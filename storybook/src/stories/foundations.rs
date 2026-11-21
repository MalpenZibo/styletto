use crate::chrome::appearance::{DENSITIES, SHAPES};
use guido::prelude::*;
use styletto::components::button::{Variant, button};
use styletto::components::list::{page_header, row};
use styletto::components::slider::slider_row;
use styletto::components::tag::tag;
use styletto::components::tile::small_tile;
use styletto::components::toggle::toggle;
use styletto::foundations::control::Size;
use styletto::foundations::tokens::{radius, size, stroke};
use styletto::foundations::typography as ty;
use styletto::primitives::island;
use styletto::primitives::text::{caption, typed};
use styletto::theme::palette::Hue;
use styletto::{Icon, centered_row, icon, provide_ui, ui};

use super::{ICON_SIZES, ICON_STEPS};
use crate::stories::story::Ctx;

/// Every colour the components read, then each fill with the colour that
/// reads on it, and each hue as text on the island.
pub fn colors(_: &Ctx) -> Container {
    let c = ui().colors;
    container()
        .layout(Flex::column().spacing(size::S6.px()))
        .child(grid(
            c.named()
                .into_iter()
                .map(|(name, color)| {
                    named(
                        name,
                        container()
                            .width(size::S15.px())
                            .height(size::S10.px())
                            .corners(radius::SMALL.corners())
                            .background(color)
                            .border(stroke::HAIRLINE, c.line),
                    )
                })
                .collect(),
        ))
        .child(
            container()
                .layout(centered_row(size::S3.px()))
                .children(Hue::ALL.map(|hue| {
                    let colors = c.hue(hue);
                    container()
                        .padding([size::S2.px(), size::S4.px()])
                        .corners(radius::SMALL.corners())
                        .background(colors.fill)
                        .child(typed(format!("{hue:?}"), ty::LABEL_MEDIUM.em()).color(colors.on))
                })),
        )
        .child(
            container()
                .layout(centered_row(size::S3.px()))
                .children(Hue::ALL.map(|hue| {
                    container().padding([size::S2.px(), size::S4.px()]).child(
                        typed(format!("{hue:?}"), ty::LABEL_MEDIUM.em()).color(c.hue(hue).text),
                    )
                })),
        )
}

const CATEGORIES: &[&str] = &["Display", "Title", "Label", "Body", "Micro", "Mono"];
const SIZES: [&str; 4] = ["large", "medium", "small", "xsmall"];
const SCALE: [[ty::Type; 4]; 6] = [
    [
        ty::DISPLAY_LARGE,
        ty::DISPLAY_MEDIUM,
        ty::DISPLAY_SMALL,
        ty::DISPLAY_XSMALL,
    ],
    [
        ty::TITLE_LARGE,
        ty::TITLE_MEDIUM,
        ty::TITLE_SMALL,
        ty::TITLE_XSMALL,
    ],
    [
        ty::LABEL_LARGE,
        ty::LABEL_MEDIUM,
        ty::LABEL_SMALL,
        ty::LABEL_XSMALL,
    ],
    [
        ty::BODY_LARGE,
        ty::BODY_MEDIUM,
        ty::BODY_SMALL,
        ty::BODY_XSMALL,
    ],
    [
        ty::MICRO_LARGE,
        ty::MICRO_MEDIUM,
        ty::MICRO_SMALL,
        ty::MICRO_XSMALL,
    ],
    [
        ty::MONO_LARGE,
        ty::MONO_MEDIUM,
        ty::MONO_SMALL,
        ty::MONO_XSMALL,
    ],
];

/// Every category in its four sizes, each with its size and line height,
/// the line drawn behind it.
pub fn typography(ctx: &Ctx) -> Container {
    let sample = ctx.text("Sample", "Wi-Fi 12:45 87%");
    let em = ctx.bool("Emphasised", false);
    let lines = ctx.bool("Show line boxes", false);
    let t = ui();
    container().child(move || {
        let emphasised = em.get();
        container()
            .layout(Flex::column().spacing(size::S6.px()))
            .children(CATEGORIES.iter().zip(SCALE).map(move |(category, kinds)| {
                container()
                    .layout(Flex::column().spacing(size::S2.px()))
                    .child(typed(*category, ty::MICRO_XSMALL.em()).color(t.colors.sub))
                    .children(SIZES.into_iter().zip(kinds).map(move |(name, kind)| {
                        let kind = if emphasised { kind.em() } else { kind };
                        container()
                            .layout(centered_row(size::S4.px()))
                            .child(container().width(size::S20.px()).child(caption({
                                let drawn = kind.resolve();
                                format!("{name} {}/{}", drawn.size, drawn.line)
                            })))
                            .child(
                                container()
                                    .background(move || {
                                        if lines.get() {
                                            t.colors.surface
                                        } else {
                                            Color::TRANSPARENT
                                        }
                                    })
                                    .child(typed(sample, kind).color(t.colors.text).nowrap()),
                            )
                    }))
            }))
    })
}

/// The one scale every gap, padding and length is taken from.
pub fn scale(_: &Ctx) -> Container {
    let t = ui();
    let steps = [
        ("S0_5", size::S0_5.px()),
        ("S1", size::S1.px()),
        ("S2", size::S2.px()),
        ("S3", size::S3.px()),
        ("S4", size::S4.px()),
        ("S6", size::S6.px()),
        ("S8", size::S8.px()),
        ("S10", size::S10.px()),
        ("S15", size::S15.px()),
        ("S20", size::S20.px()),
        ("S30", size::S30.px()),
        ("S40", size::S40.px()),
    ];
    container()
        .layout(Flex::column().spacing(size::S2.px()))
        .children(steps.into_iter().map(move |(name, step)| {
            container()
                .layout(centered_row(size::S4.px()))
                .child(
                    container()
                        .width(size::S20.px())
                        .child(caption(format!("{name} · {step}"))),
                )
                .child(
                    container()
                        .width(step)
                        .height(size::S3.px())
                        .corners(radius::XSMALL.corners())
                        .background(t.colors.accent.fill),
                )
        }))
}

