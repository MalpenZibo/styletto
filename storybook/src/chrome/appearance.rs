//! What the stories are drawn with, and the page that chooses it: the
//! library's appearance panel editing a `UiConfig`, beside a preview of what
//! it makes. The page hands the panel what reaches the system: the system's
//! dark preference and a wallpaper chosen through the desktop portal, which
//! the library reads; the panel finds the families installed itself, in
//! Guido's list. The storybook's own chrome keeps the default.

use std::path::PathBuf;

use crate::system::color_scheme::{self, ColorScheme};
use futures::StreamExt;
use guido::prelude::*;
use styletto::components::appearance::appearance;
use styletto::components::button::{Variant, button};
use styletto::components::card::level_figure;
use styletto::components::level::Level;
use styletto::components::list::row;
use styletto::components::progress::ring;
use styletto::components::slider::slider_row;
use styletto::components::tile::small_tile;
use styletto::components::toggle::toggle;
use styletto::foundations::tokens::{icon as icon_box, radius, size as space};
use styletto::foundations::typography as ty;
use styletto::primitives::text::typed;
use styletto::theme::config::UiConfig;
use styletto::theme::palette::Hue;
use styletto::theme::wallpaper::{Picture, picture};
use styletto::{Icon, Ui, centered_row, icon, look, themed};

use crate::chrome::widgets::{island, wallpaper};
use crate::system::picker;

/// Asks the system's file chooser for an image, and sets the config's
/// wallpaper to it; where there is no chooser, says so in `note`. The
/// chooser is a window, so the storybook steps below windows while it is
/// open.
fn chooser(config: WriteSignal<UiConfig>, note: WriteSignal<String>) -> Service<()> {
    let choosing = create_signal(false);
    if let Some(surface) = use_context::<SurfaceHandle>() {
        create_effect(move || {
            surface.set_layer(if choosing.get() {
                Layer::Bottom
            } else {
                Layer::Top
            })
        });
    }
    let choosing = choosing.writer();
    create_service(move |mut asked, _| async move {
        while asked.recv().await.is_some() {
            choosing.set(true);
            let chosen = picker::image("Choose a wallpaper").await;
            choosing.set(false);
            match chosen {
                Ok(Some(path)) => {
                    let path = path.display().to_string();
                    config.update(move |c| c.set_wallpaper(path));
                }
                Ok(None) => {}
                Err(err) => note.set(format!("No file chooser: {err}")),
            }
        }
    })
}

#[derive(Clone, Copy)]
pub struct Appearance {
    config: RwSignal<UiConfig>,
    /// What the system prefers: dark, light, or nothing said.
    system: Signal<Option<bool>>,
    chooser: Service<()>,
    /// What the chooser said, until another wallpaper is set.
    chooser_note: RwSignal<String>,
    /// The picture read last, shown behind the stories in place of the
    /// shapes, whatever the source is now.
    backdrop: Memo<Option<PathBuf>>,
}

impl Appearance {
    pub fn new() -> Self {
        let config = create_signal(UiConfig::default());
        let chooser_note = create_signal(String::new());
        let wallpaper = create_memo(move || config.with(UiConfig::wallpaper_path));
        // The last path set, kept when the source changes; a new one
        // clears what the chooser said.
        let last = create_signal(None);
        create_effect(move || {
            if let Some(path) = wallpaper.get() {
                last.set(Some(path));
                chooser_note.set(String::new());
            }
        });
        let backdrop = create_memo(move || {
            last.get()
                .filter(|path| matches!(picture(path), Picture::Read(_)))
        });
        Self {
            config,
            system: system_dark(),
            chooser: chooser(config.writer(), chooser_note.writer()),
            chooser_note,
            backdrop,
        }
    }

    pub fn backdrop(self) -> Memo<Option<PathBuf>> {
        self.backdrop
    }

    /// The config, the wallpaper in it showing as the theme while it is
    /// being read.
    pub fn ui(self) -> Ui {
        let system = self.system.get();
        self.config.with(|c| c.ui(system))
    }
}

/// The system's preference, from the desktop portal, as it changes.
fn system_dark() -> Signal<Option<bool>> {
    let dark = create_signal(None);
    let writer = dark.writer();
    create_service::<(), _, _>(move |_, _| async move {
        let appearance = match color_scheme::Appearance::connect().await {
            Ok(appearance) => appearance,
            Err(err) => {
                eprintln!("storybook: no colour scheme: {err}");
                return;
            }
        };
        let mut updates = std::pin::pin!(appearance.updates());
        while let Some(scheme) = updates.next().await {
            writer.set(match scheme {
                ColorScheme::Dark => Some(true),
                ColorScheme::Light => Some(false),
                ColorScheme::NoPreference => None,
            });
        }
    });
    dark.read_only()
}

