//! Text in the theme's colours, set in the type scale.

use guido::prelude::*;

use crate::foundations::motion;
use crate::foundations::typography::{
    LABEL_MEDIUM, LABEL_SMALL, MICRO_XSMALL, TITLE_SMALL, Type, Typeset,
};
use crate::ui::look;

/// `content` set in `kind`, in its case.
pub fn typed<M>(content: impl IntoSignal<String, M>, kind: Type) -> Text {
    let content = content.into_signal();
    text(move || content.with(|content| kind.cased(content))).typeset(kind)
}

pub fn label<M>(content: impl IntoSignal<String, M>) -> Text {
    typed(content, LABEL_MEDIUM)
        .color(look().eased(|c| c.text))
        .nowrap()
}

pub fn caption<M>(content: impl IntoSignal<String, M>) -> Text {
    typed(content, LABEL_SMALL)
        .color(look().eased(|c| c.sub))
        .nowrap()
}

pub fn heading<M>(content: impl IntoSignal<String, M>) -> Text {
    typed(content, TITLE_SMALL.em())
        .color(look().eased(|c| c.text))
        .nowrap()
}

/// A small uppercase label over a group.
pub fn overline<M>(content: impl IntoSignal<String, M>) -> Text {
    typed(content, MICRO_XSMALL.em()).color(look().eased(|c| c.sub))
}

/// A number that keeps its width while its digits change.
pub fn figure<M>(content: impl IntoSignal<String, M>) -> Text {
    typed(content, LABEL_MEDIUM.tabular())
        .color(look().eased(|c| c.text))
        .nowrap()
}

/// A text field's text in the palette's colours: `text`, `sub` for the
/// placeholder, `accent.text` for the cursor and `selection`, each easing
/// into a new palette. For a `text_input` or a `password_input`:
/// `text_input(typed).palette_colors()`.
pub trait PaletteColors {
    fn palette_colors(self) -> Self;
}

/// One impl per kind of field: Guido's `TextInput` is generic over what it
/// holds, and that is sealed.
macro_rules! palette_colors {
    ($($input:ty),*) => {$(
        impl PaletteColors for $input {
            fn palette_colors(self) -> Self {
                let t = look();
                self.color(t.eased(|c| c.text))
                    .placeholder_color(t.eased(|c| c.sub))
                    .cursor_color(t.eased(|c| c.accent.text))
                    .selection_color(t.eased(|c| c.selection))
            }
        }
    )*};
}

palette_colors!(TextInput, PasswordInput);

/// A figure beside its icon (a level's) or set apart (a slider's): set in
/// `kind`, its digits tabular, its box trimmed to the capitals, in `ink`.
/// In a line of icons and labels `kind` keeps its regular weight, which
/// matches the icons' stroke; a readout set apart passes it `em`.
pub(crate) fn figure_text<M>(
    content: impl IntoSignal<String, M>,
    kind: Type,
    ink: impl Fn() -> Color + 'static,
) -> Text {
    text(content)
        .typeset(kind.tabular())
        .text_box(TextEdge::Cap, TextEdge::Alphabetic)
        .color(motion::eased_color(ink))
        .nowrap()
}
