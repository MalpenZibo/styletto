//! A surface's choices for its components: palette, density, shape, fonts,
//! material and motion. Set with [`provide_ui`], read with [`ui`].

use guido::prelude::*;

use crate::theme::palette::Palette;

/// How much room things take: type, controls and space move one step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Density {
    Compact,
    #[default]
    Default,
    Comfortable,
}

impl Density {
    pub(crate) fn shift(self) -> i8 {
        match self {
            Density::Compact => -1,
            Density::Default => 0,
            Density::Comfortable => 1,
        }
    }
}

/// How round corners are: every radius moves one step; pills stay pills.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Shape {
    Square,
    #[default]
    Soft,
    Round,
}

impl Shape {
    pub(crate) fn shift(self) -> i8 {
        match self {
            Shape::Square => -1,
            Shape::Soft => 0,
            Shape::Round => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fonts {
    pub sans: FontFamily,
    pub mono: FontFamily,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            sans: FontFamily::SansSerif,
            mono: FontFamily::Monospace,
        }
    }
}

/// What the island is made of: its fill's opacity and the blur behind it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Material {
    #[default]
    Solid,
    Translucent,
    Glass,
}

impl Material {
    pub fn opacity(self) -> f32 {
        match self {
            Material::Solid => 1.0,
            Material::Translucent => 0.86,
            Material::Glass => 0.6,
        }
    }

    /// The compositor's blur behind it.
    pub fn blur(self) -> f32 {
        match self {
            Material::Solid => 0.0,
            Material::Translucent => 24.0,
            Material::Glass => 48.0,
        }
    }
}

/// How much things move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Motion {
    #[default]
    Full,
    /// Fades only: no spring, no growth, no shake.
    Reduced,
    /// Everything changes at once.
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ui {
    /// The palette, with the island's alpha set by the material.
    pub colors: Palette,
    palette: Palette,
    pub density: Density,
    pub shape: Shape,
    pub fonts: Fonts,
    pub material: Material,
    pub motion: Motion,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new(Palette::default())
    }
}

impl Ui {
    pub fn new(palette: Palette) -> Self {
        Self {
            colors: palette,
            palette,
            density: Density::default(),
            shape: Shape::default(),
            fonts: Fonts::default(),
            material: Material::default(),
            motion: Motion::default(),
        }
        .settled()
    }

    pub fn with_density(self, density: Density) -> Self {
        Self { density, ..self }
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        Self { shape, ..self }
    }

    pub fn with_fonts(self, fonts: Fonts) -> Self {
        Self { fonts, ..self }
    }

    pub fn with_material(self, material: Material) -> Self {
        Self { material, ..self }.settled()
    }

    pub fn with_motion(self, motion: Motion) -> Self {
        Self { motion, ..self }
    }

    fn settled(mut self) -> Self {
        let island = self.palette.island;
        self.colors = Palette {
            island: island.with_alpha(island.a * self.material.opacity()),
            ..self.palette
        };
        self
    }
}

pub fn provide_ui(ui: Ui) {
    provide_context(ui);
}

/// The nearest [`Ui`], or the default. Read while building, not in event
/// handlers.
pub fn ui() -> Ui {
    use_context::<Ui>().unwrap_or_default()
}

/// Reads one field of the nearest [`Ui`] without cloning it.
pub(crate) fn with_ui<R>(f: impl Fn(&Ui) -> R) -> R {
    with_context(&f).unwrap_or_else(|| f(&Ui::default()))
}

/// The accent while `on`, the surface while off.
pub fn on_fill(on: Signal<bool>) -> impl Fn() -> Color + Copy + 'static {
    let t = ui();
    move || {
        if on.get() {
            t.colors.accent
        } else {
            t.colors.surface
        }
    }
}

/// The ink that reads on [`on_fill`].
pub fn on_ink(on: Signal<bool>) -> impl Fn() -> Color + Copy + 'static {
    let t = ui();
    move || {
        if on.get() {
            t.colors.on_accent
        } else {
            t.colors.text
        }
    }
}

/// Asks `on_toggle` for the opposite of `on`.
pub(crate) fn flip(
    on: Signal<bool>,
    on_toggle: Option<Callback<(bool,)>>,
) -> impl Fn() + Copy + 'static {
    move || {
        if let Some(on_toggle) = on_toggle {
            on_toggle.run(!on.get_untracked());
        }
    }
}

/// Tabular figures, so a number keeps its width.
pub fn tabular() -> FontFeatures {
    FontFeatures::new().enable(b"tnum")
}
