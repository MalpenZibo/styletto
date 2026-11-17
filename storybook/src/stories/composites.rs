use guido::prelude::*;
use styletto::components::button::{Fill, icon_button};
use styletto::components::card::{Level, level_figure, stat_card};
use styletto::components::list::{back, empty, page_header, pages, row, section, separator};
use styletto::components::notification::{app_icon, notification, notification_group};
use styletto::components::slider::slider_row;
use styletto::components::tile::{small_tile, tile};
use styletto::components::toggle::toggle;
use styletto::foundations::control::Size;
use styletto::foundations::tokens;
use styletto::foundations::tokens::{icon as icon_box, size as space, tile};
use styletto::primitives::island;
use styletto::{Icon, centered_row, icon, ui};

use super::{CPU_LEVELS, PANEL, TEMP_LEVELS};
use crate::stories::story::Ctx;

pub fn tiles(ctx: &Ctx) -> Container {
    let wifi = ctx.bool("Wi-Fi", true);
    let bluetooth = ctx.bool("Bluetooth", false);
    let night = ctx.bool("Night light", false);
    let dnd = ctx.bool("Do not disturb", true);
    let mic = ctx.bool("Microphone", false);
    let opened = ctx.action_with::<&'static str>("open");
    let toggled = ctx.action_with::<(&'static str, bool)>("toggle");

    let toggler = |name: &'static str, signal: RwSignal<bool>| {
        let toggled = toggled.clone();
        move |on: bool| {
            signal.set(on);
            toggled((name, on));
        }
    };
    let opener = |name: &'static str| {
        let opened = opened.clone();
        move || opened(name)
    };

    container()
        .width(PANEL)
        .layout(Flex::column().spacing(space::S2.px()))
        .child(
            container()
                .width(fill())
                .layout(Flex::row().spacing(space::S2.px()))
                .child(
                    tile(
                        move || {
                            if wifi.get() {
                                Icon::Wifi
                            } else {
                                Icon::WifiOff
                            }
                        },
                        "Wi-Fi",
                    )
                    .subtitle(move || if wifi.get() { "Home 5G" } else { "Off" }.to_string())
                    .on(wifi)
                    .on_toggle(toggler("wifi", wifi))
                    .on_open(opener("wifi")),
                )
                .child(
                    tile(Icon::Bluetooth, "Bluetooth")
                        .subtitle(move || {
                            if bluetooth.get() { "2 devices" } else { "Off" }.to_string()
                        })
                        .on(bluetooth)
                        .on_toggle(toggler("bluetooth", bluetooth))
                        .on_open(opener("bluetooth")),
                ),
        )
        .child(
            container()
                .width(fill())
                .layout(Flex::row().spacing(space::S2.px()))
                .child(
                    small_tile(Icon::Moon)
                        .on(night)
                        .on_toggle(toggler("night", night)),
                )
                .child(
                    small_tile(move || if dnd.get() { Icon::BellOff } else { Icon::Bell })
                        .on(dnd)
                        .on_toggle(toggler("dnd", dnd)),
                )
                .child(
                    small_tile(move || if mic.get() { Icon::Mic } else { Icon::MicOff })
                        .on(mic)
                        .on_toggle(toggler("mic", mic)),
                )
                .child(small_tile(Icon::Zap)),
        )
}

/// The settings rows the panel starts with: a slider per level, the icon
/// mutes, the chevron opens the device list.
pub fn slider_rows(ctx: &Ctx) -> Container {
    let volume = ctx.number("Volume", 0.0..=100.0, 64.0);
    let brightness = ctx.number("Brightness", 0.0..=100.0, 80.0);
    let muted = ctx.bool("Muted", false);
    let opened = ctx.action_with::<&'static str>("open");
    container()
        .width(PANEL)
        .layout(Flex::column().spacing(space::S1.px()))
        .child(
            slider_row(
                move || {
                    if muted.get() {
                        Icon::VolumeOff
                    } else {
                        Icon::Volume
                    }
                },
                volume,
            )
            .on_change(move |v| volume.set(v))
            .on_icon(move || muted.set(!muted.get_untracked()))
            .on_more(move || opened("audio")),
        )
        .child(slider_row(Icon::Sun, brightness).on_change(move |v| brightness.set(v)))
}

