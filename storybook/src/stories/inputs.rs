use guido::prelude::*;
use styletto::centered_row;
use styletto::components::button::{Fill, icon_button};
use styletto::components::input::{field as frame, password_field as secret};
use styletto::components::slider::slider as quick_slider;
use styletto::components::toggle::toggle as switch;
use styletto::foundations::control::Size;
use styletto::foundations::tokens::{icon as icon_box, size as space};
use styletto::foundations::typography as ty;
use styletto::foundations::typography::Typeset;
use styletto::primitives::text::label;
use styletto::{Icon, icon, ui};

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

/// A slider; a max past 100 shows where 100 is.
pub fn slider(ctx: &Ctx) -> Container {
    let value = ctx.number("Value", 0.0..=150.0, 60.0);
    let max = ctx.number("Max", 100.0..=150.0, 100.0);
    let width = ctx.number("Width", 120.0..=480.0, 280.0);
    let changed = ctx.action_with::<f32>("on_change");
    container()
        .width(move || width.get())
        .child(quick_slider(value).max(max).on_change(move |v| {
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
    let t = ui();
    container().width(space::S80.px()).child(move || {
        let submitted = submitted.clone();
        frame()
            .size(if small.get() { Size::Sm } else { Size::Md })
            .ring(move || {
                if refused.get() {
                    t.colors.danger
                } else {
                    t.colors.accent.with_alpha(0.7)
                }
            })
            .child(icon(Icon::Search, icon_box::SMALL.px(), t.colors.dim))
            .child(
                container().width(fill()).child(
                    text_input(typed)
                        .typeset(ty::TITLE_XSMALL)
                        .color(t.colors.text)
                        .placeholder("Search apps".to_string())
                        .placeholder_color(t.colors.sub)
                        .cursor_color(t.colors.accent)
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
    let t = ui();
    container().width(space::S80.px()).child(move || {
        let reveal = revealer.get().then(|| {
            icon_button(Icon::Eye)
                .size(Size::Sm)
                .fill(Fill::Ghost)
                .on_click(revealed.clone())
        });
        let mut field = secret()
            .refused(refused)
            .busy(busy)
            .held(held)
            .on_submit(submitted.clone())
            .child(icon(Icon::Lock, icon_box::SMALL.px(), t.colors.sub))
            .child(
                container().width(fill()).child(
                    password_input(password)
                        .typeset(ty::LABEL_MEDIUM)
                        .color(t.colors.text)
                        .placeholder("Password".to_string())
                        .placeholder_color(t.colors.sub)
                        .cursor_color(t.colors.accent)
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
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::time::Instant;

    use guido::prelude::*;
    use guido::testing::Headless;
    use styletto::components::slider::slider;

    #[test]
    fn wheel_notches_add_up_while_the_owner_answers() {
        let Some(mut h) = Headless::new() else {
            return;
        };
        let asked = Rc::new(RefCell::new(Vec::new()));
        let probe = Rc::new(Cell::new(None));
        let (log, seen) = (asked.clone(), probe.clone());
        let id = h.surface(SurfaceConfig::new(), move || {
            // An owner that answers only when the test says so.
            let value = create_signal(60.0);
            seen.set(Some(value));
            let log = log.clone();
            container()
                .width(200.0)
                .child(slider(value).on_change(move |v| log.borrow_mut().push(v)))
        });
        h.configure(id, 200, 100, 1.0);
        h.step();
        let value: RwSignal<f32> = probe.take().unwrap();
        let notch = |h: &mut Headless, up: bool| {
            let dy = if up { -1.0 } else { 1.0 };
            h.event_at(
                id,
                Event::scroll(100.0, 8.0, 0.0, dy, ScrollSource::Wheel),
                Instant::now(),
            );
            h.step();
            asked.borrow_mut().drain(..).collect::<Vec<f32>>()
        };

        // No answer yet: each notch steps from the one before.
        assert_eq!(notch(&mut h, true), [65.0]);
        assert_eq!(notch(&mut h, true), [70.0]);
        // A late answer to the first notch does not undo the second.
        value.set(65.0);
        h.step();
        assert_eq!(notch(&mut h, true), [75.0]);
        // Caught up, a value set from outside is where the next notch starts.
        value.set(75.0);
        h.step();
        value.set(20.0);
        h.step();
        assert_eq!(notch(&mut h, false), [15.0]);
        // A refused step holds until the pointer leaves.
        assert_eq!(notch(&mut h, false), [10.0]);
        h.event_at(id, Event::MouseLeave, Instant::now());
        h.step();
        assert_eq!(notch(&mut h, false), [15.0]);
        // At an end, a notch past it asks nothing.
        value.set(15.0);
        h.step();
        value.set(100.0);
        h.step();
        assert_eq!(notch(&mut h, true), Vec::<f32>::new());
    }

    #[test]
    fn wheel_and_press_reach_the_max() {
        let Some(mut h) = Headless::new() else {
            return;
        };
        let asked = Rc::new(RefCell::new(Vec::new()));
        let probe = Rc::new(Cell::new(None));
        let (log, seen) = (asked.clone(), probe.clone());
        let id = h.surface(SurfaceConfig::new(), move || {
            // An owner that takes every value at once.
            let value = create_signal(140.0);
            let max = create_signal(150.0);
            seen.set(Some((value, max)));
            let log = log.clone();
            container()
                .width(200.0)
                .child(slider(value).max(max).on_change(move |v| {
                    value.set(v);
                    log.borrow_mut().push(v);
                }))
        });
        h.configure(id, 200, 100, 1.0);
        h.step();
        let (_, max): (RwSignal<f32>, RwSignal<f32>) = probe.take().unwrap();
        let send = |h: &mut Headless, event: Event| {
            h.event_at(id, event, Instant::now());
            h.step();
            asked.borrow_mut().drain(..).collect::<Vec<f32>>()
        };
        let up = || Event::scroll(100.0, 8.0, 0.0, -1.0, ScrollSource::Wheel);
        let down = || Event::scroll(100.0, 8.0, 0.0, 1.0, ScrollSource::Wheel);

        // Past 100, up to the max and no further.
        assert_eq!(send(&mut h, up()), [145.0]);
        assert_eq!(send(&mut h, up()), [150.0]);
        assert_eq!(send(&mut h, up()), Vec::<f32>::new());
        // A press halfway asks for half the max.
        let press = Event::mouse_down(100.0, 8.0, MouseButton::Left);
        assert_eq!(send(&mut h, press), [75.0]);
        send(&mut h, Event::mouse_up(100.0, 8.0, MouseButton::Left));
        // A max lowered under the value asks nothing by itself; the next
        // notch steps from the max, where the value is drawn.
        max.set(50.0);
        h.step();
        assert!(asked.borrow().is_empty());
        assert_eq!(send(&mut h, down()), [45.0]);
        // A max that is not a positive finite number counts as 100.
        max.set(0.0);
        h.step();
        assert_eq!(send(&mut h, up()), [50.0]);
        max.set(f32::NAN);
        let press = Event::mouse_down(150.0, 8.0, MouseButton::Left);
        assert_eq!(send(&mut h, press), [75.0]);
        send(&mut h, Event::mouse_up(150.0, 8.0, MouseButton::Left));
        max.set(f32::INFINITY);
        h.step();
        assert_eq!(send(&mut h, up()), [80.0]);
        max.set(-10.0);
        h.step();
        assert_eq!(send(&mut h, up()), [85.0]);
    }
}
