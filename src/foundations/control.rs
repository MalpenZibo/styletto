//! Control sizes. A control is its icon box with equal space above and
//! below; density may move it a size up or down.

use crate::foundations::tokens::{IconSize, Space, icon, size};
use crate::foundations::typography::{LABEL_LARGE, LABEL_MEDIUM, LABEL_SMALL, LABEL_XSMALL, Type};
use crate::ui::with_ui;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// A chip in the island at rest, a switch.
    Xxs,
    /// Inside a row, beside its text.
    Xs,
    #[default]
    Sm,
    Md,
    /// A field to type in, a tile's icon.
    Lg,
    Xl,
}

const ALL: [Size; 6] = [Size::Xxs, Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl];

impl Size {
    /// The size after the density shift.
    fn drawn(self) -> Size {
        ALL[(self as i8 + with_ui(|u| u.density.shift())).clamp(0, ALL.len() as i8 - 1) as usize]
    }

    pub fn height(self) -> f32 {
        match self.drawn() {
            Size::Xxs => 20.0,
            Size::Xs => 24.0,
            Size::Sm => 28.0,
            Size::Md => 36.0,
            Size::Lg => 40.0,
            Size::Xl => 48.0,
        }
    }

    pub fn icon(self) -> f32 {
        let step: IconSize = match self.drawn() {
            Size::Xxs | Size::Xs => icon::XSMALL,
            Size::Sm => icon::SMALL,
            Size::Md => icon::MEDIUM,
            Size::Lg | Size::Xl => icon::LARGE,
        };
        step.base()
    }

    /// Padding from a pill's ends to its content.
    pub fn inline(self) -> f32 {
        let step: Space = match self.drawn() {
            Size::Xxs | Size::Xs => size::S2,
            Size::Sm => size::S3,
            Size::Md | Size::Lg | Size::Xl => size::S4,
        };
        step.base()
    }

    /// Between an icon and its label.
    pub fn gap(self) -> f32 {
        let step = match self.drawn() {
            Size::Xxs | Size::Xs | Size::Sm => size::S1,
            Size::Md | Size::Lg | Size::Xl => size::S2,
        };
        step.base()
    }

    /// Already shifted for density.
    pub fn label(self) -> Type {
        match self.drawn() {
            Size::Xxs => LABEL_XSMALL,
            Size::Xs | Size::Sm => LABEL_SMALL,
            Size::Md | Size::Lg => LABEL_MEDIUM,
            Size::Xl => LABEL_LARGE,
        }
        .fixed()
    }
}
