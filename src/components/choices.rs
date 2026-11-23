//! One of several values to choose, as a row of buttons with the chosen one
//! `active`: a setting's options, as the appearance controls offer them, or
//! an app's own; or the tabs in a window's header.

use guido::prelude::*;

use crate::components::button::{Fill, button};
use crate::foundations::tokens::size;
use crate::primitives::layout::centered_row;

/// One of `options` to choose, as buttons, the chosen one `active`, as
/// many to a line as fit. A press calls `on_choose` with its value; the
/// app sets `chosen`, as it does a toggle's `active`. Generic over the
/// value, an app's own enum as well as styletto's. Disabled, every option
/// fades as a disabled button does and takes no press.
#[component]
pub fn choices<T: Copy + PartialEq + 'static>(
    /// Each option's label and value, in the order they show.
    #[prop(required)]
    options: Vec<(String, T)>,
    /// The value chosen; an option is `active` while its value is it.
    #[prop(required)]
    chosen: T,
    /// Whether the options take a press.
    #[prop(default = true)]
    enabled: bool,
    /// Called with the value of the option pressed.
    #[prop(callback)]
    on_choose: fn(T),
) -> impl Widget {
    buttons(
        Flex::row().spacing(size::S1.px()).wrap(true),
        Fill::Solid,
        options,
        chosen,
        enabled,
        on_choose,
    )
    .width(fill())
}

/// The pages of a window, as tabs: ghost buttons on one line, the current
/// one `active`, as tall as a header's buttons, to sit between its title
/// and its close button. They never wrap: a header holds as many as fit on
/// its line. As [`choices`], a press calls `on_choose` with its value and
/// the app sets `chosen`, and disabled they fade and take no press.
#[component]
pub fn tabs<T: Copy + PartialEq + 'static>(
    /// Each tab's label and value, in the order they show.
    #[prop(required)]
    options: Vec<(String, T)>,
    /// The current tab's value.
    #[prop(required)]
    chosen: T,
    /// Whether the tabs take a press.
    #[prop(default = true)]
    enabled: bool,
    /// Called with the value of the tab pressed.
    #[prop(callback)]
    on_choose: fn(T),
) -> impl Widget {
    buttons(
        centered_row(size::S1.px()),
        Fill::Ghost,
        options,
        chosen,
        enabled,
        on_choose,
    )
}

/// The options as buttons in `fill`, laid out by `layout`, the one whose
/// value is `chosen` `active`.
fn buttons<T: Copy + PartialEq + 'static>(
    layout: Flex,
    fill: Fill,
    options: Signal<Vec<(String, T)>>,
    chosen: Signal<T>,
    enabled: Signal<bool>,
    on_choose: Option<Callback<(T,)>>,
) -> Container {
    container().layout(layout).children(move || {
        options.get().into_iter().map(move |(name, value)| {
            button()
                .label(name)
                .fill(fill)
                .active(move || chosen.get() == value)
                .enabled(enabled)
                .on_click(move || {
                    if let Some(on_choose) = on_choose {
                        on_choose.run(value);
                    }
                })
        })
    })
}
