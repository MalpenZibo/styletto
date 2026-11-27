use guido::prelude::*;
use styletto::centered_row;
use styletto::components::button::{Fill, button};
use styletto::components::input::{field as frame, password_field as secret};
use styletto::components::slider::slider as quick_slider;
use styletto::components::toggle::toggle as switch;
use styletto::foundations::control::Size;
use styletto::foundations::tokens::{icon as icon_box, size as space};
use styletto::foundations::typography as ty;
use styletto::foundations::typography::Typeset;
use styletto::primitives::text::{PaletteColors, label};
use styletto::{Icon, icon, look};

use crate::stories::story::Ctx;

pub fn toggle(ctx: &Ctx) -> Container {
    let on = ctx.bool("On", true);
    let enabled = ctx.bool("Enabled", true);
    let toggled = ctx.action_with::<bool>("on_toggle");
    container()
        .layout(centered_row(space::S3.px()))
        .child(label("Bluetooth"))
        .child(switch(on).enabled(enabled).on_toggle(move |value| {
            on.set(value);
            toggled(value);
        }))
}

/// A slider; a max past 100 shows where 100 is, and a wheel notch, or a
/// touchpad swipe as long, moves it by its step.
pub fn slider(ctx: &Ctx) -> Container {
    let value = ctx.number("Value", 0.0..=150.0, 60.0);
    let max = ctx.number("Max", 100.0..=150.0, 100.0);
    let step = ctx.number("Step", 1.0..=20.0, 5.0);
    let width = ctx.number("Width", 120.0..=480.0, 280.0);
    let changed = ctx.action_with::<f32>("on_change");
    container()
        .width(move || width.get())
        .child(quick_slider(value).max(max).step(step).on_change(move |v| {
            value.set(v);
            changed(v);
        }))
}

/// The frame a text field sits in, ringed in the accent or, refused, in red,
/// in the launcher's size or the Wi-Fi password's.
pub fn field(ctx: &Ctx) -> Container {
    let refused = ctx.bool("Refused", false);
    let small = ctx.bool("Small", false);
    let typed = create_signal(String::new());
    let submitted = ctx.action_with::<String>("on_submit");
    let t = look();
    container().width(space::S80.px()).child(move || {
        let submitted = submitted.clone();
        frame()
            .size(if small.get() { Size::Sm } else { Size::Md })
            .refused(refused)
            .child(icon(Icon::Search, icon_box::SMALL.px(), t.color(|c| c.dim)))
            .child(
                container().width(fill()).child(
                    text_input(typed)
                        .typeset(ty::TITLE_XSMALL)
                        .palette_colors()
                        .placeholder("Search apps".to_string())
                        .on_submit(move |value| submitted(value.to_string())),
                ),
            )
    })
}

