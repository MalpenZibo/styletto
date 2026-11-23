//! The pickers in another language: every word they show is asked for by its
//! key, a translator that has none leaves the English, and a new language
//! shows in place.

use std::collections::HashSet;

use guido::layout::Constraints;
use guido::reactive::__internal::with_owner;
use guido::renderer::{DrawCommand, RenderNode};
use guido::tree::Tree;

use super::*;
use crate::theme::presets::THEMES;
use crate::ui::provide_translations;

/// Every text `build` draws, laid out `width` wide, as it is drawn (a
/// section's title is upper case).
fn drawn(build: impl FnOnce() -> Container) -> Vec<String> {
    let (texts, _scope) = with_owner(|| {
        let mut tree = Tree::new();
        let root = tree.register(Box::new(build()));
        tree.with_widget_mut(root, |w, id, t| w.register_children(t, id));
        tree.set_frame_instant(Some(std::time::Instant::now()));
        tree.layout_widget(root, Constraints::new(0.0, 0.0, 400.0, 100_000.0));
        let mut node = RenderNode::new(root.as_u64());
        tree.paint_widget(root, &mut node, 1.0);
        let mut texts = Vec::new();
        collect(&node, &mut texts);
        // Kept: the icons it decoded are cached for the next drawing, and
        // would go with it.
        std::mem::forget(tree);
        texts
    });
    texts
}

fn collect(node: &RenderNode, texts: &mut Vec<String>) {
    for command in node.commands.iter().chain(&node.overlay_commands) {
        if let DrawCommand::Text { text, .. } = &**command {
            texts.push(text.to_string());
        }
    }
    for child in &node.children {
        collect(child, texts);
    }
}

/// The families the font picker is handed, one of them with no semibold.
fn families() -> Vec<Family> {
    vec![
        Family {
            name: "Inter".to_string(),
            monospace: false,
            semibold: true,
        },
        Family {
            name: "Thin Mono".to_string(),
            monospace: true,
            semibold: false,
        },
    ]
}

/// A wallpaper's path, as typed.
const PATH: &str = "/nowhere/wallpaper.png";

/// Configs that between them show every state the pickers have words for:
/// each source, a theme with no light flavour, a family chosen that isn't
/// installed and one with no semibold.
fn configs() -> Vec<UiConfig> {
    let mut configs = Vec::new();
    for mode in [Mode::Dark, Mode::Light, Mode::Auto] {
        for source in SOURCES.iter().map(|&(_, source)| source) {
            let mut config = UiConfig {
                mode,
                ..UiConfig::default()
            };
            source.apply(&mut config, &[]);
            configs.push(config);
        }
    }
    // A wallpaper chosen, showing either side: being read (none is here).
    for mode in [Mode::Dark, Mode::Light] {
        let mut wallpaper = UiConfig {
            mode,
            ..UiConfig::default()
        };
        wallpaper.set_wallpaper(PATH.to_string());
        configs.push(wallpaper);
    }
    // Every theme, a side of one with no flavour of its own among them.
    configs.extend(THEMES.iter().map(|theme| UiConfig {
        theme: Some(theme.name.to_string()),
        ..UiConfig::default()
    }));
    let mut fonts = UiConfig::default();
    fonts.fonts.sans = Some("Gone Sans".to_string());
    fonts.fonts.mono = Some("Thin Mono".to_string());
    configs.push(fonts);
    configs
}

/// Every picker over `config`, the system's preference said and not.
fn pickers(config: RwSignal<UiConfig>) -> Vec<Container> {
    [Some(true), None]
        .into_iter()
        .map(|system| {
            container()
                .width(fill())
                .child(appearance(config).system_dark(system).families(families()))
                // Each piece also alone, as an app may set it.
                .child(mode_picker(config).system_dark(system))
                .child(
                    theme_picker(config)
                        .system_dark(system)
                        .on_pick_wallpaper(|| {}),
                )
                .child(color_editor(config, true))
                .child(layout_picker(config))
                .child(font_picker(config).families(families()))
        })
        .collect()
}

/// A word as the marking translator gives it: its key and its arguments.
fn marked(key: &str, args: &[(&str, &str)]) -> String {
    let args: Vec<String> = args.iter().map(|(n, v)| format!(" {n}={v}")).collect();
    format!("«{key}{}»", args.concat())
}

/// The names the pickers may show untranslated: themes, flavours and
/// accents, the families, a theme with its flavour, the colours of the
/// editor, by their names in the config file, and what is typed.
fn names() -> HashSet<String> {
    let mut names: HashSet<String> = families().into_iter().map(|f| f.name).collect();
    names.extend(["Gone Sans".to_string(), PATH.to_string()]);
    names.extend(
        Scheme::default()
            .named()
            .map(|(name, _)| name.replace('_', " ")),
    );
    for theme in THEMES {
        names.insert(theme.name.to_string());
        for flavor in theme.flavors {
            names.insert(flavor.name.to_string());
            names.insert(format!("{} {}", theme.name, flavor.name));
            names.extend(flavor.accents.iter().map(|a| a.name.to_string()));
        }
    }
    names
}