/// The settings on the left, what they make on the right, drawn with
/// `drawn`, the `Ui` the settings make.
pub fn page(a: Appearance, drawn: Memo<Ui>) -> Container {
    container()
        .width(fill())
        .height(fill())
        .layout(Flex::row())
        .child(
            container()
                .width(space::S100.px())
                .height(fill())
                .padding([0.0, space::S4.px(), space::S6.px(), space::S6.px()])
                .scroll(Scroll::vertical())
                .child(
                    appearance(a.config)
                        .system_dark(a.system)
                        .on_pick_wallpaper(move || a.chooser.send(()))
                        .chooser_note(a.chooser_note),
                ),
        )
        .child(preview(a, drawn))
}

/// What the controls in the preview were left at, kept across a redraw so a
/// change of theme does not reset them.
#[derive(Clone, Copy)]
struct Controls {
    wifi: RwSignal<bool>,
    volume: RwSignal<f32>,
    muted: RwSignal<bool>,
    auto: RwSignal<bool>,
    night: RwSignal<bool>,
    silent: RwSignal<bool>,
}

/// The colours at work, drawn with what is chosen: controls that answer a
/// press, text at its three levels on the island and on a surface, and
/// every fill with the colour that reads on it.
fn preview(a: Appearance, drawn: Memo<Ui>) -> Container {
    let controls = Controls {
        wifi: create_signal(true),
        volume: create_signal(60.0),
        muted: create_signal(false),
        auto: create_signal(true),
        night: create_signal(true),
        silent: create_signal(false),
    };
    container()
        .width(fill())
        .height(fill())
        .padding(space::S3.px())
        .child(wallpaper(
            a.backdrop,
            themed(drawn, move || {
                styletto::primitives::island::border(island(space::S4.px()))
                    .width(space::S80.px())
                    .layout(Flex::column().spacing(space::S4.px()))
                    .child(panel(controls))
                    .child(texts())
                    .child(fills())
                    .child(levels())
            }),
        ))
}

/// A piece of a panel whose controls answer: the row's switch, the slider
/// and its mute, a button that toggles, two tiles.
fn panel(c: Controls) -> Container {
    let flip = |on: RwSignal<bool>| move || on.set(!on.get_untracked());
    container()
        .width(fill())
        .layout(Flex::column().spacing(space::S2.px()))
        .child(
            row("Wi-Fi")
                .icon(Icon::Wifi)
                .subtitle(move || if c.wifi.get() { "Home 5G" } else { "Off" }.to_string())
                .selected(c.wifi)
                .trailing(toggle(c.wifi).on_toggle(move |on| c.wifi.set(on)))
                .on_click(flip(c.wifi)),
        )
        .child(
            slider_row(
                move || {
                    if c.muted.get() {
                        Icon::VolumeOff
                    } else {
                        Icon::Volume
                    }
                },
                move || if c.muted.get() { 0.0 } else { c.volume.get() },
            )
            .on_change(move |v| {
                c.volume.set(v);
                c.muted.set(false);
            })
            .on_icon(flip(c.muted)),
        )
        .child(
            container()
                .width(fill())
                .layout(Flex::row().spacing(space::S2.px()))
                .child(
                    container().width(fill()).child(
                        small_tile(Icon::Moon)
                            .active(c.night)
                            .on_toggle(move |on| c.night.set(on)),
                    ),
                )
                .child(
                    container().width(fill()).child(
                        small_tile(move || {
                            if c.silent.get() {
                                Icon::BellOff
                            } else {
                                Icon::Bell
                            }
                        })
                        .active(c.silent)
                        .on_toggle(move |on| c.silent.set(on)),
                    ),
                ),
        )
        .child(
            container()
                .layout(centered_row(space::S1.px()))
                .child(
                    button()
                        .label("Auto")
                        .active(c.auto)
                        .on_toggle(move |on| c.auto.set(on)),
                )
                .child(button().label("Join").variant(Variant::Accent))
                .child(button().label("Forget").variant(Variant::Danger)),
        )
}

/// The two levels of text and the dim of an icon, on the island and on a
/// surface.
fn texts() -> Container {
    let t = look();
    let levels = || {
        container()
            .layout(Flex::column().spacing(space::S1.px()))
            .child(typed("Text", ty::LABEL_MEDIUM.em()).color(t.eased(|c| c.text)))
            .child(typed("Sub", ty::LABEL_SMALL).color(t.eased(|c| c.sub)))
            .child(icon(
                Icon::ChevronRight,
                icon_box::SMALL.px(),
                t.color(|c| c.dim),
            ))
    };
    container()
        .width(fill())
        .layout(Flex::row().spacing(space::S2.px()))
        .child(
            container()
                .width(fill())
                .padding(space::S2.px())
                .child(levels()),
        )
        .child(
            container()
                .width(fill())
                .padding(space::S2.px())
                .corners(radius::SMALL.corners())
                .background(t.eased(|c| c.surface))
                .child(levels()),
        )
}

