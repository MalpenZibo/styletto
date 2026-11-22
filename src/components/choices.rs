//! One of several values to choose, as a row of buttons with the chosen one
//! `active`: a setting's options, as the appearance controls offer them, or
//! an app's own.

use guido::prelude::*;

use crate::components::button::button;
use crate::foundations::tokens::size;

/// One of `options` to choose, as buttons, the chosen one `active`, as
/// many to a line as fit. A press calls `on_choose` with its value; the
/// app sets `chosen`, as it does a toggle's `active`. Generic over the
/// value, an app's own enum as well as styletto's.
#[component]
pub fn choices<T: Copy + PartialEq + 'static>(
    /// Each option's label and value, in the order they show.
    #[prop(required)]
    options: Vec<(String, T)>,
    /// The value chosen; an option is `active` while its value is it.
    #[prop(required)]
    chosen: T,
    /// Called with the value of the option pressed.
    #[prop(callback)]
    on_choose: fn(T),
) -> impl Widget {
    container()
        .width(fill())
        .layout(Flex::row().spacing(size::S1.px()).wrap(true))
        .children(move || {
            options.get().into_iter().map(move |(name, value)| {
                button()
                    .label(name)
                    .active(move || chosen.get() == value)
                    .on_click(move || {
                        if let Some(on_choose) = on_choose {
                            on_choose.run(value);
                        }
                    })
            })
        })
}
