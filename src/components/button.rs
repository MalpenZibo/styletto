//! Pill buttons, icon buttons and toggle chips.

use std::cell::Cell;

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{opacity, stroke};
use crate::foundations::typography::LABEL_SMALL;
use crate::primitives::layout::centered_row;
use crate::primitives::text::typed;
use crate::ui::{on_fill, on_ink, ui};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Neutral,
    Accent,
    Danger,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fill {
    #[default]
    Solid,
    Ghost,
}

/// A fill one step brighter: more opaque if translucent, else lighter.
pub fn hovered(color: Color) -> Color {
    if color.a < 1.0 {
        color.with_alpha((color.a + 0.06).min(1.0))
    } else {
        color.lighter(0.12)
    }
}

/// A pill with a label, an icon before it, or only the icon in a circle.
#[component]
pub fn button(
    label: String,
    icon: Option<Icon>,
    variant: Variant,
    size: Size,
    #[prop(default = true)] enabled: bool,
    #[prop(callback)] on_click: (),
) -> impl Widget {
    let t = ui();
    let size = size.get_untracked();
    let (fill, ink) = match variant.get_untracked() {
        Variant::Neutral => (t.colors.surface2, t.colors.text),
        Variant::Accent => (t.colors.accent, t.colors.on_accent),
        Variant::Danger => (t.colors.danger, t.colors.on_danger),
    };
    pressable(
        sized(container(), label, size)
            .background(fill.transition(motion::hover()))
            .when_hovered(move |s| s.background(hovered(fill))),
        0.95,
    )
    .enabled(enabled)
    .on_click(on_click)
    .layout(centered_row(size.gap()).main_alignment(MainAlignment::Center))
    .children(content(label, icon, move || ink, size))
}

/// A round button holding one icon, filled with the accent while on.
#[component]
pub fn icon_button(
    #[prop(required)] icon: Icon,
    size: Size,
    fill: Fill,
    /// Fills it with the accent, as a tile that is on, whatever its `fill`.
    active: bool,
    #[prop(default = true)] enabled: bool,
    #[prop(callback)] on_click: (),
) -> impl Widget {
    let t = ui();
    let size = size.get_untracked();
    let side = size.height();
    let (rest, hover) = match fill.get_untracked() {
        Fill::Solid => (t.colors.surface, t.colors.surface2),
        Fill::Ghost => (Color::TRANSPARENT, t.colors.surface),
    };
    let lit = move |on: Color, off: Color| if active.get() { on } else { off };
    pressable(
        container()
            .width(side)
            .height(side)
            .corners(radius::pill(side))
            .background((move || lit(t.colors.accent, rest)).transition(toggling(active)))
            .when_hovered(move |s| s.background(move || lit(hovered(t.colors.accent), hover))),
        0.9,
    )
    .enabled(enabled)
    .on_click(on_click)
    .layout(Flex::column().center())
    .child(icons::icon(icon, size.icon(), on_ink(active)))
}

/// Eases a fill turning on or off as a chip's, a hover as a button's: a
/// button that is never turned on keeps its quicker hover.
fn toggling(active: Signal<bool>) -> impl Fn(Segment<Color>) -> Transition + 'static {
    let (hover, toggle) = (motion::hover(), motion::color());
    let was = Cell::new(active.get_untracked());
    move |_| {
        let now = active.get_untracked();
        if was.replace(now) == now {
            hover.clone()
        } else {
            toggle.clone()
        }
    }
}

/// A small on/off pill, filled with the accent while on.
#[component]
pub fn chip_button(
    #[prop(required)] label: String,
    icon: Option<Icon>,
    active: bool,
    #[prop(callback)] on_click: (),
) -> impl Widget {
    let size = Size::Sm;
    let fill = on_fill(active);
    pressable(
        sized(container(), label, size)
            .background(fill.transition(motion::color()))
            .when_hovered(move |s| s.background(move || hovered(fill()))),
        0.95,
    )
    .on_click(on_click)
    .layout(centered_row(size.gap()).main_alignment(MainAlignment::Center))
    .children(content(label, icon, on_ink(active), size))
}

/// A circle without a label, a padded pill with one.
fn sized(base: Container, label: Signal<String>, size: Size) -> Container {
    let height = size.height();
    let base = base.height(height).corners(radius::pill(height));
    if label.with_untracked(String::is_empty) {
        base.width(height)
    } else {
        base.padding([0.0, size.inline()])
    }
}

/// Shared button behaviour: squish and ripple on press, fade when disabled.
pub fn pressable(base: Container, pressed_scale: f32) -> Container {
    let t = ui();
    base.scale(Scale::NONE.transition(motion::tactile()))
        .when_pressed(move |s| {
            s.scale(Scale::uniform(pressed_scale))
                .ripple_with_color(t.colors.ripple)
        })
        .opacity(1.0.transition(motion::hover()))
        .when_disabled(|s| s.opacity(opacity::DISABLED).scale(Scale::NONE))
}

fn content(
    label: Signal<String>,
    kind: Signal<Option<Icon>>,
    ink: impl Fn() -> Color + Copy + 'static,
    size: Size,
) -> Vec<Container> {
    let mut children = Vec::new();
    if let Some(kind) = kind.get_untracked() {
        children.push(icons::icon(kind, size.icon(), ink));
    }
    if !label.with_untracked(String::is_empty) {
        children.push(
            container().child(
                typed(label, size.label().em())
                    .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                    .color(ink)
                    .nowrap(),
            ),
        );
    }
    children
}

/// A pill in a row of them, as under the island's resting row: clear until
/// hovered.
#[component]
pub fn pill(
    /// Fills it with the accent.
    active: bool,
    /// A ring inside it, for attention.
    ring: Option<Color>,
    #[prop(callback)] on_click: (),
    #[prop(children)] children: (),
) -> impl Widget {
    let t = ui();
    let size = Size::Sm;
    let fill = move |rest: Color| if active.get() { t.colors.accent } else { rest };
    container()
        .height(size.height())
        .padding([0.0, size.inline()])
        .corners(radius::pill(size.height()))
        .background((move || fill(Color::TRANSPARENT)).transition(motion::hover()))
        .when_hovered(move |s| s.background(move || fill(t.colors.surface2)))
        .when_pressed(move |s| s.ripple_with_color(t.colors.ripple))
        .shadow(move || {
            Shadow::new(
                (0.0, 0.0),
                0.0,
                stroke::RING,
                ring.get().unwrap_or(Color::TRANSPARENT),
            )
            .inset()
        })
        .on_click(on_click)
        .layout(centered_row(size.gap()))
        .children(children)
}

/// A pill's label: quiet until hovered, ellipsized when too narrow.
#[component]
pub fn pill_text(
    #[prop(required)] text: String,
    /// In the accent's ink, for an active pill.
    active: bool,
    /// The emphasised weight.
    bold: bool,
    /// Replaces the quiet colour.
    ink: Option<Color>,
) -> impl Widget {
    let t = ui();
    let (regular, emphasised) = (
        LABEL_SMALL.resolve().weight,
        LABEL_SMALL.em().resolve().weight,
    );
    let shown = move |color: Color| {
        if active.get() {
            t.colors.on_accent
        } else {
            color
        }
    };
    typed(text, LABEL_SMALL.tabular())
        .font_weight(move || if bold.get() { emphasised } else { regular })
        .text_box(TextEdge::Cap, TextEdge::Alphabetic)
        .color(move || shown(ink.get().unwrap_or(t.colors.sub)))
        .when_hovered(move |s| s.color(move || shown(t.colors.text)))
        .nowrap()
        .overflow(TextOverflow::Ellipsis)
}