/// The corners: a box's at each step, continuous as a squircle's, and a
/// pill's round ends.
pub fn radii(_: &Ctx) -> Container {
    let t = ui();
    let steps = [
        ("XSMALL", radius::XSMALL),
        ("SMALL", radius::SMALL),
        ("MEDIUM", radius::MEDIUM),
        ("LARGE", radius::LARGE),
    ];
    let shape = move |corners: Corners, width: f32| {
        container()
            .width(width)
            .height(size::S20.px())
            .corners(corners)
            .background(t.colors.surface2)
    };
    container()
        .layout(centered_row(size::S6.px()))
        .children(steps.into_iter().map(move |(name, step)| {
            named(
                format!("{name} · {}", step.px()),
                shape(step.corners(), size::S20.px()),
            )
        }))
        .child(named(
            "pill",
            shape(radius::pill(size::S20.px()), size::S30.px()),
        ))
}

const CONTROLS: [(&str, Size); 5] = [
    ("Xxs", Size::Xxs),
    ("Xs", Size::Xs),
    ("Sm", Size::Sm),
    ("Md", Size::Md),
    ("Lg", Size::Lg),
];

/// Every control size: its height, its icon and its label, as an icon
/// button, a button and a tag.
pub fn controls(_: &Ctx) -> Container {
    let t = ui();
    container()
        .layout(Flex::column().spacing(size::S4.px()))
        .children(CONTROLS.into_iter().map(move |(name, control)| {
            container()
                .layout(centered_row(size::S4.px()))
                .child(container().width(size::S20.px()).child(caption(format!(
                    "{name} · {} / {}",
                    control.height(),
                    control.icon()
                ))))
                .child(button().icon(Icon::Bell).size(control))
                .child(
                    button()
                        .label("Lock")
                        .icon(Icon::Lock)
                        .variant(Variant::Accent)
                        .size(control),
                )
                .child(
                    tag()
                        .size(control)
                        .child(icon(Icon::Download, control.icon(), t.colors.accent.text))
                        .child(
                            typed("12", control.label().em())
                                .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                                .color(t.colors.accent.text)
                                .nowrap(),
                        ),
                )
        }))
}

pub fn icons(ctx: &Ctx) -> Container {
    let side = ctx.choice("Size", ICON_STEPS, 1);
    let colored = ctx.bool("Accent", false);
    let t = ui();
    let color = move || {
        if colored.get() {
            t.colors.accent.text
        } else {
            t.colors.text
        }
    };
    container().child(move || {
        let side = ICON_SIZES[side.get()].px();
        grid(
            Icon::ALL
                .iter()
                .map(|&kind| {
                    container()
                        .width(size::S30.px())
                        .child(named(format!("{kind:?}"), icon(kind, side, color)))
                })
                .collect(),
        )
    })
}

/// `shown` over its name.
fn named(name: impl Into<String>, shown: impl Widget + 'static) -> Container {
    container()
        .layout(
            Flex::column()
                .spacing(size::S2.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(shown)
        .child(
            typed(name.into(), ty::LABEL_XSMALL)
                .color(ui().colors.sub)
                .nowrap()
                .overflow(TextOverflow::Ellipsis),
        )
}

/// The cells in rows, as many to a row as fit.
fn grid(cells: Vec<Container>) -> Container {
    container()
        .width(fill())
        .layout(
            Flex::row()
                .spacing(size::S3.px())
                .line_spacing(size::S4.px())
                .wrap(true),
        )
        .children(cells)
}

/// The same piece of a panel in every density and shape, side by side, so
/// a change to a token shows wherever it lands.
pub fn matrix(_: &Ctx) -> Container {
    let base = ui();
    container()
        .layout(Flex::column().spacing(size::S4.px()))
        .children(DENSITIES.iter().map(move |&(density_name, density)| {
            container()
                .layout(Flex::row().spacing(size::S4.px()))
                .children(SHAPES.iter().map(move |&(shape_name, shape)| {
                    let drawn = base.with_density(density).with_shape(shape);
                    container().child(move || {
                        provide_ui(drawn);
                        named(format!("{density_name} · {shape_name}"), sample())
                    })
                }))
        }))
}

/// A header, a row, a slider and the controls a panel holds.
fn sample() -> Container {
    island::border(island::paint(container()))
        .width(size::S60.px())
        .padding(size::S2.px())
        .corners(radius::shape(radius::around(
            radius::MEDIUM.px(),
            size::S2.px(),
        )))
        .layout(Flex::column().spacing(size::S1.px()))
        .child(page_header("Wi-Fi").on_back(|| {}))
        .child(
            row("Home 5G")
                .icon(Icon::Wifi)
                .subtitle("Connected")
                .selected(true)
                .trailing(toggle(true)),
        )
        .child(slider_row(Icon::Volume, 60.0))
        .child(
            container()
                .layout(centered_row(size::S1.px()))
                .child(button().label("Join").variant(Variant::Accent))
                .child(button().label("Auto").active(true))
                .child(button().icon(Icon::Refresh)),
        )
        .child(
            container()
                .width(fill())
                .layout(Flex::row().spacing(size::S1.px()))
                .child(
                    container()
                        .width(fill())
                        .child(small_tile(Icon::Moon).active(true)),
                )
                .child(container().width(fill()).child(small_tile(Icon::Bell))),
        )
}
