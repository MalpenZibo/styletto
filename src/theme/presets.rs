//! The preset themes, each with dark and light flavours and its own
//! accents, and the choice of one of them a surface shows.

use guido::prelude::*;

#[cfg(test)]
use super::palette::{BODY_CONTRAST, Hue, HueColors, TEXT_CONTRAST, contrast};
use super::palette::{Palette, Scheme, SchemeHue};

mod catppuccin;
mod dracula;
mod everforest;
mod gruvbox;
mod kanagawa;
mod nord;
mod rose_pine;
mod tokyo_night;

use catppuccin::CATPPUCCIN;
use dracula::DRACULA;
use everforest::EVERFOREST;
use gruvbox::GRUVBOX;
use kanagawa::KANAGAWA;
use nord::NORD;
use rose_pine::ROSE_PINE;
use tokyo_night::TOKYO_NIGHT;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Accent {
    pub name: &'static str,
    pub color: Color,
}

const fn accent(name: &'static str, hex: u32) -> Accent {
    Accent {
        name,
        color: Color::from_hex(hex),
    }
}

/// One flavour of a theme: neutrals, state colours and accents. `sub` and
/// `dim` are colours the theme itself gives text (labels, comments, subtle
/// text), not its borders or overlays; where none reads well enough,
/// [`Scheme::readable`] works one out from the nearest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flavor {
    pub name: &'static str,
    pub light: bool,
    island: u32,
    text: u32,
    sub: u32,
    dim: u32,
    success: u32,
    warning: u32,
    danger: u32,
    pub accents: &'static [Accent],
    /// The default accent.
    pub accent: usize,
}

impl Flavor {
    pub fn scheme(&self, accent: usize) -> Scheme {
        let fill = |hex: u32| SchemeHue::new(Color::from_hex(hex));
        Scheme {
            background: Color::from_hex(self.island),
            text: Color::from_hex(self.text),
            sub: Color::from_hex(self.sub),
            dim: Color::from_hex(self.dim),
            accent: SchemeHue::new(
                self.accents
                    .get(accent)
                    .unwrap_or(&self.accents[self.accent])
                    .color,
            ),
            success: fill(self.success),
            warning: fill(self.warning),
            danger: fill(self.danger),
        }
        .readable()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub flavors: &'static [Flavor],
}

impl Theme {
    /// Its flavours of one kind, in order.
    pub fn flavors(&self, light: bool) -> Vec<&'static Flavor> {
        self.flavors.iter().filter(|f| f.light == light).collect()
    }
}

/// Which kind of flavour shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Mode {
    Light,
    Dark,
    /// Follows the system; dark when it has no preference.
    #[default]
    Auto,
}

impl Mode {
    /// Whether it is dark, given the system's preference.
    pub fn is_dark(self, system_dark: Option<bool>) -> bool {
        match self {
            Mode::Light => false,
            Mode::Dark => true,
            Mode::Auto => system_dark.unwrap_or(true),
        }
    }
}

/// A chosen theme: a dark and a light flavour, each with its accent, and
/// the mode that picks between them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeChoice {
    pub theme: &'static Theme,
    pub dark: Side,
    pub light: Side,
    pub mode: Mode,
}

/// One kind's flavour and accent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Side {
    /// An index into [`Theme::flavors`] of the kind.
    pub flavor: usize,
    pub accent: &'static str,
}

impl Default for ThemeChoice {
    fn default() -> Self {
        Self::of(&THEMES[0])
    }
}

impl ThemeChoice {
    /// The theme with its first flavours and their default accents.
    pub fn of(theme: &'static Theme) -> Self {
        let side = |light: bool| {
            let first = theme
                .flavors(light)
                .first()
                .copied()
                .unwrap_or(&theme.flavors[0]);
            Side {
                flavor: 0,
                accent: first.accents[first.accent].name,
            }
        };
        Self {
            theme,
            dark: side(false),
            light: side(true),
            mode: Mode::default(),
        }
    }

    pub fn side(&self, light: bool) -> Side {
        if light { self.light } else { self.dark }
    }

    pub fn side_mut(&mut self, light: bool) -> &mut Side {
        if light {
            &mut self.light
        } else {
            &mut self.dark
        }
    }

