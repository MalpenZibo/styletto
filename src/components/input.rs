//! Text field frames.

use guido::prelude::*;

use crate::components::button::{Fill, icon_button};
use crate::foundations::control::Size;
use crate::foundations::icons::{Icon, spinning};
use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{icon as icon_box, opacity, size as space, stroke};
use crate::primitives::layout::centered_row;
use crate::ui::ui;

/// A pill on the surface holding a field and its trailing widgets.
#[component]
pub fn field(
    /// The ring inside its edge.
    ring: Color,
    #[prop(default = Size::Lg)] size: Size,
    /// A trailing button the pill's end hugs.
    #[prop(slot)]
    trailing: (),
    #[prop(children)] children: (),
) -> impl Widget {
    let t = ui();
    let size = size.get_untracked();
    let (height, padding) = (size.height(), size.inline());
    let end = if trailing.is_some() {
        space::S1.px()
    } else {
        padding
    };
    container()
        .width(fill())
        .height(height)
        .corners(radius::pill(height))
        .background(t.colors.surface)
        .shadow(
            (move || Shadow::new((0.0, 0.0), 0.0, stroke::RING, ring.get()).inset())
                .transition(motion::color()),
        )
        .padding([0.0, end, 0.0, padding])
        .layout(centered_row(space::S2.px()))
        .children(children)
        .children(trailing)
}

/// A password field: the ring turns red on a refusal; the end button
/// submits, or spins while checking.
#[component]
pub fn password_field(
    /// The last answer was refused and nothing new is typed.
    refused: bool,
    /// An answer is being checked.
    busy: bool,
    /// Takes no input; the field fades.
    held: bool,
    /// Before the submit button, as a reveal toggle.
    #[prop(slot)]
    extra: (),
    #[prop(callback)] on_submit: (),
    #[prop(children)] children: (),
) -> impl Widget {
    let t = ui();
    let ring = move || {
        if refused.get() {
            t.colors.danger
        } else {
            t.colors.accent.with_alpha(0.7)
        }
    };
    let submit = container().child(move || {
        if busy.get() {
            container()
                .width(Size::Sm.height())
                .height(Size::Sm.height())
                .layout(Flex::column().center())
                .child(
                    spinning(Icon::Refresh)
                        .size(icon_box::SMALL)
                        .color(t.colors.accent),
                )
                .into_any()
        } else {
            icon_button(Icon::ChevronRight)
                .size(Size::Sm)
                .fill(Fill::Ghost)
                .on_click(on_submit)
                .into_any()
        }
    });
    container()
        .width(fill())
        .opacity(
            (move || if held.get() { opacity::DISABLED } else { 1.0 }).transition(motion::hover()),
        )
        .child(
            field().ring(ring).children(children).trailing(
                container()
                    .layout(centered_row(space::S1.px()))
                    .children(extra)
                    .child(submit),
            ),
        )
}
