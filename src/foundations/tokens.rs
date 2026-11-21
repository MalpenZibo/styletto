//! The scales every component is built from. [`px`](Space::px) reads a
//! step in the current [`Ui`](crate::ui::Ui), shifted for density or shape.
//! Strokes, opacities and glows are fixed.

use guido::prelude::Corners;

use crate::foundations::typography::{
    DISPLAY_SMALL, LABEL_LARGE, LABEL_SMALL, LABEL_XSMALL, TITLE_LARGE, TITLE_MEDIUM, Type,
};
use crate::ui::with_ui;

/// Moves `index` by `shift` along a scale `len` long, never past its ends.
fn step(index: u8, shift: i8, low: u8, len: usize) -> usize {
    (index as i16 + shift as i16).clamp(low as i16, len as i16 - 1) as usize
}

/// A step of space or length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Space(u8);

const SPACE: [f32; 16] = [
    0.0, 2.0, 4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 40.0, 60.0, 80.0, 120.0, 160.0, 240.0, 320.0, 400.0,
];

impl Space {
    /// The length, shifted for density up to [`size::S8`]; larger steps are
    /// layout lengths and stay. `S0` never shifts and no gap closes.
    pub fn px(self) -> f32 {
        if self.0 == 0 || self.0 > size::S8.0 {
            return self.base();
        }
        SPACE[step(self.0, with_ui(|u| u.density.shift()), 1, SPACE.len())]
    }

    /// The unshifted length, for values shifted already.
    pub(crate) fn base(self) -> f32 {
        SPACE[self.0 as usize]
    }
}

/// Space and length on a 4 px grid, plus a 2 px half step. Up to `S8` it
/// shifts with density; past it, it is a fixed layout length.
pub mod size {
    use super::Space;

    pub const S0: Space = Space(0);
    pub const S0_5: Space = Space(1);
    pub const S1: Space = Space(2);
    pub const S2: Space = Space(3);
    pub const S3: Space = Space(4);
    pub const S4: Space = Space(5);
    pub const S6: Space = Space(6);
    pub const S8: Space = Space(7);
    pub const S10: Space = Space(8);
    pub const S15: Space = Space(9);
    pub const S20: Space = Space(10);
    pub const S30: Space = Space(11);
    pub const S40: Space = Space(12);
    pub const S60: Space = Space(13);
    pub const S80: Space = Space(14);
    pub const S100: Space = Space(15);

    /// A length the app chose for its layout, such as a panel's width or a
    /// list's height before it scrolls, grown or shrunk with the density
    /// ([`Density::scale`](crate::ui::Density::scale)) as what it holds is.
    /// `scaled(392.0)` is 343 compact, 392 by default and 490 comfortable.
    pub fn scaled(length: f32) -> f32 {
        length * crate::ui::with_ui(|u| u.density.scale())
    }
}

/// A step of corner radius.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Radius(u8);

const RADIUS: [f32; 6] = [0.0, 4.0, 8.0, 16.0, 32.0, 64.0];

impl Radius {
    /// The radius, shifted for the shape.
    pub fn px(self) -> f32 {
        RADIUS[step(self.0, with_ui(|u| u.shape.shift()), 0, RADIUS.len())]
    }

    /// A box's corners at this radius.
    pub fn corners(self) -> Corners {
        radius::shape(self.px())
    }
}

/// Corner radii, doubling at each step. Boxes are squircles; pills and
/// circles are round. A box around another gets its radius from
/// [`around`](radius::around), so the curves stay concentric.
pub mod radius {
    use guido::prelude::Corners;
    use guido::widgets::CornerRadii;

    use super::Radius;

    /// Squircle corners at any radius, such as one from [`around`].
    pub fn shape(radii: impl Into<CornerRadii>) -> Corners {
        Corners::squircle(radii)
    }

    /// Round ends for a pill or circle `height` tall.
    pub fn pill(height: f32) -> Corners {
        Corners::rounded(height / 2.0)
    }

    /// The radius of a box holding one of radius `inner`, `padding` in.
    pub fn around(inner: f32, padding: f32) -> f32 {
        inner + padding
    }

    /// The radius of a box held `padding` inside one of radius `outer`.
    pub fn within(outer: f32, padding: f32) -> f32 {
        (outer - padding).max(0.0)
    }

    pub const NONE: Radius = Radius(0);
    pub const XSMALL: Radius = Radius(1);
    pub const SMALL: Radius = Radius(2);
    pub const MEDIUM: Radius = Radius(3);
    pub const LARGE: Radius = Radius(4);
    pub const XLARGE: Radius = Radius(5);
}

/// A step of the box an icon is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconSize(u8);

const ICON: [f32; 5] = [12.0, 16.0, 20.0, 24.0, 32.0];

impl IconSize {
    pub fn px(self) -> f32 {
        ICON[self.drawn()]
    }

    /// The stroke of a ring or spinner in this box.
    pub fn stroke(self) -> f32 {
        if self.drawn() < 3 {
            stroke::BOLD
        } else {
            stroke::HEAVY
        }
    }

