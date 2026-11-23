//! Controls that edit an app's [`UiConfig`], each change written to it at
//! once: where the colours come from ([`theme_picker`]), the mode
//! ([`mode_picker`]), one side's colours by hand ([`color_editor`]), the
//! fonts ([`font_picker`]) and the density, shape, material and motion
//! ([`layout_picker`]); and [`appearance`], a panel of them in sections, as
//! wide as the app makes it.
//!
//! What reaches the system is the app's, handed in: a file chooser for a
//! wallpaper, and the font families installed, which the font picker reads
//! by itself from Guido ([`installed_families`]) when the app gives none. A
//! seed is offered with the `material` feature, a wallpaper with `wallpaper`.

mod fonts;
#[cfg(test)]
mod translated;

pub use fonts::installed_families;

use guido::prelude::*;

use crate::components::button::{button, pressable};
use crate::components::choices::choices;
use crate::components::input::field as input_field;
use crate::components::list::scroll_list;
use crate::components::settings::{Setting, setting, setting_group};
use crate::foundations::control::Size;
use crate::foundations::icons::{Icon, icon};
use crate::foundations::motion;
use crate::foundations::tokens::{icon as icon_box, radius, size, stroke};
use crate::foundations::typography::{BODY_SMALL, LABEL_MEDIUM, MONO_SMALL, TITLE_XSMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::primitives::text::{PaletteColors, caption};
use crate::theme::config::{Colors, UiConfig};
use crate::theme::material::{Contrast, Variant};
use crate::theme::palette::{Palette, Scheme, parse_hex, to_hex};
use crate::theme::presets::{Mode, THEMES, ThemeChoice};
#[cfg(feature = "wallpaper")]
use crate::theme::wallpaper::{self, Picture};
use crate::ui::{Density, Material, Motion, Shape, Words, look, words};
use crate::words::english;

/// A list of choices twice: by English name, public, and by key in
/// [`KEYS`](crate::words::KEYS), what a control shows, translated. The
/// English is taken from `KEYS`, so it is written once.
macro_rules! choices_of {
    ($(#[$doc:meta])* $named:ident, $(#[$own:meta])* $keyed:ident: $kind:ty = [
        $($(#[$attr:meta])* $key:literal => $value:expr,)*
    ]) => {
        $(#[$doc])*
        pub const $named: &[(&str, $kind)] = &[$($(#[$attr])* (english($key), $value),)*];
        $(#[$own])*
        const $keyed: &[(&str, $kind)] = &[$($(#[$attr])* ($key, $value),)*];
    };
}

choices_of!(
    /// The choices of each kind, by the English name a control shows,
    /// translated by its key in [`KEYS`](crate::words::KEYS).
    MODES, MODE_KEYS: Mode = [
        "styletto-mode-light" => Mode::Light,
        "styletto-mode-dark" => Mode::Dark,
        "styletto-mode-auto" => Mode::Auto,
    ]
);
choices_of!(DENSITIES, DENSITY_KEYS: Density = [
    "styletto-density-compact" => Density::Compact,
    "styletto-density-default" => Density::Default,
    "styletto-density-comfortable" => Density::Comfortable,
]);
choices_of!(SHAPES, SHAPE_KEYS: Shape = [
    "styletto-shape-square" => Shape::Square,
    "styletto-shape-soft" => Shape::Soft,
    "styletto-shape-round" => Shape::Round,
]);
choices_of!(MATERIALS, MATERIAL_KEYS: Material = [
    "styletto-material-solid" => Material::Solid,
    "styletto-material-translucent" => Material::Translucent,
    "styletto-material-glass" => Material::Glass,
]);
choices_of!(MOTIONS, MOTION_KEYS: Motion = [
    "styletto-motion-full" => Motion::Full,
    "styletto-motion-reduced" => Motion::Reduced,
    "styletto-motion-none" => Motion::None,
]);
// What only a seed or a wallpaper offers.
choices_of!(VARIANTS, #[cfg(feature = "material")] VARIANT_KEYS: Variant = [
    "styletto-variant-tonal-spot" => Variant::TonalSpot,
    "styletto-variant-vibrant" => Variant::Vibrant,
    "styletto-variant-expressive" => Variant::Expressive,
    "styletto-variant-content" => Variant::Content,
    "styletto-variant-fidelity" => Variant::Fidelity,
    "styletto-variant-neutral" => Variant::Neutral,
    "styletto-variant-monochrome" => Variant::Monochrome,
    "styletto-variant-rainbow" => Variant::Rainbow,
    "styletto-variant-fruit-salad" => Variant::FruitSalad,
]);
choices_of!(CONTRASTS, #[cfg(feature = "material")] CONTRAST_KEYS: Contrast = [
    "styletto-contrast-standard" => Contrast::Standard,
    "styletto-contrast-medium" => Contrast::Medium,
    "styletto-contrast-high" => Contrast::High,
]);

/// Where a config's colours come from, as [`theme_picker`] offers it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    /// A preset theme, in the flavours and accent chosen.
    #[default]
    Preset,
    /// Both sides worked out from a seed colour.
    Seed,
    /// Both sides worked out from a wallpaper.
    Wallpaper,
    /// Every colour of each side set by hand, in `dark` and `light`.
    Custom,
}

choices_of!(
    /// The sources this build offers: a seed needs the `material` feature, a
    /// wallpaper the `wallpaper` one.
    SOURCES, SOURCE_KEYS: Source = [
        "styletto-source-preset" => Source::Preset,
        #[cfg(feature = "material")]
        "styletto-source-seed" => Source::Seed,
        #[cfg(feature = "wallpaper")]
        "styletto-source-wallpaper" => Source::Wallpaper,
        "styletto-source-custom" => Source::Custom,
    ]
);

impl Source {
    /// The source `config` takes its colours from: custom while any colour
    /// is set by hand (a seed or a wallpaper under them is kept, but no
    /// longer offered), then a wallpaper, a seed, and a preset theme, as
    /// `UiConfig` picks them. A seed or a wallpaper this build can't use is
    /// the preset it falls back to.
    pub fn of(config: &UiConfig) -> Self {
        if !config.dark.is_empty() || !config.light.is_empty() {
            Source::Custom
        } else if config.wallpaper.is_some() && cfg!(feature = "wallpaper") {
            Source::Wallpaper
        } else if config.seed.is_some() && cfg!(feature = "material") {
            Source::Seed
        } else {
            Source::Preset
        }
    }

    /// Makes `config` take its colours from here, leaving out what the
    /// source before had set; the theme chosen is kept, to go back to. A
    /// seed starts as the accent of the side the mode shows with no system
    /// preference (dark in `Auto`), a wallpaper as an empty path until one
    /// is chosen, and custom colours as every colour showing. A wallpaper is
    /// never read here: what shows of it is made of `wallpaper_colors`, those
    /// read from it, best first (`wallpaper::picture`), or of the theme
    /// without them.
    pub fn apply(self, config: &mut UiConfig, wallpaper_colors: &[Color]) {
        if Source::of(config) == self {
            return;
        }
        let showing = config.with_wallpaper_colors(wallpaper_colors).schemes();
        config.seed = None;
        config.wallpaper = None;
        config.wallpaper_color = 0;
        config.dark.clear();
        config.light.clear();
        match self {
            Source::Preset => {}
            Source::Seed => {
                let light = !config.mode.is_dark(None);
                config.seed = Some(to_hex(showing[light as usize].accent.color));
            }
            Source::Wallpaper => config.wallpaper = Some(String::new()),
            Source::Custom => {
                let [dark, light] = showing;
                config.dark = colors(&dark);
                config.light = colors(&light);
            }
        }
    }
}

/// Every colour `scheme` sets, by name; an on-colour or a hue's text it
/// leaves to be worked out is left out.
fn colors(scheme: &Scheme) -> Colors {
    scheme
        .named()
        .into_iter()
        .filter_map(|(name, color)| Some((name.to_string(), to_hex(color?))))
        .collect()
}

fn side(config: &UiConfig, light: bool) -> &Colors {
    if light { &config.light } else { &config.dark }
}

fn side_mut(config: &mut UiConfig, light: bool) -> &mut Colors {
    if light {
        &mut config.light
    } else {
        &mut config.dark
    }
}

/// A font family installed, for [`font_picker`] to offer: as the app found
/// it, or as [`installed_families`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Family {
    pub name: String,
    /// Every face of it is monospaced: offered for `mono` too.
    pub monospace: bool,
    /// It draws the semibold weight of the emphasised type (`.em()`); one
    /// that doesn't is drawn in its nearest.
    pub semibold: bool,
}

/// Where the colours come from, then what makes them: a preset's theme and,
/// on a card for each side, its flavour and accent; a seed or a wallpaper,
/// with how its colour spreads; or each side's colours by hand, in a
/// [`color_editor`]. The card of the side showing is ringed.
#[component]
pub fn theme_picker(
    #[prop(required)] config: RwSignal<UiConfig>,
    /// What the system prefers, dark or light, to tell the side showing
    /// in [`Mode::Auto`]; nothing said is dark.
    system_dark: Option<bool>,
    /// Shows a button beside the wallpaper's path. The app opens its file
    /// chooser and sets `wallpaper` to the path chosen.
    #[prop(callback)]
    on_pick_wallpaper: (),
    /// What the app's file chooser said, as that there is none: shown over
    /// the wallpaper while not empty, in place of what reading it came to.
    chooser_note: String,
) -> impl Widget {
    let config = config.get_untracked();
    let w = words();
    let source = create_memo(move || config.with(Source::of));
    let showing = move |light: bool| !config.with(|c| c.mode).is_dark(system_dark.get()) == light;
    #[cfg(not(feature = "wallpaper"))]
    let _ = (on_pick_wallpaper, chooser_note);
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S4.px()))
        .child(setting(w.of("styletto-setting-colours")).child(
            choices(labelled(w, SOURCE_KEYS), source).on_choose(
                // What shows of the wallpaper is what the next starts from.
                move |chosen: Source| {
                    let found = config.with_untracked(wallpaper_colors);
                    config.update(|c| chosen.apply(c, &found))
                },
            ),
        ))
        .child(move || match source.get() {
            Source::Preset => preset(config, showing),
            #[cfg(feature = "material")]
            Source::Seed => generated(config, seed_field(config)),
            #[cfg(feature = "wallpaper")]
            Source::Wallpaper => generated(
                config,
                wallpaper_field(config, on_pick_wallpaper, chooser_note),
            ),
            Source::Custom => {
                sides(move |light| side_card(light, showing, color_editor(config, light)))
            }
            #[allow(unreachable_patterns)]
            _ => container(),
        })
}

/// The mode, and what it shows: which side, of which theme.
#[component]
pub fn mode_picker(
    #[prop(required)] config: RwSignal<UiConfig>,
    /// What the system prefers, dark or light, for [`Mode::Auto`].
    system_dark: Option<bool>,
) -> impl Widget {
    let config = config.get_untracked();
    let w = words();
    let note = move || {
        let system = system_dark.get();
        config.with(|c| {
            let light = !c.mode.is_dark(system);
            // A whole phrase for each side, so a language that tells the
            // two apart otherwise can.
            let side = |dark, light_one| if light { light_one } else { dark };
            let shown = match Source::of(c) {
                Source::Custom => w.get(
                    side("styletto-shown-custom-dark", "styletto-shown-custom-light"),
                    &[],
                ),
                Source::Wallpaper if c.wallpaper_path().is_some() => w.get(
                    side(
                        "styletto-shown-wallpaper-dark",
                        "styletto-shown-wallpaper-light",
                    ),
                    &[],
                ),
                Source::Seed => w.get(
                    side("styletto-shown-seed-dark", "styletto-shown-seed-light"),
                    &[],
                ),
                Source::Preset | Source::Wallpaper => {
                    let choice = c.choice();
                    format!("{} {}", choice.theme.name, choice.flavor_of(light).name)
                }
            };
            let note = match (c.mode, system) {
                (Mode::Auto, Some(_)) => "styletto-note-system",
                (Mode::Auto, None) => "styletto-note-no-system",
                _ => "styletto-note-showing",
            };
            w.get(note, &[("shown", &shown)])
        })
    };
    choice(
        config,
        "styletto-setting-mode",
        MODE_KEYS,
        |c| c.mode,
        |c, mode| c.mode = mode,
    )
    .note(note)
}

/// The density, the shape, the material and the motion.
#[component]
pub fn layout_picker(#[prop(required)] config: RwSignal<UiConfig>) -> impl Widget {
    let config = config.get_untracked();
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S4.px()))
        .child(choice(
            config,
            "styletto-setting-density",
            DENSITY_KEYS,
            |c| c.density,
            |c, v| c.density = v,
        ))
        .child(choice(
            config,
            "styletto-setting-shape",
            SHAPE_KEYS,
            |c| c.shape,
            |c, v| c.shape = v,
        ))
        .child(choice(
            config,
            "styletto-setting-material",
            MATERIAL_KEYS,
            |c| c.material,
            |c, v| c.material = v,
        ))
        .child(choice(
            config,
            "styletto-setting-motion",
            MOTION_KEYS,
            |c| c.motion,
            |c, v| c.motion = v,
        ))
}

/// The sans and the mono family, each chosen among those installed or left
/// to the system's.
#[component]
pub fn font_picker(
    #[prop(required)] config: RwSignal<UiConfig>,
    /// The families installed, as the app finds them; monospaced ones are
    /// offered for mono too. Left out (`None` as the picker is made), they
    /// are [`installed_families`].
    families: Option<Vec<Family>>,
) -> impl Widget {
    let config = config.get_untracked();
    // Given or not is decided as the picker is made, so Guido lists its
    // fonts only when the app gives none.
    let families: Signal<Vec<Family>> = if families.with_untracked(Option::is_some) {
        create_memo(move || families.get().unwrap_or_default()).into_signal()
    } else {
        installed_families()
    };
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S4.px()))
        .child(font_field(config, false, families))
        .child(font_field(config, true, families))
}

/// Every colour of one side, by name, as a hex to edit, beside the colour
/// drawn; an on-colour or a hue's text left empty is worked out, and shows
/// what it is in its place. A way back to the preset's ends it.
///
/// Under a wallpaper, it shows the colours set over the preset's.
#[component]
pub fn color_editor(
    #[prop(required)] config: RwSignal<UiConfig>,
    /// The light side, or the dark one.
    #[prop(required)]
    light: bool,
) -> impl Widget {
    let config = config.get_untracked();
    let light = light.get_untracked();
    // Worked out once per change, for every dot of the side.
    let scheme = create_memo(move || config.with(|c| c.with_wallpaper_colors(&[]).scheme(light)));
    let palette = create_memo(move || scheme.get().palette());
    let field = move |name: &'static str| {
        let (hex, input) = hex_text(config, light, name);
        let drawn = move || palette.with(|p| drawn(p, name));
        container()
            .width(fill())
            .layout(centered_row(size::S2.px()))
            .child(dot(move || drawn().unwrap_or(Color::TRANSPARENT)))
            .child(
                container()
                    .width(size::S20.px())
                    .child(caption(name.replace('_', " "))),
            )
            .child(
                input_field()
                    .size(Size::Xs)
                    .refused(move || {
                        hex.with(|text| !text.trim().is_empty() && parse_hex(text).is_none())
                    })
                    .child(typing(
                        text_input(hex)
                            .widget_ref(input)
                            .typeset(MONO_SMALL)
                            .placeholder(move || drawn().map(to_hex).unwrap_or_default()),
                    )),
            )
    };
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S2.px()))
        .children(Scheme::default().named().map(|(name, _)| field(name)))
        .child(
            container().child(button().label(words().of("styletto-copy-preset")).on_click(
                move || {
                    config.update(|c| {
                        let preset = c.choice().scheme_of(light);
                        *side_mut(c, light) = colors(&preset);
                    })
                },
            )),
        )
}

