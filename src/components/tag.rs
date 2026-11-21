//! Tags: read-only pills that show a state, a count, a reading, what is in
//! use.

use guido::prelude::*;

use crate::components::button::pill_box;
use crate::foundations::control::Size;
use crate::foundations::motion;
use crate::foundations::tokens::{glow as spread, tint};
use crate::primitives::layout::centered_row;
use crate::theme::palette::{Hue, Palette};
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
    /// Its hue, none while neutral.
    pub fn hue(self) -> Option<Hue> {
        match self {
            Tone::Neutral => None,
            Tone::Accent => Some(Hue::Accent),
            Tone::Success => Some(Hue::Success),
            Tone::Warning => Some(Hue::Warning),
            Tone::Danger => Some(Hue::Danger),
        }
    }

    /// What a tag in this tone is filled with: `surface` while neutral,
    /// else the hue's fill at [`tint::STRONG`].
    pub fn tint(self) -> Color {
        self.tint_in(&ui().colors)
    }

    fn tint_in(self, c: &Palette) -> Color {
        self.hue()
            .map_or(c.surface, |hue| c.hue(hue).fill.with_alpha(tint::STRONG))
    }

    /// The colour for what a tag in this tone holds, its icons and its text:
    /// `text` while neutral, else the hue as text, which reads on the tint.
    pub fn ink(self) -> Color {
        ui().colors.ink(self.hue())
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
    let c = ui().colors;
    let size = size.get_untracked();
    let lit = move || {
        let hue = tone.get().hue().filter(|_| glow.get());
        hue.map(|hue| c.hue(hue).fill)
    };
    pill_box(size)
        .background((move || tone.get().tint_in(&c)).transition(motion::color()))
        .shadow((move || spread::halo(lit(), spread::SMALL)).transition(motion::color()))
        .layout(centered_row(size.gap()))
        .children(children)
}
