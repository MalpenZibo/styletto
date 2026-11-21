//! Buttons, on and off as a tile is, a round icon button among them, and
//! the bar's pills.

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
use crate::ui::ui;

/// What a button does, by its colour: anything, the main thing, or what
/// can't be undone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Neutral,
    Accent,
    Danger,
}

/// How a button sits at rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fill {
    /// In its variant's fill (`surface` for a neutral one), its content in
    /// the colour that reads on it.
    #[default]
    Solid,
    /// Clear until hovered, then on `surface`; its content in the variant's
    /// hue as text (`text` for a neutral one), which reads on both.
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
///
/// While `active` it is on, as a tile is: filled with the accent, its
/// content in `on_accent`, whatever its variant and fill. Disabled, on or
/// off, it fades and takes no press.
#[component]
pub fn button(
    label: String,
    icon: Option<Icon>,
    variant: Variant,
    fill: Fill,
    size: Size,
    /// On: filled with the accent, whatever its variant and fill. A solid
    /// accent button is filled with it already and looks the same on and
    /// off, so a toggle is a neutral or ghost one.
    active: bool,
    #[prop(default = true)] enabled: bool,
    #[prop(callback)] on_click: (),
) -> impl Widget {
    let t = ui();
    let c = t.colors;
    let size = size.get_untracked();
    // While off: the fill at rest, hovered, and the ink on both. A neutral
    // fill steps to the next surface, a hue brightens.
    let off = move || match (fill.get(), variant.get()) {
        (Fill::Solid, Variant::Neutral) => (c.surface, c.surface2, c.text),
        (Fill::Solid, Variant::Accent) => (c.accent, hovered(c.accent), c.on_accent),
        (Fill::Solid, Variant::Danger) => (c.danger, hovered(c.danger), c.on_danger),
        (Fill::Ghost, variant) => (
            Color::TRANSPARENT,
            c.surface,
            match variant {
                Variant::Neutral => c.text,
                Variant::Accent => c.accent_text,
                Variant::Danger => c.danger_text,
            },
        ),
    };
    let lit = move |on: Color, off: Color| if active.get() { on } else { off };
    pressable(
        sized(label, size)
            .background((move || lit(c.accent, off().0)).transition(toggling(active)))
            .when_hovered(move |s| s.background(move || lit(hovered(c.accent), off().1))),
        motion::PRESS_SCALE,
    )
    .enabled(enabled)
    .on_click(on_click)
    .layout(centered_row(size.gap()).main_alignment(MainAlignment::Center))
    .children(content(
        label,
        icon,
        move || lit(c.on_accent, off().2),
        size,
    ))
}

/// Eases a fill turning on or off as a tile's, a hover as a button's: a
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

/// A pill `size` tall: round ends, its content padded in from them. A
/// button with a label, a tag and the bar's pill are drawn in one.
pub(crate) fn pill_box(size: Size) -> Container {
    let height = size.height();
    container()
        .height(height)
        .padding([0.0, size.inline()])
        .corners(radius::pill(height))
}

/// A circle without a label, a pill with one.
fn sized(label: Signal<String>, size: Size) -> Container {
    if label.with_untracked(String::is_empty) {
        let side = size.height();
        container()
            .width(side)
            .height(side)
            .corners(radius::pill(side))
    } else {
        pill_box(size)
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

/// The icon, if any, then the label, if any. Whether each is there is
/// decided once; what they show follows their signals.
fn content(
    label: Signal<String>,
    kind: Signal<Option<Icon>>,
    ink: impl Fn() -> Color + Copy + 'static,
    size: Size,
) -> Vec<Container> {
    let mut children = Vec::new();
    if let Some(first) = kind.get_untracked() {
        let kind = move || kind.get().unwrap_or(first);
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
    pill_box(size)
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
