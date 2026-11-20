use guido::prelude::*;
use styletto::Icon;
use styletto::components::button::{self, Fill, Variant, pill_text};
use styletto::foundations::control::Size;
use styletto::foundations::tokens::{icon as icon_box, size as space};
use styletto::{centered_row, icon, ui};

use crate::stories::story::Ctx;

const VARIANTS: &[&str] = &["Neutral", "Accent", "Danger"];
const FILLS: &[&str] = &["Solid", "Ghost"];
const SHOWS: &[&str] = &["Label", "Icon + label", "Icon"];

/// A button in every form; a press turns it on or off, as a toggle's does.
pub fn button(ctx: &Ctx) -> Container {
    let words = ctx.text("Label", "Lock");
    let variant = ctx.choice("Variant", VARIANTS, 1);
    let fill = ctx.choice("Fill", FILLS, 0);
    let shows = ctx.choice("Content", SHOWS, 0);
    let active = ctx.bool("Active", false);
    let enabled = ctx.bool("Enabled", true);
    let pressed = ctx.action("on_press");
    let flip = move || {
        active.set(!active.get_untracked());
        pressed();
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
            .active(active)
            .enabled(enabled)
            .on_click(flip.clone());
        if shown != 2 {
            lock = lock.label(words);
        }
        if shown != 0 {
            lock = lock.icon(Icon::Lock);
        }
        lock
    })
}

pub fn icon_button(ctx: &Ctx) -> Container {
    let small = ctx.bool("Small", false);
    let ghost = ctx.bool("Ghost", false);
    let active = ctx.bool("Active", false);
    let enabled = ctx.bool("Enabled", true);
    let pressed = ctx.action("on_press");
    container().child(move || {
        let size = if small.get() { Size::Sm } else { Size::Md };
        let fill = if ghost.get() {
            Fill::Ghost
        } else {
            Fill::Solid
        };
        let row: Vec<_> = [Icon::Power, Icon::Lock, Icon::Moon, Icon::ChevronRight]
            .into_iter()
            .map(|kind| {
                button::icon_button(kind)
                    .size(size)
                    .fill(fill)
                    .active(active)
                    .enabled(enabled)
                    .on_click(pressed.clone())
            })
            .collect();
        container()
            .layout(Flex::row().spacing(space::S2.px()))
            .children(row)
    })
}

/// As in the row under the island's resting one.
pub fn bar_pill(ctx: &Ctx) -> Container {
    let active = ctx.choice("Active", &["Workspace 1", "Workspace 2", "None"], 0);
    let chosen = ctx.action_with::<usize>("on_press");
    let t = ui();
    container()
        .padding(space::S1.px())
        .layout(centered_row(space::S0_5.px()))
        .children(
            [("1", Icon::Globe), ("2", Icon::Music)]
                .into_iter()
                .enumerate()
                .map(|(index, (name, kind))| {
                    let on = move || active.get() == index;
                    let chosen = chosen.clone();
                    button::pill()
                        .active(on)
                        .on_click(move || {
                            active.set(index);
                            chosen(index);
                        })
                        .child(pill_text(name).active(on).bold(true))
                        .child(icon(kind, icon_box::SMALL.px(), move || {
                            if on() {
                                t.colors.on_accent
                            } else {
                                t.colors.sub
                            }
                        }))
                }),
        )
        .child(
            button::pill()
                .child(icon(Icon::Calendar, icon_box::SMALL.px(), t.colors.sub))
                .child(pill_text("Tuesday 20 October")),
        )
}