/// Every control, in sections: the theme and its mode, the layout, the
/// fonts. It fills the width it is given and lets it wrap its options, as
/// in a quick-settings panel; an app scrolls it where it is taller.
#[component]
pub fn appearance(
    #[prop(required)] config: RwSignal<UiConfig>,
    /// What the system prefers, dark or light, for [`Mode::Auto`].
    system_dark: Option<bool>,
    /// The font families installed, as the app finds them; see
    /// [`font_picker`].
    families: Option<Vec<Family>>,
    /// Shows a button beside the wallpaper's path; see [`theme_picker`].
    #[prop(callback)]
    on_pick_wallpaper: (),
    /// What the app's file chooser said; see [`theme_picker`].
    chooser_note: String,
) -> impl Widget {
    let config = config.get_untracked();
    let w = words();
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S4.px()))
        .child(
            setting_group(w.of("styletto-group-theme"))
                .child(mode_picker(config).system_dark(system_dark))
                .child(
                    theme_picker(config)
                        .system_dark(system_dark)
                        .on_pick_wallpaper(on_pick_wallpaper)
                        .chooser_note(chooser_note),
                ),
        )
        .child(setting_group(w.of("styletto-group-layout")).child(layout_picker(config)))
        .child(
            setting_group(w.of("styletto-group-fonts"))
                .child(font_picker(config).families(families)),
        )
}

