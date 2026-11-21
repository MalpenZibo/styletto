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

/// A slider; a max past 100 shows where 100 is, and a wheel notch moves it
/// by its step.
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
                        .cursor_color(t.colors.accent_text)
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
            .child(icon(Icon::Lock, icon_box::SMALL.px(), t.colors.sub))
            .child(
                container().width(fill()).child(
                    password_input(password)
                        .typeset(ty::LABEL_MEDIUM)
                        .color(t.colors.text)
                        .placeholder("Password".to_string())
                        .placeholder_color(t.colors.sub)
                        .cursor_color(t.colors.accent_text)
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
    use styletto::components::slider::{Slider, slider};

    /// What a slider asks its owner, in order.
    type Asked = Rc<RefCell<Vec<f32>>>;

    /// A slider's tree mounted headless, 200 wide, with what it asked.
    struct Mounted {
        h: Headless,
        id: SurfaceId,
        asked: Asked,
    }

    impl Mounted {
        /// The tree `build` makes from the log of what is asked, and the
        /// signals it hands back for the test to drive; none without a GPU.
        fn new<P: 'static>(build: impl Fn(Asked) -> (Container, P) + 'static) -> Option<(Self, P)> {
            let mut h = Headless::new()?;
            let asked = Asked::default();
            let probe = Rc::new(Cell::new(None));
            let (log, seen) = (asked.clone(), probe.clone());
            let id = h.surface(SurfaceConfig::new(), move || {
                let (tree, handed) = build(log.clone());
                seen.set(Some(handed));
                tree
            });
            h.configure(id, 200, 100, 1.0);
            h.step();
            // Only a missing GPU skips a test; a tree that was not built fails.
            let handed = probe.take().expect("the tree was built");
            Some((Self { h, id, asked }, handed))
        }

        /// Sends `event`, steps, and gives what it asked.
        fn send(&mut self, event: Event) -> Vec<f32> {
            self.h.event_at(self.id, event, Instant::now());
            self.h.step();
            self.asked.borrow_mut().drain(..).collect()
        }

        /// A wheel notch, up or down.
        fn notch(&mut self, up: bool) -> Vec<f32> {
            let dy = if up { -1.0 } else { 1.0 };
            self.send(Event::scroll(100.0, 8.0, 0.0, dy, ScrollSource::Wheel))
        }

        /// A press and a release at `x`, giving what the press asked.
        fn press(&mut self, x: f32) -> Vec<f32> {
            let asked = self.send(Event::mouse_down(x, 8.0, MouseButton::Left));
            self.send(Event::mouse_up(x, 8.0, MouseButton::Left));
            asked
        }

        fn step(&mut self) {
            self.h.step();
        }
    }

    /// A slider whose owner takes every value at once.
    fn taking(value: RwSignal<f32>, log: Asked) -> Slider {
        slider(value).on_change(move |v| {
            value.set(v);
            log.borrow_mut().push(v);
        })
    }

    #[test]
    fn wheel_notches_add_up_while_the_owner_answers() {
        let Some((mut m, value)) = Mounted::new(|log| {
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
        let Some((mut m, ())) = Mounted::new(|log| {
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
        let Some((mut m, value)) = Mounted::new(|log| {
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
        let Some((mut m, max)) = Mounted::new(|log| {
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
        let Some((mut m, step)) = Mounted::new(|log| {
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
}
