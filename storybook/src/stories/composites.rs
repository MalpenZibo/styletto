use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use guido::prelude::*;
use styletto::components::appearance::{
    Family, SOURCES, Source, appearance as appearance_panel, color_editor, font_picker,
    layout_picker, mode_picker, theme_picker,
};
use styletto::components::button::{Fill, Variant, button};
use styletto::components::card::{level_figure, stat_card};
use styletto::components::choices::{choices, tabs};
use styletto::components::level::Level;
use styletto::components::list::{
    back, empty, page_header, page_indicator, pages, row, scroll_list, section, separator,
};
use styletto::components::notification::{app_icon, notification, notification_group};
use styletto::components::settings::{setting, setting_group};
use styletto::components::slider::slider_row;
use styletto::components::tag::Tone;
use styletto::components::tile::{small_tile, tile};
use styletto::components::toggle::toggle;
use styletto::foundations::control::Size;
use styletto::foundations::tokens;
use styletto::foundations::tokens::{icon as icon_box, size as space, tile};
use styletto::primitives::island;
use styletto::primitives::text::{caption, heading};
use styletto::theme::config::UiConfig;
use styletto::theme::presets::THEMES;
use styletto::{Icon, centered_row, icon, ui};

use super::{CPU_LEVELS, TEMP_LEVELS, panel};
use crate::stories::story::Ctx;

pub fn tiles(ctx: &Ctx) -> Container {
    let wifi = ctx.bool("Wi-Fi", true);
    let bluetooth = ctx.bool("Bluetooth", false);
    let night = ctx.bool("Night light", false);
    let dnd = ctx.bool("Do not disturb", true);
    let mic = ctx.bool("Microphone", false);
    // Off, the wide tiles carry no subtitle: their title alone, centred.
    let subtitles = ctx.bool("Subtitles", true);
    // On, Bluetooth's subtitle needs a second line, as a German one does;
    // Wi-Fi beside it keeps a short one, and the two stay as tall.
    let long = ctx.bool("Long subtitle", false);
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
    let subtitle = move |text: &str| {
        if subtitles.get() {
            text.to_string()
        } else {
            String::new()
        }
    };

    container()
        .width(panel())
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
                    .subtitle(move || subtitle(if wifi.get() { "Home 5G" } else { "Off" }))
                    .active(wifi)
                    .on_toggle(toggler("wifi", wifi))
                    .on_open(opener("wifi")),
                )
                .child(
                    tile(Icon::Bluetooth, "Bluetooth")
                        .subtitle(move || {
                            subtitle(if long.get() {
                                "Reduzierte Leistungsfähigkeit und niedriger Energieverbrauch"
                            } else if bluetooth.get() {
                                "2 devices"
                            } else {
                                "Off"
                            })
                        })
                        .active(bluetooth)
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
                        .active(night)
                        .on_toggle(toggler("night", night)),
                )
                .child(
                    small_tile(move || if dnd.get() { Icon::BellOff } else { Icon::Bell })
                        .active(dnd)
                        .on_toggle(toggler("dnd", dnd)),
                )
                .child(
                    small_tile(move || if mic.get() { Icon::Mic } else { Icon::MicOff })
                        .active(mic)
                        .on_toggle(toggler("mic", mic)),
                )
                .child(small_tile(Icon::Zap)),
        )
}