/// Each option of a list, by the words of its key, for `choices`: in a new
/// language, they change in place.
fn labelled<T: Copy + 'static>(
    w: Words,
    options: &'static [(&'static str, T)],
) -> impl Fn() -> Vec<(String, T)> + 'static {
    move || {
        options
            .iter()
            .map(|&(key, value)| (w.get(key, &[]), value))
            .collect()
    }
}

/// A setting, named by the key `name`, choosing one of `options`, by key,
/// read from the config with `get` and written to it with `set`.
fn choice<T: Copy + PartialEq + 'static>(
    config: RwSignal<UiConfig>,
    name: &'static str,
    options: &'static [(&'static str, T)],
    get: fn(&UiConfig) -> T,
    set: fn(&mut UiConfig, T),
) -> Setting {
    let w = words();
    setting(w.of(name)).child(
        choices(labelled(w, options), move || config.with(get))
            .on_choose(move |value| config.update(|c| set(c, value))),
    )
}

/// The dark side's card, then the light one's.
fn sides(card: impl Fn(bool) -> Container) -> Container {
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S2.px()))
        .child(card(false))
        .child(card(true))
}

/// One side on a card, ringed while it is the one showing.
fn side_card(
    light: bool,
    showing: impl Fn(bool) -> bool + Copy + 'static,
    body: impl Widget + 'static,
) -> Container {
    let t = look();
    let w = words();
    container()
        .width(fill())
        .padding(size::S3.px())
        .corners(radius::MEDIUM.corners())
        .background(t.eased(|c| c.surface))
        .border(
            stroke::RING,
            t.eased(move |c| {
                if showing(light) {
                    c.accent.fill
                } else {
                    Color::TRANSPARENT
                }
            }),
        )
        .layout(Flex::column().spacing(size::S4.px()))
        .child(
            container()
                .width(fill())
                .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
                .child(
                    container()
                        .layout(centered_row(size::S2.px()))
                        .child(icon(
                            if light { Icon::Sun } else { Icon::Moon },
                            icon_box::SMALL.px(),
                            t.color(|c| c.sub),
                        ))
                        .child(
                            text(w.of(if light {
                                "styletto-side-light"
                            } else {
                                "styletto-side-dark"
                            }))
                            .typeset(TITLE_XSMALL.em())
                            .color(t.eased(|c| c.text)),
                        ),
                )
                .child(caption(move || {
                    if showing(light) {
                        w.get("styletto-side-showing", &[])
                    } else {
                        String::new()
                    }
                })),
        )
        .child(body)
}

