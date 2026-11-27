//! A shape that splits off the end of another and merges back. Both sit in a
//! merging row, so while close they draw as one shape through a neck.

use guido::prelude::*;

use crate::foundations::motion;
use crate::foundations::tokens::{radius, size};
use crate::primitives::island::paint;

/// Padding between the satellite's ends and its content.
fn padding() -> f32 {
    size::S3.px()
}

/// Out past its host's end, or shrunk into it. Placed after the host, it
/// takes the pointer only while out and is [`paint`]ed like the island.
#[component]
pub fn satellite(
    /// Its content's width.
    width: f32,
    height: f32,
    /// Distance from the host while out.
    gap: f32,
    out: bool,
    #[prop(callback)] on_hover: fn(bool),
    #[prop(callback)] on_click: (),
    #[prop(children)] children: (),
) -> impl Widget {
    let outer = move || width.get() + 2.0 * padding();
    let height = height.get_untracked();
    paint(container())
        .width((move || Length::exact(outer())).transition(motion::spring()))
        .height(height)
        .corners(radius::pill(height))
        .padding([0.0, padding()])
        .translate(
            (move || {
                let x = if out.get() {
                    gap.get()
                } else {
                    -(outer() + height) / 2.0
                };
                Translate::new(x, 0.0)
            })
            .transition(motion::spring()),
        )
        .pivot(Pivot::CENTER)
        .scale(
            (move || Scale::uniform(if out.get() { 1.0 } else { 0.0 }))
                .transition(motion::spring()),
        )
        .takes_input(out)
        .on_hover(move |inside| {
            if let Some(on_hover) = on_hover.get_untracked() {
                on_hover.run(inside);
            }
        })
        .on_click(on_click)
        .layout(Flex::row().cross_alignment(CrossAlignment::Center))
        .children(children)
}

/// A satellite-wide spacer before the host, keeping the host centred.
#[component]
pub fn balance(
    /// The satellite's content width.
    #[prop(required)]
    width: f32,
) -> impl Widget {
    container()
        .width((move || Length::exact(width.get() + 2.0 * padding())).transition(motion::spring()))
}
