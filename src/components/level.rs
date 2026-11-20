//! Levels: how worrying a value is, and an icon showing how full it is.
//!
//! The app decides the level, with its own thresholds; styletto draws it.

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::typography::Typeset;
use crate::primitives::layout::centered_row;
use crate::ui::ui;

/// How a value stands: as it should, well, worrying, or alarming.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Level {
    #[default]
    Normal,
    /// Better than normal: a battery charging.
    Good,
    Warn,
    Alert,
}

impl Level {
    /// Where `value` stands when high is bad, as a temperature: an alert at
    /// `alert` or above, a warning at `warn` or above.
    pub fn of(value: f32, warn: f32, alert: f32) -> Self {
        if value >= alert {
            Level::Alert
        } else if value >= warn {
            Level::Warn
        } else {
            Level::Normal
        }
    }

    /// Where `value` stands when low is bad, as a battery: an alert at
    /// `alert` or below, a warning at `warn` or below.
    pub fn below(value: f32, warn: f32, alert: f32) -> Self {
        if value <= alert {
            Level::Alert
        } else if value <= warn {
            Level::Warn
        } else {
            Level::Normal
        }
    }
}

/// The ink of `level`, for an icon or a figure: the text's while normal,
/// else its hue as text.
pub fn level_ink(level: Level) -> Color {
    let t = ui();
    match level {
        Level::Normal => t.colors.text,
        Level::Good => t.colors.success_text,
        Level::Warn => t.colors.warning_text,
        Level::Alert => t.colors.danger_text,
    }
}

/// The icons of a value's levels, each with the value it starts from, in
/// rising order; the first also stands for anything below it. The value is
/// a percentage, as the indicator's figure shows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Levels(pub &'static [(f32, Icon)]);

impl Levels {
    /// A battery's charge, its bars rounded up: a bar stays while there is
    /// charge in its third. Empty below 15%, one bar of three from 15%, two
    /// past a third, full past two thirds. Rounded to the nearest, one bar
    /// would stand for anything up to half the charge, and say low where
    /// the level is normal; this way one bar falls about where a battery is
    /// a warning (30% or less, as an app usually colours it) and the empty
    /// one where it is an alert (15% or less).
    pub const BATTERY: Levels = Levels(&[
        (0.0, Icon::Battery),
        (15.0, Icon::BatteryLow),
        (100.0 / 3.0, Icon::BatteryMedium),
        (200.0 / 3.0, Icon::BatteryFull),
    ]);

    /// A volume: no waves while silent (below 1%), one wave from 1%, two
    /// from 50% (past 100% too, amplified).
    pub const VOLUME: Levels = Levels(&[
        (0.0, Icon::VolumeZero),
        (1.0, Icon::VolumeLow),
        (50.0, Icon::Volume),
    ]);

    /// A Wi-Fi signal's strength: the icon with the nearest number of arcs,
    /// none below 17%, one of three from 17%, two from 50%, all from 83%.
    pub const WIFI: Levels = Levels(&[
        (0.0, Icon::WifiZero),
        (17.0, Icon::WifiLow),
        (50.0, Icon::WifiHigh),
        (83.0, Icon::Wifi),
    ]);

    /// The icon for `value`: the last one whose start it reached, or the
    /// first; none from an empty set.
    pub fn icon(self, value: f32) -> Option<Icon> {
        let Levels(steps) = self;
        steps
            .iter()
            .rfind(|(from, _)| value >= *from)
            .or(steps.first())
            .map(|&(_, icon)| icon)
    }
}

/// A value as an icon from its set, chosen by the value, and optionally
/// its percentage beside it, both in the ink of its level.
///
/// A state off the scale takes its own icon in place of the set's: a
/// battery charging (`Icon::BatteryCharging`, with `Level::Good`), a volume
/// muted (`Icon::VolumeOff`), a Wi-Fi off (`Icon::WifiOff`).
///
/// With an empty set and no icon given it draws no icon: the percentage
/// alone if asked, else nothing.
#[component]
pub fn level_indicator(
    #[prop(required)] levels: Levels,
    /// A percentage: from 0 to 100, or past it.
    #[prop(required)]
    value: f32,
    level: Level,
    /// In place of the one `levels` picks.
    icon: Option<Icon>,
    /// Shows the value beside the icon.
    percent: bool,
    /// The icon, the type and the gap of a control this size.
    #[prop(default = Size::Sm)]
    size: Size,
) -> impl Widget {
    let size = size.get_untracked();
    let ink = move || level_ink(level.get());
    // A memo, so the icon is built again only when it changes, not with
    // every value.
    let kind = create_memo(move || icon.get().or_else(|| levels.get().icon(value.get())));
    container()
        .layout(centered_row(size.gap()))
        .child(move || kind.get().map(|kind| icons::icon(kind, size.icon(), ink)))
        .child(move || {
            percent.get().then(|| {
                text(move || format!("{:.0}%", value.get()))
                    .typeset(size.label().em().tabular())
                    .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                    .color(ink.transition(motion::color()))
                    .nowrap()
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_is_bad_from_the_thresholds_up() {
        assert_eq!(Level::of(59.0, 60.0, 80.0), Level::Normal);
        assert_eq!(Level::of(60.0, 60.0, 80.0), Level::Warn);
        assert_eq!(Level::of(80.0, 60.0, 80.0), Level::Alert);
    }

    #[test]
    fn low_is_bad_from_the_thresholds_down() {
        assert_eq!(Level::below(31.0, 30.0, 15.0), Level::Normal);
        assert_eq!(Level::below(30.0, 30.0, 15.0), Level::Warn);
        assert_eq!(Level::below(15.0, 30.0, 15.0), Level::Alert);
        assert_eq!(Level::below(0.0, 30.0, 15.0), Level::Alert);
    }

    #[test]
    fn a_value_picks_the_icon_it_reached() {
        // A battery's bars are rounded up.
        for (charge, icon) in [
            (12.0, Icon::Battery),
            (14.9, Icon::Battery),
            (15.0, Icon::BatteryLow),
            (33.0, Icon::BatteryLow),
            (33.5, Icon::BatteryMedium),
            (46.0, Icon::BatteryMedium),
            (66.0, Icon::BatteryMedium),
            (67.0, Icon::BatteryFull),
            (100.0, Icon::BatteryFull),
        ] {
            assert_eq!(Levels::BATTERY.icon(charge), Some(icon), "{charge}%");
        }
        assert_eq!(Levels::VOLUME.icon(0.0), Some(Icon::VolumeZero));
        assert_eq!(Levels::VOLUME.icon(30.0), Some(Icon::VolumeLow));
        assert_eq!(Levels::VOLUME.icon(150.0), Some(Icon::Volume));
        assert_eq!(Levels::WIFI.icon(70.0), Some(Icon::WifiHigh));
        // Below the first start, or not a number, the first.
        assert_eq!(Levels::WIFI.icon(-5.0), Some(Icon::WifiZero));
        assert_eq!(Levels::WIFI.icon(f32::NAN), Some(Icon::WifiZero));
    }

    #[test]
    fn an_empty_set_picks_no_icon() {
        assert_eq!(Levels(&[]).icon(50.0), None);
        assert_eq!(Levels(&[]).icon(f32::NAN), None);
    }

    #[test]
    fn every_preset_rises() {
        for Levels(steps) in [Levels::BATTERY, Levels::VOLUME, Levels::WIFI] {
            assert!(!steps.is_empty());
            assert!(steps.windows(2).all(|pair| pair[0].0 < pair[1].0));
        }
    }
}