    /// One kind's flavour, or the other kind's if the theme has none.
    pub fn flavor_of(&self, light: bool) -> &'static Flavor {
        let pick = |light: bool| {
            let flavors = self.theme.flavors(light);
            let index = self.side(light).flavor;
            flavors.get(index).or(flavors.first()).copied()
        };
        pick(light)
            .or_else(|| pick(!light))
            .unwrap_or(&self.theme.flavors[0])
    }

    /// Whether the light side shows, given what the system prefers.
    pub fn shows_light(&self, system_dark: Option<bool>) -> bool {
        !self.mode.is_dark(system_dark)
    }

    /// One kind's scheme, its accent looked up by name.
    pub fn scheme_of(&self, light: bool) -> Scheme {
        let flavor = self.flavor_of(light);
        let name = self.side(light).accent;
        let accent = flavor
            .accents
            .iter()
            .position(|a| a.name == name)
            .unwrap_or(flavor.accent);
        flavor.scheme(accent)
    }

    pub fn palette_of(&self, light: bool) -> Palette {
        self.scheme_of(light).palette()
    }

    /// The palette that shows.
    pub fn palette(&self, system_dark: Option<bool>) -> Palette {
        self.palette_of(self.shows_light(system_dark))
    }
}

/// The built-in themes.
pub static THEMES: &[Theme] = &[
    TOKYO_NIGHT,
    CATPPUCCIN,
    NORD,
    GRUVBOX,
    DRACULA,
    ROSE_PINE,
    EVERFOREST,
    KANAGAWA,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fill_reads() {
        let mut worst = (f32::MAX, String::new());
        for theme in THEMES {
            for flavor in theme.flavors {
                for (index, accent) in flavor.accents.iter().enumerate() {
                    let scheme = flavor.scheme(index);
                    let palette = scheme.palette();
                    for hue in Hue::ALL {
                        let HueColors { fill, on, .. } = palette.hue(hue);
                        let role = match hue {
                            Hue::Accent => accent.name,
                            hue => hue.name(),
                        };
                        let ratio = contrast(fill, on);
                        if ratio < worst.0 {
                            worst = (ratio, format!("{} {} {role}", theme.name, flavor.name));
                        }
                    }
                }
            }
        }
        assert!(
            worst.0 >= TEXT_CONTRAST,
            "{} reads at {:.2}",
            worst.1,
            worst.0
        );
    }

    #[test]
    fn every_text_reads() {
        // The text, and every hue as text with each of the accents; and the
        // text at 7:1 on the island.
        for theme in THEMES {
            for flavor in theme.flavors {
                for (index, accent) in flavor.accents.iter().enumerate() {
                    let palette = flavor.scheme(index).palette();
                    let name = format!("{} {} {}", theme.name, flavor.name, accent.name);
                    assert_eq!(palette.unreadable(palette.island), None, "{name}");
                    let body = contrast(palette.text, palette.island);
                    assert!(body >= BODY_CONTRAST, "{name}: text at {body:.2}");
                }
            }
        }
    }

    #[test]
    fn a_light_theme_keeps_two_levels_of_text() {
        // Their own text only just read: held to 7:1, it leaves `sub` room
        // to be lighter than it.
        for (theme, flavor) in [(TOKYO_NIGHT, "Day"), (EVERFOREST, "Light")] {
            let flavor = theme.flavors.iter().find(|f| f.name == flavor).unwrap();
            let scheme = flavor.scheme(flavor.accent);
            assert_ne!(scheme.sub, scheme.text, "{} {}", theme.name, flavor.name);
        }
    }

    #[test]
    fn a_theme_keeps_the_colours_that_read() {
        // Catppuccin Mocha's own text colours all read: none is worked out,
        // and its hues read as text as they are.
        let mocha = CATPPUCCIN.flavors[0];
        let scheme = mocha.scheme(mocha.accent);
        assert_eq!(scheme.text, Color::from_hex(mocha.text));
        assert_eq!(scheme.sub, Color::from_hex(mocha.sub));
        assert_eq!(scheme.dim, Color::from_hex(mocha.dim));
        let palette = scheme.palette();
        assert_eq!(palette.accent.text, palette.accent.fill);
        assert_eq!(palette.danger.text, palette.danger.fill);
    }
}
