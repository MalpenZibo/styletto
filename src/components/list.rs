//! Lists, the pages a panel navigates between, and where a flow of them
//! stands.

use std::cell::Cell;
use std::rc::Rc;

use guido::prelude::*;

use crate::components::button::{Fill, button};
use crate::components::tag::Tone;
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::{icon as icon_box, indicator, opacity, radius, size, stroke};
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
    /// The choice in use: selects the row and ends it with a check, after
    /// the trailing widget.
    checked: bool,
    /// Draws the title and the icon in the tone's hue as text, and tints the
    /// hover and the press with it, as a row whose action needs care
    /// (`Warning`) or ends something (`Danger`). The subtitle, the
    /// selection and the trailing widget keep their colours.
    tone: Tone,
    #[prop(default = true)] enabled: bool,
    #[prop(slot)] trailing: (),
    #[prop(callback)] on_click: (),
) -> impl Widget {
    let t = ui();
    let selected = move || selected.get() || checked.get();
    let hue = move || tone.get().hue().map(|hue| t.colors.hue(hue).text);
    // With a tone, the hover and the ripple are `surface` and `ripple` with
    // the hue in place of their white or black, as faint: a hue is never
    // brighter than white nor darker than black, so the hovered title reads
    // at least as well as it does on `surface`.
    let tinted = move |neutral: Color| hue().map_or(neutral, |hue| hue.with_alpha(neutral.a));
    let rest = move || {
        if selected() {
            t.colors.surface2
        } else {
            Color::TRANSPARENT
        }
    };
    let words = container()
        .width(fill())
        .layout(Flex::column())
        .child(
            label(title)
                .color(move || hue().unwrap_or(t.colors.text))
                .overflow(TextOverflow::Ellipsis),
        )
        .child(subtitle_line(subtitle));
    container()
        .width(fill())
        .padding([size::S2.px(), size::S3.px()])
        .corners(radius::SMALL.corners())
        .background(rest.transition(motion::hover()))
        .when_hovered(move |s| {
            s.background(move || {
                if selected() {
                    t.colors.surface2
                } else {
                    tinted(t.colors.surface)
                }
            })
        })
        .when_pressed(move |s| s.ripple_with_color(move || tinted(t.colors.ripple)))
        .opacity(1.0.transition(motion::hover()))
        .when_disabled(|s| s.opacity(opacity::DISABLED).background(Color::TRANSPARENT))
        .enabled(enabled)
        .on_click(on_click)
        .layout(centered_row(size::S3.px()))
        .child(move || {
            icon.get().map(|kind| {
                icons::icon(kind, icon_box::SMALL.px(), move || {
                    hue().unwrap_or(t.colors.sub)
                })
            })
        })
        .child(words)
        .children(trailing)
        .child(move || {
            checked
                .get()
                .then(|| icons::icon(Icon::Check, icon_box::SMALL.px(), t.colors.accent.text))
        })
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
                .color(t.colors.sub)
                .nowrap(),
        )
}

/// A page's title row: back button, title, subtitle and a trailing widget.
/// With a subtitle the row grows to hold both lines.
#[component]
pub fn page_header(
    #[prop(required)] title: String,
    /// Where the page's feature stands. Empty means none.
    subtitle: String,
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
            button()
                .icon(Icon::ChevronLeft)
                .size(Size::Sm)
                .fill(Fill::Ghost)
                .on_click(back)
        }))
        .child(
            container()
                .width(fill())
                .layout(Flex::column())
                .child(
                    text(title)
                        .typeset(TITLE_XSMALL.em())
                        .color(t.colors.text)
                        .nowrap()
                        .overflow(TextOverflow::Ellipsis),
                )
                .child(subtitle_line(subtitle)),
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

/// Where a flow of pages stands, as a first start's steps: a dot for each
/// page in `dim`, the current one stretched into a short pill in the
/// accent as text, springing to the next as `current` moves. The palette
/// holds both to 3:1 on the island whatever shows through it; the accent's
/// fill falls to 1.3:1 on glass (Catppuccin Latte's rosewater). It shows
/// and takes no press: the flow's Back and Next buttons move it.
#[component]
pub fn page_indicator(
    /// How many pages there are.
    #[prop(required)]
    count: usize,
    /// The page showing, from 0; one past the last shows as the last.
    #[prop(required)]
    current: usize,
) -> impl Widget {
    let t = ui();
    // Stretching is a movement: reduced motion swaps the pill over at once
    // and only fades the colours.
    let moves = motion::moves();
    // A page past the last shows as the last, rather than as none.
    let current = move || current.get().min(count.get().saturating_sub(1));
    container()
        .layout(centered_row(indicator::DOT))
        .children(move || {
            (0..count.get()).map(move |page| {
                let on = move || current() == page;
                container()
                    .width(
                        (move || {
                            if on() {
                                indicator::CURRENT
                            } else {
                                indicator::DOT
                            }
                        })
                        .transition(if moves {
                            motion::spring()
                        } else {
                            motion::instant()
                        }),
                    )
                    .height(indicator::DOT)
                    .corners(radius::pill(indicator::DOT))
                    .background(
                        (move || {
                            if on() {
                                t.colors.accent.text
                            } else {
                                t.colors.dim
                            }
                        })
                        .transition(motion::color()),
                    )
            })
        })
}

/// A list of rows that grows to `max_height`, then scrolls. Its scrollbar is
/// the theme's: a thin pill in `dim` that brightens and widens under the
/// pointer, drawn over the rows' ends unless `reserve_gutter` gives it a
/// lane of its own.
#[component]
pub fn scroll_list(
    /// The height past which it scrolls.
    #[prop(required)]
    max_height: f32,
    /// Keeps a lane for the scrollbar, so it never covers what a row has at
    /// its end; the rows are narrower by it, scrolling or not.
    reserve_gutter: bool,
    #[prop(children)] children: (),
) -> impl Widget {
    use crate::foundations::tokens::scrollbar;

    let t = ui();
    // The growth under the pointer is a movement: reduced motion keeps the
    // bar as it is and only brightens it.
    let grown = if motion::moves() {
        scrollbar::HOVER_WIDTH
    } else {
        scrollbar::WIDTH
    };
    let bar = || container().corners(radius::pill(scrollbar::WIDTH));
    container()
        .width(fill())
        .height(move || at_most(max_height.get()))
        .layout(Flex::column().spacing(size::S0_5.px()))
        .scroll(
            Scroll::vertical()
                .width(scrollbar::WIDTH)
                .hover_width(grown)
                .margin(scrollbar::MARGIN)
                .min_handle_size(scrollbar::MIN_HANDLE)
                .reserve_gutter(reserve_gutter)
                .track(move |_| bar().background(t.colors.surface))
                .handle(move |_| {
                    bar()
                        .background(t.colors.dim.transition(motion::hover()))
                        .when_hovered(move |s| s.background(t.colors.sub))
                        .when_pressed(move |s| s.background(t.colors.text))
                }),
        )
        .children(children)
}

/// A caption under a title while `subtitle` holds something; none while it
/// is empty.
fn subtitle_line(subtitle: Signal<String>) -> impl Fn() -> Option<Text> + 'static {
    move || {
        (!subtitle.with(String::is_empty))
            .then(|| caption(subtitle).overflow(TextOverflow::Ellipsis))
    }
}