#[test]
fn every_word_shown_is_asked_for_by_its_key() {
    let names = names();
    let asked = std::rc::Rc::new(std::cell::RefCell::new(HashSet::new()));
    let mut shown = 0;
    for config in configs() {
        let asked = asked.clone();
        let texts = drawn(move || {
            provide_translations(move |key, args| {
                asked.borrow_mut().insert(key.to_string());
                Some(marked(key, args))
            });
            container().children(pickers(create_signal(config)))
        });
        shown += texts.len();
        for text in texts {
            let marked = text.starts_with('«') && text.ends_with('»');
            // An argument is a name, or words asked for by key.
            let arguments_are_names = || {
                text.trim_matches(['«', '»'])
                    .split_once('=')
                    .is_none_or(|(_, value)| names.contains(value) || value.starts_with('«'))
            };
            let hex = text.starts_with('#') && parse_hex(&text).is_some();
            assert!(
                (marked && arguments_are_names()) || names.contains(&text) || hex,
                "{text:?} is shown in no language of the app's"
            );
        }
    }
    assert!(shown > 100, "the pickers drew {shown} texts");
    // Every source offered, every key was asked for but these: no preset
    // lacks a dark flavour, a wallpaper isn't read here (the test below
    // sees why one can't be), and the font list is shown once opened.
    if cfg!(all(feature = "material", feature = "wallpaper")) {
        let asked = asked.borrow();
        let unseen: Vec<_> = crate::words::KEYS
            .iter()
            .map(|&(key, _)| key)
            .filter(|key| !asked.contains(*key))
            .collect();
        assert_eq!(
            unseen,
            [
                "styletto-note-no-dark-flavour",
                "styletto-wallpaper-no-colour",
                "styletto-wallpaper-unreadable",
                "styletto-font-search",
            ],
            "words never asked for"
        );
    }
}

#[cfg(feature = "wallpaper")]
#[test]
fn wallpaper_reasons_are_translated_and_the_systems_are_not() {
    let ((), _scope) = with_owner(|| {
        provide_translations(|key, args| Some(marked(key, args)));
        let w = words();
        for key in [
            "styletto-wallpaper-no-colour",
            "styletto-wallpaper-unreadable",
        ] {
            assert_eq!(failed(w, english(key).to_string()), marked(key, &[]));
        }
        let system = "No such file or directory (os error 2)";
        assert_eq!(failed(w, system.to_string()), system);
    });
}

#[test]
fn a_translator_that_has_no_words_leaves_the_english() {
    let english = |translate: bool| {
        let mut config = UiConfig {
            mode: Mode::Auto,
            ..UiConfig::default()
        };
        Source::Custom.apply(&mut config, &[]);
        drawn(move || {
            if translate {
                provide_translations(|_, _| None);
            }
            container().children(pickers(create_signal(config)))
        })
    };
    let texts = english(true);
    assert_eq!(texts, english(false), "not as with no translator");
    for word in [
        "Density",
        "Comfortable",
        "System: your dark colours",
        "No system preference: your dark colours",
        "Copy from the preset",
    ] {
        assert!(texts.iter().any(|t| t == word), "no {word:?} in {texts:?}");
    }
}

#[test]
fn a_new_language_shows_in_place() {
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use guido::testing::Headless;

    let Some(mut h) = Headless::new_or_skip() else {
        return;
    };
    // The panel, its words marked or not as `marking` says, counting its
    // builds.
    let mount = |h: &mut Headless, marking: RwSignal<bool>, builds: Rc<Cell<u32>>| {
        let id = h.surface(SurfaceConfig::new(), move || {
            provide_translations(move |key, args| marking.get().then(|| marked(key, args)));
            let config = create_signal(UiConfig::default());
            let builds = builds.clone();
            container().width(fill()).child(move || {
                builds.set(builds.get() + 1);
                appearance(config).system_dark(true).families(families())
            })
        });
        h.configure(id, 400, 1400, 1.0);
        id
    };
    let start = Instant::now();
    let settle = |h: &mut Headless, at: Instant| {
        for frame in 0..120 {
            h.step_at(at + Duration::from_millis(frame * 16));
        }
    };
    let marking = create_signal(false);
    let builds = Rc::new(Cell::new(0));
    let id = mount(&mut h, marking, builds.clone());
    settle(&mut h, start);
    let english = h.read_frame(id);
    marking.set(true);
    settle(&mut h, start + Duration::from_secs(2));
    let marked = h.read_frame(id);
    let fresh = mount(&mut h, create_signal(true), Rc::default());
    settle(&mut h, start + Duration::from_secs(4));
    assert_ne!(english, marked, "the language changed nothing");
    assert!(
        marked == h.read_frame(fresh),
        "not as a panel built in the new language"
    );
    assert_eq!(builds.get(), 1, "the panel was built again");
}
