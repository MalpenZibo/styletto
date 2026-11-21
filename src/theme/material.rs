//! Themes worked out from one colour, as Material You does it: a colour
//! given, or the one that fits a wallpaper best. Its roles fill a
//! [`Scheme`]; success and warning, which it has none of, are a green and an
//! amber turned toward that colour. The generation needs the `material`
//! feature, reading a wallpaper the `wallpaper` one.

#[cfg(feature = "wallpaper")]
use std::path::Path;

#[cfg(feature = "material")]
use guido::prelude::Color;
#[cfg(feature = "material")]
use material_colors::blend::harmonize;
#[cfg(feature = "material")]
use material_colors::color::Rgb;
#[cfg(feature = "material")]
use material_colors::dynamic_color::DynamicScheme;
#[cfg(feature = "material")]
use material_colors::dynamic_color::variant::Variant as Mcu;
#[cfg(feature = "material")]
use material_colors::hct::Hct;
#[cfg(feature = "material")]
use material_colors::palette::TonalPalette;
#[cfg(feature = "wallpaper")]
use material_colors::quantize::{Quantizer, QuantizerCelebi};
#[cfg(feature = "wallpaper")]
use material_colors::score::Score;

#[cfg(feature = "material")]
use crate::theme::palette::{Scheme, SchemeHue};

/// How the scheme spreads the seed over its roles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "kebab-case")
)]
pub enum Variant {
    #[default]
    TonalSpot,
    Vibrant,
    Expressive,
    Content,
    Fidelity,
    Neutral,
    Monochrome,
    Rainbow,
    FruitSalad,
}

/// How far apart the tones of a pair are pushed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Contrast {
    #[default]
    Standard,
    Medium,
    High,
}

#[cfg(feature = "material")]
impl Contrast {
    fn level(self) -> f64 {
        match self {
            Contrast::Standard => 0.0,
            Contrast::Medium => 0.5,
            Contrast::High => 1.0,
        }
    }
}

#[cfg(feature = "material")]
const SUCCESS: u32 = 0x4caf50;
#[cfg(feature = "material")]
const WARNING: u32 = 0xffb300;

/// The dark scheme, then the light one.
#[cfg(feature = "material")]
pub fn schemes(seed: Color, variant: Variant, contrast: Contrast) -> [Scheme; 2] {
    [false, true].map(|light| scheme(rgb(seed), variant, contrast, light))
}

#[cfg(feature = "material")]
fn scheme(seed: Rgb, variant: Variant, contrast: Contrast, light: bool) -> Scheme {
    let m = DynamicScheme::by_variant(seed, &mcu(variant), !light, Some(contrast.level()));
    let paired = |color: Rgb, on: Rgb| SchemeHue {
        color: color_of(color),
        on: Some(color_of(on)),
        text: None,
    };
    // A hue's colour and its ink, at the tones the scheme gives its primary.
    let custom = |hex: u32| {
        let hct: Hct = harmonize(rgb(Color::from_hex(hex)), seed).into();
        let tones = TonalPalette::of(hct.get_hue(), hct.get_chroma().max(48.0));
        let (tone, on) = if light { (40, 100) } else { (80, 20) };
        paired(tones.tone(tone), tones.tone(on))
    };
    Scheme {
        background: color_of(m.surface_container()),
        text: color_of(m.on_surface()),
        sub: color_of(m.on_surface_variant()),
        dim: color_of(m.outline()),
        accent: paired(m.primary(), m.on_primary()),
        success: custom(SUCCESS),
        warning: custom(WARNING),
        danger: paired(m.error(), m.on_error()),
    }
    .readable()
}

/// The colours that would make a theme of the image at `path`, best first.
#[cfg(feature = "wallpaper")]
pub fn seeds(path: &Path) -> Result<Vec<Color>, String> {
    let image = image::open(path).map_err(|err| err.to_string())?;
    // A thumbnail holds the same colours in a fraction of the pixels.
    // Transparent pixels show no colour of the image.
    let pixels: Vec<Rgb> = image
        .thumbnail(128, 128)
        .to_rgba8()
        .pixels()
        .filter(|p| p[3] == 255)
        .map(|p| Rgb::new(p[0], p[1], p[2]))
        .collect();
    let quantized = QuantizerCelebi::quantize(&pixels, 128);
    Ok(Score::score(&quantized.color_to_count, Some(4), None, None)
        .into_iter()
        .map(color_of)
        .collect())
}

#[cfg(feature = "material")]
fn mcu(variant: Variant) -> Mcu {
    match variant {
        Variant::TonalSpot => Mcu::TonalSpot,
        Variant::Vibrant => Mcu::Vibrant,
        Variant::Expressive => Mcu::Expressive,
        Variant::Content => Mcu::Content,
        Variant::Fidelity => Mcu::Fidelity,
        Variant::Neutral => Mcu::Neutral,
        Variant::Monochrome => Mcu::Monochrome,
        Variant::Rainbow => Mcu::Rainbow,
        Variant::FruitSalad => Mcu::FruitSalad,
    }
}

#[cfg(feature = "material")]
fn rgb(color: Color) -> Rgb {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    Rgb::new(byte(color.r), byte(color.g), byte(color.b))
}

#[cfg(feature = "material")]
fn color_of(rgb: Rgb) -> Color {
    Color::from_hex(rgb.as_u32())
}

#[cfg(all(test, feature = "material"))]
mod tests {
    use super::*;
    use crate::theme::palette::{BODY_CONTRAST, Hue, HueColors, TEXT_CONTRAST};

    const VARIANTS: [Variant; 9] = [
        Variant::TonalSpot,
        Variant::Vibrant,
        Variant::Expressive,
        Variant::Content,
        Variant::Fidelity,
        Variant::Neutral,
        Variant::Monochrome,
        Variant::Rainbow,
        Variant::FruitSalad,
    ];

    #[test]
    fn every_variant_reads() {
        for variant in VARIANTS {
            for contrast in [Contrast::Standard, Contrast::Medium, Contrast::High] {
                for scheme in schemes(Color::from_hex(0x6750a4), variant, contrast) {
                    let palette = scheme.palette();
                    for hue in Hue::ALL {
                        let HueColors { fill, on, .. } = palette.hue(hue);
                        let ratio = crate::theme::palette::contrast(fill, on);
                        assert!(
                            ratio >= TEXT_CONTRAST,
                            "{variant:?} {contrast:?} {hue:?}: {ratio:.2}"
                        );
                    }
                    assert_eq!(
                        palette.unreadable(palette.island),
                        None,
                        "{variant:?} {contrast:?}"
                    );
                    let body = crate::theme::palette::contrast(palette.text, palette.island);
                    assert!(
                        body >= BODY_CONTRAST,
                        "{variant:?} {contrast:?}: text at {body:.2}"
                    );
                }
            }
        }
    }

    #[cfg(feature = "wallpaper")]
    #[test]
    fn an_image_gives_its_colours() {
        let path = std::env::temp_dir().join("styletto-seeds.png");
        image::RgbaImage::from_fn(64, 64, |x, _| {
            if x < 40 {
                image::Rgba([30, 90, 200, 255])
            } else {
                image::Rgba([220, 120, 40, 255])
            }
        })
        .save(&path)
        .unwrap();
        let found = seeds(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert!(!found.is_empty());
        assert!(found[0].b > found[0].r, "the larger blue should come first");
        assert!(seeds(Path::new("/nowhere.png")).is_err());
    }
}
