mod buttons;
mod composites;
mod feedback;
mod foundations;
mod inputs;
mod motion;
pub mod story;

use styletto::foundations::tokens::{IconSize, icon};

use story::{Ctx, Story};

/// The steps of an icon's box, by name, for a story's choice.
const ICON_STEPS: &[&str] = &["XSmall", "Small", "Medium", "Large", "XLarge"];
const ICON_SIZES: [IconSize; 5] = [
    icon::XSMALL,
    icon::SMALL,
    icon::MEDIUM,
    icon::LARGE,
    icon::XLARGE,
];

/// The width of a panel under the island.
const PANEL: f32 = 392.0;
/// Where CPU use and temperature turn to a warning, then an alert.
const CPU_LEVELS: (f32, f32) = (60.0, 80.0);
const TEMP_LEVELS: (f32, f32) = (70.0, 85.0);

const fn story(
    group: &'static str,
    name: &'static str,
    render: fn(&Ctx) -> guido::prelude::Container,
) -> Story {
    Story {
        group,
        name,
        render,
    }
}

pub const STORIES: &[Story] = &[
    story("Foundations", "Colors", foundations::colors),
    story("Foundations", "Typography", foundations::typography),
    story("Foundations", "Scale", foundations::scale),
    story("Foundations", "Radius", foundations::radii),
    story("Foundations", "Controls", foundations::controls),
    story("Foundations", "Icons", foundations::icons),
    story("Foundations", "Matrix", foundations::matrix),
    story("Buttons", "Button", buttons::button),
    story("Buttons", "Icon button", buttons::icon_button),
    story("Buttons", "Chip", buttons::chip),
    story("Buttons", "Pill", buttons::pill),
    story("Inputs", "Toggle", inputs::toggle),
    story("Inputs", "Slider", inputs::slider),
    story("Inputs", "Field", inputs::field),
    story("Inputs", "Password field", inputs::password_field),
    story("Feedback", "Progress", feedback::progress),
    story("Feedback", "Level indicator", feedback::level_indicators),
    story("Feedback", "Sparkline", feedback::sparklines),
    story("Feedback", "Rolling digits", feedback::rolling),
    story("Feedback", "Marks", feedback::marks),
    story("Feedback", "Chips", feedback::chips),
    story("Feedback", "Pictures", feedback::pictures),
    story("Composites", "Quick tiles", composites::tiles),
    story("Composites", "Slider rows", composites::slider_rows),
    story("Composites", "List & pages", composites::list_pages),
    story("Composites", "Scrolling list", composites::scrolling_list),
    story("Composites", "Power menu", composites::power_menu),
    story("Composites", "Cards", composites::cards),
    story("Composites", "Notifications", composites::notifications),
    story("Motion", "Island", motion::island),
    story("Motion", "Cross-fade", motion::cross_fade),
    story("Motion", "Shake", motion::shake),
    story("Motion", "Satellite", motion::satellite),
];
