//! The words the components show, by key, with the English of each: what an
//! app's translator ([`provide_translations`]) is asked for, and what shows
//! where it has none.
//!
//! A component asks for a key, never for English of its own: [`KEYS`] is the
//! one place the English is written. Names stay as they are, in every
//! language: a theme's, a flavour's, an accent's, a font family's.
//!
//! An argument is written as Fluent writes it, `{ $name }`, so each line of
//! [`KEYS`] is a Fluent message as it stands:
//!
//! ```text
//! styletto-note-system-preset = System: { $theme } { $flavour }
//! ```

// Only the appearance controls, with `serde`, show words: without it, the
// handle and the lookups go unused.
#![cfg_attr(not(feature = "serde"), allow(dead_code))]

use std::rc::Rc;

use guido::prelude::*;

/// Every word a component shows, as `(key, English)`. The key is what a
/// translator is asked for; the English is what shows where it gives
/// nothing, its `{ $name }` arguments filled in. A key is never taken away
/// nor its arguments changed; a new one shows in English until an app's
/// catalog has it.
pub const KEYS: &[(&str, &str)] = &[
    // Where the colours come from.
    ("styletto-source-preset", "Preset"),
    ("styletto-source-seed", "Seed"),
    ("styletto-source-wallpaper", "Wallpaper"),
    ("styletto-source-custom", "Custom"),
    // The choices of each setting.
    ("styletto-mode-light", "Light"),
    ("styletto-mode-dark", "Dark"),
    ("styletto-mode-auto", "Auto"),
    ("styletto-density-compact", "Compact"),
    ("styletto-density-default", "Default"),
    ("styletto-density-comfortable", "Comfortable"),
    ("styletto-shape-square", "Square"),
    ("styletto-shape-soft", "Soft"),
    ("styletto-shape-round", "Round"),
    ("styletto-material-solid", "Solid"),
    ("styletto-material-translucent", "Translucent"),
    ("styletto-material-glass", "Glass"),
    ("styletto-motion-full", "Full"),
    ("styletto-motion-reduced", "Reduced"),
    ("styletto-motion-none", "None"),
    ("styletto-variant-tonal-spot", "Tonal spot"),
    ("styletto-variant-vibrant", "Vibrant"),
    ("styletto-variant-expressive", "Expressive"),
    ("styletto-variant-content", "Content"),
    ("styletto-variant-fidelity", "Fidelity"),
    ("styletto-variant-neutral", "Neutral"),
    ("styletto-variant-monochrome", "Monochrome"),
    ("styletto-variant-rainbow", "Rainbow"),
    ("styletto-variant-fruit-salad", "Fruit salad"),
    ("styletto-contrast-standard", "Standard"),
    ("styletto-contrast-medium", "Medium"),
    ("styletto-contrast-high", "High"),
    // The settings' names, and the groups they are in.
    ("styletto-setting-colours", "Colours"),
    ("styletto-setting-mode", "Mode"),
    ("styletto-setting-density", "Density"),
    ("styletto-setting-shape", "Shape"),
    ("styletto-setting-material", "Material"),
    ("styletto-setting-motion", "Motion"),
    ("styletto-setting-preset", "Preset"),
    ("styletto-setting-flavour", "Flavour"),
    ("styletto-setting-accent", "Accent"),
    ("styletto-setting-variant", "Variant"),
    ("styletto-setting-contrast", "Contrast"),
    ("styletto-setting-seed", "Seed"),
    ("styletto-setting-wallpaper", "Wallpaper"),
    ("styletto-setting-sans", "Sans"),
    ("styletto-setting-mono", "Mono"),
    ("styletto-group-theme", "Theme"),
    ("styletto-group-layout", "Layout"),
    ("styletto-group-fonts", "Fonts"),
    // What the mode shows, one whole sentence for each: told by the system,
    // with no system preference, or by the mode itself; of a preset, named,
    // or of the colours made by hand, the wallpaper or the seed, by side.
    (
        "styletto-note-system-preset",
        "System: { $theme } { $flavour }",
    ),
    (
        "styletto-note-system-custom-dark",
        "System: your dark colours",
    ),
    (
        "styletto-note-system-custom-light",
        "System: your light colours",
    ),
    (
        "styletto-note-system-wallpaper-dark",
        "System: the wallpaper's dark colours",
    ),
    (
        "styletto-note-system-wallpaper-light",
        "System: the wallpaper's light colours",
    ),
    (
        "styletto-note-system-seed-dark",
        "System: the seed's dark colours",
    ),
    (
        "styletto-note-system-seed-light",
        "System: the seed's light colours",
    ),
    (
        "styletto-note-no-system-preset",
        "No system preference: { $theme } { $flavour }",
    ),
    (
        "styletto-note-no-system-custom-dark",
        "No system preference: your dark colours",
    ),
    (
        "styletto-note-no-system-custom-light",
        "No system preference: your light colours",
    ),
    (
        "styletto-note-no-system-wallpaper-dark",
        "No system preference: the wallpaper's dark colours",
    ),
    (
        "styletto-note-no-system-wallpaper-light",
        "No system preference: the wallpaper's light colours",
    ),
    (
        "styletto-note-no-system-seed-dark",
        "No system preference: the seed's dark colours",
    ),
    (
        "styletto-note-no-system-seed-light",
        "No system preference: the seed's light colours",
    ),
    (
        "styletto-note-showing-preset",
        "Showing { $theme } { $flavour }",
    ),
    (
        "styletto-note-showing-custom-dark",
        "Showing your dark colours",
    ),
    (
        "styletto-note-showing-custom-light",
        "Showing your light colours",
    ),
    (
        "styletto-note-showing-wallpaper-dark",
        "Showing the wallpaper's dark colours",
    ),
    (
        "styletto-note-showing-wallpaper-light",
        "Showing the wallpaper's light colours",
    ),
    (
        "styletto-note-showing-seed-dark",
        "Showing the seed's dark colours",
    ),
    (
        "styletto-note-showing-seed-light",
        "Showing the seed's light colours",
    ),
    // A side's card.
    ("styletto-side-light", "Light"),
    ("styletto-side-dark", "Dark"),
    ("styletto-side-showing", "Showing"),
    (
        "styletto-note-no-light-flavour",
        "{ $theme } has no light flavour: the dark one shows instead.",
    ),
    (
        "styletto-note-no-dark-flavour",
        "{ $theme } has no dark flavour: the light one shows instead.",
    ),
    ("styletto-copy-preset", "Copy from the preset"),
    // The wallpaper's field.
    ("styletto-wallpaper-placeholder", "~/Pictures/wallpaper.jpg"),
    (
        "styletto-wallpaper-none",
        "Choose one, or type a path and Enter",
    ),
    ("styletto-wallpaper-reading", "Reading…"),
    ("styletto-wallpaper-no-colour", "No colour found"),
    ("styletto-wallpaper-unreadable", "Can't be read"),
    // The fonts.
    ("styletto-font-system", "System"),
    ("styletto-font-not-installed", "Not installed"),
    ("styletto-font-no-semibold", "No semibold: the nearest"),
    ("styletto-font-search", "Search the installed fonts"),
];

