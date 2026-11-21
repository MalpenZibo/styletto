//! The `[appearance]` table of a config file, resolved into a [`Ui`]: the
//! colours from a theme, a seed or a wallpaper, each side's own set over
//! them, and the fonts, density, shape, material and motion. Names match in
//! any case; one that matches nothing is logged and left out.

use std::collections::BTreeMap;

use guido::prelude::FontFamily;
use serde::Deserialize;

use crate::theme::material::{Contrast, Variant};
use crate::theme::palette::{Scheme, parse_hex};
use crate::theme::presets::{Mode, Side, THEMES, Theme, ThemeChoice};
use crate::ui::{Density, Fonts, Material, Motion, Shape, Ui};

/// Colours by [`Scheme::named`] name, `_` for a space ("on_accent"), as
/// `#rrggbb` or `#rrggbbaa`.
pub type Colors = BTreeMap<String, String>;

/// The source of the colours is one of `theme`, `seed` and `wallpaper`: the
/// default theme when none is set, the wallpaper then the seed when more
/// than one is. `dark` and `light` are set over it, whichever it is.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// One of [`THEMES`] by name.
    pub theme: Option<String>,
    pub dark_flavor: Option<String>,
    pub light_flavor: Option<String>,
    /// Set on each side whose flavour has it.
    pub accent: Option<String>,
    /// Both sides worked out from this colour. Needs the `material` feature.
    pub seed: Option<String>,
    /// Both sides worked out from this image. Needs the `wallpaper` feature.
    pub wallpaper: Option<String>,
    /// How a seed or a wallpaper spreads over the roles.
    pub variant: Variant,
    pub contrast: Contrast,
    pub mode: Mode,
    pub dark: Colors,
    pub light: Colors,
    pub fonts: FontsConfig,
    pub density: Density,
    pub shape: Shape,
    pub material: Material,
    pub motion: Motion,
}

/// A family by name, or the system's when unset.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct FontsConfig {
    pub sans: Option<String>,
    pub mono: Option<String>,
}

impl UiConfig {
    /// The [`Ui`] it asks for, with the side the mode picks given what the
    /// system prefers: dark, light, or nothing said. A wallpaper is read
    /// here, so call it off the UI thread or keep what it gives.
    pub fn ui(&self, system_dark: Option<bool>) -> Ui {
        let system = Fonts::default();
        let family =
            |name: &Option<String>, system| name.as_deref().map_or(system, FontFamily::name);
        let light = !self.mode.is_dark(system_dark);
        Ui::new(self.scheme(light).palette())
            .with_density(self.density)
            .with_shape(self.shape)
            .with_material(self.material)
            .with_motion(self.motion)
            .with_fonts(Fonts {
                sans: family(&self.fonts.sans, system.sans),
                mono: family(&self.fonts.mono, system.mono),
            })
    }

    /// Both sides, the dark one first, with their colours set over them.
    pub fn schemes(&self) -> [Scheme; 2] {
        let [mut dark, mut light] = self.generated().unwrap_or_else(|| {
            let choice = self.choice();
            [choice.scheme_of(false), choice.scheme_of(true)]
        });
        set(&mut dark, &self.dark, "dark");
        set(&mut light, &self.light, "light");
        [dark, light]
    }

    pub fn scheme(&self, light: bool) -> Scheme {
        self.schemes()[light as usize]
    }

    /// The theme in the flavours and accent asked for, in its mode.
    pub fn choice(&self) -> ThemeChoice {
        let theme = self.theme();
        let mut choice = ThemeChoice::of(theme);
        choice.mode = self.mode;
        for (light, name) in [(false, &self.dark_flavor), (true, &self.light_flavor)] {
            let Some(name) = name else { continue };
            let flavors = theme.flavors(light);
            match flavors.iter().position(|f| same(f.name, name)) {
                Some(index) => {
                    let flavor = flavors[index];
                    *choice.side_mut(light) = Side {
                        flavor: index,
                        accent: flavor.accents[flavor.accent].name,
                    };
                }
                None => log::warn!("{} has no flavour {name:?}", theme.name),
            }
        }
        if let Some(name) = &self.accent {
            let mut found = false;
            for light in [false, true] {
                if let Some(accent) = choice
                    .flavor_of(light)
                    .accents
                    .iter()
                    .find(|a| same(a.name, name))
                {
                    choice.side_mut(light).accent = accent.name;
                    found = true;
                }
            }
            if !found {
                log::warn!("{} has no accent {name:?}", theme.name);
            }
        }
        choice
    }

