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

/// A figure beside its icon (a level's, a slider's): set in `kind`,
/// emphasised, its digits tabular, its box trimmed to the capitals, in
/// `ink`.
pub(crate) fn figure_text<M>(
    content: impl IntoSignal<String, M>,
    kind: Type,
    ink: impl Fn() -> Color + 'static,
) -> Text {
    text(content)
        .typeset(kind.em().tabular())
        .text_box(TextEdge::Cap, TextEdge::Alphabetic)
        .color(motion::eased_color(ink))
        .nowrap()
}
