//! Colours by meaning. A theme gives a [`Scheme`]; the [`Palette`]
//! components read is derived from it.

use guido::prelude::*;

use super::presets::ThemeChoice;

/// A colour a surface can be painted with, and the one that reads on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fill {
    pub color: Color,
    /// Derived by [`Scheme::on`] when `None`.
    pub on: Option<Color>,
}

impl Fill {
    pub const fn new(color: Color) -> Self {
        Self { color, on: None }
    }
}

/// The contrast a text keeps on everything it can sit on: WCAG's AA for
/// body text.
pub(crate) const TEXT_CONTRAST: f32 = 4.5;
/// The contrast `dim` keeps: WCAG's floor for large text and for what is not
/// text, which is all `dim` carries.
const DIM_CONTRAST: f32 = 3.0;
/// What `text`, `sub` and `dim` reach, in that order.
const FLOORS: [f32; 3] = [TEXT_CONTRAST, TEXT_CONTRAST, DIM_CONTRAST];

/// What a theme sets; the rest is derived.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scheme {
    /// The island's fill: darkest in a dark scheme, lightest in a light one.
    pub background: Color,
    pub text: Color,
    /// Secondary text: captions, placeholders, small labels.
    pub sub: Color,
    /// Icons, dividers and large text; never small text.
    pub dim: Color,
    pub accent: Fill,
    pub success: Fill,
    pub warning: Fill,
    pub danger: Fill,
}

/// The colours the components read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub island: Color,
    /// A control's resting fill.
    pub surface: Color,
    /// Its hovered or chosen fill.
    pub surface2: Color,
    pub line: Color,
    pub text: Color,
    pub sub: Color,
    pub dim: Color,
    pub accent: Color,
    pub on_accent: Color,
    pub success: Color,
    pub on_success: Color,
    pub warning: Color,
    pub on_warning: Color,
    pub danger: Color,
    pub on_danger: Color,
    /// The ripple of a press.
    pub ripple: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Scheme::default().palette()
    }
}

