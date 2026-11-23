//! The `[appearance]` table of a config file, resolved into a [`Ui`]: the
//! colours from a theme, a seed or a wallpaper, each side's own set over
//! them, and the fonts, density, shape, material and motion. Names match in
//! any case; one that matches nothing is logged and left out.

use std::collections::BTreeMap;
use std::path::PathBuf;

use guido::prelude::{Color, FontFamily};
use serde::{Deserialize, Serialize};

use crate::theme::material::{Contrast, Variant};
use crate::theme::palette::{Scheme, parse_hex, to_hex};
use crate::theme::presets::{Mode, Side, THEMES, Theme, ThemeChoice};
use crate::ui::{Density, Fonts, Material, Motion, Shape, Ui};

/// Colours by [`Scheme::named`] name ("on_accent"), as `#rrggbb` or
/// `#rrggbbaa`.
pub type Colors = BTreeMap<String, String>;

/// The source of the colours is one of `theme`, `seed` and `wallpaper`: the
/// default theme when none is set, the wallpaper then the seed when more
/// than one is. A theme beside a seed or a wallpaper is kept for when they
/// give no colour, and to go back to. `dark` and `light` are set over it,
/// whichever it is.
///
/// Written back (`toml::to_string(&config)`, or JSON, or another format that
/// names its fields), it leaves out every field at its default, so it holds
/// only what was chosen.
/// Its keys are at the top level: the app puts them under whatever table it
/// keeps them in.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct UiConfig {
    /// One of [`THEMES`] by name.
    #[serde(skip_serializing_if = "is_default")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub dark_flavor: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub light_flavor: Option<String>,
    /// Set on each side whose flavour has it.
    #[serde(skip_serializing_if = "is_default")]
    pub accent: Option<String>,
    /// The dark side's accent, over `accent`.
    #[serde(skip_serializing_if = "is_default")]
    pub dark_accent: Option<String>,
    /// The light side's accent, over `accent`.
    #[serde(skip_serializing_if = "is_default")]
    pub light_accent: Option<String>,
    /// Both sides worked out from this colour. Needs the `material` feature.
    #[serde(skip_serializing_if = "is_default")]
    pub seed: Option<String>,
    /// Both sides worked out from this image. Needs the `wallpaper` feature.
    /// Empty, it is one not chosen yet, and the theme shows meanwhile.
    #[serde(skip_serializing_if = "is_default")]
    pub wallpaper: Option<String>,
    /// Which of the colours the wallpaper yields, best first, the sides are
    /// made of: 0, the best, unless another is chosen; any past the last is
    /// the best.
    #[serde(skip_serializing_if = "is_default")]
    pub wallpaper_color: usize,
    /// How a seed or a wallpaper spreads over the roles.
    #[serde(skip_serializing_if = "is_default")]
    pub variant: Variant,
    #[serde(skip_serializing_if = "is_default")]
    pub contrast: Contrast,
    #[serde(skip_serializing_if = "is_default")]
    pub mode: Mode,
    #[serde(skip_serializing_if = "is_default")]
    pub dark: Colors,
    #[serde(skip_serializing_if = "is_default")]
    pub light: Colors,
    #[serde(skip_serializing_if = "is_default")]
    pub fonts: FontsConfig,
    #[serde(skip_serializing_if = "is_default")]
    pub density: Density,
    #[serde(skip_serializing_if = "is_default")]
    pub shape: Shape,
    #[serde(skip_serializing_if = "is_default")]
    pub material: Material,
    #[serde(skip_serializing_if = "is_default")]
    pub motion: Motion,
}

/// A family by name, or the system's when unset.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct FontsConfig {
    #[serde(skip_serializing_if = "is_default")]
    pub sans: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub mono: Option<String>,
}

/// Whether a value is its type's default, and so left out when written.
fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

impl UiConfig {
    /// The [`Ui`] it asks for, with the side the mode picks given what the
    /// system prefers: dark, light, or nothing said.
    ///
    /// A wallpaper is never read here: it is read once per path, and kept
    /// until it is read again, on a thread of its own (`wallpaper::picture`, with the
    /// `wallpaper` feature). Until it is, the theme shows, as
    /// [`with_wallpaper_colors`] shows it with no colours, and a reactive
    /// closure that calls this runs again once it is. Call it on the `App`'s
    /// thread; an app whose first frame must already be in the wallpaper's
    /// colours, or a program with no running `App`, calls [`ui_blocking`]
    /// instead.
    ///
    /// [`with_wallpaper_colors`]: Self::with_wallpaper_colors
    /// [`ui_blocking`]: Self::ui_blocking
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

