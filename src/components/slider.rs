//! A percentage slider: a thin line at rest that swells into a pill showing
//! its value while hovered or dragged. Press, drag or scroll to set it. It
//! may go past 100, an amplified volume, say: a mark then shows where 100 is.

use std::cell::Cell;

use guido::prelude::*;

use crate::components::button::{Fill, button};
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::size as space;
use crate::foundations::tokens::{radius, stroke};
use crate::foundations::typography::LABEL_SMALL;
use crate::primitives::layout::centered_row;
use crate::primitives::text::figure_text;
use crate::ui::look;

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
/// The pixels Guido gives a classic wheel's notch (its
/// `SCROLL_PIXELS_PER_LINE`, not public). A scroll moves the slider
/// a step for each notch's worth of it, whatever sent it: a touchpad's swipe
/// as far as the same distance on a wheel, a high-resolution wheel's eighths
/// of a notch once a whole notch is gathered.
const NOTCH: f32 = 40.0;
/// Where the mark goes, and the maximum past which it shows.
const FULL: f32 = 100.0;

/// Adds `dy` to `left`, the pixels a scroll was short of a notch, and gives
/// the whole notches in them and the pixels short of the next. A scroll
/// turning the other way drops what was left, so it answers at once.
fn gather(left: f32, dy: f32) -> (f32, f32) {
    let left = if left * dy < 0.0 { 0.0 } else { left };
    let total = left + dy;
    let notches = (total / NOTCH).trunc();
    // `%` is exact: what is left stays short of a notch, however large
    // the delta.
    (notches, total % NOTCH)
}

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
    /// The percentage a wheel notch, or a swipe as long, adds or takes, 5
    /// unless set; the same past 100. A whole percentage: a fraction is rounded, to 1 at least.
    /// One that is not a positive finite number counts as 5.
    #[prop(default = STEP)]
    step: f32,
    /// Called with each new whole percentage.
    #[prop(callback)]
    on_change: fn(f32),
) -> impl Widget {
    let t = look();
    let on_change = move |v: f32| {
        if let Some(on_change) = on_change.get_untracked() {
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
    // Likewise the wheel's steps the owner has not answered yet, in order: the
    // last one is shown, and the next notch steps from it, not from an answer
    // on its way.
    let wheeled = create_signal(Vec::<f32>::new());
    // The pixels scrolled short of a notch, kept for the next event.
    let scrolled = create_signal(0.0_f32);
    create_effect(move || {
        // Rounded, as the steps are, for an owner that keeps finer levels (a
        // backlight's) than whole percentages.
        let v = value.get().round();
        wheeled.update(|asks| match asks.iter().position(|&a| a == v) {
            // A late answer: the steps asked after it still hold.
            Some(i) => {
                asks.drain(..=i);
            }
            // Anything else, a level of the owner's own or a change from
            // outside, is believed: the next notch steps from it.
            None => asks.clear(),
        });
    });
    let held = move || wheeled.with(|asks| asks.last().copied());
    let shown = create_memo(move || {
        dragging
            .get()
            .or_else(held)
            .unwrap_or_else(|| value.get())
            .clamp(0.0, max.get())
    });
    let width = create_memo(move || bounds.rect().get().width);
    let active = create_memo(move || hovered.get() || dragging.get().is_some());

    let set = move |v: f32| {
        let v = v.clamp(0.0, max.get_untracked()).round();
        // A press or a drag ends the wheel's run and drops a scroll short
        // of a notch.
        wheeled.set(Vec::new());
        scrolled.set(0.0);
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
        .background(t.eased(|c| c.accent.fill));
    // A cut through the track in the island's colour: it reads on the empty
    // part as much as the track reads on the island, and on the fill as much
    // as the fill does, in any theme, and stays as quiet as the track.
    let amplified = create_memo(move || max.get() > FULL);
    let mark = move || {
        amplified.get().then(|| {
            container()
                .width(stroke::HAIRLINE)
                .height(fill())
                .background(t.eased(|c| c.island))
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
                .child(figure_text(
                    move || format!("{}%", shown.get().round()),
                    LABEL_SMALL.em(),
                    move || {
                        let c = t.colors();
                        if inside.get() { c.accent.on } else { c.sub }
                    },
                )),
        );
    let track_height = move || if active.get() { height() } else { thin() };
    let track = container()
        .width(fill())
        .height(track_height.transition(motion::spring()))
        .corners((move || radius::pill(track_height())).transition(motion::spring()))
        .overflow(Overflow::Hidden)
        .background(t.eased(|c| c.surface2))
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
            // never answered, is believed again, and a scroll short of a
            // notch is dropped.
            if !h {
                wheeled.set(Vec::new());
                scrolled.set(0.0);
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
            // A delta that is not a number moves nothing.
            if !dy.is_finite() {
                return;
            }
            let (notches, left) = gather(scrolled.get_untracked(), dy);
            scrolled.set(left);
            // Short of a notch (or sideways) nothing is asked: stepping by
            // none would still round an owner's 50.4 to 50.
            if notches == 0.0 {
                return;
            }
            // From what is shown: a value past a maximum lowered under it
            // steps down from the maximum.
            let max = max.get_untracked();
            let from = wheeled
                .with_untracked(|asks| asks.last().copied())
                .unwrap_or_else(|| value.get_untracked())
                .clamp(0.0, max);
            let next = (from - notches * step.get_untracked())
                .clamp(0.0, max)
                .round();
            if next != from {
                // Held before the owner is told, so an answer given at once
                // is seen catching up.
                wheeled.update(|asks| asks.push(next));
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
    /// Shows the chevron, which calls it. One that comes and goes with
    /// state, there only while there is more to choose from (several
    /// outputs, say), fades in and out of a room kept for it, so the slider
    /// never moves and is never built again.
    #[prop(callback)]
    on_more: (),
) -> impl Widget {
    let t = look();
    // Built again only when it turns into a button or back.
    let icon_button = create_memo(move || on_icon.with(Option::is_some));
    let lead = move || {
        if icon_button.get() {
            container().child(button().icon(icon).size(Size::Sm).on_click(on_icon))
        } else {
            container()
                .width(Size::Sm.height())
                .height(Size::Sm.height())
                .layout(Flex::column().center())
                .child(icons::icon(icon, Size::Sm.icon(), t.color(|c| c.text)))
        }
    };
    // The room is always kept; the chevron is built only when it comes, and
    // fades in and out of it. One there from the start is drawn at once, as
    // the rest of the row is. Stacked, so one coming back while it still
    // fades out takes its place, not the room beside it.
    let more = create_memo(move || on_more.with(Option::is_some));
    let first = Cell::new(true);
    let tail = container()
        .width(Size::Sm.height())
        .layout(ZStack::new())
        .child(move || {
            let fades = !first.replace(false);
            more.get().then(|| {
                let shown = 1.0.transition(motion::hover()).exiting_to(0.0);
                container()
                    .opacity(if fades {
                        shown.entering_from(0.0)
                    } else {
                        shown
                    })
                    .child(
                        button()
                            .icon(Icon::ChevronRight)
                            .size(Size::Sm)
                            .fill(Fill::Ghost)
                            .on_click(on_more),
                    )
            })
        });
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

#[cfg(test)]
mod tests {
    use super::{NOTCH, gather};

    #[test]
    fn a_scroll_steps_once_for_each_notch_it_gathers() {
        // A classic wheel: a notch at a time.
        assert_eq!(gather(0.0, NOTCH), (1.0, 0.0));
        assert_eq!(gather(0.0, -NOTCH), (-1.0, 0.0));
        // A swipe's small deltas add up, and what is short of a notch waits.
        let mut left = 0.0;
        let mut notches = 0.0;
        for _ in 0..25 {
            let (n, l) = gather(left, 4.0);
            notches += n;
            left = l;
        }
        assert_eq!((notches, left), (2.0, 20.0));
        // A fast one moves several at once.
        assert_eq!(gather(0.0, 3.5 * NOTCH), (3.0, 0.5 * NOTCH));
        // However large, what is left stays short of a notch.
        assert!(gather(0.0, 1e10).1.abs() < NOTCH);
        // Turning back drops what was left the other way.
        assert_eq!(gather(30.0, -10.0), (0.0, -10.0));
        assert_eq!(gather(-30.0, NOTCH), (1.0, 0.0));
    }
}
