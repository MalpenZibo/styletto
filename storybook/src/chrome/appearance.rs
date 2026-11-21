//! What the stories are drawn with, and the page that chooses it: a theme
//! with a dark and a light side, each a flavour and an accent, the mode that
//! picks between them, colours set one by one, or both sides worked out from
//! a wallpaper or a seed colour; then the density, the shape, the material,
//! the motion and the fonts. The storybook's own chrome keeps the default.

use std::path::PathBuf;

use crate::system::color_scheme::{self, ColorScheme};
use futures::StreamExt;
use guido::prelude::*;
use styletto::components::button::{Variant, button};
use styletto::components::card::level_figure;
use styletto::components::level::Level;
use styletto::components::list::row;
use styletto::components::progress::ring;
use styletto::components::slider::slider_row;
use styletto::components::tile::small_tile;
use styletto::components::toggle::toggle;
use styletto::foundations::control::Size;
use styletto::foundations::tokens::{icon as icon_box, radius, size as space, stroke};
use styletto::foundations::typography as ty;
use styletto::foundations::typography::Typeset;
use styletto::primitives::island as look;
use styletto::primitives::text::{caption, typed};
use styletto::theme::material::{self, Contrast};
use styletto::theme::palette::{Hue, Palette, Scheme, parse_hex, to_hex};
use styletto::theme::presets::{Mode, THEMES, ThemeChoice};
use styletto::ui::{Density, Fonts, Material, Motion, Shape};
use styletto::{Icon, Ui, centered_row, icon, provide_ui, ui};

use crate::chrome::widgets::{choice, choices, field, island, section, swatch, wallpaper};
use crate::system::fonts::{self, Family};
use crate::system::picker;

pub const DENSITIES: &[(&str, Density)] = &[
    ("Compact", Density::Compact),
    ("Default", Density::Default),
    ("Comfortable", Density::Comfortable),
];
pub const SHAPES: &[(&str, Shape)] = &[
    ("Square", Shape::Square),
    ("Soft", Shape::Soft),
    ("Round", Shape::Round),
];
const MATERIALS: &[(&str, Material)] = &[
    ("Solid", Material::Solid),
    ("Translucent", Material::Translucent),
    ("Glass", Material::Glass),
];
const MOTIONS: &[(&str, Motion)] = &[
    ("Full", Motion::Full),
    ("Reduced", Motion::Reduced),
    ("None", Motion::None),
];
const VARIANTS: &[(&str, material::Variant)] = &[
    ("Tonal spot", material::Variant::TonalSpot),
    ("Vibrant", material::Variant::Vibrant),
    ("Expressive", material::Variant::Expressive),
    ("Content", material::Variant::Content),
    ("Fidelity", material::Variant::Fidelity),
    ("Neutral", material::Variant::Neutral),
    ("Monochrome", material::Variant::Monochrome),
    ("Rainbow", material::Variant::Rainbow),
    ("Fruit salad", material::Variant::FruitSalad),
];
const CONTRASTS: &[(&str, Contrast)] = &[
    ("Standard", Contrast::Standard),
    ("Medium", Contrast::Medium),
    ("High", Contrast::High),
];
const MODES: &[(&str, Mode)] = &[
    ("Light", Mode::Light),
    ("Dark", Mode::Dark),
    ("Auto", Mode::Auto),
];

/// Where the colours come from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Source {
    #[default]
    Preset,
    /// Set one by one, in `colors`.
    Custom,
    /// Worked out from the colour of the wallpaper picked.
    Wallpaper,
    /// Worked out from the colour in `seed`.
    Seed,
}

const SOURCES: &[(&str, Source)] = &[
    ("Preset", Source::Preset),
    ("Seed", Source::Seed),
    ("Wallpaper", Source::Wallpaper),
    ("Custom", Source::Custom),
];

/// Reads an image for its colours on a thread of its own; from the chooser's
/// task too, so it holds only writers.
#[derive(Clone, Copy)]
struct Reader {
    picture: WriteSignal<Picture>,
    picked: WriteSignal<usize>,
    path: WriteSignal<String>,
}

