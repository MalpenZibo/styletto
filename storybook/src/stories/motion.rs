use guido::prelude::*;
use styletto::components::button::{Variant, button};
use styletto::components::card::level_figure;
use styletto::components::level::Level;
use styletto::foundations::control::Size;
use styletto::foundations::motion::{self, armed, arriving, shake as shake_frames};
use styletto::foundations::tokens;
use styletto::foundations::tokens::{icon as icon_box, size as space};
use styletto::primitives::island::{self, island as island_shape};
use styletto::primitives::satellite::{balance, satellite as satellite_shape};
use styletto::primitives::text::{caption, heading};
use styletto::{Icon, centered_row, icon, ui};

use super::{CPU_LEVELS, PANEL, TEMP_LEVELS};
use crate::stories::story::Ctx;

pub fn cross_fade(ctx: &Ctx) -> Container {
    let which = create_signal(false);
    let swapped = ctx.action("swap");
    // The views draw no background of their own: the island shape behind
    // them does, as in the shell, so the one arriving never covers the blur
    // of the one leaving.
    let card = move |title: &'static str, note: &'static str| {
        container()
            .layout(Flex::column().spacing(space::S1.px()))
            .child(heading(title))
            .child(caption(note))
    };
    // Each view is centred in a stage of its own size, so neither the one
    // leaving nor the one arriving moves the other when it comes or goes.
    let view = move |title, note| arriving(card(title, note));
    container()
        .layout(
            Flex::column()
                .spacing(space::S6.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(
            island::border(island::paint(container()))
                .width(space::S60.px())
                .height(space::S30.px())
                .corners(tokens::radius::LARGE.corners())
                .layout(ZStack::new().align(Alignment::CENTER))
                .child(move || (!which.get()).then(|| view("Now playing", "Blurs out growing")))
                .child(move || which.get().then(|| view("Wi-Fi", "Fades in from a blur"))),
        )
        .child(
            button()
                .label("Swap")
                .variant(Variant::Accent)
                .on_click(move || {
                    which.set(!which.get_untracked());
                    swapped();
                }),
        )
}

pub fn shake(ctx: &Ctx) -> Container {
    let shakes = create_signal(0u32);
    let generation = create_signal(0u32);
    let rejected = ctx.action("rejected");
    let accepted = ctx.action("accepted");
    let rebuilt = ctx.action("rebuilt");
    container()
        .layout(
            Flex::column()
                .spacing(space::S4.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(move || {
            // Read so that "Rebuild" builds the card again, and arms it again.
            generation.get();
            let ready = armed();
            let (rejected, accepted) = (rejected.clone(), accepted.clone());
            island::border(island::paint(container()))
                .padding([space::S3.px(), space::S4.px()])
                .corners(tokens::radius::LARGE.corners())
                .translate(Translate::NONE.timeline(shake_frames().played_by(shakes)))
                .layout(
                    Flex::column()
                        .spacing(space::S2.px())
                        .cross_alignment(CrossAlignment::Center),
                )
                .child(caption(move || {
                    if ready.get() {
                        "Armed: presses are accepted".to_string()
                    } else {
                        "Arming: presses shake".to_string()
                    }
                }))
                .child(button().label("Press me").on_click(move || {
                    if ready.get_untracked() {
                        accepted();
                    } else {
                        shakes.update(|n| *n += 1);
                        rejected();
                    }
                }))
        })
        .child(
            container()
                .layout(Flex::row().spacing(space::S2.px()))
                .child(
                    button()
                        .label("Shake")
                        .variant(Variant::Danger)
                        .on_click(move || shakes.update(|n| *n += 1)),
                )
                .child(
                    button()
                        .label("Rebuild")
                        .variant(Variant::Accent)
                        .on_click(move || {
                            generation.update(|n| *n += 1);
                            rebuilt();
                        }),
                ),
        )
        .child(caption(format!(
            "Rebuild, then press within {} ms",
            motion::ARM_MS
        )))
}

/// The island at rest, and as a panel.
const ISLAND_HEIGHT: f32 = 34.0;
const PANEL_HEIGHT: f32 = 220.0;
const FIGURE_WIDTH: f32 = 50.0;
const FIGURE_SPACING: f32 = 6.0;

/// What runs hot splits off the island into a bubble of its own, and goes
/// back into it when it cools down or the island opens.
pub fn satellite(ctx: &Ctx) -> Container {
    let cpu = ctx.number("CPU %", 0.0..=100.0, 86.0);
    let temp = ctx.number("Temperature °C", 30.0..=100.0, 76.0);
    let ram = ctx.number("RAM %", 0.0..=100.0, 40.0);
    let open = ctx.bool("Island open", false);
    let width = ctx.number("Island width", 160.0..=360.0, 340.0);
    let gap = ctx.number("Gap", 0.0..=30.0, 8.0);
    let merge = ctx.number("Merge distance", 0.0..=40.0, 18.0);
    let clicked = ctx.action("open system info");

    let hot = create_memo(move || {
        [
            (Icon::Cpu, cpu.get(), "%", CPU_LEVELS),
            (Icon::Thermometer, temp.get(), "°", TEMP_LEVELS),
            (Icon::MemoryStick, ram.get(), "%", (80.0, 90.0)),
        ]
        .into_iter()
        .filter_map(|(kind, value, unit, (warn, alert))| {
            let level = Level::of(value, warn, alert);
            (level != Level::Normal).then(|| (kind, format!("{}{unit}", value.round()), level))
        })
        .collect::<Vec<_>>()
    });
    let out = create_memo(move || hot.with(|hot| !hot.is_empty()) && !open.get());
    let bubble_width = move || {
        let count = hot.with(Vec::len) as f32;
        (count * (FIGURE_WIDTH + FIGURE_SPACING) - FIGURE_SPACING).max(0.0)
    };

    let island = island::paint(container())
        .width(
            (move || Length::exact(if open.get() { PANEL } else { width.get() }))
                .transition(motion::spring()),
        )
        .height(
            (move || {
                Length::exact(if open.get() {
                    PANEL_HEIGHT
                } else {
                    ISLAND_HEIGHT
                })
            })
            .transition(motion::spring()),
        )
        .corners(
            (move || {
                if open.get() {
                    tokens::radius::LARGE.corners()
                } else {
                    tokens::radius::pill(ISLAND_HEIGHT)
                }
            })
            .transition(motion::spring()),
        )
        .padding([0.0, space::S3.px()])
        .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
        .child(move || (!open.get()).then(|| caption("1  2  3   Firefox".to_string())))
        .child(move || (!open.get()).then(|| caption("12:41".to_string())));

    let figures = container()
        .layout(centered_row(FIGURE_SPACING))
        .children(keyed(
            move || hot.with(|hot| hot.iter().map(|(kind, _, _)| *kind).collect::<Vec<_>>()),
            |kind| *kind,
            move |kind| {
                let reading =
                    move || hot.with(|hot| hot.iter().find(|(k, _, _)| *k == kind).cloned());
                container()
                    .width(FIGURE_WIDTH)
                    .layout(Flex::row().main_alignment(MainAlignment::Center))
                    .opacity(
                        1.0.transition(motion::hover())
                            .entering_from(0.0)
                            .exiting_to(0.0),
                    )
                    .child(level_figure(
                        kind,
                        move || reading().map(|(_, figure, _)| figure).unwrap_or_default(),
                        move || reading().map_or(Level::Warn, |(_, _, level)| level),
                    ))
            },
        ));
    let bubble = satellite_shape()
        .width(bubble_width)
        .height(ISLAND_HEIGHT)
        .gap(gap)
        .out(out)
        .on_click(clicked)
        .child(figures);

    container()
        .height(PANEL_HEIGHT)
        .layout(
            Flex::row()
                .main_alignment(MainAlignment::Center)
                .cross_alignment(CrossAlignment::Start),
        )
        .child(
            island::edge(container().merge_children(merge))
                .layout(Flex::row().cross_alignment(CrossAlignment::Start))
                .child(balance(bubble_width))
                .child(island)
                .child(bubble),
        )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Shape {
    Seed,
    Pill,
    Card,
}

const SHAPES: &[&str] = &["Seed", "Pill", "Card"];
const GLOWS: &[&str] = &["None", "Accent", "Success", "Danger"];

/// The island springing between a seed, a pill and a card, shaking, and
/// lit by a state.
pub fn island(ctx: &Ctx) -> Container {
    let shape = ctx.choice("Shape", SHAPES, 1);
    let glow = ctx.choice("Glow", GLOWS, 0);
    let shakes = create_signal(0u32);
    let shook = ctx.action("shake");
    let t = ui();
    let shown = create_memo(move || [Shape::Seed, Shape::Pill, Shape::Card][shape.get()]);
    let seed = space::S8.px();
    let pill = Size::Md.height();
    let inset = space::S3.px();
    container()
        .height(space::S60.px())
        .layout(
            Flex::column()
                .spacing(space::S6.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(
            island_shape(
                shown,
                move |shown| match shown {
                    Shape::Seed => container().width(seed).height(seed),
                    Shape::Pill => container()
                        .height(pill)
                        .padding([0.0, space::S4.px()])
                        .layout(centered_row(space::S2.px()))
                        .child(icon(Icon::Music, icon_box::SMALL.px(), t.colors.text))
                        .child(caption("Now playing")),
                    Shape::Card => container()
                        .width(space::S60.px())
                        .padding(inset)
                        .layout(Flex::column().spacing(space::S1.px()))
                        .child(heading("Wi-Fi"))
                        .child(caption("Home 5G, connected"))
                        .child(caption("Two other networks in range")),
                },
                move || match shown.get() {
                    Shape::Seed => tokens::radius::pill(seed),
                    Shape::Pill => tokens::radius::pill(pill),
                    Shape::Card => tokens::radius::shape(tokens::radius::around(
                        tokens::radius::SMALL.px(),
                        inset,
                    )),
                },
            )
            .translate(Translate::NONE.timeline(shake_frames().played_by(shakes)))
            .shadow(
                (move || {
                    island::shadow(match glow.get() {
                        1 => t.colors.accent,
                        2 => t.colors.success,
                        3 => t.colors.danger,
                        _ => Color::TRANSPARENT,
                    })
                })
                .transition(motion::color()),
            ),
        )
        .child(button().label("Shake").on_click(move || {
            shakes.update(|n| *n += 1);
            shook();
        }))
}
