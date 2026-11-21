//! Quick-settings tiles: a pill on the surface while off, filled with the
//! accent and squircle-cornered while on.

use guido::prelude::*;

use crate::components::button::{hovered, pressable};
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::{radius, size};
use crate::foundations::typography::{LABEL_MEDIUM, LABEL_SMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::ui::{active_fill, active_ink, flip, ui};

fn icon_box() -> f32 {
    Size::Lg.height()
}
/// Inset from the tile's edges.
fn inset() -> f32 {
    size::S3.px()
}
fn height() -> f32 {
    icon_box() + 2.0 * inset()
}

/// A pill while off, a squircle of `on_radius` while on.
fn shape(on: Signal<bool>, height: f32, on_radius: f32) -> impl Fn() -> Corners + Copy {
    move || {
        if on.get() {
            radius::shape(on_radius)
        } else {
            radius::pill(height)
        }
    }
}

/// The icon box's radius, concentric with the tile's at `inset`.
fn box_radius() -> f32 {
    radius::within(radius::MEDIUM.px(), inset())
}

/// A wide tile with two targets, as on Android: the icon toggles, the rest
/// opens the details.
///
/// As tall as a small tile, unless its subtitle takes two lines. In a row
/// that stretches its children, as a `Flex::row()` does by default, the
/// tiles side by side are all as tall as the tallest.
#[component]
pub fn tile(
    #[prop(required)] icon: Icon,
    #[prop(required)] title: String,
    /// Empty, the title stands alone in the middle of the tile. A long one
    /// takes a second line, then is cut with an ellipsis.
    subtitle: String,
    on: bool,
    #[prop(callback)] on_toggle: fn(bool),
    /// Without it, the whole tile toggles.
    #[prop(callback)]
    on_open: (),
) -> impl Widget {
    let t = ui();
    let ink = active_ink(on);
    let toggle = flip(on, on_toggle);
    let label_click = move || match on_open {
        Some(open) => open.run(),
        None => toggle(),
    };

    let icon_box = pressable(
        container()
            .width(icon_box())
            .height(icon_box())
            .corners(shape(on, icon_box(), box_radius()).transition(motion::spring()))
            .background(
                (move || {
                    if on.get() {
                        Color::BLACK.with_alpha(0.13)
                    } else {
                        t.colors.surface2
                    }
                })
                .transition(motion::color()),
            )
            .when_hovered(move |s| {
                s.background(move || {
                    if on.get() {
                        Color::BLACK.with_alpha(0.2)
                    } else {
                        hovered(t.colors.surface2)
                    }
                })
            }),
        motion::PRESS_SCALE_INNER,
    )
    .layout(Flex::column().center())
    .on_click(toggle)
    .child(icons::icon(icon, Size::Md.icon(), ink));

    // An empty subtitle takes no line, so the title sits in the middle. A
    // memo, so that the line is rebuilt only when it comes or goes. As tall
    // as its lines, the label is centred on the icon box by the tile's row,
    // whether the subtitle takes one line or two.
    let has_subtitle = create_memo(move || subtitle.with(|s| !s.is_empty()));
    let label = container()
        .width(fill())
        .padding([0.0, size::S4.px(), 0.0, inset()])
        .layout(Flex::column().spacing(size::S0_5.px()))
        .child(
            text(title)
                .typeset(LABEL_MEDIUM.em())
                .color(ink)
                .nowrap()
                .overflow(TextOverflow::Ellipsis),
        )
        .child(move || {
            has_subtitle.get().then(|| {
                text(subtitle)
                    .typeset(LABEL_SMALL)
                    // On a fill the on-colour is the only one that reads; off
                    // it, the secondary text.
                    .color(move || if on.get() { ink() } else { t.colors.sub })
                    // A long one, as a German one often is, takes a second
                    // line before it is cut.
                    .max_lines(Some(2))
                    .overflow(TextOverflow::Ellipsis)
            })
        });

    // The icon box or the label, whichever is taller, with the inset around
    // it: `height()` unless the subtitle takes two lines. No fixed height, so
    // a row that stretches its children gives the tiles in it the height of
    // the tallest. The corners stay those of `height()`, as the neighbours'.
    pressable(
        container()
            .width(fill())
            .corners(shape(on, height(), radius::MEDIUM.px()).transition(motion::spring()))
            .background(active_fill(on).transition(motion::color()))
            .overflow(Overflow::Hidden),
        motion::PRESS_SCALE_WIDE,
    )
    .layout(centered_row(0.0))
    .padding([inset(), 0.0, inset(), inset()])
    .on_click(label_click)
    .child(icon_box)
    .child(label)
}

/// A compact icon-only tile; the whole tile toggles.
#[component]
pub fn small_tile(
    #[prop(required)] icon: Icon,
    on: bool,
    #[prop(callback)] on_toggle: fn(bool),
) -> impl Widget {
    let paint = active_fill(on);
    pressable(
        container()
            .width(fill())
            // At least, so that beside a wide tile whose subtitle takes two
            // lines it can be stretched to its height.
            .height(at_least(height()))
            .corners(shape(on, height(), radius::MEDIUM.px()).transition(motion::spring()))
            .background(paint.transition(motion::color()))
            .when_hovered(move |s| s.background(move || hovered(paint()))),
        motion::PRESS_SCALE,
    )
    .layout(Flex::column().center())
    .on_click(flip(on, on_toggle))
    .child(icons::icon(icon, Size::Md.icon(), active_ink(on)))
}
