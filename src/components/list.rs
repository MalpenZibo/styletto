//! Lists and the pages a panel navigates between.

use std::cell::Cell;
use std::rc::Rc;

use guido::prelude::*;

use crate::components::button::{Fill, icon_button};
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::{icon as icon_box, opacity, radius, size, stroke};
use crate::foundations::typography::{LABEL_MEDIUM, TITLE_XSMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::primitives::text::{caption, label, overline};
use crate::ui::ui;

/// A list row: icon, title, subtitle and a trailing widget, each optional.
#[component]
pub fn row(
    icon: Option<Icon>,
    #[prop(required)] title: String,
    /// Empty means none.
    subtitle: String,
    selected: bool,
    #[prop(default = true)] enabled: bool,
    #[prop(slot)] trailing: (),
    #[prop(callback)] on_click: (),
) -> impl Widget {
    let t = ui();
    let rest = move || {
        if selected.get() {
            t.colors.surface2
        } else {
            Color::TRANSPARENT
        }
    };
    let words = container()
        .width(fill())
        .layout(Flex::column())
        .child(label(title).overflow(TextOverflow::Ellipsis))
        .child(move || {
            (!subtitle.with(String::is_empty))
                .then(|| caption(subtitle).overflow(TextOverflow::Ellipsis))
        });
    container()
        .width(fill())
        .padding([size::S2.px(), size::S3.px()])
        .corners(radius::SMALL.corners())
        .background(rest.transition(motion::hover()))
        .when_hovered(move |s| {
            s.background(move || {
                if selected.get() {
                    t.colors.surface2
                } else {
                    t.colors.surface
                }
            })
        })
        .when_pressed(move |s| s.ripple_with_color(t.colors.ripple))
        .opacity(1.0.transition(motion::hover()))
        .when_disabled(|s| s.opacity(opacity::DISABLED).background(Color::TRANSPARENT))
        .enabled(enabled)
        .on_click(on_click)
        .layout(centered_row(size::S3.px()))
        .child(move || {
            icon.get()
                .map(|kind| icons::icon(kind, icon_box::SMALL.px(), t.colors.sub))
        })
        .child(words)
        .children(trailing)
}

/// A small uppercase label over a group of rows.
#[component]
pub fn section(#[prop(required)] title: String) -> impl Widget {
    container()
        .padding([size::S2.px(), size::S3.px(), size::S1.px(), size::S3.px()])
        .child(overline(title))
}

#[component]
pub fn separator() -> impl Widget {
    let t = ui();
    container()
        .width(fill())
        .padding([size::S1.px(), size::S2.px()])
        .child(
            container()
                .width(fill())
                .height(stroke::HAIRLINE)
                .background(t.colors.line),
        )
}

/// What an empty list shows.
#[component]
pub fn empty(#[prop(required)] icon: Icon, #[prop(required)] message: String) -> impl Widget {
    let t = ui();
    container()
        .width(fill())
        .padding([size::S6.px(), 0.0])
        .layout(
            Flex::column()
                .spacing(size::S2.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(icons::icon(icon, icon_box::LARGE.px(), t.colors.dim))
        .child(
            text(message)
                .typeset(LABEL_MEDIUM)
                .color(t.colors.dim)
                .nowrap(),
        )
}

/// A page's title row: back button, title and a trailing widget.
#[component]
pub fn page_header(
    #[prop(required)] title: String,
    /// Shows the back button, which calls it.
    #[prop(callback)]
    on_back: (),
    #[prop(slot)] trailing: (),
) -> impl Widget {
    let t = ui();
    container()
        .width(fill())
        .height(at_least(Size::Sm.height()))
        .layout(centered_row(size::S2.px()))
        .children(on_back.map(|back| {
            icon_button(Icon::ChevronLeft)
                .size(Size::Sm)
                .fill(Fill::Ghost)
                .on_click(back)
        }))
        .child(
            container().width(fill()).child(
                text(title)
                    .typeset(TITLE_XSMALL.em())
                    .color(t.colors.text)
                    .nowrap()
                    .overflow(TextOverflow::Ellipsis),
            ),
        )
        .children(trailing)
}

/// A page header's back action: pops `stack`.
pub fn back<P: Clone + PartialEq + 'static>(stack: RwSignal<Vec<P>>) -> impl Fn() + 'static {
    move || {
        stack.update(|pages| {
            pages.pop();
        })
    }
}

/// A panel's pages as a stack; the last shows. A push slides in from the
/// right, a pop from the left.
#[component]
pub fn pages<P>(
    #[prop(required)] stack: Vec<P>,
    #[prop(required, callback)] render: fn(P) -> impl Widget,
) -> impl Widget
where
    P: Clone + PartialEq + 'static,
{
    let depth = Rc::new(Cell::new(stack.get_untracked().len()));
    container().width(fill()).child(move || {
        let (len, page) = stack.with(|s| (s.len(), s.last().cloned()));
        let was = depth.replace(len);
        let from = if len >= was {
            size::S4.px()
        } else {
            -size::S4.px()
        };
        let page = page.map(|page| render.run(page));
        container()
            .width(fill())
            .opacity(1.0.transition(motion::color()).entering_from(0.0))
            .translate(
                Translate::NONE
                    .transition(motion::spring())
                    .entering_from(Translate::new(from, 0.0)),
            )
            .children(page)
    })
}