    /// The [`Ui`] it asks for, as [`ui`] gives it, with the wallpaper already
    /// read: for an app whose first frame must be in the wallpaper's colours,
    /// as a lock screen or a greeter, or a program with no running `App`, as
    /// a one-off render.
    ///
    /// A wallpaper not read yet is read here, on the calling thread, which
    /// waits for it. What it finds goes into the store [`ui`] reads
    /// (`wallpaper::picture`), so every surface and every call after this
    /// one takes it at once, without reading it again; a read of it still
    /// under way on a thread is dropped. A picture the store has already,
    /// read or failed, is taken as it is, even one being read again
    /// (`reread_wallpaper`). A picture that can't be read is logged, and the
    /// theme shows, as with [`ui`]. Without the `wallpaper` feature it is
    /// [`ui`].
    ///
    /// Call it once, before the app draws, on the `App`'s thread if there is
    /// one. Not in a reactive closure: it blocks the thread for the read, and
    /// writes the store the closure reads; there, [`ui`] follows the config.
    /// For a `Ui` worked out in a memo, see [`read_wallpaper_blocking`].
    ///
    /// [`ui`]: Self::ui
    /// [`read_wallpaper_blocking`]: Self::read_wallpaper_blocking
    pub fn ui_blocking(&self, system_dark: Option<bool>) -> Ui {
        self.read_wallpaper_blocking();
        // Called outside a reactive closure, as it is meant to be: Guido's
        // debug builds would warn there that the store's read is a snapshot,
        // which is what this asks for. It silences the warning, nothing else.
        guido::reactive::diagnostics::snapshot_zone(|| self.ui(system_dark))
    }