/// The English of `key`, if it is one of [`KEYS`].
pub(crate) const fn lookup(key: &str) -> Option<&'static str> {
    let mut i = 0;
    while i < KEYS.len() {
        if same(KEYS[i].0, key) {
            return Some(KEYS[i].1);
        }
        i += 1;
    }
    None
}

/// The English of `key`, which must be one of [`KEYS`]: in a constant, a key
/// misspelt doesn't build.
pub(crate) const fn english(key: &str) -> &'static str {
    match lookup(key) {
        Some(english) => english,
        None => panic!("not a key of words::KEYS"),
    }
}

/// `str` equality, as a `const fn` can write it.
const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// `text` with each `{ $name }` of `args` replaced by its value, as the
/// English of [`KEYS`] is filled in: for a translator whose catalog writes
/// its arguments so, a map or gettext's. As in Fluent, the spaces inside
/// the braces are free, `{$name}` as well. It is one pass over `text`, so
/// a value is never filled in again. A placeholder `args` has no value for,
/// or anything else in braces, is left as it is; an argument the text
/// doesn't take is left out.
///
/// ```
/// use styletto::words::fill;
///
/// let args = [("theme", "Nord"), ("flavour", "Polar")];
/// assert_eq!(fill("Système : { $theme } {$flavour}", &args), "Système : Nord Polar");
/// ```
pub fn fill(text: &str, args: &[(&str, &str)]) -> String {
    let mut filled = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        filled.push_str(&rest[..open]);
        let from = &rest[open..];
        let value = from.find('}').and_then(|close| {
            let name = from[1..close].trim().strip_prefix('$')?;
            let (_, value) = args.iter().find(|(n, _)| *n == name)?;
            Some((*value, close))
        });
        match value {
            Some((value, close)) => {
                filled.push_str(value);
                rest = &from[close + 1..];
            }
            // Not an argument of `args`: kept, and read on past its brace.
            None => {
                filled.push('{');
                rest = &from[1..];
            }
        }
    }
    filled.push_str(rest);
    filled
}

