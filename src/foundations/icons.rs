//! Lucide line icons, from the SVG files in `icons/`, drawn white and tinted.

use std::cell::RefCell;
use std::collections::HashMap;

use guido::prelude::*;

use crate::foundations::motion;
use crate::foundations::tokens::{self, IconSize, TileSize, tile};
use crate::foundations::typography::Typeset;

/// Declares [`Icon`] from its variants and the Lucide file each draws.
macro_rules! icons {
    ($($name:ident => $file:literal,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Icon {
            $($name,)*
        }

        impl Icon {
            pub const ALL: &[Icon] = &[$(Icon::$name,)*];

            fn svg(self) -> &'static str {
                match self {
                    $(Icon::$name => include_str!(concat!("../../icons/", $file, ".svg")),)*
                }
            }
        }
    };
}

icons! {
    ArrowDown => "arrow-down",
    ArrowRight => "arrow-right",
    ArrowUp => "arrow-up",
    Battery => "battery",
    BatteryCharging => "battery-charging",
    BatteryFull => "battery-full",
    BatteryLow => "battery-low",
    BatteryMedium => "battery-medium",
    BatteryWarning => "battery-warning",
    Bell => "bell",
    BellOff => "bell-off",
    Bluetooth => "bluetooth",
    BluetoothConnected => "bluetooth-connected",
    BluetoothOff => "bluetooth-off",
    BluetoothSearching => "bluetooth-searching",
    Cable => "cable",
    Calculator => "calculator",
    Calendar => "calendar",
    Camera => "video",
    CapsLock => "arrow-big-up-dash",
    Check => "check",
    ChevronDown => "chevron-down",
    ChevronLeft => "chevron-left",
    ChevronRight => "chevron-right",
    ChevronUp => "chevron-up",
    Clipboard => "clipboard",
    Cloud => "cloud",
    CloudDrizzle => "cloud-drizzle",
    CloudFog => "cloud-fog",
    CloudLightning => "cloud-lightning",
    CloudMoon => "cloud-moon",
    CloudRain => "cloud-rain",
    CloudSnow => "cloud-snow",
    CloudSun => "cloud-sun",
    Coffee => "coffee",
    Cpu => "cpu",
    Download => "download",
    Droplet => "droplet",
    EthernetPort => "ethernet-port",
    Eye => "eye",
    EyeOff => "eye-off",
    File => "file",
    Fingerprint => "fingerprint",
    Folder => "folder",
    Gamepad => "gamepad",
    Gauge => "gauge",
    Globe => "globe",
    Grid => "layout-grid",
    HardDrive => "hard-drive",
    Headphones => "headphones",
    Headset => "headset",
    Image => "image",
    Keyboard => "keyboard",
    Leaf => "leaf",
    Lock => "lock",
    LogOut => "log-out",
    MemoryStick => "memory-stick",
    Mic => "mic",
    MicOff => "mic-off",
    Minus => "minus",
    Monitor => "monitor",
    Moon => "moon",
    Mouse => "mouse",
    Music => "music",
    Package => "package",
    Pause => "pause",
    Plane => "plane",
    Play => "play",
    Plus => "plus",
    Power => "power",
    Refresh => "rotate-cw",
    Search => "search",
    Settings => "settings-2",
    Shield => "shield",
    SkipBack => "skip-back",
    SkipForward => "skip-forward",
    Snowflake => "snowflake",
    Speaker => "speaker",
    Sun => "sun",
    Thermometer => "thermometer",
    Tv => "tv",
    Usb => "usb",
    User => "user-round",
    Volume => "volume-2",
    VolumeLow => "volume-1",
    VolumeOff => "volume-x",
    VolumeZero => "volume",
    Wifi => "wifi",
    WifiHigh => "wifi-high",
    WifiLow => "wifi-low",
    WifiOff => "wifi-off",
    WifiZero => "wifi-zero",
    Wind => "wind",
    X => "x",
    Zap => "zap",
}