/// The preset themes, then each side's flavour and accent.
fn preset(
    config: RwSignal<UiConfig>,
    showing: impl Fn(bool) -> bool + Copy + 'static,
) -> Container {
    // Worked out once per change, for every button and swatch.
    let choice = create_memo(move || config.with(UiConfig::choice));
    let theme = create_memo(move || choice.get().theme.name);
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S4.px()))
        .child(
            setting(words().of("styletto-setting-preset")).child(
                choices(
                    THEMES
                        .iter()
                        .map(|theme| (theme.name.to_string(), theme.name))
                        .collect::<Vec<_>>(),
                    theme,
                )
                .on_choose(move |name: &'static str| {
                    config.update(|c| c.set_theme(Some(name.to_string())))
                }),
            ),
        )
        .child(move || {
            // Built again with the theme, whose flavours these are.
            theme.get();
            sides(move |light| side_card(light, showing, flavor_and_accent(config, choice, light)))
        })
}

/// A side's flavours, as buttons, or why there are none; then its accents.
fn flavor_and_accent(
    config: RwSignal<UiConfig>,
    choice: Memo<ThemeChoice>,
    light: bool,
) -> Container {
    let t = look();
    let w = words();
    let theme = config.with_untracked(|c| c.choice().theme);
    let flavors = theme.flavors(light);
    let flavor = create_memo(move || choice.get().flavor_of(light));
    // Running text when there is none: it wraps inside the card rather than
    // a caption's one line.
    let none = flavors.is_empty().then(|| {
        container().width(fill()).child(
            text(move || {
                let none = if light {
                    "styletto-note-no-light-flavour"
                } else {
                    "styletto-note-no-dark-flavour"
                };
                w.get(none, &[("theme", theme.name)])
            })
            .typeset(BODY_SMALL)
            .color(t.eased(|c| c.sub)),
        )
    });
    let flavors = (!flavors.is_empty()).then(|| {
        setting(w.of("styletto-setting-flavour")).child(
            choices(
                flavors
                    .iter()
                    .map(|flavor| (flavor.name.to_string(), flavor.name))
                    .collect::<Vec<_>>(),
                move || flavor.get().name,
            )
            .on_choose(move |name: &'static str| {
                config.update(|c| {
                    let name = Some(name.to_string());
                    if light {
                        c.light_flavor = name;
                    } else {
                        c.dark_flavor = name;
                    }
                })
            }),
        )
    });
    let accent = move || choice.get().side(light).accent;
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S4.px()))
        .child(none)
        .child(flavors)
        .child(move || {
            let flavor = flavor.get();
            setting(w.of("styletto-setting-accent"))
                .note(move || accent().to_string())
                .child(container().layout(centered_row(size::S1.px())).children(
                    flavor.accents.iter().map(move |a| {
                        let name = a.name;
                        swatch(a.color, move || accent() == name)
                            .on_click(move || config.update(|c| c.set_accent(light, name)))
                    }),
                ))
        })
}