#[derive(Clone, PartialEq)]
enum Page {
    Main,
    Wifi,
    Network(&'static str),
}

const NETWORKS: &[(&str, bool)] = &[
    ("Home 5G", true),
    ("Home", true),
    ("Café Centrale", false),
    ("Office", true),
];

pub fn list_pages(ctx: &Ctx) -> Container {
    let none = ctx.bool("No networks", false);
    let connected = create_signal(0usize);
    let stack = create_signal(vec![Page::Main]);
    let chosen = ctx.action_with::<&'static str>("connect");
    let t = ui();
    let push = move |page: Page| stack.update(|s| s.push(page));

    island::paint(container())
        .width(PANEL)
        .padding(space::S3.px())
        .corners(tokens::radius::shape(tokens::radius::around(
            tokens::radius::SMALL.px(),
            space::S3.px(),
        )))
        .child(pages(stack, move |page| match page {
            Page::Main => container()
                .width(fill())
                .layout(Flex::column().spacing(space::S1.px()))
                .child(page_header("Settings"))
                .child(
                    row("Wi-Fi")
                        .icon(Icon::Wifi)
                        .subtitle(move || NETWORKS[connected.get()].0.to_string())
                        .trailing(icon(Icon::ChevronRight, icon_box::SMALL.px(), t.colors.dim))
                        .on_click(move || push(Page::Wifi)),
                )
                .child(
                    row("Bluetooth")
                        .icon(Icon::Bluetooth)
                        .subtitle("Off")
                        .trailing(toggle(false)),
                )
                .child(separator())
                .child(row("Lock").icon(Icon::Lock))
                .child(row("Power off").icon(Icon::Power).enabled(false)),
            Page::Wifi => {
                let chosen = chosen.clone();
                container()
                    .width(fill())
                    .layout(Flex::column().spacing(space::S0_5.px()))
                    .child(
                        page_header("Wi-Fi")
                            .on_back(back(stack))
                            .trailing(icon_button(Icon::Refresh).size(Size::Sm).fill(Fill::Ghost)),
                    )
                    .child(move || {
                        if none.get() {
                            return empty(Icon::WifiOff, "No networks in range").into_any();
                        }
                        let chosen = chosen.clone();
                        container()
                            .width(fill())
                            .layout(Flex::column().spacing(space::S0_5.px()))
                            .child(section("Known"))
                            .children(NETWORKS.iter().enumerate().map(
                                move |(i, &(name, secured))| {
                                    let chosen = chosen.clone();
                                    let details = container()
                                        .layout(centered_row(space::S1.px()))
                                        .children(secured.then(|| {
                                            icon(Icon::Lock, icon_box::SMALL.px(), t.colors.dim)
                                        }))
                                        .child(
                                            icon_button(Icon::ChevronRight)
                                                .size(Size::Xs)
                                                .fill(Fill::Ghost)
                                                .on_click(move || push(Page::Network(name))),
                                        );
                                    row(name)
                                        .icon(Icon::Wifi)
                                        .selected(move || connected.get() == i)
                                        .subtitle(move || {
                                            if connected.get() == i {
                                                "Connected"
                                            } else {
                                                "Saved"
                                            }
                                            .to_string()
                                        })
                                        .trailing(details)
                                        .on_click(move || {
                                            connected.set(i);
                                            chosen(name);
                                        })
                                },
                            ))
                            .into_any()
                    })
            }
            Page::Network(name) => container()
                .width(fill())
                .layout(Flex::column().spacing(space::S1.px()))
                .child(page_header(name).on_back(back(stack)))
                .child(row("Auto-connect").trailing(toggle(true)))
                .child(row("Forget network").icon(Icon::X)),
        }))
}

/// Cards for the panel, and the figures under the island, both coloured
/// past their thresholds.
pub fn cards(ctx: &Ctx) -> Container {
    let cpu = ctx.number("CPU %", 0.0..=100.0, 23.0);
    let temp = ctx.number("Temperature °C", 30.0..=100.0, 61.0);
    let cpu_level = move || Level::of(cpu.get(), CPU_LEVELS.0, CPU_LEVELS.1);
    let temp_level = move || Level::of(temp.get(), TEMP_LEVELS.0, TEMP_LEVELS.1);
    let cpu_figure = move || format!("{:.0}%", cpu.get());
    let temp_figure = move || format!("{:.0}°", temp.get());
    container()
        .width(PANEL)
        .layout(Flex::column().spacing(space::S4.px()))
        .child(
            container()
                .width(fill())
                .layout(Flex::row().spacing(space::S2.px()))
                .child(stat_card(Icon::Cpu, "CPU", cpu_figure).level(cpu_level))
                .child(stat_card(Icon::Thermometer, "Temperature", temp_figure).level(temp_level)),
        )
        .child(
            container()
                .layout(Flex::row().spacing(space::S3.px()))
                .child(level_figure(Icon::Cpu, cpu_figure, cpu_level))
                .child(level_figure(Icon::Thermometer, temp_figure, temp_level)),
        )
}

pub fn notifications(ctx: &Ctx) -> Container {
    let urgent = ctx.bool("Urgent", true);
    let folded = ctx.bool("Folded", false);
    let dismissed = ctx.action_with::<&'static str>("dismiss");
    let opened = ctx.action_with::<&'static str>("open");
    let t = ui();
    let item = |who: &'static str, title: &str, body: &str, time: &str| {
        let (dismissed, opened) = (dismissed.clone(), opened.clone());
        notification(title)
            .body(body)
            .time(time)
            .on_dismiss(move || dismissed(who))
            .on_click(move || opened(who))
    };
    let first = item(
        "anna",
        "Anna",
        "Are we still on for tonight? I booked the table for eight.",
        "2m",
    );
    let second = item(
        "marco",
        "Marco",
        "Sent you the photos from the weekend, have a look when you can and tell me which ones to print.",
        "14m",
    );
    container()
        .width(PANEL)
        .layout(Flex::column().spacing(space::S2.px()))
        .child(
            notification_group("Messages")
                .count(2usize)
                .folded(folded)
                .lead(app_icon(Icon::Bell, t.colors.accent).size(tile::XSMALL))
                .on_toggle(move || folded.set(!folded.get_untracked()))
                .child(first)
                .child(
                    container()
                        .width(fill())
                        .visible(move || !folded.get())
                        .child(second),
                ),
        )
        .child(
            notification("Battery low")
                .body("8% left. Plug in the charger.")
                .time("now")
                .urgent(urgent)
                .lead(app_icon(Icon::Battery, t.colors.danger))
                .on_dismiss(move || dismissed("battery")),
        )
}