    fn drawn(self) -> usize {
        step(self.0, with_ui(|u| u.density.shift()), 0, ICON.len())
    }

    pub(crate) fn base(self) -> f32 {
        ICON[self.0 as usize]
    }
}

/// The box an icon is drawn in.
pub mod icon {
    use super::IconSize;

    pub const XSMALL: IconSize = IconSize(0);
    pub const SMALL: IconSize = IconSize(1);
    pub const MEDIUM: IconSize = IconSize(2);
    pub const LARGE: IconSize = IconSize(3);
    pub const XLARGE: IconSize = IconSize(4);
}

/// A step of a square tile showing a picture: an app icon, a cover, an
/// initial. Corners, inner icon and letter follow from the step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileSize(u8);

const TILE: [f32; 6] = [16.0, 24.0, 32.0, 40.0, 48.0, 64.0];

impl TileSize {
    pub fn px(self) -> f32 {
        TILE[self.drawn()]
    }

    pub fn radius(self) -> Radius {
        match self.drawn() {
            0 => radius::XSMALL,
            1 | 2 => radius::SMALL,
            _ => radius::MEDIUM,
        }
    }

    /// The icon drawn on it, about half its side.
    pub fn icon(self) -> f32 {
        let step = match self.drawn() {
            0 | 1 => icon::XSMALL,
            2 => icon::SMALL,
            3 => icon::MEDIUM,
            4 => icon::LARGE,
            _ => icon::XLARGE,
        };
        step.base()
    }

    /// The initial's letter, already shifted for density.
    pub fn letter(self) -> Type {
        match self.drawn() {
            0 => LABEL_XSMALL,
            1 => LABEL_SMALL,
            2 => LABEL_LARGE,
            3 => TITLE_MEDIUM,
            4 => TITLE_LARGE,
            _ => DISPLAY_SMALL,
        }
        .em()
        .fixed()
    }

    fn drawn(self) -> usize {
        step(self.0, with_ui(|u| u.density.shift()), 0, TILE.len())
    }
}

/// A square tile showing a picture.
pub mod tile {
    use super::TileSize;

    pub const XSMALL: TileSize = TileSize(0);
    pub const SMALL: TileSize = TileSize(1);
    pub const MEDIUM: TileSize = TileSize(2);
    pub const LARGE: TileSize = TileSize(3);
    pub const XLARGE: TileSize = TileSize(4);
    pub const XXLARGE: TileSize = TileSize(5);
}

/// Lines drawn around or between things.
pub mod stroke {
    /// A separator, a border.
    pub const HAIRLINE: f32 = 1.0;
    /// The ring inside a field or a pill.
    pub const RING: f32 = 1.5;
    /// A mark meant to be seen: a dot's ring, the urgent bar.
    pub const BOLD: f32 = 2.0;
    /// A ring or a spinner in a large icon box.
    pub const HEAVY: f32 = 3.0;
}

/// The alpha of a whole control or of a stroke. A hue's wash over the
/// island is a [`tint`] step instead.
pub mod opacity {
    /// A disabled control.
    pub const DISABLED: f32 = 0.4;
    /// A field's ring in the accent, softened so it frames what is typed.
    pub const RING: f32 = 0.7;
}

/// How far a glow spreads around what it lights.
pub mod glow {
    use guido::prelude::{Color, Shadow};

    use super::tint;

    pub const SMALL: f32 = 4.0;
    pub const MEDIUM: f32 = 8.0;

    /// A glow `spread` wide around what lights up in a hue: its fill at
    /// [`tint::MEDIUM`], nothing when there is none. A tag's and a card's.
    pub(crate) fn halo(fill: Option<Color>, spread: f32) -> Shadow {
        let color = fill.map_or(Color::TRANSPARENT, |fill| fill.with_alpha(tint::MEDIUM));
        Shadow::new((0.0, 0.0), spread, 0.0, color)
    }
}

/// How strongly a hue tints what it fills or lights: the alpha its fill is
/// laid over the island with. What sits on a tint is drawn in the hue as
/// text.
pub mod tint {
    /// A card's wash while it warns or does well: faint, as a card is large.
    pub const SUBTLE: f32 = 0.12;
    /// A card's wash while it alerts, and a glow around a mark or a card.
    pub const MEDIUM: f32 = 0.15;
    /// Behind a small mark's ink: a tag, an alert's tile.
    pub const STRONG: f32 = 0.2;
}

/// A scrollbar's measures. It shows where a list is rather than taking room
/// in it, so they stay the same in every density.
pub mod scrollbar {
    /// Its width at rest.
    pub const WIDTH: f32 = 4.0;
    /// Its width under the pointer.
    pub const HOVER_WIDTH: f32 = 8.0;
    /// The gap between it and the list's edge.
    pub const MARGIN: f32 = 2.0;
    /// How short the handle may get, so it stays grabbable.
    pub const MIN_HANDLE: f32 = 24.0;
}