/// What an app's translator is: a key of [`KEYS`] and its arguments, by
/// name, to the words in the app's language, or `None` to leave them in
/// English.
type Translate = dyn Fn(&str, &[(&str, &str)]) -> Option<String>;

/// The translator provided.
#[derive(Clone, Copy)]
struct Translations(Signal<Rc<Translate>>);

/// Gives the components built from here on the words of the app's language:
/// `translate` is asked for each by its key in [`KEYS`], with its arguments
/// by name (`[("theme", "Nord"), ("flavour", "Polar")]`), and gives the
/// words with those filled in, or `None` for the English of `KEYS`. A Fluent
/// bundle's `format_pattern`, gettext or a map all fit; [`fill`] fills in a
/// catalog's `{ $name }`.
///
/// Each word is asked for where it is drawn, so a translator that reads a
/// signal, the app's language, puts a new language in place without
/// building anything again. Without one, every word is English.
pub fn provide_translations(translate: impl Fn(&str, &[(&str, &str)]) -> Option<String> + 'static) {
    let translate: Rc<Translate> = Rc::new(translate);
    provide_context(Translations(create_stored(translate)));
}

/// A handle on the nearest translator, taken while building, as
/// [`look`](crate::look) is, and asked inside the closures a widget's
/// properties take.
#[derive(Clone, Copy)]
pub(crate) struct Words(Option<Signal<Rc<Translate>>>);

/// The nearest translator, or English.
pub(crate) fn words() -> Words {
    Words(use_context::<Translations>().map(|translations| translations.0))
}

impl Words {
    /// The words of `key` with `args` filled in, as the translator gives
    /// them or else in English. Whatever the translator reads is tracked.
    pub(crate) fn get(self, key: &str, args: &[(&str, &str)]) -> String {
        self.translated(key, args)
            .unwrap_or_else(|| fill(lookup(key).unwrap_or(key), args))
    }

    /// The words of `key`, which takes no argument, as a closure a property
    /// takes. Its English is looked up once, here, not on every run.
    pub(crate) fn of(self, key: &'static str) -> impl Fn() -> String + Copy + 'static {
        let english = lookup(key).unwrap_or(key);
        move || {
            self.translated(key, &[])
                .unwrap_or_else(|| english.to_string())
        }
    }

    /// What the translator gives for `key`, if there is one and it has
    /// words for it.
    fn translated(self, key: &str, args: &[(&str, &str)]) -> Option<String> {
        // Called outside the stored signal's read, so a signal the
        // translator reads is tracked as any other.
        let translate = self.0?.with_untracked(Rc::clone);
        translate(key, args)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::Path;

    use super::*;

    /// Every Rust file under `dir`, with its text.
    fn sources(dir: &Path, found: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                found.push((path.display().to_string(), text));
            }
        }
    }

