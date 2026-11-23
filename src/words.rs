//! The words the components show, by key, with the English of each: what an
//! app's translator ([`provide_translations`](crate::provide_translations))
//! is asked for, and what shows where it has none.
//!
//! A component asks for a key, never for English of its own: [`KEYS`] is the
//! one place the English is written. Names stay as they are, in every
//! language: a theme's, a flavour's, an accent's, a font family's.
//!
//! An argument is written as Fluent writes it, `{ $name }`, so each line of
//! [`KEYS`] is a Fluent message as it stands:
//!
//! ```text
//! styletto-note-system = System: { $shown }
//! ```

// Only the appearance controls, with `serde`, show words: without it the
// lookups below go unused.

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
    // What the mode shows: `shown` is one of the `styletto-shown-` words
    // below, or a preset's theme and flavour, by name.
    ("styletto-note-system", "System: { $shown }"),
    (
        "styletto-note-no-system",
        "No system preference: { $shown }",
    ),
    ("styletto-note-showing", "Showing { $shown }"),
    ("styletto-shown-custom-light", "your light colours"),
    ("styletto-shown-custom-dark", "your dark colours"),
    (
        "styletto-shown-wallpaper-light",
        "the wallpaper's light colours",
    ),
    (
        "styletto-shown-wallpaper-dark",
        "the wallpaper's dark colours",
    ),
    ("styletto-shown-seed-light", "the seed's light colours"),
    ("styletto-shown-seed-dark", "the seed's dark colours"),
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
#[cfg_attr(not(feature = "serde"), allow(dead_code))]
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
#[cfg_attr(not(feature = "serde"), allow(dead_code))]
pub(crate) const fn english(key: &str) -> &'static str {
    match lookup(key) {
        Some(english) => english,
        None => panic!("not a key of words::KEYS"),
    }
}

/// `str` equality, as a `const fn` can write it.
#[cfg_attr(not(feature = "serde"), allow(dead_code))]
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

/// `text` with each `{ $name }` of `args` replaced by its value.
#[cfg_attr(not(feature = "serde"), allow(dead_code))]
pub(crate) fn fill(text: &str, args: &[(&str, &str)]) -> String {
    args.iter().fold(text.to_string(), |text, (name, value)| {
        text.replace(&format!("{{ ${name} }}"), value)
    })
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
            fill(english("styletto-note-system"), &[("shown", "Nord Polar")]),
            "System: Nord Polar"
        );
        let both = fill(
            "{ $theme } and { $theme }, { $other }",
            &[("theme", "a"), ("other", "b")],
        );
        assert_eq!(both, "a and a, b");
        // An argument the text doesn't take changes nothing.
        assert_eq!(fill("Reading…", &[("shown", "x")]), "Reading…");
    }
}