impl Reader {
    fn read(self, path: PathBuf) {
        self.picture.set(Picture::Reading);
        self.picked.set(0);
        self.path.set(path.display().to_string());
        std::thread::spawn(move || {
            self.picture.set(match material::seeds(&path) {
                Ok(seeds) if seeds.is_empty() => Picture::Failed("No colour found".to_string()),
                Ok(seeds) => Picture::Read { path, seeds },
                Err(err) => Picture::Failed(err),
            })
        });
    }
}

/// Asks the system's file chooser for an image, then reads it. The chooser
/// is a window, so the storybook steps below windows while it is open.
fn chooser(reader: Reader) -> Service<()> {
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
                Ok(Some(path)) => reader.read(path),
                Ok(None) => {}
                Err(err) => reader
                    .picture
                    .set(Picture::Failed(format!("No file chooser: {err}"))),
            }
        }
    })
}

/// An image read for the colours a theme could be made of.
#[derive(Clone, Debug, Default, PartialEq)]
enum Picture {
    #[default]
    None,
    Reading,
    Read {
        path: PathBuf,
        /// Best first.
        seeds: Vec<Color>,
    },
    Failed(String),
}

#[derive(Clone, Copy)]
pub struct Appearance {
    choice: RwSignal<ThemeChoice>,
    /// What the system prefers: dark, light, or nothing said.
    system: Signal<Option<bool>>,
    source: RwSignal<Source>,
    /// The colours set one by one: the dark palette, then the light one.
    colors: RwSignal<[Scheme; 2]>,
    picture: RwSignal<Picture>,
    /// The image's path as typed, or as the chooser gave it.
    path: RwSignal<String>,
    reader: Reader,
    chooser: Service<()>,
    /// Which of the picture's seeds the theme is made of.
    picked: RwSignal<usize>,
    /// A seed colour as typed.
    seed: RwSignal<String>,
    variant: RwSignal<material::Variant>,
    contrast: RwSignal<Contrast>,
    /// Both sides worked out from the wallpaper or the seed, while one of
    /// them is the source and has a colour to give.
    generated: Memo<Option<[Scheme; 2]>>,
    /// The picture read, shown behind the stories in place of the shapes.
    backdrop: Memo<Option<PathBuf>>,
    density: RwSignal<Density>,
    shape: RwSignal<Shape>,
    material: RwSignal<Material>,
    motion: RwSignal<Motion>,
    /// A family by name, or the system's.
    sans: RwSignal<Option<String>>,
    mono: RwSignal<Option<String>>,
    /// The families installed, as the font database sees them.
    families: Signal<Vec<Family>>,
}

impl Appearance {
    pub fn new() -> Self {
        let source = create_signal(Source::default());
        let picture = create_signal(Picture::default());
        let picked = create_signal(0);
        let path = create_signal(String::new());
        let reader = Reader {
            picture: picture.writer(),
            picked: picked.writer(),
            path: path.writer(),
        };
        let seed = create_signal("#6750a4".to_string());
        let variant = create_signal(material::Variant::default());
        let contrast = create_signal(Contrast::default());
        let generated = create_memo(move || {
            let color = match source.get() {
                Source::Wallpaper => picture.with(|p| match p {
                    Picture::Read { seeds, .. } => seeds.get(picked.get()).copied(),
                    _ => None,
                }),
                Source::Seed => seed.with(|s| parse_hex(s)),
                Source::Preset | Source::Custom => None,
            };
            color.map(|color| material::schemes(color, variant.get(), contrast.get()))
        });
        let backdrop = create_memo(move || {
            picture.with(|p| match p {
                Picture::Read { path, .. } => Some(path.clone()),
                _ => None,
            })
        });
        Self {
            choice: create_signal(ThemeChoice::default()),
            system: system_dark(),
            source,
            colors: create_signal([Scheme::default(); 2]),
            picture,
            path,
            reader,
            chooser: chooser(reader),
            picked,
            seed,
            variant,
            contrast,
            generated,
            backdrop,
            density: create_signal(Density::default()),
            shape: create_signal(Shape::default()),
            material: create_signal(Material::default()),
            motion: create_signal(Motion::default()),
            sans: create_signal(None),
            mono: create_signal(None),
            families: installed_families(),
        }
    }

    /// Whether the light side shows.
    fn light(self) -> bool {
        self.choice.get().shows_light(self.system.get())
    }

    fn palette(self) -> Palette {
        self.schemes()[self.light() as usize].palette()
    }

