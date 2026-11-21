//! A percentage slider: a thin line at rest that swells into a pill showing
//! its value while hovered or dragged. Press, drag or scroll to set it. It
//! may go past 100, an amplified volume, say: a mark then shows where 100 is.

use guido::prelude::*;

use crate::components::button::{Fill, button};
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::size as space;
use crate::foundations::tokens::{radius, stroke};
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
/// Percent per wheel notch unless the app sets its own; whatever the
/// maximum, a notch changes a level by the same amount past 100 as below it.
const STEP: f32 = 5.0;
/// Where the mark goes, and the maximum past which it shows.
const FULL: f32 = 100.0;

/// `value` if it is a positive finite number, else `default`.
fn positive_or(value: f32, default: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        default
    }
}

#[component]
pub fn slider(
    /// A percentage, 0 to `max`. Past `max`, it is drawn at `max`.
    #[prop(required)]
    value: f32,
    /// The highest percentage it can be set to. Above 100, a mark on the
    /// track shows where 100 is. One that is not a positive finite number
    /// counts as 100.
    #[prop(default = FULL)]
    max: f32,
    /// The percentage a wheel notch adds or takes, 5 unless set; the same
    /// past 100. A whole percentage: a fraction is rounded, to 1 at least.
    /// One that is not a positive finite number counts as 5.
    #[prop(default = STEP)]
    step: f32,
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
    // Every read of the max goes through this: a bad one would divide by
    // zero or make a clamp panic.
    let max = create_memo(move || positive_or(max.get(), FULL)).into_signal();
    // Whole, as the values it steps between are.
    let step = create_memo(move || positive_or(step.get(), STEP).round().max(1.0)).into_signal();
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
            .clamp(0.0, max.get())
    });
    let width = create_memo(move || bounds.rect().get().width);
    let active = create_memo(move || hovered.get() || dragging.get().is_some());

    let set = move |v: f32| {
        let v = v.clamp(0.0, max.get_untracked()).round();
        wheeled.set(None);
        if dragging.get_untracked() != Some(v) {
            dragging.set(Some(v));
            on_change(v);
        }
    };
    let locate = move |x: f32| {
        let w = bounds.rect().get_untracked().width;
        if w > 0.0 {
            x / w * max.get_untracked()
        } else {
            0.0
        }
    };

    let reach = move || shown.get() / max.get() * width.get();
    let filled = container()
        .width(reach.transition(motion::tactile()))
        .height(fill())
        .background(t.colors.accent);
    // A cut through the track in the island's colour: it reads on the empty
    // part as much as the track reads on the island, and on the fill as much
    // as the fill does, in any theme, and stays as quiet as the track.
    let amplified = create_memo(move || max.get() > FULL);
    let mark = move || {
        amplified.get().then(|| {
            container()
                .width(stroke::HAIRLINE)
                .height(fill())
                .background(t.colors.island)
                .translate(move || {
                    let x = FULL / max.get() * width.get() - stroke::HAIRLINE / 2.0;
                    Translate::new(x, 0.0)
                })
        })
    };
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
        .child(mark)
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
            // From what is shown: a value past a maximum lowered under it
            // steps down from the maximum.
            let max = max.get_untracked();
            let from = wheeled
                .get_untracked()
                .unwrap_or_else(|| value.get_untracked())
                .clamp(0.0, max);
            let next = (from - dy.signum() * step.get_untracked())
                .clamp(0.0, max)
                .round();
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
    /// The slider's `max`; one that is not a positive finite number counts
    /// as 100.
    #[prop(default = FULL)]
    max: f32,
    /// The slider's `step`, 5 unless set.
    #[prop(default = STEP)]
    step: f32,
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
        Some(pressed) => container().child(button().icon(icon).size(Size::Sm).on_click(pressed)),
        None => container()
            .width(Size::Sm.height())
            .height(Size::Sm.height())
            .layout(Flex::column().center())
            .child(icons::icon(icon, Size::Sm.icon(), t.colors.text)),
    };
    let tail = match on_more {
        Some(pressed) => container().child(
            button()
                .icon(Icon::ChevronRight)
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
                .child(slider(value).max(max).step(step).on_change(on_change)),
        )
        .child(tail)
}