    fn crate_sources() -> Vec<(String, String)> {
        let mut found = Vec::new();
        sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut found,
        );
        found
    }

    #[test]
    fn every_key_is_one_of_a_kind_and_shown_somewhere() {
        let mut seen = HashSet::new();
        for (key, english) in KEYS {
            assert!(seen.insert(key), "{key} twice");
            assert!(
                key.starts_with("styletto-")
                    && key.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{key} is not a Fluent identifier of styletto's"
            );
            assert!(!english.is_empty(), "{key} says nothing");
        }
        // Each key written, quoted, in the source of what shows it.
        let sources = crate_sources();
        for (key, _) in KEYS {
            let quoted = format!("\"{key}\"");
            assert!(
                sources
                    .iter()
                    .any(|(path, text)| !path.ends_with("words.rs") && text.contains(&quoted)),
                "{key} is shown nowhere"
            );
        }
    }

    #[test]
    fn every_key_a_component_asks_for_is_one() {
        // A quoted `styletto-` word in a component is a key; one missing
        // from KEYS would show the key itself.
        for (path, text) in crate_sources()
            .iter()
            .filter(|(path, _)| path.contains("components"))
        {
            for quoted in text.split('"').skip(1).step_by(2) {
                if quoted.starts_with("styletto-") {
                    assert!(lookup(quoted).is_some(), "{path}: {quoted} is not a key");
                }
            }
        }
    }

    #[test]
    fn arguments_are_filled_in_by_name() {
        assert_eq!(
            fill(
                english("styletto-note-system-preset"),
                &[("theme", "Nord"), ("flavour", "Polar")]
            ),
            "System: Nord Polar"
        );
        let both = fill(
            "{ $theme } and { $theme }, { $other }",
            &[("theme", "a"), ("other", "b")],
        );
        assert_eq!(both, "a and a, b");
        // An argument the text doesn't take changes nothing.
        assert_eq!(fill("Reading…", &[("theme", "x")]), "Reading…");
    }

    #[test]
    fn any_spacing_fluent_allows_is_filled_in() {
        let args = [("theme", "Nord")];
        for text in ["{ $theme }", "{$theme}", "{  $theme}", "{$theme   }"] {
            assert_eq!(fill(text, &args), "Nord", "{text:?}");
        }
    }

    #[test]
    fn a_value_is_never_filled_in_again() {
        let args = [("theme", "A { $flavour }"), ("flavour", "Dark")];
        assert_eq!(
            fill("{ $theme } { $flavour }", &args),
            "A { $flavour } Dark"
        );
    }

    #[test]
    fn what_has_no_value_is_left_as_it_is() {
        let args = [("theme", "Nord")];
        assert_eq!(
            fill("{ $other } {theme} { $theme } {", &args),
            "{ $other } {theme} Nord {"
        );
    }

    #[test]
    fn a_translator_with_no_words_leaves_the_english() {
        use guido::reactive::__internal::with_owner;

        let args = [("theme", "Nord"), ("flavour", "Polar")];
        let english = |translate: bool| {
            with_owner(|| {
                if translate {
                    provide_translations(|_, _| None);
                }
                let w = words();
                [
                    w.get("styletto-density-compact", &[]),
                    w.get("styletto-note-no-system-preset", &args),
                    w.get("styletto-note-showing-wallpaper-dark", &[]),
                ]
            })
            .0
        };
        let expected = [
            "Compact",
            "No system preference: Nord Polar",
            "Showing the wallpaper's dark colours",
        ];
        assert_eq!(english(true), expected);
        assert_eq!(english(false), expected, "not as with no translator");
    }
}
