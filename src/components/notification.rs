//! Notifications, one by one and grouped by application.

use guido::prelude::*;

use crate::components::button::{Fill, button};
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon, icon};
use crate::foundations::motion;
use crate::foundations::tokens::{
    TileSize, icon as icon_box, lift, opacity, radius, size as space, stroke, tile,
};
use crate::foundations::typography::{BODY_SMALL, LABEL_MEDIUM, LABEL_XSMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::theme::palette;
use crate::ui::ui;

/// One notification. The sender is the caller's to show, as a lead or a
/// group's header. The close button shows on hover.
#[component]
pub fn notification(
    #[prop(required)] title: String,
    body: String,
    /// When it came: "now", "5m".
    time: String,
    urgent: bool,
    /// The sender, before it.
    #[prop(slot)]
    lead: (),
    #[prop(callback)] on_dismiss: (),
    #[prop(callback)] on_click: (),
) -> impl Widget {
    let t = ui();
    let hovered = create_signal(false);
    let top = container()
        .width(fill())
        .layout(centered_row(space::S2.px()))
        .child(
            container().width(fill()).child(
                text(title)
                    .typeset(LABEL_MEDIUM.em())
                    .color(t.colors.text)
                    .nowrap()
                    .overflow(TextOverflow::Ellipsis),
            ),
        )
        .child(
            text(time)
                .typeset(LABEL_XSMALL)
                .color(t.colors.sub)
                .nowrap(),
        );
    let close = container()
        .opacity((move || if hovered.get() { 1.0 } else { 0.0 }).transition(motion::hover()))
        .child(
            button()
                .icon(Icon::X)
                .size(Size::Xs)
                .fill(Fill::Ghost)
                .on_click(on_dismiss),
        );
    let content = container()
        .width(fill())
        .padding([space::S2.px(), space::S3.px()])
        .layout(Flex::row().spacing(space::S3.px()))
        .children(lead)
        .child(
            container()
                .width(fill())
                .layout(Flex::column().spacing(space::S0_5.px()))
                .child(top)
                .child(move || {
                    body.with(|body| !body.is_empty()).then(|| {
                        text(body)
                            .typeset(BODY_SMALL)
                            .color(t.colors.sub)
                            .max_lines(Some(3))
                            .overflow(TextOverflow::Ellipsis)
                    })
                }),
        )
        .child(close);
    // The urgent mark: a red bar down the left edge, clipped by the corners,
    // so the content doesn't move.
    let urgent = container()
        .visible(urgent)
        .width(stroke::BOLD)
        .height(fill())
        .background(t.colors.danger.fill);

    container()
        .width(fill())
        .corners(radius::shape(corners()))
        .overflow(Overflow::Hidden)
        .background(Color::TRANSPARENT.transition(motion::hover()))
        .when_hovered(move |s| s.background(t.colors.surface))
        .on_hover(move |h| hovered.set(h))
        .on_click(on_click)
        .layout(ZStack::new())
        .child(content)
        .child(urgent)
}

fn corners() -> f32 {
    radius::MEDIUM.px()
}
fn group_padding() -> f32 {
    space::S1.px()
}

/// An app's notifications under a header with its name and count.
#[component]
pub fn notification_group(
    #[prop(required)] app: String,
    count: usize,
    /// The caller passes only the newest; the others' edge shows under it.
    folded: bool,
    #[prop(slot)] lead: (),
    /// Called when the header is pressed.
    #[prop(callback)]
    on_toggle: (),
    #[prop(children)] children: (),
) -> impl Widget {
    let t = ui();
    let head = container()
        .padding([
            space::S2.px(),
            space::S2.px(),
            space::S1.px(),
            space::S2.px(),
        ])
        .layout(centered_row(space::S2.px()))
        .on_click(on_toggle)
        .children(lead)
        .child(
            container().width(fill()).child(
                text(move || format!("{} · {}", app.get(), count.get()))
                    .typeset(LABEL_XSMALL.em())
                    .color(t.colors.text)
                    .nowrap()
                    .overflow(TextOverflow::Ellipsis),
            ),
        )
        .child(move || {
            (count.get() > 1).then(|| {
                let kind = if folded.get() {
                    Icon::ChevronDown
                } else {
                    Icon::ChevronLeft
                };
                icon(kind, icon_box::XSMALL.px(), t.colors.dim)
            })
        });
    let edge = t.colors.surface.scale_alpha(opacity::STACKED);
    container()
        .width(fill())
        .layout(Flex::column())
        .child(
            container()
                .width(fill())
                .padding(group_padding())
                .corners(radius::shape(radius::around(corners(), group_padding())))
                .background(t.colors.surface)
                .layout(Flex::column())
                .child(head)
                .children(children),
        )
        .child(move || {
            folded.get().then(|| {
                container()
                    .width(fill())
                    .padding([0.0, space::S4.px()])
                    .child(
                        container()
                            .width(fill())
                            .height(space::S1.px())
                            .corners(radius::shape([
                                0.0,
                                0.0,
                                radius::SMALL.px(),
                                radius::SMALL.px(),
                            ]))
                            .background(edge),
                    )
            })
        })
}

/// An application's icon on a tile of its colour.
#[component]
pub fn app_icon(
    #[prop(required)] icon: Icon,
    #[prop(required)] color: Color,
    #[prop(default = tile::MEDIUM)] size: TileSize,
) -> impl Widget {
    let tile = size.get_untracked();
    let size = tile.px();
    let color = color.get_untracked();
    container()
        .width(size)
        .height(size)
        .corners(tile.radius().corners())
        .gradient(LinearGradient::new(
            color.lighter(lift::SHEEN),
            color.darker(lift::SHEEN),
            GradientDirection::Diagonal,
        ))
        .layout(Flex::column().center())
        .child(icons::icon(
            icon,
            tile.icon(),
            palette::black_or_white(color),
        ))
}