    /// Reads the wallpaper on the calling thread, as [`ui_blocking`] does,
    /// and builds no `Ui`: for an app whose `Ui` is a memo over the config,
    /// so the memo's first run finds the picture read and is already in its
    /// colours.
    ///
    /// ```no_run
    /// # use guido::prelude::*;
    /// # use styletto::theme::config::UiConfig;
    /// let config = create_signal(UiConfig::default());
    /// config.with_untracked(|c| c.read_wallpaper_blocking());
    /// let ui = create_memo(move || config.with(|c| c.ui(None)));
    /// ```
    ///
    /// As with [`ui_blocking`]: a picture the store has already, read or
    /// failed, is left as it is, and one that can't be read is logged, so
    /// the theme shows. Call it once, before the app draws, on the `App`'s
    /// thread (the store is that thread's), and not in a reactive closure:
    /// it blocks for the read and writes the store the closure reads. A
    /// wallpaper set later is read as [`ui`] reads it, on a thread of its
    /// own. Without a wallpaper chosen, or without the `wallpaper` feature,
    /// it does nothing.
    ///
    /// [`ui`]: Self::ui
    /// [`ui_blocking`]: Self::ui_blocking
    pub fn read_wallpaper_blocking(&self) {
        #[cfg(feature = "wallpaper")]
        if let Some(path) = self.wallpaper_path() {
            crate::theme::wallpaper::read_blocking(&path);
        }
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
        // The accent of both sides, then each side's own over it.
        let accents = [
            (&self.accent, [false, true].as_slice()),
            (&self.dark_accent, &[false]),
            (&self.light_accent, &[true]),
        ];
        for (name, sides) in accents {
            let Some(name) = name else { continue };
            let mut found = false;
            for &light in sides {
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

    /// The wallpaper's path, `~/` standing for the home directory; none
    /// while it is unset or not chosen yet (empty).
    pub fn wallpaper_path(&self) -> Option<PathBuf> {
        let path = self.wallpaper.as_deref()?.trim();
        if path.is_empty() {
            return None;
        }
        Some(match path.strip_prefix("~/") {
            Some(rest) => std::env::home_dir().unwrap_or_default().join(rest),
            None => PathBuf::from(path),
        })
    }

    /// Sets the wallpaper's path; another picture starts from its best
    /// colour.
    pub fn set_wallpaper(&mut self, path: String) {
        if self.wallpaper.as_ref() != Some(&path) {
            self.wallpaper = Some(path);
            self.wallpaper_color = 0;
        }
    }

    /// Reads its wallpaper again, as when the file has changed, and starts
    /// from the new picture's best colour, as [`set_wallpaper`] does for a
    /// new path; see `wallpaper::reread`. Until the new picture lands, the
    /// best colour of the one read before shows. Call it on the `App`'s
    /// thread.
    ///
    /// [`set_wallpaper`]: Self::set_wallpaper
    #[cfg(feature = "wallpaper")]
    pub fn reread_wallpaper(&mut self) {
        if let Some(path) = self.wallpaper_path() {
            crate::theme::wallpaper::reread(&path);
            self.wallpaper_color = 0;
        }
    }

    /// Which of `count` colours a wallpaper yields, best first, the sides
    /// are made of: `wallpaper_color`, or the best when it is past the last.
    pub fn wallpaper_index(&self, count: usize) -> usize {
        if self.wallpaper_color < count {
            self.wallpaper_color
        } else {
            0
        }
    }

    /// This config with its wallpaper as read: the colour chosen among
    /// `colors`, those it yields best first (`material::seeds`), stands as
    /// the seed; with none yet, the theme shows. For a program that reads
    /// the wallpaper itself rather than through [`ui`](Self::ui) or
    /// [`ui_blocking`](Self::ui_blocking).
    pub fn with_wallpaper_colors(&self, colors: &[Color]) -> UiConfig {
        let mut config = self.clone();
        if config.wallpaper.take().is_some() {
            config.seed = colors
                .get(self.wallpaper_index(colors.len()))
                .map(|&color| to_hex(color));
        }
        config
    }

    /// Chooses a theme by name, or the default one, in its own flavours and
    /// accents.
    pub fn set_theme(&mut self, theme: Option<String>) {
        self.theme = theme;
        self.dark_flavor = None;
        self.light_flavor = None;
        self.accent = None;
        self.dark_accent = None;
        self.light_accent = None;
    }

    /// Sets one side's accent by name, writing as little as keeps both sides
    /// as they show: no key while each shows its flavour's own, `accent`
    /// while both show the same one, and each side's own key where they
    /// differ and it isn't its flavour's. A name its flavour hasn't is
    /// logged and left out.
    pub fn set_accent(&mut self, light: bool, name: &str) {
        let mut choice = self.choice();
        let flavor = choice.flavor_of(light);
        let Some(accent) = flavor.accents.iter().find(|a| same(a.name, name)) else {
            log::warn!("{} has no accent {name:?}", flavor.name);
            return;
        };
        choice.side_mut(light).accent = accent.name;
        let shown = [false, true].map(|light| choice.side(light).accent);
        let own = [false, true].map(|light| {
            let flavor = choice.flavor_of(light);
            flavor.accents[flavor.accent].name
        });
        let set = |side: usize| (shown[side] != own[side]).then(|| shown[side].to_string());
        (self.accent, self.dark_accent, self.light_accent) = if shown == own {
            (None, None, None)
        } else if shown[0] == shown[1] {
            (Some(shown[0].to_string()), None, None)
        } else {
            (None, set(0), set(1))
        };
    }

    /// Both sides from the wallpaper or the seed, where one is set and
    /// gives a colour.
    fn generated(&self) -> Option<[Scheme; 2]> {
        // A wallpaper not chosen yet (empty) is none.
        let wallpaper = self.wallpaper_path();
        if wallpaper.is_some() && self.seed.is_some() {
            log::warn!("appearance: both wallpaper and seed are set; using the wallpaper");
        }
        let seed = match (wallpaper, &self.seed) {
            (Some(path), _) => self.wallpaper_seed(path),
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
fn generate(seed: Color, variant: Variant, contrast: Contrast) -> Option<[Scheme; 2]> {
    Some(crate::theme::material::schemes(seed, variant, contrast))
}

#[cfg(not(feature = "material"))]
fn generate(_: Color, _: Variant, _: Contrast) -> Option<[Scheme; 2]> {
    log::warn!("appearance: a seed or a wallpaper needs styletto's material feature");
    None
}

impl UiConfig {
    /// The colour chosen among those the wallpaper at `path` yields; none
    /// until it is read, or if it can't be (the reading logs why).
    #[cfg(feature = "wallpaper")]
    fn wallpaper_seed(&self, path: PathBuf) -> Option<Color> {
        let picture = crate::theme::wallpaper::picture(&path);
        let colors = picture.colors();
        colors.get(self.wallpaper_index(colors.len())).copied()
    }

    /// The colours its wallpaper yields, best first, as far as it is read
    /// (`wallpaper::picture`); none without one.
    #[cfg(feature = "wallpaper")]
    pub(crate) fn wallpaper_colors(&self) -> Vec<Color> {
        self.wallpaper_path()
            .map(|path| crate::theme::wallpaper::picture(&path).colors().to_vec())
            .unwrap_or_default()
    }

    #[cfg(not(feature = "wallpaper"))]
    fn wallpaper_seed(&self, _: PathBuf) -> Option<Color> {
        log::warn!("appearance: a wallpaper needs styletto's wallpaper feature");
        None
    }
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
        let name = name.trim().to_lowercase();
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
    fn the_default_writes_nothing() {
        assert_eq!(toml::to_string(&UiConfig::default()).unwrap(), "");
    }

    #[test]
    fn a_key_at_its_default_is_not_written() {
        let c = config(
            "mode = \"auto\"\ndensity = \"default\"\nvariant = \"tonal-spot\"\n\
             shape = \"round\"\n[fonts]\n[dark]",
        );
        assert_eq!(toml::to_string(&c).unwrap(), "shape = \"round\"\n");
    }

    #[test]
    fn what_is_written_reads_back_the_same() {
        let tables = [
            // Every key at once, the tables among the values when written.
            "theme = \"catppuccin\"\ndark_flavor = \"macchiato\"\nlight_flavor = \"latte\"\n\
             accent = \"peach\"\ndark_accent = \"red\"\nlight_accent = \"teal\"\nseed = \"#6750a4\"\nwallpaper = \"~/Pictures/w.jpg\"\nwallpaper_color = 2\n\
             variant = \"fruit-salad\"\ncontrast = \"high\"\nmode = \"light\"\n\
             density = \"compact\"\nshape = \"round\"\nmaterial = \"glass\"\n\
             motion = \"reduced\"\n[dark]\naccent = \"#ff79c6\"\non_accent = \"#1a1b26\"\n\
             [light]\ndanger_text = \"#a00000cc\"\n[fonts]\nsans = \"Inter\"\nmono = \"Iosevka\"",
            // The other values of each choice, and one font of two.
            "variant = \"vibrant\"\ncontrast = \"medium\"\nmode = \"dark\"\n\
             density = \"comfortable\"\nshape = \"square\"\nmaterial = \"translucent\"\n\
             motion = \"none\"\n[fonts]\nmono = \"JetBrains Mono\"",
        ];
        for table in tables {
            let c = config(table);
            assert_ne!(c, UiConfig::default(), "{table}");
            let written = toml::to_string(&c).unwrap();
            assert_eq!(config(&written), c, "{written}");
        }
    }

    #[test]
    fn a_seed_that_is_no_colour_leaves_the_theme() {
        assert_eq!(
            config("seed = \"purple\"").schemes(),
            UiConfig::default().schemes()
        );
    }

    #[test]
    fn a_sides_accent_wins_over_the_accent_on_that_side_only() {
        let c = config("theme = \"catppuccin\"\naccent = \"peach\"\nlight_accent = \"teal\"");
        let choice = c.choice();
        assert_eq!(choice.dark.accent, "Peach");
        assert_eq!(choice.light.accent, "Teal");
        let c = config("theme = \"catppuccin\"\ndark_accent = \"red\"");
        let choice = c.choice();
        assert_eq!(choice.dark.accent, "Red");
        assert_eq!(choice.light, ThemeChoice::of(&THEMES[1]).light);
    }

    #[test]
    fn the_wallpaper_colour_picks_among_the_candidates() {
        let candidates = [Color::BLACK, Color::WHITE];
        let pick = |index: usize, candidates: &[Color]| {
            UiConfig {
                wallpaper: Some("~/w.jpg".to_string()),
                wallpaper_color: index,
                ..UiConfig::default()
            }
            .with_wallpaper_colors(candidates)
            .seed
        };
        assert_eq!(pick(0, &candidates), Some(to_hex(Color::BLACK)));
        assert_eq!(pick(1, &candidates), Some(to_hex(Color::WHITE)));
        assert_eq!(
            pick(2, &candidates),
            Some(to_hex(Color::BLACK)),
            "past the end, the best"
        );
        assert_eq!(pick(1, &[]), None, "none read yet");
        // Another picture starts from its best colour; the same one keeps it.
        let mut c = config("wallpaper = \"~/w.jpg\"\nwallpaper_color = 1");
        c.set_wallpaper("~/w.jpg".to_string());
        assert_eq!(c.wallpaper_color, 1);
        c.set_wallpaper("~/other.jpg".to_string());
        assert_eq!(c.wallpaper_color, 0);
    }

    #[test]
    fn an_accent_is_written_per_side_only_where_the_sides_differ() {
        let mut c = config("theme = \"catppuccin\"");
        let own = c.choice().dark.accent;
        let keys = |c: &UiConfig| {
            (
                c.accent.clone(),
                c.dark_accent.clone(),
                c.light_accent.clone(),
            )
        };
        c.set_accent(true, "peach");
        assert_eq!(keys(&c), (None, None, Some("Peach".to_string())));
        assert_eq!(c.choice().dark.accent, own, "the dark side moved");
        c.set_accent(false, "Peach");
        assert_eq!(keys(&c), (Some("Peach".to_string()), None, None));
        c.set_accent(false, "Red");
        assert_eq!(
            keys(&c),
            (None, Some("Red".to_string()), Some("Peach".to_string()))
        );
        c.set_accent(false, own);
        c.set_accent(true, own);
        assert_eq!(keys(&c), (None, None, None), "the flavours' own");
    }

    #[test]
    fn a_wallpaper_not_chosen_yet_leaves_the_theme() {
        assert_eq!(
            config("wallpaper = \"\"").schemes(),
            UiConfig::default().schemes()
        );
    }
}
