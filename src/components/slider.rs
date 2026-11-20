//! A percentage slider: a thin line at rest that swells into a pill showing
//! its value while hovered or dragged. Press, drag or scroll to set it.

use guido::prelude::*;

use crate::components::button::{Fill, icon_button};
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::size as space;
use crate::foundations::typography::{LABEL_SMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::ui::ui;

fn height() -> f32 {
    Size::Sm.height()
}
fn thin() -> f32 {
    space::S1.px()
}
/// Space between the value and the end it sits against.
fn inset() -> f32 {
    space::S3.px()
}
/// Percent per wheel notch.
const WHEEL_STEP: f32 = 5.0;

#[component]
pub fn slider(
    /// A percentage, 0 to 100.
    #[prop(required)]
    value: f32,
    /// Called with each new whole percentage.
    #[prop(callback)]
    on_change: fn(f32),
) -> impl Widget {
    let t = ui();
    let on_change = move |v: f32| {
        if let Some(on_change) = on_change {
            on_change.run(v);
        }
    };
    let bounds = create_widget_ref();
    let hovered = create_signal(false);
    // While dragging, show where the pointer put it, not the owner's answer.
    let dragging = create_signal(None::<f32>);
    // Likewise the wheel's last step, until the owner's value catches up with
    // it: the next notch steps from there, not from an answer on its way. The
    // owner's value is rounded, as the steps are, for an owner that keeps
    // finer levels (a backlight's) than whole percentages.
    let wheeled = create_signal(None::<f32>);
    create_effect(move || {
        if wheeled.get_untracked() == Some(value.get().round()) {
            wheeled.set(None);
        }
    });
    let shown = create_memo(move || {
        dragging
            .get()
            .or_else(|| wheeled.get())
            .unwrap_or_else(|| value.get())
            .clamp(0.0, 100.0)
    });
    let width = create_memo(move || bounds.rect().get().width);
    let active = create_memo(move || hovered.get() || dragging.get().is_some());

    let set = move |v: f32| {
        let v = v.clamp(0.0, 100.0).round();
        wheeled.set(None);
        if dragging.get_untracked() != Some(v) {
            dragging.set(Some(v));
            on_change(v);
        }
    };
    let locate = move |x: f32| {
        let w = bounds.rect().get_untracked().width;
        if w > 0.0 { x / w * 100.0 } else { 0.0 }
    };

    let reach = move || shown.get() / 100.0 * width.get();
    let filled = container()
        .width(reach.transition(motion::tactile()))
        .height(fill())
        .background(t.colors.accent);
    // The value sits at the far end until the fill reaches it, then rides the
    // fill's edge in the accent's ink.
    let label = create_widget_ref();
    let label_width = create_memo(move || label.rect().get().width);
    let inside = create_memo(move || reach() > width.get() - inset() - label_width.get());
    let readout = container()
        .width(fill())
        .height(fill())
        .padding([0.0, inset()])
        .opacity((move || if active.get() { 1.0 } else { 0.0 }).transition(motion::hover()))
        .layout(
            Flex::row()
                .main_alignment(MainAlignment::End)
                .cross_alignment(CrossAlignment::Center),
        )
        .child(
            container()
                .widget_ref(label)
                .translate(
                    (move || {
                        let x = if inside.get() {
                            reach() - width.get()
                        } else {
                            0.0
                        };
                        Translate::new(x, 0.0)
                    })
                    .transition(motion::tactile()),
                )
                .child(
                    text(move || format!("{}%", shown.get().round()))
                        .typeset(LABEL_SMALL.em().tabular())
                        .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                        .color(
                            (move || {
                                if inside.get() {
                                    t.colors.on_accent
                                } else {
                                    t.colors.sub
                                }
                            })
                            .transition(motion::color()),
                        )
                        .nowrap(),
                ),
        );
    let track_height = move || if active.get() { height() } else { thin() };
    let track = container()
        .width(fill())
        .height(track_height.transition(motion::spring()))
        .corners((move || radius::pill(track_height())).transition(motion::spring()))
        .overflow(Overflow::Hidden)
        .background(t.colors.surface2)
        .layout(ZStack::new())
        .child(filled)
        .child(readout);

    container()
        .widget_ref(bounds)
        .width(fill())
        .height(height())
        .layout(Flex::column().main_alignment(MainAlignment::Center))
        .on_hover(move |h| {
            hovered.set(h);
            // Leaving ends the wheel's run: an owner that refused a step, or
            // never answered, is believed again.
            if !h {
                wheeled.set(None);
            }
        })
        .on_mouse_down(move |x, _| set(locate(x)))
        .on_pointer_move(move |x, _| {
            if dragging.get_untracked().is_some() {
                set(locate(x))
            }
        })
        .on_mouse_up(move |_, _| dragging.set(None))
        .on_scroll(move |_, dy, _| {
            let from = wheeled
                .get_untracked()
                .unwrap_or_else(|| value.get_untracked());
            let next = (from - dy.signum() * WHEEL_STEP).clamp(0.0, 100.0).round();
            if next != from {
                // Held before the owner is told, so an answer given at once
                // is seen catching up.
                wheeled.set(Some(next));
                on_change(next);
            }
        })
        .child(track)
}

/// A level as a row: icon, slider, and a chevron to more (or room for one).
#[component]
pub fn slider_row(
    #[prop(required)] icon: Icon,
    #[prop(required)] value: f32,
    #[prop(callback)] on_change: fn(f32),
    /// Makes the icon a button (mute, say).
    #[prop(callback)]
    on_icon: (),
    /// Shows the chevron, which calls it.
    #[prop(callback)]
    on_more: (),
) -> impl Widget {
    let t = ui();
    let lead = match on_icon {
        Some(pressed) => container().child(icon_button(icon).size(Size::Sm).on_click(pressed)),
        None => container()
            .width(Size::Sm.height())
            .height(Size::Sm.height())
            .layout(Flex::column().center())
            .child(icons::icon(icon, Size::Sm.icon(), t.colors.text)),
    };
    let tail = match on_more {
        Some(pressed) => container().child(
            icon_button(Icon::ChevronRight)
                .size(Size::Sm)
                .fill(Fill::Ghost)
                .on_click(pressed),
        ),
        None => container().width(Size::Sm.height()),
    };
    container()
        .width(fill())
        .layout(centered_row(space::S2.px()))
        .child(lead)
        .child(
            container()
                .width(fill())
                .child(slider(value).on_change(on_change)),
        )
        .child(tail)
}