    /// Both sides from the wallpaper or the seed, where one is set and
    /// gives a colour.
    fn generated(&self) -> Option<[Scheme; 2]> {
        if self.wallpaper.is_some() && self.seed.is_some() {
            log::warn!("appearance: both wallpaper and seed are set; using the wallpaper");
        }
        if (self.wallpaper.is_some() || self.seed.is_some()) && self.theme.is_some() {
            log::warn!("appearance: theme is set beside a wallpaper or a seed, which wins");
        }
        let seed = match (&self.wallpaper, &self.seed) {
            (Some(path), _) => wallpaper_seed(path),
            (None, Some(hex)) => {
                let color = parse_hex(hex);
                if color.is_none() {
                    log::warn!("appearance.seed: {hex:?} is not #rrggbb");
                }
                color
            }
            (None, None) => None,
        }?;
        generate(seed, self.variant, self.contrast)
    }

    fn theme(&self) -> &'static Theme {
        let Some(name) = &self.theme else {
            return &THEMES[0];
        };
        THEMES
            .iter()
            .find(|t| same(t.name, name))
            .unwrap_or_else(|| {
                log::warn!("no theme {name:?}; using {}", THEMES[0].name);
                &THEMES[0]
            })
    }
}

#[cfg(feature = "material")]
fn generate(
    seed: guido::prelude::Color,
    variant: Variant,
    contrast: Contrast,
) -> Option<[Scheme; 2]> {
    Some(crate::theme::material::schemes(seed, variant, contrast))
}

#[cfg(not(feature = "material"))]
fn generate(_: guido::prelude::Color, _: Variant, _: Contrast) -> Option<[Scheme; 2]> {
    log::warn!("appearance: a seed or a wallpaper needs styletto's material feature");
    None
}

#[cfg(feature = "wallpaper")]
fn wallpaper_seed(path: &str) -> Option<guido::prelude::Color> {
    let path = match path.trim().strip_prefix("~/") {
        Some(rest) => std::env::home_dir().unwrap_or_default().join(rest),
        None => std::path::PathBuf::from(path.trim()),
    };
    match crate::theme::material::seeds(&path) {
        Ok(seeds) => seeds.first().copied(),
        Err(err) => {
            log::warn!("appearance.wallpaper {}: {err}", path.display());
            None
        }
    }
}

#[cfg(not(feature = "wallpaper"))]
fn wallpaper_seed(_: &str) -> Option<guido::prelude::Color> {
    log::warn!("appearance: a wallpaper needs styletto's wallpaper feature");
    None
}

/// Sets `colors` over `scheme`, by name. A colour set is used as given; the
/// text it leaves to the source is made to read again, on a background set
/// as on the source's, and a hue's text follows its fill.
fn set(scheme: &mut Scheme, colors: &Colors, side: &str) {
    if colors.is_empty() {
        return;
    }
    let mut kept = Vec::new();
    for (name, hex) in colors {
        let name = name.trim().to_lowercase().replace('_', " ");
        if !scheme.named().iter().any(|(known, _)| *known == name) {
            log::warn!("appearance.{side}: no colour {name:?}");
            continue;
        }
        match parse_hex(hex) {
            Some(color) => {
                *scheme = scheme.with(&name, Some(color));
                kept.push(name);
            }
            None => log::warn!("appearance.{side}.{name}: {hex:?} is not #rrggbb or #rrggbbaa"),
        }
    }
    *scheme = scheme.readable_but(|name| kept.iter().any(|set| set == name));
}

fn same(name: &str, asked: &str) -> bool {
    name.eq_ignore_ascii_case(asked.trim())
}

#[cfg(test)]
mod tests {
    use guido::prelude::Color;

    use super::*;

    fn config(toml: &str) -> UiConfig {
        toml::from_str(toml).unwrap()
    }

    #[test]
    fn left_alone_it_is_the_default_ui() {
        assert_eq!(UiConfig::default().ui(None), Ui::default());
    }