    /// Both sides' colours as the source has them; the preset's where the
    /// wallpaper or the seed has none to give yet.
    fn schemes(self) -> [Scheme; 2] {
        let preset = || {
            let choice = self.choice.get();
            [choice.scheme_of(false), choice.scheme_of(true)]
        };
        match self.source.get() {
            Source::Custom => self.colors.get(),
            Source::Wallpaper | Source::Seed => self.generated.get().unwrap_or_else(preset),
            Source::Preset => preset(),
        }
    }

    pub fn backdrop(self) -> Memo<Option<PathBuf>> {
        self.backdrop
    }

    /// Reads the image at `path` for its colours, on a thread of its own.
    fn read_picture(self, path: &str) {
        let path = match path.trim().strip_prefix("~/") {
            Some(rest) => std::env::home_dir().unwrap_or_default().join(rest),
            None => PathBuf::from(path.trim()),
        };
        self.reader.read(path);
    }

    pub fn ui(self) -> Ui {
        let system = Fonts::default();
        let family = |chosen: Option<String>, system: FontFamily| {
            chosen.map_or(system, |name| FontFamily::name(&name))
        };
        Ui::new(self.palette())
            .with_density(self.density.get())
            .with_shape(self.shape.get())
            .with_material(self.material.get())
            .with_motion(self.motion.get())
            .with_fonts(Fonts {
                sans: family(self.sans.get(), system.sans),
                mono: family(self.mono.get(), system.mono),
            })
    }

    fn choose(self, change: impl FnOnce(&mut ThemeChoice)) {
        self.choice.update(change);
    }
}

/// The families installed, read once on a thread of their own: reading
/// every font file takes a moment.
fn installed_families() -> Signal<Vec<Family>> {
    let families = create_signal(Vec::new());
    let writer = families.writer();
    std::thread::spawn(move || writer.set(fonts::families()));
    families.read_only()
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

/// The settings on the left, what they make on the right.
pub fn page(appearance: Appearance) -> Container {
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
                .layout(Flex::column().spacing(space::S6.px()))
                .child(themes(appearance))
                .child(group(
                    "Layout",
                    vec![
                        choice("Density", DENSITIES, appearance.density),
                        choice("Shape", SHAPES, appearance.shape),
                    ],
                ))
                .child(group(
                    "Surface",
                    vec![
                        choice("Material", MATERIALS, appearance.material),
                        choice("Motion", MOTIONS, appearance.motion),
                    ],
                ))
                .child(group(
                    "Fonts",
                    vec![
                        font_field("Sans", appearance.sans, appearance.families, false),
                        font_field("Mono", appearance.mono, appearance.families, true),
                    ],
                )),
        )
        .child(preview(appearance))
}

