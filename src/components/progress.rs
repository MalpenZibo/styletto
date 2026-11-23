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
                .background((move || color.get()).transition(motion::color())),
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
    arc()
        .size(size.px())
        .stroke(size.stroke())
        .caps(Cap::Round)
        // A track takes no motion: it changes palette at once.
        .track(look().color(|c| c.surface2))
        .color(move || color.get())
        .progress((move || value.get().clamp(0.0, 1.0)).transition(motion::spring()))
}
