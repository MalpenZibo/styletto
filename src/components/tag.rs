//! Tags: read-only pills that show a state, a count, a reading, what is in
//! use.

use guido::prelude::*;

use crate::components::button::pill_box;
use crate::foundations::control::Size;
use crate::foundations::motion;
use crate::foundations::tokens::{glow as spread, tint};
use crate::primitives::layout::centered_row;
use crate::ui::ui;

/// The hue a tag is tinted with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// On `surface`, untinted.
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

impl Tone {
    /// The hue's fill, none while neutral.
    fn hue(self) -> Option<Color> {
        let c = ui().colors;
        match self {
            Tone::Neutral => None,
            Tone::Accent => Some(c.accent),
            Tone::Success => Some(c.success),
            Tone::Warning => Some(c.warning),
            Tone::Danger => Some(c.danger),
        }
    }

    /// What a tag in this tone is filled with: `surface` while neutral,
    /// else the hue's fill at [`tint::STRONG`].
    pub fn tint(self) -> Color {
        self.hue()
            .map_or_else(|| ui().colors.surface, |hue| hue.with_alpha(tint::STRONG))
    }

    /// The colour for what a tag in this tone holds, its icons and its text:
    /// `text` while neutral, else the hue as text (`accent_text` and the
    /// rest), which reads on the tint.
    pub fn ink(self) -> Color {
        let c = ui().colors;
        match self {
            Tone::Neutral => c.text,
            Tone::Accent => c.accent_text,
            Tone::Success => c.success_text,
            Tone::Warning => c.warning_text,
            Tone::Danger => c.danger_text,
        }
    }
}

/// A pill holding its children in a centred row, tinted by its tone. It
/// shows, and takes no press: a toggle is a `button` with `active`.
///
/// What it holds is drawn by the app, in [`Tone::ink`] for the tag's hue
/// or in a colour of its own, as a battery's level.
#[component]
pub fn tag(
    tone: Tone,
    #[prop(default = Size::Xxs)] size: Size,
    /// A halo of the tone's hue around it, for what should be seen at a
    /// glance: a microphone in use. None while neutral.
    glow: bool,
    #[prop(children)] children: (),
) -> impl Widget {
    let size = size.get_untracked();
    let halo = move || match tone.get().hue() {
        Some(hue) if glow.get() => hue.with_alpha(tint::MEDIUM),
        _ => Color::TRANSPARENT,
    };
    pill_box(size)
        .background((move || tone.get().tint()).transition(motion::color()))
        .shadow(
            (move || Shadow::new((0.0, 0.0), spread::SMALL, 0.0, halo()))
                .transition(motion::color()),
        )
        .layout(centered_row(size.gap()))
        .children(children)
}