/// A family to choose among those installed: the one chosen, written in
/// itself, and a list to search when it is opened. Monospaced families
/// only for `mono`; the system's first.
fn font_field(
    name: &'static str,
    chosen: RwSignal<Option<String>>,
    families: Signal<Vec<Family>>,
    mono: bool,
) -> Container {
    let t = ui();
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
    let note = caption(move || {
        let Some(name) = chosen.get() else {
            return String::new();
        };
        families.with(|families| match families.iter().find(|f| f.name == name) {
            None => "Not installed".to_string(),
            Some(family) if !family.has_weight(600) => "No semibold: the nearest".to_string(),
            Some(_) => String::new(),
        })
    });
    let current = container()
        .width(fill())
        .padding([space::S2.px(), space::S3.px()])
        .corners(radius::SMALL.corners())
        .background(t.colors.surface)
        .when_hovered(move |s| s.background(t.colors.surface2))
        .on_click(move || open.set(!open.get_untracked()))
        .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
        .child(
            text(move || chosen.get().unwrap_or_else(|| "System".to_string()))
                .typeset(ty::LABEL_MEDIUM)
                .font_family(move || chosen.with(drawn_in))
                .color(t.colors.text),
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
            t.colors.sub,
        ));
    let list = move || {
        open.get().then(|| {
            let option = move |label: String, family: Option<String>| {
                let picked = family.clone();
                let face = drawn_in(&family);
                container()
                    .width(fill())
                    .padding([space::S1.px(), space::S2.px()])
                    .corners(radius::SMALL.corners())
                    .background(move || {
                        if chosen.with(|c| *c == family) {
                            t.colors.surface2
                        } else {
                            Color::TRANSPARENT
                        }
                    })
                    .when_hovered(move |s| s.background(t.colors.surface))
                    .on_click(move || {
                        chosen.set(picked.clone());
                        open.set(false);
                    })
                    .child(
                        text(label)
                            .typeset(ty::LABEL_MEDIUM)
                            .font_family(face)
                            .color(t.colors.text)
                            .nowrap(),
                    )
            };
            container()
                .width(fill())
                .layout(Flex::column().spacing(space::S1.px()))
                .child(
                    container()
                        .width(fill())
                        .padding([space::S1.px(), space::S2.px()])
                        .corners(radius::SMALL.corners())
                        .background(t.colors.surface)
                        .child(
                            text_input(query)
                                .typeset(ty::LABEL_SMALL)
                                .color(t.colors.text)
                                .placeholder("Search the installed fonts".to_string())
                                .placeholder_color(t.colors.sub)
                                .autofocus(),
                        ),
                )
                .child(
                    container()
                        .width(fill())
                        .height(space::S60.px())
                        .scroll(Scroll::vertical())
                        .child(move || {
                            // Built once and hidden as the query leaves them
                            // out: a keystroke shapes no text.
                            let options: Vec<Container> = families.with(|families| {
                                families
                                    .iter()
                                    .filter(|f| !mono || f.monospace)
                                    .map(|f| {
                                        let lower = f.name.to_lowercase();
                                        option(f.name.clone(), Some(f.name.clone())).visible(
                                            move || needle.with(|n| lower.contains(n.as_str())),
                                        )
                                    })
                                    .collect()
                            });
                            container()
                                .width(fill())
                                .layout(Flex::column())
                                .child(option("System".to_string(), None))
                                .children(options)
                        }),
                )
        })
    };
    field(
        name,
        Some(note),
        container()
            .width(fill())
            .layout(Flex::column().spacing(space::S1.px()))
            .child(current)
            .child(list),
    )
}

/// A titled run of settings.
fn group(title: &str, settings: Vec<Container>) -> Container {
    container()
        .width(fill())
        .layout(Flex::column().spacing(space::S3.px()))
        .child(section(title))
        .children(settings)
}

/// Where the colours come from, the mode, then the dark and the light side:
/// a theme's flavour and accent, or every colour set by hand.
fn themes(a: Appearance) -> Container {
    let theme = create_memo(move || a.choice.with(|c| c.theme.name));
    // Bumped to rebuild the colour fields after the colours are reset.
    let generation = create_signal(0u32);
    group(
        "Theme",
        vec![
            field(
                "Colours",
                None,
                choices(
                    SOURCES.iter().map(|(name, _)| *name).collect(),
                    move |index| a.source.get() == SOURCES[index].1,
                    move |index| {
                        let source = SOURCES[index].1;
                        // Set by hand from what shows until now.
                        if source == Source::Custom && a.source.get_untracked() != source {
                            a.colors.set(a.schemes());
                            generation.update(|n| *n += 1);
                        }
                        a.source.set(source);
                    },
                ),
            ),
            field(
                "Mode",
                Some(caption(move || {
                    let light = a.light();
                    let kind = if light { "light" } else { "dark" };
                    let shown = match a.source.get() {
                        Source::Custom => format!("your {kind} colours"),
                        Source::Wallpaper => format!("the wallpaper's {kind} colours"),
                        Source::Seed => format!("the seed's {kind} colours"),
                        Source::Preset => {
                            let choice = a.choice.get();
                            format!("{} {}", choice.theme.name, choice.flavor_of(light).name)
                        }
                    };
                    match (a.choice.get().mode, a.system.get()) {
                        (Mode::Auto, Some(_)) => format!("System: {shown}"),
                        (Mode::Auto, None) => format!("No system preference: {shown}"),
                        _ => format!("Showing {shown}"),
                    }
                })),
                choices(
                    MODES.iter().map(|(name, _)| *name).collect(),
                    move |index| a.choice.with(|c| c.mode) == MODES[index].1,
                    move |index| a.choose(|c| c.mode = MODES[index].1),
                ),
            ),
            container().width(fill()).child(move || {
                match a.source.get() {
                    Source::Preset => {}
                    Source::Custom => return container().width(fill()),
                    Source::Wallpaper => return generated(a, picture_field(a)),
                    Source::Seed => return generated(a, seed_field(a)),
                }
                field(
                    "Preset",
                    None,
                    choices(
                        THEMES.iter().map(|theme| theme.name).collect(),
                        move |index| a.choice.get().theme.name == THEMES[index].name,
                        move |index| a.choice.set(ThemeChoice::of(&THEMES[index])),
                    ),
                )
            }),
            container().width(fill()).child(move || {
                // Rebuilt with the theme, whose flavours and accents these
                // are, and with the source of the colours.
                theme.get();
                generation.get();
                let custom = match a.source.get() {
                    Source::Preset => false,
                    Source::Custom => true,
                    Source::Wallpaper | Source::Seed => return container(),
                };
                container()
                    .width(fill())
                    .layout(Flex::column().spacing(space::S2.px()))
                    .child(side(a, false, custom, generation))
                    .child(side(a, true, custom, generation))
            }),
        ],
    )
}