/// Where a seed or a wallpaper's colour comes from, then how the theme is
/// made of it.
#[cfg(feature = "material")]
fn generated(config: RwSignal<UiConfig>, from: impl Widget + 'static) -> Container {
    container()
        .width(fill())
        .layout(Flex::column().spacing(size::S4.px()))
        .child(from)
        .child(choice(
            config,
            "styletto-setting-variant",
            VARIANT_KEYS,
            |c| c.variant,
            |c, v| c.variant = v,
        ))
        .child(choice(
            config,
            "styletto-setting-contrast",
            CONTRAST_KEYS,
            |c| c.contrast,
            |c, v| c.contrast = v,
        ))
}

/// A seed typed as a hex, beside the dot it makes; it is set once it reads
/// as a colour.
#[cfg(feature = "material")]
fn seed_field(config: RwSignal<UiConfig>) -> Setting {
    let (typed, input) = bound_text(
        config,
        |c| c.seed.clone().unwrap_or_default(),
        |text| parse_hex(text).is_some(),
        |c, text| c.seed = Some(text.to_string()),
    );
    setting(words().of("styletto-setting-seed")).child(
        container()
            .width(fill())
            .layout(centered_row(size::S2.px()))
            .child(dot(move || {
                config
                    .with(|c| c.seed.as_deref().and_then(parse_hex))
                    .unwrap_or(Color::TRANSPARENT)
            }))
            .child(
                input_field()
                    .size(Size::Sm)
                    .refused(move || typed.with(|text| parse_hex(text).is_none()))
                    .child(typing(
                        text_input(typed).widget_ref(input).typeset(MONO_SMALL),
                    )),
            ),
    )
}

/// The colours the config's wallpaper yields, best first, as far as it is
/// read; none without one.
fn wallpaper_colors(config: &UiConfig) -> Vec<Color> {
    #[cfg(feature = "wallpaper")]
    return config.wallpaper_colors();
    #[cfg(not(feature = "wallpaper"))]
    {
        let _ = config;
        Vec::new()
    }
}

/// The wallpaper's path, set on Enter or by the app's file chooser, and how
/// to set one or what reading it came to.
#[cfg(feature = "wallpaper")]
fn wallpaper_field(
    config: RwSignal<UiConfig>,
    pick: Option<Callback>,
    chooser_note: Signal<String>,
) -> Setting {
    let w = words();
    let set = create_memo(move || config.with(|c| c.wallpaper.clone().unwrap_or_default()));
    let picture = create_memo(move || {
        config
            .with(UiConfig::wallpaper_path)
            .map(|path| wallpaper::picture(&path))
    });
    let typed = create_signal(config.with_untracked(|c| c.wallpaper.clone().unwrap_or_default()));
    // A new path, as the app's chooser sets it, shows in the field; a path
    // typed and not sent stays until then.
    create_effect(move || {
        let path = set.get();
        if typed.with_untracked(|typed| *typed != path) {
            typed.set(path);
        }
    });
    let input = text_input(typed)
        .typeset(MONO_SMALL)
        .placeholder(w.of("styletto-wallpaper-placeholder"))
        .on_submit(move |path: &str| {
            let path = path.trim().to_string();
            let typed = UiConfig {
                wallpaper: Some(path.clone()),
                ..UiConfig::default()
            }
            .wallpaper_path();
            // The same file again, however it is written, is read again:
            // it may have changed, or come to be since it failed.
            let same = typed.is_some() && config.with_untracked(|c| c.wallpaper_path() == typed);
            if same {
                config.update(UiConfig::reread_wallpaper);
            } else {
                config.update(|c| c.set_wallpaper(path));
            }
        });
    let field = input_field().size(Size::Sm).child(typing(input));
    let field = match pick {
        Some(pick) => field.trailing(
            button()
                .icon(Icon::Folder)
                .size(Size::Xs)
                .fill(crate::components::button::Fill::Ghost)
                .on_click(pick),
        ),
        None => field,
    };
    // What the chooser said, or what reading the picture came to, or until
    // a path is set, how to set one.
    let note = move || {
        let said = chooser_note.get();
        if !said.is_empty() {
            return said;
        }
        match picture.get() {
            None => w.get("styletto-wallpaper-none", &[]),
            Some(Picture::Reading) => w.get("styletto-wallpaper-reading", &[]),
            Some(Picture::Failed(why)) => failed(w, why),
            Some(Picture::Read(_)) => String::new(),
        }
    };
    let candidates = create_memo(move || {
        picture.with(|p| p.as_ref().map_or(Vec::new(), |p| p.colors().to_vec()))
    });
    // The colours found, to choose among: the best first.
    let chosen = move || {
        let count = candidates.with(Vec::len);
        config.with(|c| c.wallpaper_index(count))
    };
    let found = move || {
        let found = candidates.get();
        (!found.is_empty()).then(|| {
            container().layout(centered_row(size::S1.px())).children(
                found.into_iter().enumerate().map(move |(index, color)| {
                    swatch(color, move || chosen() == index)
                        .on_click(move || config.update(|c| c.wallpaper_color = index))
                }),
            )
        })
    };
    setting(w.of("styletto-setting-wallpaper"))
        .note(note)
        .child(
            container()
                .width(fill())
                .layout(Flex::column().spacing(size::S2.px()))
                .child(field)
                .child(found),
        )
}

