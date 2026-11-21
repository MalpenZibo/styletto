//! The island: one shape that springs around whatever view it shows. A
//! primitive, so the app adds where it sits and what it answers to, and can
//! merge it with what splits off it.
//!
//! Its look comes in pieces, for the shapes that should match it: [`paint`]
//! fills and blurs, [`border`] rims, [`shadow`] is what it casts, and
//! [`edge`] is the border and the shadow together. Merging an island with
//! what comes out of it, the holder takes the edge
//! (`edge(holder.merge_children(reach))`), the shapes coming out take the
//! paint, and the island leaves its shadow to the holder
//! (`.shadow(Shadows::none())`): cast by the island, it would show through
//! the translucent shapes beside it.

use std::hash::Hash;

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::motion::{self, arriving};
use crate::foundations::tokens::{self, stroke};
use crate::theme::palette;
use crate::ui::ui;

const BORDER: f32 = stroke::HAIRLINE;

/// The height a view is given so the island holding it is `size` tall, its
/// border included.
pub fn inner(size: Size) -> f32 {
    size.height() - 2.0 * BORDER
}

/// The island showing `content` of `view`, with `corners` for it. Each view
/// keeps its own size and the two cross-fade while the shape springs between
/// them: growing, it widens and then drops; shrinking, both go at once. It
/// is [`paint`]ed, with an [`edge`].
pub fn island<K, W, M1, M2>(
    view: impl IntoSignal<K, M1>,
    content: impl Fn(K) -> W + 'static,
    corners: impl IntoSignal<Corners, M2>,
) -> Container
where
    K: Clone + PartialEq + Eq + Hash + 'static,
    W: Widget + 'static,
{
    let (view, corners) = (view.into_signal(), corners.into_signal());
    // Whether the next view is taller only the layout knows, so the height
    // chooses from the change itself.
    let (grow, shrink) = (
        motion::spring().delay(motion::HEIGHT_LAG_MS),
        motion::spring(),
    );
    edge(paint(container()))
        .width(at_least(0.0).transition(motion::spring()))
        .height(at_least(0.0).transition(move |s: Segment<f32>| {
            if s.to > s.from {
                grow.clone()
            } else {
                shrink.clone()
            }
        }))
        .corners((move || corners.get()).transition(motion::spring()))
        .overflow(Overflow::Hidden)
        .layout(ZStack::new().align(Alignment::TOP_CENTER))
        .children(keyed(
            move || [view.get()],
            |view| view.clone(),
            move |view| arriving(content(view)).width(natural()).height(natural()),
        ))
}

/// `shape` filled like the island, and blurring what is behind it as the
/// material asks. With no blur to ask for, the compositor is still told the
/// shape is there: a compositor rule that blurs a surface declaring no blur
/// (niri's `layer-rule` `blur`) blurs all of it, empty space included.
pub fn paint(shape: Container) -> Container {
    let t = ui();
    let radius = t.material.blur();
    let sources = if radius > 0.0 {
        BackdropSources::all()
    } else {
        BackdropSources::COMPOSITOR
    };
    shape
        .background(t.colors.island)
        .backdrop_blur(BackdropBlur::new(radius).sources(sources))
}

/// `shape` rimmed like the island.
pub fn border(shape: Container) -> Container {
    shape.border(BORDER, ui().colors.line)
}

/// `shape` with the island's [`border`] and its [`shadow`], without a glow:
/// for an island alone, or the holder an island merges in.
pub fn edge(shape: Container) -> Container {
    border(shape).shadow(shadow(Color::TRANSPARENT))
}

/// The shadow an island casts, with a ring and a halo of `glow` for a state
/// worth seeing.
pub fn shadow(glow: Color) -> Shadows {
    let drop = palette::SHADE.with_alpha(tokens::shadow::ALPHA);
    let halo = glow.scale_alpha(tokens::opacity::HALO);
    Shadows::new([
        Shadow::new((0.0, tokens::shadow::DROP), tokens::shadow::BLUR, 0.0, drop),
        Shadow::new((0.0, 0.0), 0.0, stroke::HAIRLINE, glow),
        Shadow::new(
            (0.0, 0.0),
            tokens::glow::LARGE,
            -tokens::shadow::HALO_INSET,
            halo,
        ),
    ])
}

/// How far up an island `top` from the edge and `height` tall goes to leave
/// the screen, its shadow with it.
pub fn above_edge(top: f32, height: f32) -> Translate {
    let reach = shadow(Color::WHITE).outsets().bottom;

    Translate::new(0.0, -(top + height + reach))
}
