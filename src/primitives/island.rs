//! The island: one shape that springs around whatever view it shows. A
//! primitive, so the app adds where it sits and what it answers to, and can
//! merge it with what splits off it.

use std::hash::Hash;

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::motion::{self, arriving};
use crate::foundations::tokens::stroke;
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
/// blurs as the material asks and casts [`shadow`] without a glow.
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
    let t = ui();
    // Whether the next view is taller only the layout knows, so the height
    // chooses from the change itself.
    let (grow, shrink) = (
        motion::spring().delay(motion::HEIGHT_LAG_MS),
        motion::spring(),
    );
    container()
        .width(at_least(0.0).transition(motion::spring()))
        .height(at_least(0.0).transition(move |s: Segment<f32>| {
            if s.to > s.from {
                grow.clone()
            } else {
                shrink.clone()
            }
        }))
        .corners((move || corners.get()).transition(motion::spring()))
        .background(t.colors.island)
        .border(BORDER, t.colors.line)
        .backdrop_blur(BackdropBlur::new(t.material.blur()))
        .shadow(shadow(Color::TRANSPARENT))
        .overflow(Overflow::Hidden)
        .layout(ZStack::new().align(Alignment::TOP_CENTER))
        .children(keyed(
            move || [view.get()],
            |view| view.clone(),
            move |view| arriving(content(view)).width(natural()).height(natural()),
        ))
}

/// The shadow an island casts, with a ring and a halo of `glow` for a state
/// worth seeing.
pub fn shadow(glow: Color) -> Shadows {
    Shadows::new([
        Shadow::new((0.0, 4.0), 12.0, 0.0, Color::BLACK.with_alpha(0.3)),
        Shadow::new((0.0, 0.0), 0.0, stroke::HAIRLINE, glow),
        Shadow::new((0.0, 0.0), 16.0, -4.0, glow.with_alpha(glow.a * 0.5)),
    ])
}

/// How far up an island `top` from the edge and `height` tall goes to leave
/// the screen, its shadow with it.
pub fn above_edge(top: f32, height: f32) -> Translate {
    let reach = shadow(Color::WHITE).outsets().bottom;

    Translate::new(0.0, -(top + height + reach))
}