/// Why a wallpaper can't be read: styletto's own reasons in the app's
/// language, what the system said as it said it.
#[cfg(feature = "wallpaper")]
fn failed(w: Words, why: String) -> String {
    [
        "styletto-wallpaper-no-colour",
        "styletto-wallpaper-unreadable",
    ]
    .into_iter()
    .find(|key| english(key) == why)
    .map_or(why, |key| w.get(key, &[]))
}

/// A family to choose among those installed: the one chosen, written in
/// itself, and a list to search when it is opened. Monospaced families
/// only for `mono`; the system's first.
fn font_field(config: RwSignal<UiConfig>, mono: bool, families: Signal<Vec<Family>>) -> Setting {
    let t = look();
    let w = words();
    let chosen = create_memo(move || {
        config.with(|c| if mono { &c.fonts.mono } else { &c.fonts.sans }.clone())
    });
    let choose = move |family: Option<String>| {
        config.update(|c| {
            *if mono {
                &mut c.fonts.mono
            } else {
                &mut c.fonts.sans
            } = family
        })
    };
    let open = create_signal(false);
    let query = create_signal(String::new());
    let needle = create_memo(move || query.get().to_lowercase());
    let system = if mono {
        FontFamily::Monospace
    } else {
        FontFamily::SansSerif
    };
    let drawn_in =
        move |family: &Option<String>| family.as_deref().map_or(system, FontFamily::name);
    let note = move || {
        // Nothing to say until the app has found the families.
        let Some(name) = chosen.get().filter(|_| families.with(|f| !f.is_empty())) else {
            return String::new();
        };
        families.with(|families| {
            match families.iter().find(|f| f.name.eq_ignore_ascii_case(&name)) {
                None => w.get("styletto-font-not-installed", &[]),
                Some(family) if !family.semibold => w.get("styletto-font-no-semibold", &[]),
                Some(_) => String::new(),
            }
        })
    };
    let current = container()
        .width(fill())
        .padding([size::S2.px(), size::S3.px()])
        .corners(radius::SMALL.corners())
        .background(t.color(|c| c.surface).transition(motion::hover()))
        .when_hovered(move |s| s.background(t.color(|c| c.surface2)))
        .on_click(move || open.set(!open.get_untracked()))
        .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
        .child(
            text(move || {
                chosen
                    .get()
                    .unwrap_or_else(|| w.get("styletto-font-system", &[]))
            })
            .typeset(LABEL_MEDIUM)
            .font_family(move || chosen.with(drawn_in))
            .color(t.eased(|c| c.text)),
        )
        .child(icon(
            move || {
                if open.get() {
                    Icon::ChevronDown
                } else {
                    Icon::ChevronRight
                }
            },
            icon_box::SMALL.px(),
            t.color(|c| c.sub),
        ));
    let list = move || {
        open.get().then(|| {
            let option = move |label: Signal<String>, family: Option<String>| {
                let picked = family.clone();
                let face = drawn_in(&family);
                container()
                    .width(fill())
                    .padding([size::S1.px(), size::S2.px()])
                    .corners(radius::SMALL.corners())
                    .background(
                        (move || {
                            if chosen.with(|c| *c == family) {
                                t.colors().surface2
                            } else {
                                Color::TRANSPARENT
                            }
                        })
                        .transition(motion::hover()),
                    )
                    .when_hovered(move |s| s.background(t.color(|c| c.surface)))
                    .on_click(move || {
                        choose(picked.clone());
                        open.set(false);
                    })
                    .child(
                        text(label)
                            .typeset(LABEL_MEDIUM)
                            .font_family(face)
                            .color(t.eased(|c| c.text))
                            .nowrap(),
                    )
            };
            // Built once and hidden as the query leaves them out: a
            // keystroke shapes no text.
            let options: Vec<Container> = families.with(|families| {
                families
                    .iter()
                    .filter(|f| !mono || f.monospace)
                    .map(|f| {
                        let lower = f.name.to_lowercase();
                        option(f.name.clone().into_signal(), Some(f.name.clone()))
                            .visible(move || needle.with(|n| lower.contains(n.as_str())))
                    })
                    .collect()
            });
            container()
                .width(fill())
                .layout(Flex::column().spacing(size::S1.px()))
                .child(
                    input_field().size(Size::Sm).child(typing(
                        text_input(query)
                            .typeset(LABEL_MEDIUM)
                            .placeholder(w.of("styletto-font-search"))
                            .autofocus(),
                    )),
                )
                .child(
                    scroll_list(size::scaled(size::S60.px()))
                        .child(option(w.of("styletto-font-system").into_signal(), None))
                        .children(options),
                )
        })
    };
    setting(w.of(if mono {
        "styletto-setting-mono"
    } else {
        "styletto-setting-sans"
    }))
    .note(note)
    .child(
        container()
            .width(fill())
            .layout(Flex::column().spacing(size::S1.px()))
            .child(current)
            .child(list),
    )
}

