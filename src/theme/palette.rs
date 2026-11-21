//! Colours by meaning. A theme gives a [`Scheme`]; the [`Palette`]
//! components read is derived from it.

use guido::prelude::*;

use super::presets::ThemeChoice;

/// One of the four hues a theme sets a fill for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hue {
    Accent,
    Success,
    Warning,
    Danger,
}

impl Hue {
    pub const ALL: [Hue; 4] = [Hue::Accent, Hue::Success, Hue::Warning, Hue::Danger];

    /// Its fill's name, in a [`Scheme`] and a [`Palette`]: `accent` and the
    /// rest.
    pub const fn name(self) -> &'static str {
        match self {
            Hue::Accent => "accent",
            Hue::Success => "success",
            Hue::Warning => "warning",
            Hue::Danger => "danger",
        }
    }
}

/// A colour a surface can be painted with, the one that reads on it, and
/// its hue as text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fill {
    pub color: Color,
    /// Derived by [`Scheme::on`] when `None`.
    pub on: Option<Color>,
    /// The hue as text or an icon on the island. Worked out from `color`
    /// when `None`: see [`HueColors::text`].
    pub text: Option<Color>,
}

impl Fill {
    pub const fn new(color: Color) -> Self {
        Self {
            color,
            on: None,
            text: None,
        }
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
/// The contrast a hue's text keeps on a see-through island, whatever shows
/// through: `dim`'s. 4.5:1 there would turn the hues of a dark theme to
/// near white on glass, as light as its text, and a warning would no
/// longer look like one; on a solid island they keep 4.5:1.
const SEE_THROUGH_HUE_CONTRAST: f32 = DIM_CONTRAST;

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

/// A hue's colours in the [`Palette`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HueColors {
    /// What paints a surface, a bar, a ring or a border.
    pub fill: Color,
    /// The text and icons on `fill`.
    pub on: Color,
    /// The hue as text or an icon on the island, `surface` and `surface2`,
    /// at 4.5:1 on all three. Unless a theme or a config sets it, the fill
    /// itself moved in lightness only, hue and chroma kept, as far as it
    /// must to read: the fill where it already does. On a see-through
    /// island ([`Ui::colors`](crate::Ui::colors)) it keeps 3:1 whatever
    /// shows through, as `dim` does: 4.5:1 there would turn it near white
    /// on dark glass.
    pub text: Color,
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
    pub accent: HueColors,
    pub success: HueColors,
    pub warning: HueColors,
    pub danger: HueColors,
    /// The ripple of a press.
    pub ripple: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Scheme::default().palette()
    }
}

impl Palette {
    /// What reads as text in `hue`: the hue as text, or `text` without one.
    pub fn ink(&self, hue: Option<Hue>) -> Color {
        hue.map_or(self.text, |hue| self.hue(hue).text)
    }

    /// The colours of `hue`.
    pub fn hue(&self, hue: Hue) -> &HueColors {
        match hue {
            Hue::Accent => &self.accent,
            Hue::Success => &self.success,
            Hue::Warning => &self.warning,
            Hue::Danger => &self.danger,
        }
    }

    fn hue_mut(&mut self, hue: Hue) -> &mut HueColors {
        match hue {
            Hue::Accent => &mut self.accent,
            Hue::Success => &mut self.success,
            Hue::Warning => &mut self.warning,
            Hue::Danger => &mut self.danger,
        }
    }