impl Icon {
    /// One source per icon, so every use shares one texture.
    fn source(self) -> ImageSource {
        thread_local! {
            static SOURCES: RefCell<HashMap<Icon, ImageSource>> = RefCell::default();
        }
        SOURCES.with_borrow_mut(|sources| {
            sources
                .entry(self)
                .or_insert_with(|| {
                    // Drawn white, so a tint gives it any colour.
                    let svg = self.svg().replace("currentColor", "white");
                    ImageSource::SvgBytes(svg.into_bytes().into())
                })
                .clone()
        })
    }
}

/// `kind` drawn `size` square in `color`: a value, a signal or a closure,
/// easing into a new one at [`motion::color`], as text does.
pub fn icon<M1, M2>(
    kind: impl IntoSignal<Icon, M1>,
    size: f32,
    color: impl IntoSignal<Color, M2>,
) -> Container {
    let kind = kind.into_signal();
    container()
        .width(size)
        .height(size)
        .child(image(move || kind.get().source()).tint(motion::eased_color(color)))
}

/// An icon that turns while something runs.
#[component]
pub fn spinning(
    #[prop(required)] icon: Icon,
    #[prop(default = tokens::icon::SMALL)] size: IconSize,
    color: Color,
) -> impl Widget {
    let turn = Keyframes::new(900.0)
        .at(0.0, 0.0)
        .at(1.0, 360.0)
        .repeat(Repeat::Forever);
    container()
        .pivot(Pivot::CENTER)
        .rotate(0.0.timeline(turn))
        .child(self::icon(icon, size.get_untracked().px(), color))
}

/// A name's first letter on a tile, for what has no icon.
#[component]
pub fn initial(
    #[prop(required)] name: String,
    #[prop(default = tile::MEDIUM)] size: TileSize,
) -> impl Widget {
    let t = crate::look();
    let tile = size.get_untracked();
    let size = tile.px();
    let letter = move || {
        name.get()
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string()
    };
    container()
        .width(size)
        .height(size)
        .corners(tile.radius().corners())
        .background(t.eased(|c| c.surface2))
        .layout(Flex::column().center())
        .child(
            text(letter)
                .typeset(tile.letter())
                .color(t.eased(|c| c.sub))
                .nowrap(),
        )
}

/// An image in a square.
#[component]
pub fn picture(
    source: Option<ImageSource>,
    #[prop(default = tile::MEDIUM)] size: TileSize,
    /// What shows while there is no source.
    #[prop(slot)]
    fallback: (),
) -> impl Widget {
    let size = size.get_untracked().px();
    match source.get_untracked() {
        Some(source) => container()
            .width(size)
            .height(size)
            .child(image(source).content_fit(ContentFit::Contain)),
        None => container().children(fallback),
    }
}

/// An image covering a squircle: a cover, a notification's picture.
#[component]
pub fn artwork(
    source: Option<ImageSource>,
    #[prop(default = tile::MEDIUM)] size: TileSize,
    /// What shows while there is no source.
    #[prop(slot)]
    fallback: (),
) -> impl Widget {
    let tile = size.get_untracked();
    let size = tile.px();
    match source.get_untracked() {
        Some(source) => container()
            .width(size)
            .height(size)
            .corners(tile.radius().corners())
            .overflow(Overflow::Hidden)
            .child(image(source).content_fit(ContentFit::Cover)),
        None => container().children(fallback),
    }
}

#[cfg(test)]
mod tests {
    use super::Icon;

    #[test]
    fn every_icon_is_a_lucide_line_icon() {
        for icon in Icon::ALL {
            let svg = icon.svg();
            assert!(svg.contains(r#"viewBox="0 0 24 24""#), "{icon:?}");
            assert!(svg.contains(r#"stroke="currentColor""#), "{icon:?}");
        }
    }
}