/// Where the colour comes from, then how the theme is made of it.
fn generated(a: Appearance, from: Container) -> Container {
    container()
        .width(fill())
        .layout(Flex::column().spacing(space::S6.px()))
        .child(from)
        .child(choice("Variant", VARIANTS, a.variant))
        .child(choice("Contrast", CONTRASTS, a.contrast))
}

/// The path of an image to read, and the colours found in it to pick from.
fn picture_field(a: Appearance) -> Container {
    let note = caption(move || {
        a.picture.with(|p| match p {
            Picture::None => "Choose one, or type a path and Enter".to_string(),
            Picture::Reading => "Reading…".to_string(),
            Picture::Read { seeds, .. } => format!("{} colours found", seeds.len()),
            Picture::Failed(err) => err.clone(),
        })
    });
    let input = text_input(a.path)
        .typeset(ty::MONO_SMALL)
        .color(ui().colors.text)
        .placeholder("~/Pictures/wallpaper.jpg".to_string())
        .placeholder_color(ui().colors.sub)
        .on_submit(move |path: &str| a.read_picture(path));
    container()
        .width(fill())
        .layout(Flex::column().spacing(space::S3.px()))
        .child(field(
            "Image",
            Some(note),
            container()
                .width(fill())
                .layout(centered_row(space::S2.px()))
                .child(boxed(input))
                .child(
                    button()
                        .icon(Icon::Folder)
                        .size(Size::Sm)
                        .on_click(move || a.chooser.send(())),
                ),
        ))
        .child(move || {
            let seeds = a.picture.with(|p| match p {
                Picture::Read { seeds, .. } => seeds.clone(),
                _ => Vec::new(),
            });
            (!seeds.is_empty()).then(|| {
                container().layout(centered_row(space::S1.px())).children(
                    seeds.into_iter().enumerate().map(move |(index, color)| {
                        swatch(color, move || a.picked.get() == index)
                            .on_click(move || a.picked.set(index))
                    }),
                )
            })
        })
}

/// A colour typed as a hex, beside the dot it makes.
fn seed_field(a: Appearance) -> Container {
    let dot = container()
        .width(space::S4.px())
        .height(space::S4.px())
        .corners(radius::pill(space::S4.px()))
        .border(stroke::HAIRLINE, ui().colors.line)
        .background(move || a.seed.with(|s| parse_hex(s)).unwrap_or(Color::TRANSPARENT));
    let input = text_input(a.seed)
        .typeset(ty::MONO_SMALL)
        .color(ui().colors.text);
    field(
        "Seed",
        None,
        container()
            .width(fill())
            .layout(centered_row(space::S2.px()))
            .child(dot)
            .child(boxed(input)),
    )
}

/// A field to type in, on the surface.
fn boxed(input: impl Widget + 'static) -> Container {
    container()
        .width(fill())
        .padding([space::S1.px(), space::S2.px()])
        .corners(radius::SMALL.corners())
        .background(ui().colors.surface)
        .child(input)
}