/// What is typed in a field, in the text's colours, filling the field up
/// to its trailing button.
fn typing(input: TextInput) -> Container {
    container().width(fill()).child(input.palette_colors())
}

/// A dot of a colour, as a seed or a colour set by hand.
fn dot(color: impl Fn() -> Color + 'static) -> Container {
    container()
        .width(size::S4.px())
        .height(size::S4.px())
        .corners(radius::pill(size::S4.px()))
        .border(stroke::HAIRLINE, look().eased(|c| c.line))
        .background(motion::eased_color(color))
}

/// A dot of `color` to press, ringed in the text's colour while `on`.
fn swatch(color: Color, on: impl Fn() -> bool + 'static) -> Container {
    let t = look();
    pressable(
        container()
            .width(size::S4.px())
            .height(size::S4.px())
            .corners(radius::pill(size::S4.px()))
            .background(color)
            .border(
                stroke::BOLD,
                (move || {
                    if on() {
                        t.colors().text
                    } else {
                        Color::TRANSPARENT
                    }
                })
                .transition(motion::hover()),
            ),
        // A small target: it presses deeper than a button.
        motion::PRESS_SCALE_INNER,
    )
}

/// The field for colour `name` of one side, bound to the config: what
/// parses as a colour is set on that side, nothing at all unsets it, and
/// anything else waits to be finished.
fn hex_text(
    config: RwSignal<UiConfig>,
    light: bool,
    name: &'static str,
) -> (RwSignal<String>, WidgetRef) {
    bound_text(
        config,
        move |c| side(c, light).get(name).cloned().unwrap_or_default(),
        |text| text.is_empty() || parse_hex(text).is_some(),
        move |c, text| {
            let colors = side_mut(c, light);
            if text.is_empty() {
                colors.remove(name);
            } else {
                colors.insert(name.to_string(), text.to_string());
            }
        },
    )
}

/// A field's text bound to one value of the config, both ways, and the ref
/// its input takes. What is typed is written, trimmed, once `takes` it. A
/// change made elsewhere shows in the field, over an edit not finished,
/// but not while the field has the focus, so it never moves the text
/// under the cursor: the field catches up once it is left.
fn bound_text(
    config: RwSignal<UiConfig>,
    read: impl Fn(&UiConfig) -> String + Copy + 'static,
    takes: fn(&str) -> bool,
    write: impl Fn(&mut UiConfig, &str) + Copy + 'static,
) -> (RwSignal<String>, WidgetRef) {
    let start = config.with_untracked(read);
    let text = create_signal(start.clone());
    // The config's value the field last followed.
    let seen = create_signal(start);
    let input = create_widget_ref();
    create_effect(move || {
        let typed = text.get();
        let typed = typed.trim();
        if takes(typed) && config.with_untracked(read) != typed {
            config.update(|c| write(c, typed));
        }
    });
    let outside = create_memo(move || config.with(read));
    create_effect(move || {
        let value = outside.get();
        // The focus is read only for a change: a field in step doesn't
        // follow it.
        if seen.with_untracked(|seen| *seen == value) || input.is_focused() {
            return;
        }
        seen.set(value.clone());
        if text.with_untracked(|text| text.trim() != value) {
            text.set(value);
        }
    });
    (text, input)
}

