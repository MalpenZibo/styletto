//! Linear progress bars and progress rings.

use guido::prelude::*;

use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{IconSize, icon, size as space};
use crate::theme::palette::Palette;
use crate::ui::ui;

/// A thin bar.
#[component]
pub fn progress_bar(
    /// From 0 to 1.
    #[prop(required)]
    value: f32,
    color: Color,
) -> impl Widget {
    let t = ui();
    container()
        .width(fill())
        .height(space::S1.px())
        .corners(radius::pill(space::S1.px()))
        .background(t.colors.surface2)
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
    let t = ui();
    let size = size.get_untracked();
    arc()
        .size(size.px())
        .stroke(size.stroke())
        .caps(Cap::Round)
        .track(t.colors.surface2)
        .color(move || color.get())
        .progress((move || value.get().clamp(0.0, 1.0)).transition(motion::spring()))
}

/// A battery's ink: the success colour on the charger, danger at 15% or
/// less, warning at 30% or less or while saving power, else the text's.
pub fn battery_ink(colors: &Palette, percent: f32, plugged: bool, saving: bool) -> Color {
    if plugged {
        colors.success
    } else if percent <= 15.0 {
        colors.danger
    } else if percent <= 30.0 || saving {
        colors.warning
    } else {
        colors.text
    }
}