/// One side of the theme on a card, marked while it is the one that shows:
/// its flavour and accent, or its colours set by hand.
fn side(a: Appearance, light: bool, custom: bool, generation: RwSignal<u32>) -> Container {
    let t = ui();
    let title = if light { "Light" } else { "Dark" };
    let card = container()
        .width(fill())
        .padding(space::S3.px())
        .corners(radius::MEDIUM.corners())
        .background(t.colors.surface)
        .border(stroke::RING, move || {
            if a.light() == light {
                t.colors.accent.fill
            } else {
                Color::TRANSPARENT
            }
        })
        .layout(Flex::column().spacing(space::S4.px()))
        .child(
            container()
                .width(fill())
                .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
                .child(
                    container()
                        .layout(centered_row(space::S2.px()))
                        .child(icon(
                            if light { Icon::Sun } else { Icon::Moon },
                            icon_box::SMALL.px(),
                            t.colors.sub,
                        ))
                        .child(
                            text(title)
                                .typeset(ty::TITLE_XSMALL.em())
                                .color(t.colors.text),
                        ),
                )
                .child(caption(move || {
                    if a.light() == light {
                        "Showing".to_string()
                    } else {
                        String::new()
                    }
                })),
        );
    if custom {
        card.children(palette_fields(a, light, generation))
    } else {
        let flavor = create_memo(move || a.choice.with(|c| c.flavor_of(light)));
        card.child(field("Flavour", None, flavors(a, light)))
            .child(move || {
                let flavor = flavor.get();
                field(
                    "Accent",
                    Some(caption(move || {
                        a.choice.get().side(light).accent.to_string()
                    })),
                    container().layout(centered_row(space::S1.px())).children(
                        flavor.accents.iter().map(move |accent| {
                            let name = accent.name;
                            swatch(accent.color, move || {
                                a.choice.get().side(light).accent == name
                            })
                            .on_click(move || a.choose(|c| c.side_mut(light).accent = name))
                        }),
                    ),
                )
            })
    }
}

/// The theme's flavours of one kind, as buttons, or why there are none.
fn flavors(a: Appearance, light: bool) -> AnyWidget {
    let t = ui();
    let theme = a.choice.get_untracked().theme;
    let flavors = theme.flavors(light);
    if flavors.is_empty() {
        // Running text: it wraps inside the card rather than a caption's
        // one line.
        return container()
            .width(fill())
            .child(
                text(format!(
                    "{} has no {} flavour: the {} one shows instead.",
                    theme.name,
                    if light { "light" } else { "dark" },
                    if light { "dark" } else { "light" }
                ))
                .typeset(ty::BODY_SMALL)
                .color(t.colors.sub),
            )
            .into_any();
    }
    choices(
        flavors.iter().map(|flavor| flavor.name).collect(),
        move |index| a.choice.get().side(light).flavor == index,
        move |index| a.choose(|c| c.side_mut(light).flavor = index),
    )
    .into_any()
}