    #[test]
    fn a_flavour_and_an_accent_are_found_in_any_case() {
        let c = config("theme = \"catppuccin\"\ndark_flavor = \"macchiato\"\naccent = \"peach\"");
        let catppuccin = THEMES.iter().find(|t| t.name == "Catppuccin").unwrap();
        let macchiato = catppuccin.flavors(false)[1];
        let peach = macchiato
            .accents
            .iter()
            .find(|a| a.name == "Peach")
            .unwrap();
        let scheme = c.scheme(false);
        assert_eq!(scheme.background, macchiato.scheme(0).background);
        assert_eq!(scheme.accent.color, peach.color);
    }

    #[test]
    fn a_colour_set_wins_over_the_themes() {
        let c = config("[dark]\naccent = \"#333333\"\non_accent = \"#ffffff\"");
        let [dark, light] = c.schemes();
        assert_eq!(dark.accent.color, Color::rgb(0.2, 0.2, 0.2));
        assert_eq!(dark.accent.on, Some(Color::WHITE));
        assert_eq!(light, UiConfig::default().scheme(true));
    }

    #[test]
    fn a_background_set_keeps_the_text_left_to_the_theme_reading() {
        let c = config("[dark]\nbackground = \"#5a5a6a\"\ndim = \"#606070\"");
        let dark = c.scheme(false);
        // dim was set, so it stays as given even where it doesn't read.
        assert_eq!(dark.dim, Color::from_hex(0x606070));
        let palette = Scheme {
            dim: dark.text,
            ..dark
        }
        .palette();
        assert_eq!(palette.unreadable(palette.island), None);
    }

    #[test]
    fn a_hues_text_set_is_kept_and_one_left_follows_its_fill() {
        let c = config("[light]\nwarning = \"#ffd000\"\ndanger_text = \"#ff8080\"");
        let light = c.scheme(true);
        let palette = light.palette();
        // Set by hand, it stays as given even where it doesn't read.
        assert_eq!(light.danger.text, Some(Color::from_hex(0xff8080)));
        assert_eq!(palette.danger.text, Color::from_hex(0xff8080));
        // Left to the source, it is the new fill darkened until it reads.
        assert_ne!(palette.warning.text, palette.warning.fill);
        let mut reading = palette;
        reading.danger.text = palette.text;
        assert_eq!(reading.unreadable(palette.island), None);
        // A text tone of the source's own follows a fill set over it.
        let mut source = UiConfig::default().scheme(true);
        source.warning.text = Some(Color::from_hex(0x6a4a00));
        let set = source
            .with("warning", Some(Color::from_hex(0xffd000)))
            .readable_but(|name| name == "warning");
        assert_eq!(set.warning.text, None);
    }

    #[test]
    fn the_mode_picks_the_side() {
        let light = config("mode = \"light\"").ui(None);
        let dark = config("mode = \"dark\"").ui(None);
        assert_ne!(light, dark);
        let auto = config("mode = \"auto\"");
        assert_eq!(auto.ui(Some(false)), light);
        assert_eq!(auto.ui(Some(true)), dark);
    }

    #[test]
    fn the_choices_read_in_lowercase() {
        let c = config(
            "density = \"compact\"\nshape = \"round\"\nmaterial = \"glass\"\nmotion = \"none\"",
        );
        assert_eq!(
            (c.density, c.shape, c.material, c.motion),
            (
                Density::Compact,
                Shape::Round,
                Material::Glass,
                Motion::None
            )
        );
    }

    #[cfg(feature = "material")]
    #[test]
    fn a_seed_makes_both_sides_and_wins_over_the_theme() {
        let c = config(
            "theme = \"nord\"\nseed = \"#6750a4\"\nvariant = \"fruit-salad\"\ncontrast = \"high\"",
        );
        let made = crate::theme::material::schemes(
            Color::from_hex(0x6750a4),
            Variant::FruitSalad,
            Contrast::High,
        );
        assert_eq!(c.schemes(), made);
    }

    #[test]
    fn a_seed_that_is_no_colour_leaves_the_theme() {
        assert_eq!(
            config("seed = \"purple\"").schemes(),
            UiConfig::default().schemes()
        );
    }
}
