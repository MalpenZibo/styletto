use std::time::Duration;

use guido::prelude::*;
use styletto::components::badge::{dot, icon_tile};
use styletto::components::level::{Level, Levels, level_indicator};
use styletto::components::notification::app_icon;
use styletto::components::progress::{progress_bar, ring};
use styletto::components::roll;
use styletto::components::sparkline::sparkline;
use styletto::components::tag::{Tone, tag};
use styletto::foundations::control::Size;
use styletto::foundations::icons::{artwork, initial, picture, spinning};
use styletto::foundations::tokens::{TileSize, icon as icon_box, size as space, tile};
use styletto::foundations::typography as ty;
use styletto::foundations::typography::Typeset;
use styletto::primitives::text::caption;
use styletto::{Icon, centered_row, icon, look};

use super::{ICON_SIZES, ICON_STEPS};
use crate::stories::story::Ctx;

pub fn progress(ctx: &Ctx) -> Container {
    let value = ctx.number("Value", 0.0..=100.0, 40.0);
    let size = ctx.choice("Ring size", ICON_STEPS, 4);
    let t = look();
    let fraction = move || value.get() / 100.0;
    let color = t.color(move |c| match value.get() {
        v if v <= 15.0 => c.danger.fill,
        v if v <= 30.0 => c.warning.fill,
        _ => c.accent.fill,
    });
    container()
        .layout(
            Flex::column()
                .spacing(space::S6.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(
            container()
                .width(space::S60.px())
                .child(progress_bar(fraction).color(color)),
        )
        .child(container().child(move || ring(fraction).color(color).size(ICON_SIZES[size.get()])))
}

pub fn sparklines(ctx: &Ctx) -> Container {
    const SLOTS: usize = 40;
    let running = ctx.bool("Running", true);
    let fixed = ctx.bool("Scale to 100", true);
    let height = ctx.choice("Height", &["S4", "S6", "S8", "S10"], 2);
    let samples = create_signal(Vec::<f32>::new());
    // A random walk, from a seed of its own: no dependency for a demo.
    let seed = std::cell::Cell::new(0x2545_f491_u32);
    set_interval(Duration::from_millis(300), move || {
        if !running.get_untracked() {
            return;
        }
        let next = seed
            .get()
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        seed.set(next);
        let step = (next >> 16) as f32 / 65_536.0 * 24.0 - 12.0;
        samples.update(|samples| {
            let last = samples.last().copied().unwrap_or(30.0);
            samples.push((last + step).clamp(0.0, 100.0));
            if samples.len() > SLOTS {
                samples.remove(0);
            }
        });
    });
    let samples = samples.read_only();
    container().width(space::S60.px()).child(move || {
        let steps = [space::S4, space::S6, space::S8, space::S10];
        let line = sparkline(samples, SLOTS).height(steps[height.get()]);
        if fixed.get() { line.top(100.0) } else { line }
    })
}

const LOOKS: &[&str] = &["Label", "Title", "Display S", "Display M", "Display L"];
const LOOK_TYPES: [ty::Type; 5] = [
    ty::LABEL_MEDIUM,
    ty::TITLE_MEDIUM,
    ty::DISPLAY_SMALL,
    ty::DISPLAY_MEDIUM,
    ty::DISPLAY_LARGE,
];

pub fn rolling(ctx: &Ctx) -> Container {
    let value = ctx.number("Value", 0.0..=999.0, 128.0);
    let ticking = ctx.bool("Tick every second", true);
    let kind = ctx.choice("Type", LOOKS, 3);
    let seconds = create_signal(0u32);
    set_interval(Duration::from_secs(1), move || {
        if ticking.get_untracked() {
            seconds.update(|s| *s += 1);
        }
    });
    let t = look();
    container()
        .layout(
            Flex::column()
                .spacing(space::S4.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(container().child(move || {
            roll::rolling(move || {
                let s = seconds.get();
                format!("{:02}:{:02}", s / 60 % 60, s % 60)
            })
            .look(LOOK_TYPES[kind.get()].em())
            .color(t.color(|c| c.text))
        }))
        .child(
            roll::rolling(move || format!("{:.0}%", value.get()))
                .look(ty::DISPLAY_XSMALL.em())
                .color(t.color(|c| c.accent.text)),
        )
}

const PRESETS: &[&str] = &["Battery", "Volume", "Wi-Fi"];
const PRESET_LEVELS: [Levels; 3] = [Levels::BATTERY, Levels::VOLUME, Levels::WIFI];
const LEVELS: &[&str] = &["As the app decides", "Normal", "Good", "Warn", "Alert"];
const LEVEL_CHOICES: [Option<Level>; 5] = [
    None,
    Some(Level::Normal),
    Some(Level::Good),
    Some(Level::Warn),
    Some(Level::Alert),
];
/// Where a battery turns to a warning, then an alert: the app's policy,
/// not styletto's.
const BATTERY_LEVELS: (f32, f32) = (30.0, 15.0);

/// The level an app would give a reading: a battery is good on the charger,
/// warns at 30% and alerts at 15%; a volume and a signal stay normal.
fn decided(preset: usize, value: f32, charging: bool) -> Level {
    match preset {
        0 if charging => Level::Good,
        0 => Level::below(value, BATTERY_LEVELS.0, BATTERY_LEVELS.1),
        _ => Level::Normal,
    }
}

/// The icon of a state off the scale: a battery charging, a volume muted.
fn state_icon(preset: usize, charging: bool, muted: bool) -> Option<Icon> {
    match preset {
        0 => charging.then_some(Icon::BatteryCharging),
        1 => muted.then_some(Icon::VolumeOff),
        _ => None,
    }
}

/// A value as the icon of its level, picked from a preset's set, inked by a
/// level the app decides. Charging applies to the battery, muted to the
/// volume. Below, every preset at a few values and in its states.
pub fn level_indicators(ctx: &Ctx) -> Container {
    let preset = ctx.choice("Preset", PRESETS, 0);
    let value = ctx.number("Value", 0.0..=100.0, 12.0);
    let charging = ctx.bool("Charging", false);
    let muted = ctx.bool("Muted", false);
    let percent = ctx.bool("Show percentage", true);
    let level = ctx.choice("Level", LEVELS, 0);
    let chosen = level_indicator(move || PRESET_LEVELS[preset.get()], value)
        .icon(move || state_icon(preset.get(), charging.get(), muted.get()))
        .level(move || {
            LEVEL_CHOICES[level.get()]
                .unwrap_or_else(|| decided(preset.get(), value.get(), charging.get()))
        })
        .percent(percent)
        .size(Size::Md);
    let gallery = |preset: usize, name: &'static str| {
        let at = move |value: f32, charging: bool, muted: bool| {
            level_indicator(PRESET_LEVELS[preset], value)
                .icon(state_icon(preset, charging, muted))
                .level(decided(preset, value, charging))
                .percent(percent)
        };
        let state = match preset {
            0 => Some(at(50.0, true, false)),
            1 => Some(at(0.0, false, true)),
            _ => None,
        };
        container()
            .layout(centered_row(space::S4.px()))
            .child(container().width(space::S15.px()).child(caption(name)))
            .children([0.0, 12.0, 30.0, 60.0, 100.0].map(move |value| at(value, false, false)))
            .child(state)
    };
    container()
        .layout(
            Flex::column()
                .spacing(space::S6.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(chosen)
        .child(
            container()
                .layout(Flex::column().spacing(space::S3.px()))
                .child(gallery(0, "Battery"))
                .child(gallery(1, "Volume"))
                .child(gallery(2, "Wi-Fi")),
        )
}

/// What marks something wanting attention, as the island shows it: the
/// unread dot, the glowing tag of what is in use, the tile an alert opens
/// with, the initial of what has no icon, the spinner of what runs.
pub fn badges(ctx: &Ctx) -> Container {
    let urgent = ctx.bool("Urgent", false);
    let microphone = ctx.bool("Microphone", true);
    let camera = ctx.bool("Camera", true);
    let screen = ctx.bool("Screen share", false);
    let name = ctx.text("Initial of", "Firefox");
    let t = look();
    // A screen shared is graver than a microphone or a camera in use.
    let tone = move || {
        if screen.get() {
            Tone::Danger
        } else {
            Tone::Warning
        }
    };
    let used = |on: RwSignal<bool>, kind: Icon| {
        container()
            .visible(on)
            .child(icon(kind, icon_box::XSMALL.px(), move || tone().ink()))
    };
    container()
        .layout(centered_row(space::S8.px()))
        .child(labelled(
            "unread",
            MARK,
            dot(t.color(move |c| {
                if urgent.get() {
                    c.danger.fill
                } else {
                    c.accent.fill
                }
            })),
        ))
        .child(labelled(
            "privacy",
            MARK,
            container()
                .visible(move || microphone.get() || camera.get() || screen.get())
                .child(
                    tag()
                        .tone(tone)
                        .glow(true)
                        .child(used(microphone, Icon::Mic))
                        .child(used(camera, Icon::Camera))
                        .child(used(screen, Icon::Monitor)),
                ),
        ))
        .child(labelled(
            "alert",
            MARK,
            icon_tile(Icon::Battery, t.color(|c| c.danger.text)),
        ))
        .child(labelled("initial", MARK, initial(name)))
        .child(labelled(
            "running",
            MARK,
            spinning(Icon::Refresh)
                .size(icon_box::MEDIUM)
                .color(t.color(|c| c.accent.text)),
        ))
}

const TONES: &[&str] = &["Neutral", "Accent", "Success", "Warning", "Danger"];
const TONE_OF: [Tone; 5] = [
    Tone::Neutral,
    Tone::Accent,
    Tone::Success,
    Tone::Warning,
    Tone::Danger,
];

/// Tags, which show and take no press: a count in a tone, the player
/// shown, readings on the surface.
pub fn tags(ctx: &Ctx) -> Container {
    let count = ctx.number("Updates", 0.0..=99.0, 12.0);
    let tone = ctx.choice("Tone", TONES, 1);
    let glow = ctx.bool("Glow", false);
    let chosen = ctx.bool("Player chosen", true);
    let t = look();
    let tone = move || TONE_OF[tone.get()];
    let player = move || {
        if chosen.get() {
            Tone::Accent
        } else {
            Tone::Neutral
        }
    };
    container()
        .layout(Flex::column().spacing(space::S4.px()))
        .child(sized(
            Size::Xxs,
            "Small",
            container()
                .layout(centered_row(space::S2.px()))
                .child(
                    tag()
                        .tone(tone)
                        .glow(glow)
                        .child(icon(Icon::Download, icon_box::SMALL.px(), move || {
                            tone().ink()
                        }))
                        .child(
                            text(move || count.get().round().to_string())
                                .typeset(ty::LABEL_SMALL.em())
                                .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                                .color(move || tone().ink())
                                .nowrap(),
                        ),
                )
                .child(
                    tag().tone(player).child(
                        text("Spotify")
                            .typeset(ty::LABEL_XSMALL.em())
                            .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                            .color(move || player().ink())
                            .nowrap(),
                    ),
                ),
        ))
        .child(sized(
            Size::Sm,
            "Medium",
            tag()
                .size(Size::Sm)
                .child(icon(
                    Icon::Headphones,
                    icon_box::SMALL.px(),
                    t.color(|c| c.sub),
                ))
                .child(
                    text("WH-1000XM4 · 70%")
                        .typeset(ty::LABEL_XSMALL)
                        .color(t.eased(|c| c.sub))
                        .nowrap(),
                ),
        ))
        .child(sized(
            Size::Md,
            "Large",
            tag()
                .size(Size::Md)
                .child(icon(
                    Icon::Battery,
                    icon_box::MEDIUM.px(),
                    t.color(|c| c.text),
                ))
                .child(
                    text("84%")
                        .typeset(ty::LABEL_MEDIUM.em())
                        .color(t.eased(|c| c.text))
                        .nowrap(),
                )
                .child(caption("Empty in 3h 10m")),
        ))
}

/// An app's icon and a cover drawn as SVGs, as the icon theme or a player
/// would hand them over.
const ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect x="4" y="4" width="40" height="40" rx="11" fill="#9ece6a"/><circle cx="24" cy="24" r="9" fill="#1a1b26"/></svg>"##;
const COVER_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#bb9af7"/><stop offset="1" stop-color="#ff9e64"/></linearGradient></defs><rect width="64" height="64" fill="url(#g)"/><circle cx="32" cy="32" r="12" fill="#1a1b26" fill-opacity="0.5"/></svg>"##;

fn svg(source: &str) -> ImageSource {
    ImageSource::SvgBytes(source.as_bytes().to_vec().into())
}

const TILES: &[&str] = &["XSmall", "Small", "Medium", "Large", "XLarge", "XXLarge"];
const TILE_SIZES: [TileSize; 6] = [
    tile::XSMALL,
    tile::SMALL,
    tile::MEDIUM,
    tile::LARGE,
    tile::XLARGE,
    tile::XXLARGE,
];

pub fn pictures(ctx: &Ctx) -> Container {
    let found = ctx.bool("Found", true);
    let size = ctx.choice("Size", TILES, 2);
    let t = look();
    container()
        .layout(centered_row(space::S8.px()))
        .child(labelled(
            "picture",
            PICTURE,
            container().child(move || {
                let size = TILE_SIZES[size.get()];
                picture()
                    .source(found.get().then(|| svg(ICON_SVG)))
                    .size(size)
                    .fallback(initial("Slack").size(size))
            }),
        ))
        .child(labelled(
            "artwork",
            PICTURE,
            container().child(move || {
                let size = TILE_SIZES[size.get()];
                artwork()
                    .source(found.get().then(|| svg(COVER_SVG)))
                    .size(size)
                    .fallback(app_icon(Icon::Music, t.color(|c| c.accent.fill)).size(size))
            }),
        ))
}

/// The cells marks and pictures are centred in.
const MARK: f32 = 38.0;
const PICTURE: f32 = 64.0;

/// A mark centred in a cell `height` tall, named under it.
fn labelled(name: &'static str, height: f32, shown: impl Widget + 'static) -> Container {
    container()
        .layout(
            Flex::column()
                .spacing(space::S2.px())
                .cross_alignment(CrossAlignment::Center),
        )
        .child(
            container()
                .height(height)
                .layout(Flex::column().center())
                .child(shown),
        )
        .child(caption(name))
}

/// A tag of `size` in its row, named before it.
fn sized(size: Size, name: &'static str, shown: impl Widget + 'static) -> Container {
    container()
        .layout(centered_row(space::S4.px()))
        .child(container().width(space::S15.px()).child(caption(name)))
        .child(container().height(size.height()).child(shown))
}