/// The password's field as a polkit request or a lock screen asks for it:
/// the ring turns red on a refusal, the end spins while the answer is
/// checked, and a reveal button can go before it.
pub fn password_field(ctx: &Ctx) -> Container {
    let refused = ctx.bool("Refused", false);
    let busy = ctx.bool("Busy", false);
    let held = ctx.bool("Held", false);
    let revealer = ctx.bool("Reveal button", true);
    let submitted = ctx.action("on_submit");
    let revealed = ctx.action("reveal");
    let password = create_password();
    let t = look();
    container().width(space::S80.px()).child(move || {
        let reveal = revealer.get().then(|| {
            button()
                .icon(Icon::Eye)
                .size(Size::Sm)
                .fill(Fill::Ghost)
                .on_click(revealed.clone())
        });
        let mut field = secret()
            .refused(refused)
            .busy(busy)
            .held(held)
            .on_submit(submitted.clone())
            .child(icon(Icon::Lock, icon_box::SMALL.px(), t.color(|c| c.sub)))
            .child(
                container().width(fill()).child(
                    password_input(password)
                        .typeset(ty::LABEL_MEDIUM)
                        .palette_colors()
                        .placeholder("Password".to_string())
                        .readonly(held)
                        .on_submit(submitted.clone()),
                ),
            );
        if let Some(reveal) = reveal {
            field = field.extra(reveal);
        }
        field
    })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use guido::prelude::*;
    use styletto::components::input::field;
    use styletto::components::slider::{Slider, slider};
    use styletto::foundations::tokens::opacity;
    use styletto::look;

    use crate::stories::mounted::{Asked, Mounted};

    /// A slider whose owner takes every value at once.
    fn taking(value: RwSignal<f32>, log: Asked<f32>) -> Slider {
        slider(value).on_change(move |v| {
            value.set(v);
            log.borrow_mut().push(v);
        })
    }

    #[test]
    fn wheel_notches_add_up_while_the_owner_answers() {
        let Some((mut m, value)) = Mounted::<f32>::new(|log| {
            // An owner that answers only when the test says so.
            let value = create_signal(60.0);
            let slider = slider(value).on_change(move |v| log.borrow_mut().push(v));
            (container().width(200.0).child(slider), value)
        }) else {
            return;
        };

        // No answer yet: each notch steps from the one before.
        assert_eq!(m.notch(true), [65.0]);
        assert_eq!(m.notch(true), [70.0]);
        // A late answer to the first notch does not undo the second.
        value.set(65.0);
        m.step();
        assert_eq!(m.notch(true), [75.0]);
        // Caught up, a value set from outside is where the next notch starts.
        value.set(75.0);
        m.step();
        value.set(20.0);
        m.step();
        assert_eq!(m.notch(false), [15.0]);
        // A refused step holds until the pointer leaves.
        assert_eq!(m.notch(false), [10.0]);
        m.send(Event::MouseLeave);
        assert_eq!(m.notch(false), [15.0]);
        // At an end, a notch past it asks nothing.
        value.set(15.0);
        m.step();
        value.set(100.0);
        m.step();
        assert_eq!(m.notch(true), Vec::<f32>::new());
    }

    #[test]
    fn a_notch_steps_from_the_level_an_owner_snaps_to() {
        let Some((mut m, ())) = Mounted::<f32>::new(|log| {
            // A backlight of 16 levels: it takes the one nearest the ask.
            let value = create_signal(40.0);
            let slider = slider(value).on_change(move |v: f32| {
                value.set((v / 100.0 * 15.0).round() / 15.0 * 100.0);
                log.borrow_mut().push(v);
            });
            (container().width(200.0).child(slider), ())
        }) else {
            return;
        };

        // Asked 45, it answers 46.67: the next notch steps from there, and
        // asks 51.67 rounded, not 50.
        assert_eq!(m.notch(true), [45.0]);
        assert_eq!(m.notch(true), [52.0]);
        // 53.33, then down: 48.33 and 41.67, rounded.
        assert_eq!(m.notch(false), [48.0]);
        assert_eq!(m.notch(false), [42.0]);
    }

    #[test]
    fn a_change_from_outside_ends_the_wheels_run() {
        let Some((mut m, value)) = Mounted::<f32>::new(|log| {
            // An owner that answers only when the test says so.
            let value = create_signal(60.0);
            let slider = slider(value).on_change(move |v| log.borrow_mut().push(v));
            (container().width(200.0).child(slider), value)
        }) else {
            return;
        };

        assert_eq!(m.notch(true), [65.0]);
        assert_eq!(m.notch(true), [70.0]);
        // A media key, with the pointer still on the slider: the next notch
        // steps from what it set, not from the asks it overtook.
        value.set(30.0);
        m.step();
        assert_eq!(m.notch(true), [35.0]);
        // A late answer still holds a run started after the change.
        assert_eq!(m.notch(true), [40.0]);
        value.set(35.0);
        m.step();
        assert_eq!(m.notch(true), [45.0]);
    }

    #[test]
    fn wheel_and_press_reach_the_max() {
        let Some((mut m, max)) = Mounted::<f32>::new(|log| {
            let (value, max) = (create_signal(140.0), create_signal(150.0));
            (
                container().width(200.0).child(taking(value, log).max(max)),
                max,
            )
        }) else {
            return;
        };

        // Past 100, up to the max and no further.
        assert_eq!(m.notch(true), [145.0]);
        assert_eq!(m.notch(true), [150.0]);
        assert_eq!(m.notch(true), Vec::<f32>::new());
        // A press halfway asks for half the max.
        assert_eq!(m.press(100.0), [75.0]);
        // A max lowered under the value asks nothing by itself; the next
        // notch steps from the max, where the value is drawn.
        max.set(50.0);
        m.step();
        assert!(m.asked.borrow().is_empty());
        assert_eq!(m.notch(false), [45.0]);
        // A max that is not a positive finite number counts as 100.
        max.set(0.0);
        m.step();
        assert_eq!(m.notch(true), [50.0]);
        max.set(f32::NAN);
        assert_eq!(m.press(150.0), [75.0]);
        max.set(f32::INFINITY);
        m.step();
        assert_eq!(m.notch(true), [80.0]);
        max.set(-10.0);
        m.step();
        assert_eq!(m.notch(true), [85.0]);
    }

    #[test]
    fn a_notch_moves_by_the_step() {
        let Some((mut m, step)) = Mounted::<f32>::new(|log| {
            let (value, step) = (create_signal(50.0), create_signal(2.0));
            let slider = taking(value, log).max(150.0).step(step);
            (container().width(200.0).child(slider), step)
        }) else {
            return;
        };

        assert_eq!(m.notch(true), [52.0]);
        assert_eq!(m.notch(false), [50.0]);
        // A sideways scroll moves nothing.
        let sideways = Event::scroll(100.0, 8.0, 1.0, 0.0, ScrollSource::Wheel);
        assert_eq!(m.send(sideways), Vec::<f32>::new());
        // A fraction is rounded, to 1 at least.
        step.set(2.6);
        m.step();
        assert_eq!(m.notch(true), [53.0]);
        step.set(0.2);
        m.step();
        assert_eq!(m.notch(true), [54.0]);
        // One that is not a positive finite number counts as 5.
        for bad in [0.0, -3.0, f32::NAN, f32::INFINITY] {
            step.set(bad);
            m.step();
            assert_eq!(m.notch(false), [49.0], "{bad}");
            assert_eq!(m.notch(true), [54.0], "{bad}");
        }
        // The same past 100.
        step.set(20.0);
        m.step();
        assert_eq!(
            [m.notch(true), m.notch(true), m.notch(true)].concat(),
            [74.0, 94.0, 114.0]
        );
    }

    #[test]
    fn a_swipe_steps_as_far_as_a_wheel_would() {
        let Some((mut m, ())) = Mounted::<f32>::new(|log| {
            let slider = taking(create_signal(50.0), log);
            (container().width(200.0).child(slider), ())
        }) else {
            return;
        };
        let swipe = |m: &mut Mounted<f32>, dy: f32, times: usize, source: ScrollSource| {
            (0..times)
                .flat_map(|_| m.scroll(dy, source))
                .collect::<Vec<_>>()
        };

        // A touchpad's small deltas step once for each notch's worth, 40
        // pixels: 100 pixels down are two steps, and 20 wait.
        assert_eq!(swipe(&mut m, 4.0, 25, ScrollSource::Finger), [45.0, 40.0]);
        // The 20 left and 20 more make a third.
        assert_eq!(swipe(&mut m, 4.0, 5, ScrollSource::Finger), [35.0]);
        // Turning back answers at once, from nothing left.
        assert_eq!(swipe(&mut m, -8.0, 5, ScrollSource::Finger), [40.0]);
        // A high-resolution wheel's eighths make one step per notch.
        assert_eq!(swipe(&mut m, -5.0, 8, ScrollSource::Wheel), [45.0]);
        // A continuous source gathers as a finger does.
        assert_eq!(swipe(&mut m, -20.0, 2, ScrollSource::Continuous), [50.0]);
        // Leaving drops what is short of a notch.
        assert_eq!(
            swipe(&mut m, 30.0, 1, ScrollSource::Finger),
            Vec::<f32>::new()
        );
        m.send(Event::mouse_move(100.0, 90.0));
        m.send(Event::mouse_move(100.0, 8.0));
        assert_eq!(
            swipe(&mut m, 30.0, 1, ScrollSource::Finger),
            Vec::<f32>::new()
        );
        assert_eq!(swipe(&mut m, 10.0, 1, ScrollSource::Finger), [45.0]);
        // So does a press.
        assert_eq!(
            swipe(&mut m, 30.0, 1, ScrollSource::Finger),
            Vec::<f32>::new()
        );
        assert_eq!(m.press(100.0), [50.0]);
        assert_eq!(
            swipe(&mut m, 30.0, 1, ScrollSource::Finger),
            Vec::<f32>::new()
        );
    }

    #[test]
    fn a_field_is_ringed_in_the_accent_and_refused_in_danger() {
        let Some((mut m, (refused, own))) = Mounted::<()>::new(|_| {
            let t = look();
            let refused = create_signal(false);
            let own = create_signal(Some(Color::rgb(0.0, 1.0, 0.0)));
            // The field, the rule's two rings set by hand beside it, and a
            // field with a ring of its own.
            let row = container()
                .width(200.0)
                .layout(Flex::row())
                .child(field().refused(refused))
                .child(field().ring(t.color(|c| c.accent.fill.with_alpha(opacity::RING))))
                .child(field().ring(t.color(|c| c.danger.fill)))
                .child(field().refused(refused).ring(own));
            (row, (refused, own))
        }) else {
            return;
        };
        // The middle of each one's top edge, once the ring's change has
        // played.
        let rings = |m: &mut Mounted<()>, at: Instant| {
            m.step_at(at);
            m.step_at(at + Duration::from_secs(2));
            [0, 1, 2, 3].map(|index| m.pixel(25.0 + index as f32 * 50.0, 0.0))
        };
        let start = Instant::now();

        let [field, accent, danger, green] = rings(&mut m, start);
        assert_ne!(accent, danger);
        assert_eq!(field, accent);

        refused.set(true);
        let [field, _, _, kept] = rings(&mut m, start + Duration::from_secs(3));
        assert_eq!(field, danger);
        // A ring the caller set is kept through a refusal, and `None` hands
        // it back to the rule.
        assert_eq!(kept, green);
        own.set(None);
        let [.., handed_back] = rings(&mut m, start + Duration::from_secs(6));
        assert_eq!(handed_back, danger);

        refused.set(false);
        let [field, ..] = rings(&mut m, start + Duration::from_secs(9));
        assert_eq!(field, accent);
    }
}
