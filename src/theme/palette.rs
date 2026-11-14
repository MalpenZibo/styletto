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

/// What a theme sets; the rest is derived.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scheme {
    /// The island's fill: darkest in a dark scheme, lightest in a light one.
    pub background: Color,
    pub text: Color,
    /// Secondary text and icons.
    pub sub: Color,
    /// Hints and labels.
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
            if contrast(fill.color, own) >= 4.5 {
                own
            } else {
                best(&[self.background, self.text, Color::BLACK, Color::WHITE])
            }
        })
    }

    pub fn palette(&self) -> Palette {
        // Surfaces, line and ripple: white over a dark background, black over a
        // light one.
        let light = self.is_light();
        let over = |alpha: f32| {
            if light {
                Color::rgba(0.0, 0.0, 0.0, alpha * 0.8)
            } else {
                Color::rgba(1.0, 1.0, 1.0, alpha)
            }
        };
        Palette {
            island: self.background,
            surface: over(0.065),
            surface2: over(0.11),
            line: over(0.08),
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
            ripple: over(0.16),
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
