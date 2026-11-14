//! The type scale: six categories in four sizes, regular or emphasised.
//! Sizes are even and line heights multiples of 4; density may shift a size
//! within its category.

use guido::prelude::*;

use crate::ui::{tabular, with_ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    /// Big numbers: a clock, a level, a figure a card is about.
    Display,
    /// A page's or a group's title.
    Title,
    /// One line of a control: a button, a row, a pill.
    Label,
    /// Running text, over more than one line.
    Body,
    /// Uppercase and tracked: section names. Never running text.
    Micro,
    Mono,
}

/// Size and line height, from the largest to the smallest.
const fn sizes(category: Category) -> [(f32, f32); 4] {
    match category {
        Category::Display => [(48.0, 48.0), (40.0, 40.0), (32.0, 32.0), (24.0, 24.0)],
        Category::Title => [(24.0, 28.0), (20.0, 24.0), (16.0, 20.0), (14.0, 20.0)],
        Category::Body => [(16.0, 24.0), (14.0, 20.0), (12.0, 16.0), (10.0, 16.0)],
        Category::Label | Category::Micro | Category::Mono => {
            [(16.0, 20.0), (14.0, 16.0), (12.0, 16.0), (10.0, 12.0)]
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Type {
    pub category: Category,
    /// 0 is the largest size, 3 the smallest.
    step: u8,
    em: bool,
    tabular: bool,
    /// Already shifted for density, with its control.
    fixed: bool,
}

/// A [`Type`] resolved where it is built, or a style off the scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolved {
    pub size: f32,
    pub line: f32,
    pub weight: FontWeight,
    /// Letter spacing, in pixels.
    pub tracking: f32,
    pub family: FontFamily,
    pub tabular: bool,
}

impl From<Type> for Resolved {
    fn from(kind: Type) -> Self {
        kind.resolve()
    }
}

impl Type {
    const fn of(category: Category, step: u8) -> Self {
        Self {
            category,
            step,
            em: false,
            tabular: false,
            fixed: false,
        }
    }

    pub(crate) const fn fixed(self) -> Self {
        Self {
            fixed: true,
            ..self
        }
    }

    /// The emphasised weight.
    pub const fn em(self) -> Self {
        Self { em: true, ..self }
    }

    /// Tabular figures.
    pub const fn tabular(self) -> Self {
        Self {
            tabular: true,
            ..self
        }
    }

    pub fn resolve(self) -> Resolved {
        let (density, fonts) = with_ui(|u| (u.density.shift(), u.fonts));
        let shift = if self.fixed { 0 } else { density };
        let index = (self.step as i8 - shift).clamp(0, 3) as usize;
        let (size, line) = sizes(self.category)[index];
        let tracking = if self.category == Category::Micro {
            0.06 * size
        } else {
            0.0
        };
        Resolved {
            size,
            line,
            weight: if self.em {
                FontWeight::SEMI_BOLD
            } else {
                FontWeight::NORMAL
            },
            tracking,
            family: if self.category == Category::Mono {
                fonts.mono
            } else {
                fonts.sans
            },
            tabular: self.tabular,
        }
    }

    /// The font size alone.
    pub fn size(self) -> f32 {
        self.resolve().size
    }

    /// The case the category asks for.
    pub fn cased(self, content: &str) -> String {
        if self.category == Category::Micro {
            content.to_uppercase()
        } else {
            content.to_string()
        }
    }
}

pub const DISPLAY_LARGE: Type = Type::of(Category::Display, 0);
pub const DISPLAY_MEDIUM: Type = Type::of(Category::Display, 1);
pub const DISPLAY_SMALL: Type = Type::of(Category::Display, 2);
pub const DISPLAY_XSMALL: Type = Type::of(Category::Display, 3);

pub const TITLE_LARGE: Type = Type::of(Category::Title, 0);
pub const TITLE_MEDIUM: Type = Type::of(Category::Title, 1);
pub const TITLE_SMALL: Type = Type::of(Category::Title, 2);
pub const TITLE_XSMALL: Type = Type::of(Category::Title, 3);

pub const LABEL_LARGE: Type = Type::of(Category::Label, 0);
pub const LABEL_MEDIUM: Type = Type::of(Category::Label, 1);
pub const LABEL_SMALL: Type = Type::of(Category::Label, 2);
pub const LABEL_XSMALL: Type = Type::of(Category::Label, 3);

pub const BODY_LARGE: Type = Type::of(Category::Body, 0);
pub const BODY_MEDIUM: Type = Type::of(Category::Body, 1);
pub const BODY_SMALL: Type = Type::of(Category::Body, 2);
pub const BODY_XSMALL: Type = Type::of(Category::Body, 3);

pub const MICRO_LARGE: Type = Type::of(Category::Micro, 0);
pub const MICRO_MEDIUM: Type = Type::of(Category::Micro, 1);
pub const MICRO_SMALL: Type = Type::of(Category::Micro, 2);
pub const MICRO_XSMALL: Type = Type::of(Category::Micro, 3);

pub const MONO_LARGE: Type = Type::of(Category::Mono, 0);
pub const MONO_MEDIUM: Type = Type::of(Category::Mono, 1);
pub const MONO_SMALL: Type = Type::of(Category::Mono, 2);
pub const MONO_XSMALL: Type = Type::of(Category::Mono, 3);

/// Applies a [`Type`], or any [`Resolved`] style, to a widget that draws
/// glyphs. Casing is the content's: see [`Type::cased`].
pub trait Typeset {
    fn typeset(self, kind: impl Into<Resolved>) -> Self;
}

macro_rules! typeset {
    ($($widget:ty),*) => {$(
        impl Typeset for $widget {
            fn typeset(self, kind: impl Into<Resolved>) -> Self {
                let kind = kind.into();
                let styled = self
                    .font_family(kind.family)
                    .font_size(kind.size)
                    .line_height(LineHeight::Absolute(kind.line))
                    .font_weight(kind.weight)
                    .letter_spacing(kind.tracking);
                if kind.tabular {
                    styled.font_features(tabular())
                } else {
                    styled
                }
            }
        }
    )*};
}

typeset!(Text, TextInput, PasswordInput);