    /// Every colour with its name, in the order a picker lists them: a
    /// hue's as `accent`, `on_accent` and `accent_text`.
    pub fn named(&self) -> [(&'static str, Color); 20] {
        let [a, s, w, d] = [self.accent, self.success, self.warning, self.danger];
        [
            ("island", self.island),
            ("surface", self.surface),
            ("surface2", self.surface2),
            ("line", self.line),
            ("text", self.text),
            ("sub", self.sub),
            ("dim", self.dim),
            ("accent", a.fill),
            ("on_accent", a.on),
            ("accent_text", a.text),
            ("success", s.fill),
            ("on_success", s.on),
            ("success_text", s.text),
            ("warning", w.fill),
            ("on_warning", w.on),
            ("warning_text", w.text),
            ("danger", d.fill),
            ("on_danger", d.on),
            ("danger_text", d.text),
            ("ripple", self.ripple),
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
    /// up to the floors, whatever shows through; each hue's text the same,
    /// in lightness only, up to 3:1. The worst is the opposite of the text:
    /// black behind a light island turns it grey under dark text, white
    /// behind a dark one turns it grey under light text. Where not even
    /// black or white would read, the island is drawn as much denser as
    /// lets them.
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
        let grounds = island.grounds(worst);
        let [text, sub, dim] = lifted(inks, worst, grounds, floors);
        let mut palette = Palette {
            text,
            sub,
            dim,
            ..island
        };
        for hue in Hue::ALL {
            let ink = &mut palette.hue_mut(hue).text;
            let floor = SEE_THROUGH_HUE_CONTRAST.min(lowest(*ink, solid));
            *ink = shade(*ink, worst, grounds, floor);
        }
        palette
    }
}

#[cfg(test)]
impl Palette {
    /// The first of `text`, `sub`, `dim` and the hues' texts short of its
    /// floor on the island over `backdrop`, with the contrast it has. A
    /// hue's floor is 4.5:1 on a solid island, 3:1 on a see-through one.
    pub(crate) fn unreadable(&self, backdrop: Color) -> Option<(&'static str, f32)> {
        let grounds = self.grounds(backdrop);
        let hue = if self.island.a < 1.0 {
            SEE_THROUGH_HUE_CONTRAST
        } else {
            TEXT_CONTRAST
        };
        [
            ("text", self.text),
            ("sub", self.sub),
            ("dim", self.dim),
            ("accent_text", self.accent.text),
            ("success_text", self.success.text),
            ("warning_text", self.warning.text),
            ("danger_text", self.danger.text),
        ]
        .into_iter()
        .zip(FLOORS.into_iter().chain([hue; 4]))
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

/// The alphas of a scheme's tints: a control's resting and hovered fills, a
/// line, a press's ripple.
const SURFACE: f32 = 0.065;
const SURFACE2: f32 = 0.11;
const LINE: f32 = 0.08;
const RIPPLE: f32 = 0.16;

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

/// `ink` moved in lightness toward `toward`, black or white, its hue and
/// chroma kept, as little as makes it reach `floor` on all of `grounds`;
/// `toward` itself when nothing short of it does. Mixing toward black or
/// white would grey the hue: in Oklab the lightness moves alone, and the
/// hue looks the same all the way.
fn shade(ink: Color, toward: Color, grounds: [Color; 3], floor: f32) -> Color {
    if lowest(ink, grounds) >= floor {
        return ink;
    }
    let [from, a, b] = oklab(ink);
    let to = oklab(toward)[0];
    (1..=STEPS)
        .map(|step| from + (to - from) * step as f32 / STEPS as f32)
        .map(|lightness| in_gamut(lightness, a, b).with_alpha(ink.a))
        .find(|&shaded| lowest(shaded, grounds) >= floor)
        .unwrap_or(toward)
}

/// The colour at Oklab `lightness` with the hue of `a`, `b` and as much of
/// their chroma as sRGB can show there: near black or white, little.
fn in_gamut(lightness: f32, a: f32, b: f32) -> Color {
    let at = |share: f32| from_oklab([lightness, a * share, b * share]);
    if let (color, true) = at(1.0) {
        return color;
    }
    // The largest share of the chroma that fits, halving the interval.
    let (mut fits, mut too_much) = (0.0, 1.0);
    for _ in 0..16 {
        let mid = (fits + too_much) / 2.0;
        if at(mid).1 {
            fits = mid;
        } else {
            too_much = mid;
        }
    }
    at(fits).0
}

/// A colour in Oklab: lightness, 0 to 1, and two axes whose angle is the
/// hue and whose length is the chroma. Björn Ottosson's matrices.
fn oklab(color: Color) -> [f32; 3] {
    let [r, g, b] = [color.r, color.g, color.b].map(linear);
    let [l, m, s] = [
        0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b,
        0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b,
        0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b,
    ]
    .map(f32::cbrt);
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// The sRGB colour of an Oklab one, clamped, and whether sRGB holds it.
fn from_oklab([lightness, a, b]: [f32; 3]) -> (Color, bool) {
    let [l, m, s] = [
        lightness + 0.396_337_78 * a + 0.215_803_76 * b,
        lightness - 0.105_561_346 * a - 0.063_854_17 * b,
        lightness - 0.089_484_18 * a - 1.291_485_5 * b,
    ]
    .map(|c| c * c * c);
    let rgb = [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ];
    // Rounding leaves even a grey a hair outside.
    let holds = rgb.iter().all(|c| (-1e-4..=1.0 + 1e-4).contains(c));
    let [r, g, b] = rgb.map(|c| encoded(c.clamp(0.0, 1.0)));
    (Color::rgb(r, g, b), holds)
}

/// An sRGB channel as light, for WCAG and for Oklab.
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Light back into an sRGB channel.
fn encoded(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
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
    /// A theme's own colours are kept where they already read; a hue's
    /// text that doesn't is left to be worked out from its fill.
    pub fn readable(self) -> Self {
        self.readable_but(|_| false)
    }

    /// [`readable`](Self::readable), leaving the colours `kept` says by
    /// [`named`](Self::named) name as they are. A hue's text not kept
    /// follows its fill when the fill is kept: it is worked out again.
    pub(crate) fn readable_but(self, kept: impl Fn(&str) -> bool) -> Self {
        let grounds = self.grounds();
        let floors = [0, 1, 2].map(|i| {
            if kept(["text", "sub", "dim"][i]) {
                0.0
            } else {
                FLOORS[i]
            }
        });
        let [text, sub, dim] = lifted(
            [self.text, self.sub, self.dim],
            away(self.text, self.background),
            grounds,
            floors,
        );
        let mut scheme = Self {
            text,
            sub,
            dim,
            ..self
        };
        for hue in Hue::ALL {
            let fill = scheme.fill_mut(hue);
            let follows = !kept(&format!("{} text", hue.name()))
                && (kept(hue.name())
                    || fill
                        .text
                        .is_some_and(|t| lowest(t, grounds) < TEXT_CONTRAST));
            if follows {
                fill.text = None;
            }
        }
        scheme
    }

    /// The fill of `hue`.
    pub fn fill(&self, hue: Hue) -> &Fill {
        match hue {
            Hue::Accent => &self.accent,
            Hue::Success => &self.success,
            Hue::Warning => &self.warning,
            Hue::Danger => &self.danger,
        }
    }

    fn fill_mut(&mut self, hue: Hue) -> &mut Fill {
        match hue {
            Hue::Accent => &mut self.accent,
            Hue::Success => &mut self.success,
            Hue::Warning => &mut self.warning,
            Hue::Danger => &mut self.danger,
        }
    }

    /// The hue of `fill` as text on `grounds`: its own if set, else its
    /// colour shaded until it reads.
    fn text_of(&self, fill: Fill, grounds: [Color; 3]) -> Color {
        fill.text.unwrap_or_else(|| {
            shade(
                fill.color,
                away(self.text, self.background),
                grounds,
                TEXT_CONTRAST,
            )
        })
    }

    pub fn palette(&self) -> Palette {
        let grounds = self.grounds();
        let [accent, success, warning, danger] = Hue::ALL.map(|hue| {
            let fill = *self.fill(hue);
            HueColors {
                fill: fill.color,
                on: self.on(fill),
                text: self.text_of(fill, grounds),
            }
        });
        Palette {
            island: self.background,
            surface: self.tint(SURFACE),
            surface2: self.tint(SURFACE2),
            line: self.tint(LINE),
            text: self.text,
            sub: self.sub,
            dim: self.dim,
            accent,
            success,
            warning,
            danger,
            ripple: self.tint(RIPPLE),
        }
    }

    /// Surfaces, line and ripple at `alpha`: white over a dark background,
    /// black over a light one.
    fn tint(&self, alpha: f32) -> Color {
        if self.is_light() {
            Color::rgba(0.0, 0.0, 0.0, alpha * 0.8)
        } else {
            Color::rgba(1.0, 1.0, 1.0, alpha)
        }
    }

    /// What a text sits on in the palette: the island, and a control's
    /// fills over it.
    fn grounds(&self) -> [Color; 3] {
        let island = over(self.background, self.background);
        [
            island,
            over(self.tint(SURFACE), island),
            over(self.tint(SURFACE2), island),
        ]
    }

    /// Every colour by name; an on-colour or a hue's text may be unset.
    /// A hue's as `accent`, `on accent` and `accent text`.
    pub fn named(&self) -> [(&'static str, Option<Color>); 16] {
        let [a, s, w, d] = [self.accent, self.success, self.warning, self.danger];
        [
            ("background", Some(self.background)),
            ("text", Some(self.text)),
            ("sub", Some(self.sub)),
            ("dim", Some(self.dim)),
            ("accent", Some(a.color)),
            ("on accent", a.on),
            ("accent text", a.text),
            ("success", Some(s.color)),
            ("on success", s.on),
            ("success text", s.text),
            ("warning", Some(w.color)),
            ("on warning", w.on),
            ("warning text", w.text),
            ("danger", Some(d.color)),
            ("on danger", d.on),
            ("danger text", d.text),
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
            "on accent" => self.accent.on = color,
            "accent text" => self.accent.text = color,
            "success" => solid(&mut self.success.color),
            "on success" => self.success.on = color,
            "success text" => self.success.text = color,
            "warning" => solid(&mut self.warning.color),
            "on warning" => self.warning.on = color,
            "warning text" => self.warning.text = color,
            "danger" => solid(&mut self.danger.color),
            "on danger" => self.danger.on = color,
            "danger text" => self.danger.text = color,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::presets::THEMES;
    use crate::ui::{Material, Ui};

    #[test]
    fn a_hue_as_text_keeps_its_hue() {
        // Darker or lighter, not greyed toward black or white, on a solid
        // island and on glass: the angle between fill and text in Oklab
        // stays within a degree, and a third of the chroma at least is left.
        for theme in THEMES {
            for flavor in theme.flavors {
                for index in 0..flavor.accents.len() {
                    let solid = flavor.scheme(index).palette();
                    let glass = Ui::new(solid).with_material(Material::Glass).colors;
                    for p in [solid, glass] {
                        for HueColors { fill, text, .. } in Hue::ALL.map(|hue| *p.hue(hue)) {
                            let ([_, a, b], [_, ta, tb]) = (oklab(fill), oklab(text));
                            let turn = (a * tb - b * ta).atan2(a * ta + b * tb).to_degrees();
                            let kept = ta.hypot(tb) / a.hypot(b);
                            assert!(
                                turn.abs() < 1.0 && kept > 1.0 / 3.0,
                                "{} {} {}: {} turned {turn:.1}°, kept {kept:.2} of its chroma",
                                theme.name,
                                flavor.name,
                                flavor.accents[index].name,
                                to_hex(fill)
                            );
                        }
                    }
                }
            }
        }
    }
}
