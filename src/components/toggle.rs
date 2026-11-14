//! An on/off switch whose knob springs across the track.

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{opacity, size};
use crate::ui::{flip, ui};

fn height() -> f32 {
    Size::Xxs.height()
}
fn width() -> f32 {
    height() + size::S4.px()
}
fn inset() -> f32 {
    size::S0_5.px()
}
fn knob() -> f32 {
    height() - 2.0 * inset()
}

#[component]
pub fn toggle(
    /// Follows what the owner sets.
    #[prop(required)]
    on: bool,
    #[prop(default = true)] enabled: bool,
    /// Called with the state a press asks for.
    #[prop(callback)]
    on_toggle: fn(bool),
) -> impl Widget {
    let t = ui();
    container()
        .width(width())
        .height(height())
        .corners(radius::pill(height()))
        .background(
            (move || {
                if on.get() {
                    t.colors.accent
                } else {
                    t.colors.surface2
                }
            })
            .transition(motion::color()),
        )
        .opacity(1.0.transition(motion::hover()))
        .when_disabled(|s| s.opacity(opacity::DISABLED))
        .enabled(enabled)
        .on_click(flip(on, on_toggle))
        .child(
            container()
                .width(knob())
                .height(knob())
                .corners(radius::pill(knob()))
                .background(Color::WHITE)
                .translate(
                    (move || {
                        let x = if on.get() {
                            width() - knob() - inset()
                        } else {
                            inset()
                        };
                        Translate::new(x, inset())
                    })
                    .transition(motion::spring()),
                ),
        )
}
