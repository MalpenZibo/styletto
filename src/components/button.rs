//! Buttons, an action or a toggle as a tile is, a round icon button among
//! them, and the bar's pills.

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{lift, opacity, stroke};
use crate::foundations::typography::LABEL_SMALL;
use crate::primitives::layout::centered_row;
use crate::primitives::text::typed;
use crate::theme::palette::Hue;
use crate::ui::{active_fill, active_ink, look};

/// What a button does, by its colour: anything, the main thing, or what
/// can't be undone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Neutral,
    Accent,
    Danger,
}

impl Variant {
    /// Its hue, none while neutral.
    fn hue(self) -> Option<Hue> {
        match self {
            Variant::Neutral => None,
            Variant::Accent => Some(Hue::Accent),
            Variant::Danger => Some(Hue::Danger),
        }
    }
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
        color.with_alpha((color.a + lift::HOVER_ALPHA).min(1.0))
    } else {
        color.lighter(lift::HOVER)
    }
}

/// `base` filled with `rest`, and `lifted` while hovered: the fill of
/// something that turns on and off, its hover eased quicker than a change of
/// state ([`motion::toggling`], which needs the same two colours).
pub(crate) fn toggled(
    base: Container,
    rest: impl Fn() -> Color + Copy + 'static,
    lifted: impl Fn() -> Color + Copy + 'static,
) -> Container {
    base.background(rest.transition(motion::toggling(rest, lifted)))
        .when_hovered(move |s| s.background(lifted))
}

/// A pill with a label, an icon before it, or only the icon in a circle.
///
/// An action takes `on_click`. A toggle takes `active` and `on_toggle`, as
/// a tile does: while `active` it is on, filled with the accent, its
/// content in `accent.on`, whatever its variant and fill; a press calls
/// `on_toggle` with the opposite, and the owner sets `active` to what it
/// keeps. Given both, a press calls `on_click`, then `on_toggle`. Disabled,
/// on or off, it fades and takes no press.
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
    /// Called with the state a press asks for: the opposite of `active`.
    #[prop(callback)]
    on_toggle: fn(bool),
) -> impl Widget {
    let t = look();
    let size = size.get_untracked();
    // While off: the fill at rest, hovered, and the ink on both. A neutral
    // fill steps to the next surface, a hue brightens.
    let off = move || {
        let c = t.colors();
        match (fill.get(), variant.get().hue().map(|hue| c.hue(hue))) {
            (Fill::Solid, None) => (c.surface, c.surface2, c.text),
            (Fill::Solid, Some(hue)) => (hue.fill, hovered(hue.fill), hue.on),
            (Fill::Ghost, _) => (Color::TRANSPARENT, c.surface, c.ink(variant.get().hue())),
        }
    };
    let rest = active_fill(active, move || off().0);
    // On, the accent brightens as a hue does.
    let lifted = move || {
        if active.get() {
            hovered(t.colors().accent.fill)
        } else {
            off().1
        }
    };
    // As GTK's toggle button sends `clicked` before `toggled`. The state
    // asked for is the opposite of the one pressed, read before `on_click`
    // can change it or close the view it is in.
    let press = (on_click.is_some() || on_toggle.is_some()).then(|| {
        Callback::new(move || {
            let asked = !active.get_untracked();
            if let Some(on_click) = on_click {
                on_click.run();
            }
            if let Some(on_toggle) = on_toggle {
                on_toggle.run(asked);
            }
        })
    });
    pressable(
        toggled(sized(label, size), rest, lifted),
        motion::PRESS_SCALE,
    )
    .enabled(enabled)
    .on_click(press)
    .layout(centered_row(size.gap()).main_alignment(MainAlignment::Center))
    .children(content(
        label,
        icon,
        active_ink(active, move || off().2),
        size,
    ))
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
/// `pressed_scale` is a step of `motion`: [`motion::PRESS_SCALE`],
/// [`motion::PRESS_SCALE_WIDE`] or [`motion::PRESS_SCALE_INNER`].
pub fn pressable(base: Container, pressed_scale: f32) -> Container {
    let t = look();
    base.scale(Scale::NONE.transition(motion::tactile()))
        .when_pressed(move |s| {
            s.scale(Scale::uniform(pressed_scale))
                .ripple_with_color(t.color(|c| c.ripple))
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
        children.push(icons::icon(
            kind,
            size.icon(),
            ink.transition(motion::color()),
        ));
    }
    if !label.with_untracked(String::is_empty) {
        children.push(
            container().child(
                typed(label, size.label().em())
                    .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                    .color(ink.transition(motion::color()))
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
    /// Fills it with the accent: the one chosen, as a workspace shown. A
    /// press calls `on_click` and the owner chooses, as with a row's
    /// `selected`; it is not a toggle.
    active: bool,
    /// A ring inside it, for attention.
    ring: Option<Color>,
    #[prop(callback)] on_click: (),
    #[prop(children)] children: (),
) -> impl Widget {
    let t = look();
    let size = Size::Sm;
    let lifted = active_fill(active, t.color(|c| c.surface2));
    pill_box(size)
        .background(active_fill(active, Color::TRANSPARENT).transition(motion::hover()))
        .when_hovered(move |s| s.background(lifted))
        .when_pressed(move |s| s.ripple_with_color(t.color(|c| c.ripple)))
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
    let t = look();
    let (regular, emphasised) = (
        LABEL_SMALL.resolve().weight,
        LABEL_SMALL.em().resolve().weight,
    );
    let lifted = active_ink(active, t.color(|c| c.text));
    typed(text, LABEL_SMALL.tabular())
        .font_weight(move || if bold.get() { emphasised } else { regular })
        .text_box(TextEdge::Cap, TextEdge::Alphabetic)
        .color(
            active_ink(active, move || ink.get().unwrap_or_else(|| t.colors().sub))
                .transition(motion::hover()),
        )
        .when_hovered(move |s| s.color(lifted))
        .nowrap()
        .overflow(TextOverflow::Ellipsis)
}
