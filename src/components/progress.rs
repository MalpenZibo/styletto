//! Linear progress bars and progress rings.

use guido::prelude::*;

use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{IconSize, icon, size as space};
use crate::ui::look;

/// A thin bar.
#[component]
pub fn progress_bar(
    /// From 0 to 1.
    #[prop(required)]
    value: f32,
    color: Color,
) -> impl Widget {
    container()
        .width(fill())
        .height(space::S1.px())
        .corners(radius::pill(space::S1.px()))
        .background(look().eased(|c| c.surface2))
        .overflow(Overflow::Hidden)
        .child(
            container()
                .width((move || fraction(value.get().clamp(0.0, 1.0))).transition(motion::hover()))
                .height(fill())
                .corners(radius::pill(space::S1.px()))
                .background(motion::eased_color(color)),
        )
}

/// A ring filling clockwise from twelve, sized and stroked by an icon step.
#[component]
pub fn ring(
    /// From 0 to 1.
    #[prop(required)]
    value: f32,
    color: Color,
    #[prop(default = icon::MEDIUM)] size: IconSize,
) -> impl Widget {
    let size = size.get_untracked();
    ring_arc(value, color, size.drawn_step())
}

/// A ring in the box of `step`, a step already shifted for density. A
/// value that is not a number draws it empty.
pub(crate) fn ring_arc<M1, M2>(
    value: impl IntoSignal<f32, M1>,
    color: impl IntoSignal<Color, M2>,
    step: IconSize,
) -> guido::widgets::Arc {
    let value = value.into_signal();
    let fill = move || filled(value.get());
    arc()
        .size(step.base())
        .stroke(step.base_stroke())
        .caps(Cap::Round)
        .track(look().eased(|c| c.surface2))
        .color(motion::eased_color(color))
        .progress(fill.transition(motion::spring()))
}

/// How much of a ring `value` fills: from 0 to 1, none where it is not a
/// number.
fn filled(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::filled;

    #[test]
    fn a_ring_fills_from_none_to_whole() {
        assert_eq!(filled(-0.5), 0.0);
        assert_eq!(filled(0.4), 0.4);
        assert_eq!(filled(1.5), 1.0);
        assert_eq!(filled(f32::NAN), 0.0);
    }
}