/// A field's text for colour `name` of side `index`: what parses as a
/// colour is written to that side, and nothing at all unsays an on-colour
/// or a hue's text.
fn bind_hex(
    colors: RwSignal<[Scheme; 2]>,
    index: usize,
    name: &'static str,
    color: Option<Color>,
) -> RwSignal<String> {
    let hex = create_signal(color.map(to_hex).unwrap_or_default());
    #[cfg(test)]
    tests::BOUND.with_borrow_mut(|bound| bound.push((index, name, hex)));
    create_effect(move || {
        let text = hex.get();
        let color = if text.trim().is_empty() {
            None
        } else if let Some(color) = parse_hex(&text) {
            Some(color)
        } else {
            return;
        };
        colors.update(|sides| sides[index] = sides[index].with(name, color));
    });
    hex
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

/// Every colour one side's scheme says, by name, as a hex to edit (an
/// on-colour or a hue's text may be left to be worked out), and a way back
/// to the preset's.
fn palette_fields(a: Appearance, light: bool, generation: RwSignal<u32>) -> Vec<Container> {
    let t = ui();
    let index = light as usize;
    let start = a.colors.get_untracked()[index];
    // Worked out once per change, for every swatch of the side.
    let palette = create_memo(move || a.colors.with(|sides| sides[index].palette()));
    let mut rows: Vec<Container> = start
        .named()
        .into_iter()
        .map(move |(name, color)| {
            let hex = bind_hex(a.colors, index, name, color);
            container()
                .width(fill())
                .layout(centered_row(space::S2.px()))
                .child(
                    container()
                        .width(space::S4.px())
                        .height(space::S4.px())
                        .corners(radius::pill(space::S4.px()))
                        .border(stroke::HAIRLINE, t.colors.line)
                        .background(move || {
                            palette
                                .with(|palette| drawn(palette, name).unwrap_or(Color::TRANSPARENT))
                        }),
                )
                .child(
                    container()
                        .width(space::S20.px())
                        .child(caption(name.replace('_', " "))),
                )
                .child(
                    container()
                        .width(fill())
                        .padding([space::S1.px(), space::S2.px()])
                        .corners(radius::SMALL.corners())
                        .background(t.colors.surface)
                        .child(
                            text_input(hex)
                                .typeset(ty::MONO_SMALL)
                                .color(t.colors.text)
                                .placeholder("auto".to_string())
                                .placeholder_color(t.colors.sub),
                        ),
                )
        })
        .collect();
    rows.push(
        container().child(
            button()
                .label("Copy from the preset")
                .variant(Variant::Neutral)
                .on_click(move || {
                    let preset = a.choice.get_untracked().scheme_of(light);
                    a.colors.update(|sides| sides[index] = preset);
                    generation.update(|n| *n += 1);
                }),
        ),
    );
    rows
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
fn preview(a: Appearance) -> Container {
    let controls = Controls {
        wifi: create_signal(true),
        volume: create_signal(60.0),
        muted: create_signal(false),
        auto: create_signal(true),
        night: create_signal(true),
        silent: create_signal(false),
    };
    let drawn = create_memo(move || a.ui());
    container()
        .width(fill())
        .height(fill())
        .padding(space::S3.px())
        .child(wallpaper(a.backdrop, move || {
            provide_ui(drawn.get());
            look::border(island(space::S4.px()))
                .width(space::S80.px())
                .layout(Flex::column().spacing(space::S4.px()))
                .child(panel(controls))
                .child(texts())
                .child(fills())
                .child(levels())
        }))
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
    let t = ui();
    let levels = || {
        container()
            .layout(Flex::column().spacing(space::S1.px()))
            .child(typed("Text", ty::LABEL_MEDIUM.em()).color(t.colors.text))
            .child(typed("Sub", ty::LABEL_SMALL).color(t.colors.sub))
            .child(icon(Icon::ChevronRight, icon_box::SMALL.px(), t.colors.dim))
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
                .background(t.colors.surface)
                .child(levels()),
        )
}

/// Each fill with the colour that reads on it, two to a row so a larger
/// density keeps every name on its line.
fn fills() -> Container {
    let t = ui();
    let swatch = |hue: Hue| {
        let colors = t.colors.hue(hue);
        container()
            .width(fill())
            .padding([space::S1.px(), space::S2.px()])
            .corners(radius::SMALL.corners())
            .background(colors.fill)
            .layout(Flex::row().main_alignment(MainAlignment::Center))
            .child(
                typed(format!("{hue:?}"), ty::LABEL_SMALL.em())
                    .color(colors.on)
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
    container()
        .width(fill())
        .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
        .child(level_figure(Icon::Cpu, "23%", Level::Normal))
        .child(level_figure(Icon::Thermometer, "74°", Level::Warn))
        .child(level_figure(Icon::MemoryStick, "93%", Level::Alert))
        .child(
            container().layout(centered_row(space::S2.px())).children(
                [
                    (0.64, ui().colors.success.fill),
                    (0.24, ui().colors.warning.fill),
                    (0.09, ui().colors.danger.fill),
                ]
                .map(|(value, color)| ring(value).color(color).size(icon_box::SMALL)),
            ),
        )
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use guido::prelude::*;
    use guido::testing::Headless;
    use styletto::theme::palette::{Scheme, parse_hex};
    use styletto::theme::presets::{Mode, THEMES, ThemeChoice};
    use styletto::ui::{Density, Material};

    use super::{Appearance, Source, bind_hex, drawn, palette_fields, themes};

    thread_local! {
        /// Every colour field built, as `bind_hex` makes them.
        pub static BOUND: std::cell::RefCell<Vec<(usize, &'static str, RwSignal<String>)>> =
            std::cell::RefCell::default();
    }

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

    #[test]
    fn the_light_card_edits_the_light_palette_only() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let probe = Rc::new(Cell::new(None));
        let seen = probe.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let a = Appearance::new();
            a.colors.set(a.schemes());
            a.source.set(Source::Custom);
            a.choose(|c| c.mode = Mode::Light);
            let generation = create_signal(0u32);
            seen.set(Some(a));
            container()
                .children(palette_fields(a, false, generation))
                .children(palette_fields(a, true, generation))
        });
        h.configure(id, 400, 2000, 1.0);
        h.step();
        let a: Appearance = probe.take().unwrap();
        let before = a.colors.get_untracked();
        let light_accent = BOUND.with_borrow(|bound| {
            bound
                .iter()
                .find(|(index, name, _)| *index == 1 && *name == "accent")
                .map(|(_, _, hex)| *hex)
                .unwrap()
        });
        light_accent.set("#ff0000".to_string());
        h.step();
        let after = a.colors.get_untracked();
        assert_eq!(after[1].accent.color, parse_hex("#ff0000").unwrap());
        assert_eq!(after[0], before[0], "the dark palette moved");
        assert_eq!(a.palette().accent.fill, parse_hex("#ff0000").unwrap());
        a.choose(|c| c.mode = Mode::Dark);
        h.step();
        assert_eq!(
            a.palette().accent.fill,
            before[0].accent.color,
            "dark mode shows the edit"
        );
        assert_eq!(
            a.ui().colors.accent.fill,
            before[0].accent.color,
            "the ui shows the edit"
        );
    }

    #[test]
    fn a_light_colour_leaves_the_dark_one() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let probe = Rc::new(Cell::new(None));
        let seen = probe.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let choice = ThemeChoice::of(&THEMES[1]);
            let colors = create_signal([choice.scheme_of(false), choice.scheme_of(true)]);
            let dark = bind_hex(
                colors,
                0,
                "accent",
                Some(colors.get_untracked()[0].accent.color),
            );
            let light = bind_hex(
                colors,
                1,
                "accent",
                Some(colors.get_untracked()[1].accent.color),
            );
            seen.set(Some((colors, dark, light)));
            container()
        });
        h.configure(id, 100, 100, 1.0);
        h.step();
        let (colors, _dark, light) = probe.take().unwrap();
        let before: [Scheme; 2] = colors.get_untracked();
        light.set("#ff0000".to_string());
        h.step();
        let after = colors.get_untracked();
        assert_eq!(after[1].accent.color, parse_hex("#ff0000").unwrap());
        assert_eq!(after[0].accent, before[0].accent, "the dark accent moved");
    }

    #[test]
    fn the_page_keeps_the_sides_apart() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        BOUND.with_borrow_mut(Vec::clear);
        let probe = Rc::new(Cell::new(None));
        let seen = probe.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let a = Appearance::new();
            seen.set(Some(a));
            themes(a)
        });
        h.configure(id, 400, 3000, 1.0);
        h.step();
        let a: Appearance = probe.take().unwrap();
        a.colors.set(a.schemes());
        a.source.set(Source::Custom);
        a.choose(|c| c.mode = Mode::Light);
        h.step();
        let before = a.colors.get_untracked();
        let field = |side: usize| {
            BOUND.with_borrow(|bound| {
                bound
                    .iter()
                    .rev()
                    .find(|(index, name, _)| *index == side && *name == "accent")
                    .map(|(_, _, hex)| *hex)
                    .unwrap()
            })
        };
        field(1).set("#ff0000".to_string());
        h.step();
        let after = a.colors.get_untracked();
        assert_eq!(after[1].accent.color, parse_hex("#ff0000").unwrap());
        assert_eq!(after[0], before[0], "the dark palette moved");
        a.choose(|c| c.mode = Mode::Dark);
        h.step();
        assert_eq!(
            a.colors.get_untracked()[0],
            before[0],
            "the dark palette moved on the switch"
        );
        assert_eq!(a.palette().accent.fill, before[0].accent.color);
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
                super::page(a)
            })
        });
        h.configure(id, 1200, 900, 1.0);
        h.step();
        let a: Appearance = probe.take().unwrap();
        for change in [
            Box::new(move || a.material.set(Material::Glass)) as Box<dyn Fn()>,
            Box::new(move || a.density.set(Density::Compact)),
            Box::new(move || a.choose(|c| c.mode = Mode::Light)),
            Box::new(move || a.source.set(Source::Custom)),
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
            super::page(a)
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
        a.material.set(Material::Glass);
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
