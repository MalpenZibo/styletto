//! Quick-settings tiles: a pill on the surface while off, filled with the
//! accent and squircle-cornered while on.

use guido::prelude::*;

use crate::components::button::{hovered, pressable, toggled};
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::{radius, shade, size};
use crate::foundations::typography::{LABEL_MEDIUM, LABEL_SMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::theme::palette;
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
fn shape(active: Signal<bool>, height: f32, on_radius: f32) -> impl Fn() -> Corners + Copy {
    move || {
        if active.get() {
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
/// On while `active`; a press that toggles calls `on_toggle` with the
/// opposite, and the owner sets `active` to what it keeps.
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
    active: bool,
    #[prop(callback)] on_toggle: fn(bool),
    /// Without it, the whole tile toggles.
    #[prop(callback)]
    on_open: (),
) -> impl Widget {
    let t = ui();
    let ink = active_ink(active, t.colors.text);
    let toggle = flip(active, on_toggle);
    let label_click = move || match on_open {
        Some(open) => open.run(),
        None => toggle(),
    };

    let rest = move || {
        if active.get() {
            palette::SHADE.with_alpha(shade::REST)
        } else {
            t.colors.surface2
        }
    };
    let lifted = move || {
        if active.get() {
            palette::SHADE.with_alpha(shade::HOVER)
        } else {
            hovered(t.colors.surface2)
        }
    };
    let icon_box = pressable(
        toggled(
            container()
                .width(icon_box())
                .height(icon_box())
                .corners(shape(active, icon_box(), box_radius()).transition(motion::spring())),
            rest,
            lifted,
        ),
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
                    .color(active_ink(active, t.colors.sub))
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
    //
    // Under the pointer the body lights up as a small tile does.
    let body = active_fill(active, t.colors.surface);
    pressable(
        toggled(
            container()
                .width(fill())
                .corners(shape(active, height(), radius::MEDIUM.px()).transition(motion::spring()))
                .overflow(Overflow::Hidden),
            body,
            move || hovered(body()),
        ),
        motion::PRESS_SCALE_WIDE,
    )
    .layout(centered_row(0.0))
    .padding([inset(), 0.0, inset(), inset()])
    .on_click(label_click)
    .child(icon_box)
    .child(label)
}

/// A compact icon-only tile; the whole tile toggles, as a wide tile's icon
/// does.
#[component]
pub fn small_tile(
    #[prop(required)] icon: Icon,
    active: bool,
    #[prop(callback)] on_toggle: fn(bool),
) -> impl Widget {
    let t = ui();
    let rest = active_fill(active, t.colors.surface);
    pressable(
        toggled(
            container()
                .width(fill())
                // At least, so that beside a wide tile whose subtitle takes
                // two lines it can be stretched to its height.
                .height(at_least(height()))
                .corners(shape(active, height(), radius::MEDIUM.px()).transition(motion::spring())),
            rest,
            move || hovered(rest()),
        ),
        motion::PRESS_SCALE,
    )
    .layout(Flex::column().center())
    .on_click(flip(active, on_toggle))
    .child(icons::icon(
        icon,
        Size::Md.icon(),
        active_ink(active, t.colors.text),
    ))
}