/// Each fill with the colour that reads on it, two to a row so a larger
/// density keeps every name on its line.
fn fills() -> Container {
    let t = look();
    let swatch = |hue: Hue| {
        container()
            .width(fill())
            .padding([space::S1.px(), space::S2.px()])
            .corners(radius::SMALL.corners())
            .background(t.eased(move |c| c.hue(hue).fill))
            .layout(Flex::row().main_alignment(MainAlignment::Center))
            .child(
                typed(format!("{hue:?}"), ty::LABEL_SMALL.em())
                    .color(t.eased(move |c| c.hue(hue).on))
                    .nowrap(),
            )
    };
    let pair = |a: Container, b: Container| {
        container()
            .width(fill())
            .layout(Flex::row().spacing(space::S1.px()))
            .child(a)
            .child(b)
    };
    let [accent, success, warning, danger] = Hue::ALL.map(swatch);
    container()
        .width(fill())
        .layout(Flex::column().spacing(space::S1.px()))
        .child(pair(accent, success))
        .child(pair(warning, danger))
}

/// The states as the ink of a level: figures past their thresholds and the
/// battery charging, low and nearly out.
fn levels() -> Container {
    let t = look();
    container()
        .width(fill())
        .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
        .child(level_figure(Icon::Cpu, "23%", Level::Normal))
        .child(level_figure(Icon::Thermometer, "74°", Level::Warn))
        .child(level_figure(Icon::MemoryStick, "93%", Level::Alert))
        .child(
            container().layout(centered_row(space::S2.px())).children(
                [
                    (0.64, Hue::Success),
                    (0.24, Hue::Warning),
                    (0.09, Hue::Danger),
                ]
                .map(|(value, hue)| {
                    ring(value)
                        .color(t.color(move |c| c.hue(hue).fill))
                        .size(icon_box::SMALL)
                }),
            ),
        )
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use guido::prelude::*;
    use guido::testing::Headless;
    use styletto::components::appearance::{Source, appearance, color_editor};
    use styletto::theme::config::UiConfig;
    use styletto::theme::palette::parse_hex;
    use styletto::theme::presets::Mode;
    use styletto::ui::{Density, Material};

    use super::Appearance;
    use crate::stories::mounted::{Bench, still_backdrop};

    #[test]
    fn the_page_keeps_the_sides_apart() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let probe = Rc::new(Cell::new(None));
        let seen = probe.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let a = Appearance::new();
            seen.set(Some(a));
            super::page(a, create_memo(move || a.ui()))
        });
        h.configure(id, 1200, 3000, 1.0);
        h.step();
        let a: Appearance = probe.take().unwrap();
        a.config.update(|c| {
            Source::Custom.apply(c, &[]);
            c.mode = Mode::Light;
        });
        h.step();
        let before = a.config.get_untracked();
        // The light accent, as its field sets it, with both sides' editors
        // up: neither writes what it holds back over it.
        a.config.update(|c| {
            c.light.insert("accent".to_string(), "#ff0000".to_string());
        });
        h.step();
        let red = parse_hex("#ff0000").unwrap();
        assert_eq!(
            a.config.get_untracked().dark,
            before.dark,
            "the dark side moved"
        );
        assert_eq!(a.ui().colors.accent.fill, red);
        a.config.update(|c| c.mode = Mode::Dark);
        h.step();
        let after = a.config.get_untracked();
        assert_eq!(after.dark, before.dark, "the dark side moved on the switch");
        assert_eq!(after.light["accent"], "#ff0000", "the light edit was lost");
        assert_eq!(
            Some(a.ui().colors.accent.fill),
            parse_hex(&before.dark["accent"]),
            "dark mode shows the light edit"
        );
    }

    #[test]
    fn a_colour_field_follows_a_change_made_elsewhere() {
        let custom = |accent: Option<&str>| {
            let mut c = UiConfig::default();
            Source::Custom.apply(&mut c, &[]);
            if let Some(accent) = accent {
                c.dark.insert("accent".to_string(), accent.to_string());
            }
            c
        };
        // One editor whose config changes from outside, one built with the
        // config it changes to: once it has, they draw the same.
        let editor = |config| container().width(fill()).child(color_editor(config, false));
        let Some(mut bench) = Bench::new() else {
            return;
        };
        let mut pair = bench.pair((400, 900), custom(None), custom(Some("#00ff00")), editor);
        assert!(pair.unlike().is_some());
        pair.change();
        if let Some(unlike) = pair.unlike() {
            panic!("the field did not show the colour set elsewhere: {unlike}");
        }
    }

    #[test]
    fn a_choice_in_the_panel_changes_the_config() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let probe = Rc::new(Cell::new(None));
        let seen = probe.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let config = create_signal(UiConfig::default());
            seen.set(Some(config));
            container().width(fill()).child(appearance(config))
        });
        h.configure(id, 400, 1600, 1.0);
        h.step();
        let config: RwSignal<UiConfig> = probe.take().unwrap();
        // Down the panel's left edge, past the section's title and the
        // setting's name, to the first option there: the mode's first.
        for y in (0..200).step_by(4) {
            h.click(id, 24.0, y as f32);
            h.step();
            if config.get_untracked() != UiConfig::default() {
                break;
            }
        }
        assert_eq!(
            config.get_untracked(),
            UiConfig {
                mode: Mode::Light,
                ..UiConfig::default()
            }
        );
    }

    #[test]
    fn a_new_palette_repaints_the_preview_as_built_with_it() {
        let light = UiConfig {
            mode: Mode::Light,
            material: Material::Glass,
            ..UiConfig::default()
        };
        // One preview whose config changes, one built with the config it
        // changes to.
        let preview = |config| {
            let a = Appearance {
                config,
                backdrop: still_backdrop(),
                ..Appearance::new()
            };
            super::preview(a, create_memo(move || a.ui()))
        };
        let Some(mut bench) = Bench::new() else {
            return;
        };
        let mut pair = bench.pair((400, 800), UiConfig::default(), light, preview);
        assert!(pair.unlike().is_some());
        pair.change();
        if let Some(unlike) = pair.unlike() {
            panic!("the preview kept a colour of the old palette: {unlike}");
        }
    }

    #[test]
    fn a_setting_does_not_rebuild_the_page() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let built = Rc::new(Cell::new(0));
        let probe = Rc::new(Cell::new(None));
        let (count, seen) = (built.clone(), probe.clone());
        let id = h.surface(SurfaceConfig::new(), move || {
            let a = Appearance::new();
            seen.set(Some(a));
            container().child(move || {
                count.set(count.get() + 1);
                super::page(a, create_memo(move || a.ui()))
            })
        });
        h.configure(id, 1200, 900, 1.0);
        h.step();
        let a: Appearance = probe.take().unwrap();
        for change in [
            Box::new(move || a.config.update(|c| c.material = Material::Glass)) as Box<dyn Fn()>,
            Box::new(move || a.config.update(|c| c.density = Density::Compact)),
            Box::new(move || a.config.update(|c| c.mode = Mode::Light)),
            Box::new(move || a.config.update(|c| Source::Custom.apply(c, &[]))),
        ] {
            let before = built.get();
            change();
            h.step();
            assert_eq!(built.get(), before, "the page was built again");
        }
    }

    #[test]
    fn a_setting_keeps_the_scroll() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let probe = Rc::new(Cell::new(None));
        let seen = probe.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let a = Appearance::new();
            seen.set(Some(a));
            super::page(a, create_memo(move || a.ui()))
        });
        let (width, height) = (1200, 700);
        h.configure(id, width, height, 1.0);
        h.step();
        let a: Appearance = probe.take().unwrap();
        // The top of the settings column, where no setting is drawn.
        let top = |frame: &[u8]| -> Vec<u8> {
            frame
                .chunks(width as usize * 4)
                .take(120)
                .flat_map(|line| line[..400 * 4].to_vec())
                .collect()
        };
        let unscrolled = top(&h.read_frame(id));
        let start = std::time::Instant::now();
        h.event_at(
            id,
            Event::scroll(200.0, 400.0, 0.0, 600.0, ScrollSource::Finger),
            start,
        );
        for frame in 1..60 {
            h.step_at(start + std::time::Duration::from_millis(frame * 16));
        }
        assert!(
            top(&h.read_frame(id)) != unscrolled,
            "the scroll did not move"
        );
        a.config.update(|c| c.material = Material::Glass);
        for frame in 60..120 {
            h.step_at(start + std::time::Duration::from_millis(frame * 16));
        }
        assert!(
            top(&h.read_frame(id)) != unscrolled,
            "the scroll went back to the top"
        );
        // A press anywhere down the column, as an option is chosen.
        let mut frame = 120;
        for y in (140..680).step_by(20) {
            h.click(id, 60.0, y as f32);
            for _ in 0..30 {
                frame += 1;
                h.step_at(start + std::time::Duration::from_millis(frame * 16));
            }
            assert!(
                top(&h.read_frame(id)) != unscrolled,
                "a press at {y} sent the scroll back to the top"
            );
        }
    }
}
