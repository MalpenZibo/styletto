use guido::prelude::*;
use styletto::Icon;
use styletto::components::button::{self, Fill, Variant, pill_text};
use styletto::foundations::control::Size;
use styletto::foundations::tokens::{icon as icon_box, size as space};
use styletto::ui::active_ink;
use styletto::{centered_row, icon, look};

use crate::stories::story::Ctx;

const VARIANTS: &[&str] = &["Neutral", "Accent", "Danger"];
const FILLS: &[&str] = &["Solid", "Ghost"];
const SHOWS: &[&str] = &["Label", "Icon + label", "Icon"];

/// A button in every form; a press turns it on or off through `on_toggle`,
/// as a tile's does.
pub fn button(ctx: &Ctx) -> Container {
    let words = ctx.text("Label", "Lock");
    let variant = ctx.choice("Variant", VARIANTS, 1);
    let fill = ctx.choice("Fill", FILLS, 0);
    let shows = ctx.choice("Content", SHOWS, 0);
    let small = ctx.bool("Small", true);
    let active = ctx.bool("Active", false);
    let enabled = ctx.bool("Enabled", true);
    let toggled = ctx.action_with::<bool>("on_toggle");
    let toggle = move |on: bool| {
        active.set(on);
        toggled(on);
    };
    container().child(move || {
        let variant = match variant.get() {
            1 => Variant::Accent,
            2 => Variant::Danger,
            _ => Variant::Neutral,
        };
        let fill = if fill.get() == 1 {
            Fill::Ghost
        } else {
            Fill::Solid
        };
        let shown = shows.get();
        let mut lock = button::button()
            .variant(variant)
            .fill(fill)
            .size(if small.get() { Size::Sm } else { Size::Md })
            .active(active)
            .enabled(enabled)
            .on_toggle(toggle.clone());
        if shown != 2 {
            lock = lock.label(words);
        }
        if shown != 0 {
            lock = lock.icon(Icon::Lock);
        }
        lock
    })
}