/// The settings rows the panel starts with: a slider per level, the icon
/// mutes, the chevron opens the device list. The volume may be amplified up
/// to its max.
pub fn slider_rows(ctx: &Ctx) -> Container {
    let volume = ctx.number("Volume", 0.0..=150.0, 64.0);
    let max = ctx.number("Max", 100.0..=150.0, 100.0);
    let step = ctx.number("Step", 1.0..=20.0, 5.0);
    let brightness = ctx.number("Brightness", 0.0..=100.0, 80.0);
    let muted = ctx.bool("Muted", false);
    let opened = ctx.action_with::<&'static str>("open");
    container()
        .width(panel())
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
            .max(max)
            .step(step)
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
    let subtitle = ctx.text("Subtitle", "");
    let checked = ctx.bool("Checked", false);
    let connected = create_signal(0usize);
    let stack = create_signal(vec![Page::Main]);
    let chosen = ctx.action_with::<&'static str>("connect");
    let t = ui();
    let push = move |page: Page| stack.update(|s| s.push(page));

    panel_frame().child(pages(stack, move |page| match page {
        Page::Main => container()
            .width(fill())
            .layout(Flex::column().spacing(space::S1.px()))
            .child(page_header("Settings").subtitle(subtitle))
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
            .child(row("Lock").icon(Icon::Lock).checked(checked))
            .child(row("Power off").icon(Icon::Power).enabled(false)),
        Page::Wifi => {
            let chosen = chosen.clone();
            container()
                .width(fill())
                .layout(Flex::column().spacing(space::S0_5.px()))
                .child(
                    page_header("Wi-Fi")
                        .subtitle(move || {
                            if none.get() {
                                String::new()
                            } else {
                                format!("Connected to {}", NETWORKS[connected.get()].0)
                            }
                        })
                        .on_back(back(stack))
                        .trailing(
                            button()
                                .icon(Icon::Refresh)
                                .size(Size::Sm)
                                .fill(Fill::Ghost),
                        ),
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
                        .children(
                            NETWORKS
                                .iter()
                                .enumerate()
                                .map(move |(i, &(name, secured))| {
                                    let chosen = chosen.clone();
                                    let details = container()
                                        .layout(centered_row(space::S1.px()))
                                        .children(secured.then(|| {
                                            icon(Icon::Lock, icon_box::SMALL.px(), t.colors.dim)
                                        }))
                                        .child(
                                            button()
                                                .icon(Icon::ChevronRight)
                                                .size(Size::Xs)
                                                .fill(Fill::Ghost)
                                                .on_click(move || push(Page::Network(name))),
                                        );
                                    row(name)
                                        .icon(Icon::Wifi)
                                        .checked(move || connected.get() == i)
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
                                }),
                        )
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

const TONES: &[&str] = &["None", "Warning", "Danger"];

/// A session's power page: the rows that end it in a tone of their own.
pub fn power_menu(ctx: &Ctx) -> Container {
    let tone = ctx.choice("Tone", TONES, 2);
    let chosen = ctx.action_with::<&'static str>("power");
    let tone = move || [Tone::Neutral, Tone::Warning, Tone::Danger][tone.get()];
    let item = |name: &'static str, kind: Icon| {
        let chosen = chosen.clone();
        row(name).icon(kind).on_click(move || chosen(name))
    };
    panel_frame()
        .layout(Flex::column().spacing(space::S1.px()))
        .child(page_header("Power"))
        .child(item("Lock", Icon::Lock))
        .child(item("Suspend", Icon::Moon))
        .child(item("Log out", Icon::LogOut))
        .child(separator())
        .child(item("Reboot", Icon::Refresh).tone(tone))
        .child(item("Shut down", Icon::Power).tone(tone))
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
        .width(panel())
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
        .width(panel())
        .layout(Flex::column().spacing(space::S2.px()))
        .child(
            notification_group("Messages")
                .count(2usize)
                .folded(folded)
                .lead(app_icon(Icon::Bell, t.colors.accent.fill).size(tile::XSMALL))
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
                .lead(app_icon(Icon::Battery, t.colors.danger.fill))
                .on_dismiss(move || dismissed("battery")),
        )
}

const DEVICES: &[(&str, Icon, bool)] = &[
    ("Headphones", Icon::Headphones, true),
    ("Keyboard", Icon::Keyboard, true),
    ("Mouse", Icon::Mouse, false),
    ("Speaker", Icon::Volume, false),
    ("Controller", Icon::Gamepad, false),
    ("Earbuds", Icon::Headphones, false),
    ("Monitor", Icon::Monitor, false),
    ("Microphone", Icon::Mic, false),
    ("Trackpad", Icon::Mouse, false),
    ("Camera", Icon::Camera, false),
    ("Phone", Icon::User, false),
    ("Watch", Icon::Zap, false),
];

/// Devices nearby, more than fit: the list stops growing and scrolls.
pub fn scrolling_list(ctx: &Ctx) -> Container {
    let lane = ctx.bool("Lane", false);
    let count = ctx.number("Devices", 0.0..=DEVICES.len() as f32, DEVICES.len() as f32);
    panel_frame()
        .layout(Flex::column().spacing(space::S0_5.px()))
        .child(page_header("Bluetooth").trailing(toggle(true)))
        .child(section("Nearby"))
        .child(
            scroll_list(space::scaled(space::S60.px()))
                .reserve_gutter(lane)
                .children(move || {
                    DEVICES[..count.get() as usize]
                        .iter()
                        .map(|&(name, kind, connected)| {
                            row(name)
                                .icon(kind)
                                .subtitle(if connected { "Connected" } else { "Paired" })
                                .checked(connected)
                        })
                        .collect::<Vec<_>>()
                }),
        )
}

/// Where the panel's colours come from, for its states to be seen.
const COLOURS: &[&str] = &["Preset", "Seed", "Wallpaper", "Custom"];

/// The families a font list offers, as an app would find them installed.
fn families() -> Vec<Family> {
    let family = |name: &str, monospace: bool| Family {
        name: name.to_string(),
        monospace,
        semibold: true,
    };
    vec![
        family("Inter", false),
        family("Noto Sans", false),
        family("JetBrains Mono", true),
    ]
}

/// The whole panel, or one of the pieces it is built from.
const PIECES: &[&str] = &["Panel", "Theme", "Mode", "Colour editor", "Fonts", "Layout"];

/// The appearance panel, editing a config of its own: what it chooses
/// shows in its controls, not in the story's look, which is the canvas's.
pub fn appearance(ctx: &Ctx) -> Container {
    let piece = ctx.choice("Piece", PIECES, 0);
    let colours = ctx.choice("Colours", COLOURS, 0);
    let picked = ctx.action("pick wallpaper");
    let config = create_signal(UiConfig::default());
    create_effect(move || {
        let name = COLOURS[colours.get()];
        if let Some(&(_, source)) = SOURCES.iter().find(|(known, _)| *known == name) {
            config.update(|c| {
                source.apply(c, &[]);
                // A wallpaper chosen, as an app's file chooser sets it.
                if source == Source::Wallpaper {
                    c.wallpaper = Some(wallpaper_picture().display().to_string());
                }
            });
        }
    });
    panel_frame().child(move || {
        let picked = picked.clone();
        match PIECES[piece.get()] {
            "Theme" => theme_picker(config).on_pick_wallpaper(picked).into_any(),
            "Mode" => mode_picker(config).into_any(),
            "Colour editor" => color_editor(config, false).into_any(),
            "Fonts" => font_picker(config).families(families()).into_any(),
            "Layout" => layout_picker(config).into_any(),
            _ => appearance_panel(config)
                .families(families())
                .on_pick_wallpaper(picked)
                .into_any(),
        }
    })
}

/// A picture for the panel to read as a wallpaper: a band of each of four
/// accents of the default theme, far apart in hue and narrower each, so
/// they come out as four colours in that order. Written once per run to the
/// user's runtime directory, which only they can write to, or else the
/// temporary one.
fn wallpaper_picture() -> &'static Path {
    static PICTURE: OnceLock<PathBuf> = OnceLock::new();
    PICTURE.get_or_init(|| {
        let dir =
            std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
        let path = dir.join("styletto-storybook-wallpaper.png");
        // Blue, green, orange, magenta, by their place among the accents.
        const BANDS: [(usize, u32); 4] = [(0, 48), (3, 32), (5, 24), (7, 16)];
        let accents = &THEMES[0].flavors[0].accents;
        let width = BANDS.iter().map(|&(_, w)| w).sum();
        let image = ::image::RgbImage::from_fn(width, 32, |x, _| {
            let mut x = x;
            let mut band = BANDS[0].0;
            for (accent, w) in BANDS {
                band = accent;
                if x < w {
                    break;
                }
                x -= w;
            }
            let (r, g, b, _) = accents[band].color.to_rgba8();
            ::image::Rgb([r, g, b])
        });
        // Written beside it and moved over it, so a storybook running at
        // the same time never reads half of it.
        let partial = path.with_extension(format!("{}.png", std::process::id()));
        if let Err(err) = image
            .save(&partial)
            .and_then(|()| Ok(std::fs::rename(&partial, &path)?))
        {
            eprintln!("storybook: no picture at {}: {err}", path.display());
        }
        path
    })
}

/// Where an app's bar sits on the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Edge {
    Top,
    Bottom,
}

/// How an app's clock shows the hours.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Hours {
    Twelve,
    TwentyFour,
}

/// An app's own settings, made of the pieces the appearance panel is made
/// of, above one of styletto's pickers: the same titles, names and buttons,
/// spaced the same.
pub fn settings(ctx: &Ctx) -> Container {
    let notes = ctx.bool("Notes", true);
    let enabled = ctx.bool("Enabled", true);
    let chose_edge = ctx.action_with::<Edge>("edge");
    let chose_hours = ctx.action_with::<Hours>("hours");
    let edge = create_signal(Edge::Top);
    let hours = create_signal(Hours::TwentyFour);
    let config = create_signal(UiConfig::default());
    panel_frame().child(
        container()
            .width(fill())
            .layout(Flex::column().spacing(space::S4.px()))
            .child(
                setting_group("Bar")
                    .child(
                        setting("Edge")
                            .note(move || {
                                if notes.get() {
                                    "Moves at once".to_string()
                                } else {
                                    String::new()
                                }
                            })
                            .child(
                                choices(
                                    vec![
                                        ("Top".to_string(), Edge::Top),
                                        ("Bottom".to_string(), Edge::Bottom),
                                    ],
                                    edge,
                                )
                                .enabled(enabled)
                                .on_choose(move |chosen| {
                                    edge.set(chosen);
                                    chose_edge(chosen);
                                }),
                            ),
                    )
                    .child(
                        setting("Clock").child(
                            choices(
                                vec![
                                    ("12-hour".to_string(), Hours::Twelve),
                                    ("24-hour".to_string(), Hours::TwentyFour),
                                ],
                                hours,
                            )
                            .enabled(enabled)
                            .on_choose(move |chosen| {
                                hours.set(chosen);
                                chose_hours(chosen);
                            }),
                        ),
                    ),
            )
            .child(setting_group("Layout").child(layout_picker(config))),
    )
}

/// The tabs of the window below, by the names they show.
const TABS: &[&str] = &["Bar", "Look", "About"];
/// The pages of its first start.
const STEPS: &[&str] = &["1", "2", "3", "4", "5"];

/// A first start's window: a header with its title, the tabs between it
/// and the close button, all as tall; a footer with Back and Next at its
/// ends and where the flow stands between them. Back and Next are solid,
/// so their edges line up with the title's; a ghost button's label sits
/// in by its padding.
pub fn tabs_and_steps(ctx: &Ctx) -> Container {
    let tab = ctx.choice("Tab", TABS, 1);
    let page = ctx.choice("Page", STEPS, 1);
    let enabled = ctx.bool("Enabled", true);
    let closed = ctx.action("close");
    let done = ctx.action("done");
    let t = ui();
    let header = container()
        .width(fill())
        .layout(centered_row(space::S2.px()))
        .child(side(MainAlignment::Start, heading("Welcome")))
        .child(
            tabs(
                TABS.iter()
                    .enumerate()
                    .map(|(index, name)| (name.to_string(), index))
                    .collect::<Vec<_>>(),
                tab,
            )
            .enabled(enabled)
            .on_choose(move |index| tab.set(index)),
        )
        .child(side(
            MainAlignment::End,
            button().icon(Icon::X).fill(Fill::Ghost).on_click(closed),
        ));
    let last = STEPS.len() - 1;
    let footer = container()
        .width(fill())
        .layout(centered_row(space::S2.px()))
        .child(side(
            MainAlignment::Start,
            button()
                .label("Back")
                .enabled(move || page.get() > 0)
                .on_click(move || page.update(|p| *p = p.saturating_sub(1))),
        ))
        .child(page_indicator(STEPS.len(), page))
        .child(side(
            MainAlignment::End,
            button()
                .label(move || if page.get() == last { "Done" } else { "Next" }.to_string())
                .variant(Variant::Accent)
                .on_click(move || {
                    if page.get_untracked() == last {
                        done();
                    } else {
                        page.update(|p| *p += 1);
                    }
                }),
        ));
    panel_frame().child(
        container()
            .width(fill())
            .layout(Flex::column().spacing(space::S4.px()))
            .child(header)
            .child(
                container()
                    .width(fill())
                    .height(space::S40.px())
                    .corners(tokens::radius::SMALL.corners())
                    .background(t.colors.surface)
                    .layout(Flex::column().center())
                    .child(caption(move || {
                        format!(
                            "{}: page {} of {}",
                            TABS[tab.get()],
                            page.get() + 1,
                            STEPS.len()
                        )
                    })),
            )
            .child(footer),
    )
}

/// One end of a header or a footer: as wide as the other, so what sits
/// between them is centred, holding `content` at its `end`.
fn side(end: MainAlignment, content: impl Widget + 'static) -> Container {
    container()
        .width(fill())
        .layout(Flex::row().main_alignment(end))
        .child(content)
}

/// A panel under the island, as the stories here draw it: painted like the
/// island, as wide as a panel, its corners nesting its rows'.
fn panel_frame() -> Container {
    island::paint(container())
        .width(panel())
        .padding(space::S3.px())
        .corners(tokens::radius::shape(tokens::radius::around(
            tokens::radius::SMALL.px(),
            space::S3.px(),
        )))
}

#[cfg(test)]
mod tests {
    use guido::prelude::*;
    use styletto::Icon;
    use styletto::components::choices::{choices, tabs};
    use styletto::components::tile::{small_tile, tile};
    use styletto::foundations::control::Size;
    use styletto::foundations::tokens::size as space;

    use super::Edge;
    use crate::stories::mounted::Mounted;

    /// The middle of a wide tile's icon box, its sides inset as much from
    /// the tile's top and left.
    fn icon_box() -> f32 {
        (Size::Lg.height() + 2.0 * space::S3.px()) / 2.0
    }

    #[test]
    fn a_press_on_a_tile_asks_for_the_opposite() {
        let Some((mut m, active)) = Mounted::<String>::new(|log| {
            let active = create_signal(true);
            let (toggled, opened) = (log.clone(), log);
            let wifi = tile(Icon::Wifi, "Wi-Fi")
                .active(active)
                .on_toggle(move |on| toggled.borrow_mut().push(format!("toggle({on})")))
                .on_open(move || opened.borrow_mut().push("open".into()));
            (container().width(200.0).child(wifi), active)
        }) else {
            return;
        };

        let middle = icon_box();
        assert_eq!(m.click(middle, middle), ["toggle(false)"]);
        // Until the owner keeps it, a press asks the same.
        assert_eq!(m.click(middle, middle), ["toggle(false)"]);
        // The rest of it opens the details.
        assert_eq!(m.click(150.0, middle), ["open"]);
        active.set(false);
        m.step();
        assert_eq!(m.click(middle, middle), ["toggle(true)"]);
    }

    #[test]
    fn a_wide_tile_lights_the_part_a_press_would_reach() {
        let Some((mut m, ())) = Mounted::<()>::new(|_| {
            let wifi = tile(Icon::Wifi, "Wi-Fi").on_open(|| {});
            (container().width(200.0).child(wifi), ())
        }) else {
            return;
        };
        let middle = icon_box();
        // A point of the body clear of its title.
        let body = (190.0, middle);
        let start = std::time::Instant::now();
        let settled = |n: u64| start + std::time::Duration::from_secs(n);
        m.step_at(settled(1));
        let rest = m.pixel(body.0, body.1);

        // On the body, the body lights up: its press opens.
        m.send_at(Event::mouse_move(150.0, middle), settled(2));
        m.step_at(settled(3));
        assert_ne!(m.pixel(body.0, body.1), rest);
        // On the icon box, only the box does: its press toggles.
        m.send_at(Event::mouse_move(middle, middle), settled(4));
        m.step_at(settled(5));
        assert_eq!(m.pixel(body.0, body.1), rest);
    }

    #[test]
    fn a_press_on_a_small_tile_asks_for_the_opposite() {
        let Some((mut m, active)) = Mounted::<bool>::new(|log| {
            let active = create_signal(false);
            let moon = small_tile(Icon::Moon)
                .active(active)
                .on_toggle(move |on| log.borrow_mut().push(on));
            (container().width(100.0).child(moon), active)
        }) else {
            return;
        };

        assert_eq!(m.click(50.0, 20.0), [true]);
        active.set(true);
        m.step();
        assert_eq!(m.click(50.0, 20.0), [false]);
    }

    #[test]
    fn disabled_choices_take_no_press() {
        let Some((mut m, enabled)) = Mounted::<Edge>::new(|log| {
            let enabled = create_signal(true);
            let edges = choices(
                vec![
                    ("Top".to_string(), Edge::Top),
                    ("Bottom".to_string(), Edge::Bottom),
                ],
                Edge::Bottom,
            )
            .enabled(enabled)
            .on_choose(move |edge| log.borrow_mut().push(edge));
            (container().width(200.0).child(edges), enabled)
        }) else {
            return;
        };

        assert_eq!(m.click(8.0, 8.0), [Edge::Top]);
        enabled.set(false);
        m.step();
        assert_eq!(m.click(8.0, 8.0), []);
        enabled.set(true);
        m.step();
        assert_eq!(m.click(8.0, 8.0), [Edge::Top]);
    }

    #[test]
    fn disabled_tabs_take_no_press() {
        let Some((mut m, enabled)) = Mounted::<Edge>::new(|log| {
            let enabled = create_signal(false);
            let edges = tabs(
                vec![
                    ("Top".to_string(), Edge::Top),
                    ("Bottom".to_string(), Edge::Bottom),
                ],
                Edge::Bottom,
            )
            .enabled(enabled)
            .on_choose(move |edge| log.borrow_mut().push(edge));
            (container().child(edges), enabled)
        }) else {
            return;
        };

        assert_eq!(m.click(8.0, 8.0), []);
        enabled.set(true);
        m.step();
        assert_eq!(m.click(8.0, 8.0), [Edge::Top]);
    }
}