impl Palette {
    /// Every colour with its name, in the order a picker lists them.
    pub fn named(&self) -> [(&'static str, Color); 16] {
        [
            ("island", self.island),
            ("surface", self.surface),
            ("surface2", self.surface2),
            ("line", self.line),
            ("text", self.text),
            ("sub", self.sub),
            ("dim", self.dim),
            ("accent", self.accent),
            ("on_accent", self.on_accent),
            ("success", self.success),
            ("on_success", self.on_success),
            ("warning", self.warning),
            ("on_warning", self.on_warning),
            ("danger", self.danger),
            ("on_danger", self.on_danger),
            ("ripple", self.ripple),
        ]
    }

    /// Each fill with the colour that reads on it.
    pub fn fills(&self) -> [(&'static str, Color, Color); 4] {
        [
            ("Accent", self.accent, self.on_accent),
            ("Success", self.success, self.on_success),
            ("Warning", self.warning, self.on_warning),
            ("Danger", self.danger, self.on_danger),
        ]
    }

    /// What a text can sit on: the island, as it shows over `backdrop`, and
    /// a control's fills over it.
    fn grounds(&self, backdrop: Color) -> [Color; 3] {
        let island = over(self.island, backdrop);
        [
            island,
            over(self.surface, island),
            over(self.surface2, island),
        ]
    }

    /// What shows behind a see-through island worst for its text: black
    /// under dark text, white under light text.
    pub(crate) fn worst_backdrop(&self) -> Color {
        away(self.text, self.island)
    }

    /// The palette with the island at `opacity`, its text darkened or
    /// lightened as far as keeps the contrast it has on the opaque island,
    /// up to the floors, whatever shows through. The worst is the opposite
    /// of the text: black behind a light island turns it grey under dark
    /// text, white behind a dark one turns it grey under light text. Where
    /// not even black or white would read, the island is drawn as much
    /// denser as lets them.
    pub(crate) fn see_through(&self, opacity: f32) -> Palette {
        // The text moves toward the same extreme that is worst behind.
        let worst = self.worst_backdrop();
        let solid = self.grounds(self.island);
        let inks = [self.text, self.sub, self.dim];
        // A colour set below its floor keeps the contrast it was set with.
        let floors = [0, 1, 2].map(|i| FLOORS[i].min(lowest(inks[i], solid)));
        let at = |opacity: f32| Palette {
            island: self.island.scale_alpha(opacity),
            ..*self
        };
        let opacity = (0..=STEPS)
            .map(|step| opacity + (1.0 - opacity) * step as f32 / STEPS as f32)
            .find(|&denser| lowest(worst, at(denser).grounds(worst)) >= floors[0])
            .unwrap_or(1.0);
        let island = at(opacity);
        let [text, sub, dim] = lifted(inks, worst, island.grounds(worst), floors);
        Palette {
            text,
            sub,
            dim,
            ..island
        }
    }
}

#[cfg(test)]
impl Palette {
    /// The first of `text`, `sub` and `dim` short of its floor on the
    /// island over `backdrop`, with the contrast it has.
    pub(crate) fn unreadable(&self, backdrop: Color) -> Option<(&'static str, f32)> {
        let grounds = self.grounds(backdrop);
        [("text", self.text), ("sub", self.sub), ("dim", self.dim)]
            .into_iter()
            .zip(FLOORS)
            .map(|((role, ink), floor)| (role, lowest(ink, grounds), floor))
            .find(|&(_, ratio, floor)| ratio < floor)
            .map(|(role, ratio, _)| (role, ratio))
    }
}

/// Black for text darker than `ground`, white for text lighter: where the
/// text goes to stand out more, and what behind a see-through `ground` makes
/// it stand out least.
fn away(text: Color, ground: Color) -> Color {
    if luminance(text) < luminance(ground) {
        Color::BLACK
    } else {
        Color::WHITE
    }
}

/// How finely a colour is searched for.
const STEPS: u16 = 100;

/// `top` painted over the opaque `below`, the channels mixed as they are
/// stored, as the renderer blends them.
fn over(top: Color, below: Color) -> Color {
    below.mix(top.with_alpha(1.0), top.a)
}

/// `text`, `sub` and `dim` reaching their `floors` on all of `grounds`:
/// `text` mixed toward `extreme`, the others toward `text`, each as little
/// as gets it there.
fn lifted(
    [text, sub, dim]: [Color; 3],
    extreme: Color,
    grounds: [Color; 3],
    floors: [f32; 3],
) -> [Color; 3] {
    let text = lift(text, extreme, grounds, floors[0]);
    [
        text,
        lift(sub, text, grounds, floors[1]),
        lift(dim, text, grounds, floors[2]),
    ]
}

/// `ink` mixed toward `toward` as little as makes it reach `floor` on all of
/// `grounds`; `toward` itself when nothing short of it does.
fn lift(ink: Color, toward: Color, grounds: [Color; 3], floor: f32) -> Color {
    (0..=STEPS)
        .map(|step| ink.mix(toward, step as f32 / STEPS as f32))
        .find(|&mixed| lowest(mixed, grounds) >= floor)
        .unwrap_or(toward)
}

/// The lowest contrast `ink` has on any of `grounds`.
fn lowest(ink: Color, grounds: [Color; 3]) -> f32 {
    grounds
        .map(|ground| contrast(ink, ground))
        .into_iter()
        .fold(f32::MAX, f32::min)
}

/// WCAG's relative luminance.
fn luminance(color: Color) -> f32 {
    let linear = |c: f32| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

/// WCAG's contrast ratio, 1 to 21.
pub fn contrast(a: Color, b: Color) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

impl Default for Scheme {
    fn default() -> Self {
        ThemeChoice::default().scheme_of(false)
    }
}

impl Scheme {
    /// Whether its background is light.
    pub fn is_light(&self) -> bool {
        luminance(self.background) > 0.5
    }

    /// The colour that reads on `fill`: the background or the text if one
    /// reaches 4.5:1, else the best of them, black and white.
    pub fn on(&self, fill: Fill) -> Color {
        let best = |candidates: &[Color]| {
            candidates
                .iter()
                .copied()
                .max_by(|a, b| contrast(fill.color, *a).total_cmp(&contrast(fill.color, *b)))
                .unwrap_or(self.text)
        };
        fill.on.unwrap_or_else(|| {
            let own = best(&[self.background, self.text]);
            if contrast(fill.color, own) >= TEXT_CONTRAST {
                own
            } else {
                best(&[self.background, self.text, Color::BLACK, Color::WHITE])
            }
        })
    }

    /// The scheme with `text` and `sub` reaching 4.5:1 and `dim` 3:1 on the
    /// island and a control's fills. A colour short of it is mixed toward
    /// `text`, and `text` toward black or white, as little as gets it there.
    /// A theme's own colours are kept where they already read.
    pub fn readable(self) -> Self {
        self.readable_but([false; 3])
    }

    /// [`readable`](Self::readable), leaving `text`, `sub` and `dim` as they
    /// are where `kept` says so, in that order.
    pub(crate) fn readable_but(self, kept: [bool; 3]) -> Self {
        let palette = self.palette();
        let floors = [0, 1, 2].map(|i| if kept[i] { 0.0 } else { FLOORS[i] });
        let [text, sub, dim] = lifted(
            [self.text, self.sub, self.dim],
            away(self.text, self.background),
            palette.grounds(palette.island),
            floors,
        );
        Self {
            text,
            sub,
            dim,
            ..self
        }
    }

    pub fn palette(&self) -> Palette {
        // Surfaces, line and ripple: white over a dark background, black over a
        // light one.
        let light = self.is_light();
        let tint = |alpha: f32| {
            if light {
                Color::rgba(0.0, 0.0, 0.0, alpha * 0.8)
            } else {
                Color::rgba(1.0, 1.0, 1.0, alpha)
            }
        };
        Palette {
            island: self.background,
            surface: tint(0.065),
            surface2: tint(0.11),
            line: tint(0.08),
            text: self.text,
            sub: self.sub,
            dim: self.dim,
            accent: self.accent.color,
            on_accent: self.on(self.accent),
            success: self.success.color,
            on_success: self.on(self.success),
            warning: self.warning.color,
            on_warning: self.on(self.warning),
            danger: self.danger.color,
            on_danger: self.on(self.danger),
            ripple: tint(0.16),
        }
    }

    /// Every colour by name; an on-colour may be unset.
    pub fn named(&self) -> [(&'static str, Option<Color>); 12] {
        [
            ("background", Some(self.background)),
            ("text", Some(self.text)),
            ("sub", Some(self.sub)),
            ("dim", Some(self.dim)),
            ("accent", Some(self.accent.color)),
            ("on accent", self.accent.on),
            ("success", Some(self.success.color)),
            ("on success", self.success.on),
            ("warning", Some(self.warning.color)),
            ("on warning", self.warning.on),
            ("danger", Some(self.danger.color)),
            ("on danger", self.danger.on),
        ]
    }

    /// Sets `name` to `color`. A fill's own colour can't be unset; an unknown
    /// name changes nothing.
    pub fn with(mut self, name: &str, color: Option<Color>) -> Self {
        let solid = |slot: &mut Color| {
            if let Some(color) = color {
                *slot = color;
            }
        };
        match name {
            "background" => solid(&mut self.background),
            "text" => solid(&mut self.text),
            "sub" => solid(&mut self.sub),
            "dim" => solid(&mut self.dim),
            "accent" => solid(&mut self.accent.color),
            "success" => solid(&mut self.success.color),
            "warning" => solid(&mut self.warning.color),
            "danger" => solid(&mut self.danger.color),
            "on accent" => self.accent.on = color,
            "on success" => self.success.on = color,
            "on warning" => self.warning.on = color,
            "on danger" => self.danger.on = color,
            _ => {}
        }
        self
    }
}

/// `#rrggbb` or `#rrggbbaa`, the `#` optional.
pub fn parse_hex(hex: &str) -> Option<Color> {
    let hex = hex.trim().trim_start_matches('#');
    let value = u32::from_str_radix(hex, 16).ok()?;
    match hex.len() {
        6 => Some(Color::from_hex(value)),
        8 => Some(Color::from_hex(value >> 8).with_alpha((value & 0xff) as f32 / 255.0)),
        _ => None,
    }
}

/// The colour as `#rrggbb`, or `#rrggbbaa` when it is not opaque.
pub fn to_hex(color: Color) -> String {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let (r, g, b, a) = (byte(color.r), byte(color.g), byte(color.b), byte(color.a));
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}