/// As in the row under the island's resting one.
pub fn bar_pill(ctx: &Ctx) -> Container {
    let active = ctx.choice("Active", &["Workspace 1", "Workspace 2", "None"], 0);
    let chosen = ctx.action_with::<usize>("on_press");
    let t = look();
    container()
        .padding(space::S1.px())
        .layout(centered_row(space::S0_5.px()))
        .children(
            [("1", Icon::Globe), ("2", Icon::Music)]
                .into_iter()
                .enumerate()
                .map(|(index, (name, kind))| {
                    let on = create_derived(move || active.get() == index);
                    let chosen = chosen.clone();
                    button::pill()
                        .active(on)
                        .on_click(move || {
                            active.set(index);
                            chosen(index);
                        })
                        .child(pill_text(name).active(on).bold(true))
                        .child(icon(
                            kind,
                            icon_box::SMALL.px(),
                            active_ink(on, t.color(|c| c.sub)),
                        ))
                }),
        )
        .child(
            button::pill()
                .child(icon(
                    Icon::Calendar,
                    icon_box::SMALL.px(),
                    t.color(|c| c.sub),
                ))
                .child(pill_text("Tuesday 20 October")),
        )
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use guido::prelude::*;
    use styletto::components::button::{Variant, button};
    use styletto::foundations::control::Size;
    use styletto::foundations::motion;
    use styletto::{Icon, look};

    use crate::stories::mounted::Mounted;

    #[test]
    fn a_press_asks_for_the_opposite() {
        let Some((mut m, active)) = Mounted::<String>::new(|log| {
            let active = create_signal(false);
            let (clicked, toggled) = (log.clone(), log);
            let lock = button()
                .label("Lock")
                .active(active)
                .on_click(move || clicked.borrow_mut().push("click".into()))
                .on_toggle(move |on| toggled.borrow_mut().push(format!("toggle({on})")));
            (container().child(lock), active)
        }) else {
            return;
        };

        // The action first, then the state asked for.
        assert_eq!(m.click(8.0, 8.0), ["click", "toggle(true)"]);
        // Until the owner keeps it, a press asks the same.
        assert_eq!(m.click(8.0, 8.0), ["click", "toggle(true)"]);
        active.set(true);
        m.step();
        assert_eq!(m.click(8.0, 8.0), ["click", "toggle(false)"]);
    }

    #[test]
    fn the_state_asked_for_is_the_opposite_of_the_one_pressed() {
        let Some((mut m, ())) = Mounted::<bool>::new(|log| {
            // An action that turns it on by itself, as the old `on_click`
            // toggles did.
            let active = create_signal(false);
            let lock = button()
                .label("Lock")
                .active(active)
                .on_click(move || active.set(true))
                .on_toggle(move |on| log.borrow_mut().push(on));
            (container().child(lock), ())
        }) else {
            return;
        };

        assert_eq!(m.click(8.0, 8.0), [true]);
    }

    /// Round icon buttons side by side, and boxes as big.
    fn side() -> f32 {
        Size::Sm.height()
    }

    /// Where the `index`th one's background is read: at its left, clear of
    /// its icon.
    fn at(index: usize) -> (f32, f32) {
        (index as f32 * side() + 4.0, side() / 2.0)
    }

    /// A round button with `variant`, on while `active`.
    fn lock<M>(variant: impl IntoSignal<Variant, M>, active: RwSignal<bool>) -> Container {
        container().child(
            button()
                .icon(Icon::Lock)
                .size(Size::Sm)
                .variant(variant)
                .active(active),
        )
    }

    #[test]
    fn a_hover_after_a_toggle_that_changed_nothing_eases_as_a_hover() {
        let Some((mut m, toggled)) = Mounted::<()>::new(|_| {
            // Solid accent buttons look the same on and off.
            let (toggled, untouched) = (create_signal(false), create_signal(false));
            let row = container()
                .layout(Flex::row())
                .child(lock(Variant::Accent, toggled))
                .child(lock(Variant::Accent, untouched));
            (row, toggled)
        }) else {
            return;
        };
        let hovered = |m: &mut Mounted<()>, index: usize, start: Instant| {
            let (x, y) = at(index);
            m.send_at(Event::mouse_move(x, y), start);
            m.step_at(start + Duration::from_millis(100));
            m.pixel(x, y)
        };

        toggled.set(true);
        m.step();
        let start = Instant::now() + Duration::from_secs(1);
        let after_toggle = hovered(&mut m, 0, start);
        let never_toggled = hovered(&mut m, 1, start + Duration::from_secs(1));
        assert_eq!(after_toggle, never_toggled);
    }

    #[test]
    fn a_toggle_after_one_that_changed_nothing_eases_as_a_toggle() {
        let Some((mut m, (active, variant))) = Mounted::<()>::new(|_| {
            let t = look();
            let (active, variant) = (create_signal(false), create_signal(Variant::Accent));
            // The accent while on, the surface while off, at a toggle's and
            // at a hover's speed.
            let eased = |transition: Transition| {
                container().width(side()).height(side()).background(
                    t.color(move |c| {
                        if active.get() {
                            c.accent.fill
                        } else {
                            c.surface
                        }
                    })
                    .transition(transition),
                )
            };
            let row = container()
                .layout(Flex::row())
                .child(lock(variant, active))
                .child(eased(motion::color()))
                .child(eased(motion::hover()));
            (row, (active, variant))
        }) else {
            return;
        };

        // On, then neutral: filled with the accent all along.
        active.set(true);
        m.step();
        variant.set(Variant::Neutral);
        m.step();
        // Off: from the accent to the surface.
        let start = Instant::now() + Duration::from_secs(1);
        active.set(false);
        m.step_at(start);
        m.step_at(start + Duration::from_millis(100));
        let [button, toggle, hover] = [0, 1, 2].map(|index| {
            let (x, y) = at(index);
            m.pixel(x, y)
        });
        assert_ne!(toggle, hover);
        assert_eq!(button, toggle);
    }
}