/// The colour a scheme's `name` is drawn in, from its palette: an
/// on-colour or a hue's text left unsaid is the one worked out for it. The
/// palette names every colour as the scheme does but `background`, which it
/// draws as `island`. `None` for a name the palette doesn't know.
fn drawn(palette: &Palette, name: &str) -> Option<Color> {
    let name = if name == "background" { "island" } else { name };
    palette
        .named()
        .into_iter()
        .find_map(|(named, color)| (named == name).then_some(color))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Never run, only compiled: the families optional, an app still hands
    /// them in in every form it did when they were a list.
    #[allow(dead_code)]
    fn families_are_handed_in_as_before(
        config: RwSignal<UiConfig>,
        read: Signal<Vec<Family>>,
        held: RwSignal<Vec<Family>>,
        worked_out: Memo<Vec<Family>>,
    ) {
        font_picker(config).families(read);
        font_picker(config).families(held);
        font_picker(config).families(worked_out);
        font_picker(config).families(move || read.get());
        appearance(config).families(Vec::new());
    }
    use crate::theme::presets::ThemeChoice;

    #[test]
    fn every_colour_listed_shows_its_own() {
        // A name the palette doesn't know would show as nothing.
        for theme in THEMES {
            for light in [false, true] {
                let scheme = ThemeChoice::of(theme).scheme_of(light);
                let palette = scheme.palette();
                for (name, set) in scheme.named() {
                    let shown = drawn(&palette, name);
                    assert!(shown.is_some(), "{name} shows nothing");
                    if set.is_some() {
                        assert_eq!(shown, set, "{name} shows another colour");
                    }
                }
            }
        }
    }

    /// A config whose colours are all set by hand, from the second theme,
    /// showing `mode`.
    fn custom(mode: Mode) -> UiConfig {
        let mut config = UiConfig {
            theme: Some(THEMES[1].name.to_string()),
            mode,
            ..UiConfig::default()
        };
        Source::Custom.apply(&mut config, &[]);
        config
    }

    #[test]
    fn a_light_colour_leaves_the_dark_one() {
        let config = create_signal(custom(Mode::Light));
        let before = config.get_untracked();
        let (dark, _) = hex_text(config, false, "accent");
        let (light, _) = hex_text(config, true, "accent");
        assert_eq!(
            config.get_untracked(),
            before,
            "binding a field set a colour"
        );
        light.set("#ff0000".to_string());
        let after = config.get_untracked();
        assert_eq!(after.light["accent"], "#ff0000");
        assert_eq!(after.dark, before.dark, "the dark side moved");
        assert_eq!(dark.get_untracked(), before.dark["accent"]);
    }

    #[test]
    fn the_light_side_edits_the_light_palette_only() {
        // Every field of both sides, as the two editors bind them.
        let config = create_signal(custom(Mode::Light));
        let before = config.get_untracked().schemes();
        let fields: Vec<_> = [false, true]
            .into_iter()
            .flat_map(|light| {
                Scheme::default()
                    .named()
                    .map(|(name, _)| ((light, name), hex_text(config, light, name).0))
            })
            .collect();
        let red = parse_hex("#ff0000").unwrap();
        let field = |light: bool, name: &str| {
            fields
                .iter()
                .find(|(key, _)| *key == (light, name))
                .map(|(_, hex)| *hex)
                .unwrap()
        };
        field(true, "accent").set("#ff0000".to_string());
        let after = config.get_untracked().schemes();
        assert_eq!(after[1].accent.color, red);
        assert_eq!(after[0], before[0], "the dark palette moved");
        assert_eq!(config.get_untracked().ui(None).colors.accent.fill, red);
        config.update(|c| c.mode = Mode::Dark);
        assert_eq!(
            config.get_untracked().ui(None).colors.accent.fill,
            before[0].palette().accent.fill,
            "dark mode shows the edit"
        );
        // Emptied, an on-colour is worked out again; a hue can't be.
        field(true, "on_accent").set(String::new());
        assert!(!config.get_untracked().light.contains_key("on_accent"));
        field(true, "accent").set("#ff00".to_string());
        assert_eq!(config.get_untracked().light["accent"], "#ff0000");
    }

    #[test]
    fn a_source_leaves_out_what_the_one_before_set() {
        let mut config = UiConfig {
            theme: Some("nord".to_string()),
            accent: Some("frost".to_string()),
            ..UiConfig::default()
        };
        let showing = config.schemes();
        Source::Custom.apply(&mut config, &[]);
        assert_eq!(Source::of(&config), Source::Custom);
        // As it showed, to the byte a hex keeps.
        let hexes = |schemes: [Scheme; 2]| schemes.map(|s| s.named().map(|(_, c)| c.map(to_hex)));
        assert_eq!(
            hexes(config.schemes()),
            hexes(showing),
            "custom starts as what showed"
        );
        assert_eq!(config.theme.as_deref(), Some("nord"), "the preset is kept");
        // A seed and a wallpaper are offered with their features.
        if !cfg!(feature = "wallpaper") {
            return;
        }
        Source::Seed.apply(&mut config, &[]);
        assert_eq!(Source::of(&config), Source::Seed);
        assert!(config.dark.is_empty() && config.light.is_empty());
        assert_eq!(config.theme.as_deref(), Some("nord"), "the theme is kept");
        assert_eq!(
            config.seed.as_deref().and_then(parse_hex),
            Some(showing[0].accent.color),
            "the seed starts as the accent showing"
        );
        Source::Wallpaper.apply(&mut config, &[]);
        assert_eq!(
            (config.seed.as_ref(), config.wallpaper.as_deref()),
            (None, Some(""))
        );
        // Leaving a wallpaper, what showed of it is what the next starts
        // from: the colour chosen among those read from it.
        config.set_wallpaper("~/w.jpg".to_string());
        let found = [Color::from_hex(0x2e7d32), Color::from_hex(0xc62828)];
        config.wallpaper_color = 1;
        let shown = config.with_wallpaper_colors(&found).schemes();
        let mut custom = config.clone();
        Source::Custom.apply(&mut custom, &found);
        assert_eq!(hexes(custom.schemes()), hexes(shown));
        Source::Preset.apply(&mut config, &[]);
        assert_eq!(
            config,
            UiConfig {
                theme: Some("nord".to_string()),
                accent: Some("frost".to_string()),
                ..UiConfig::default()
            },
            "back to the preset chosen"
        );
    }

    #[test]
    fn a_field_follows_a_change_made_elsewhere_but_not_over_an_edit() {
        let config = create_signal(custom(Mode::Dark));
        let (hex, _) = hex_text(config, false, "accent");
        config.update(|c| {
            c.dark.insert("accent".to_string(), "#00ff00".to_string());
        });
        assert_eq!(hex.get_untracked(), "#00ff00", "the field stayed behind");
        // Typed, then a change elsewhere: the field shows what it wrote.
        hex.set("#0000ff".to_string());
        config.update(|c| c.mode = Mode::Light);
        assert_eq!(hex.get_untracked(), "#0000ff");
        // Half typed and left, it takes a change made elsewhere.
        hex.set("#ff".to_string());
        assert_eq!(config.get_untracked().dark["accent"], "#0000ff");
        config.update(|c| {
            c.dark.insert("accent".to_string(), "#123456".to_string());
        });
        assert_eq!(hex.get_untracked(), "#123456", "the field stayed behind");
        // A change that leaves the colour as it is moves nothing.
        hex.set("#ff".to_string());
        config.update(|c| c.mode = Mode::Light);
        assert_eq!(hex.get_untracked(), "#ff", "the edit was lost");
    }
}
