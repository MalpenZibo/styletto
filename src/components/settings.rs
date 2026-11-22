//! The pieces a settings panel is made of: a titled group of settings, and
//! a setting with its name over what sets it, as `choices`. The appearance
//! controls are built from them, so an app's own settings set beside
//! styletto's keep the same spacing.

use guido::prelude::*;

use crate::components::list::section;
use crate::foundations::tokens::size;
use crate::primitives::layout::centered_row;
use crate::primitives::text::{caption, label};

/// A titled run of settings, one under another, set in from the panel's
/// edge as a row's content is, under the title.
#[component]
pub fn setting_group(
    #[prop(required)] title: String,
    /// The settings, as `setting`s or what an app sets its own with.
    #[prop(children)]
    children: (),
) -> impl Widget {
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S2.px()))
        .child(section(title))
        .child(
            container()
                .width(fill())
                .padding([0.0, size::S3.px()])
                .layout(Flex::column().spacing(size::S4.px()))
                .children(children),
        )
}

/// A setting: its name, a note at the other end of the line, and what sets
/// it underneath.
#[component]
pub fn setting(
    #[prop(required)] name: String,
    /// A word on the setting's state, in `sub`, cut short with an ellipsis
    /// where the line ends. Empty says nothing.
    note: String,
    /// What sets it, as `choices`, one under another.
    #[prop(children)]
    children: (),
) -> impl Widget {
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S2.px()))
        .child(
            container()
                .width(fill())
                .layout(centered_row(size::S3.px()))
                .child(label(name))
                .child(
                    container()
                        .width(fill())
                        .layout(Flex::row().main_alignment(MainAlignment::End))
                        // A caption is never taller than the name's label, so
                        // an empty one leaves the line as it is.
                        .child(caption(note).overflow(TextOverflow::Ellipsis)),
                ),
        )
        .children(children)
}
