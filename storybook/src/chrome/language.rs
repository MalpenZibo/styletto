//! The language styletto's words show in: English, or a pseudo-language that
//! shows what a translation does to a layout, every word longer, accented
//! and bracketed, while it still reads.

use styletto::provide_translations;
use styletto::words::{KEYS, fill};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Language {
    #[default]
    English,
    Pseudo,
}

/// The languages, by name, for a choice and for `shot language=`.
pub const LANGUAGES: &[(&str, Language)] =
    &[("English", Language::English), ("Pseudo", Language::Pseudo)];

impl Language {
    /// The language of `name`, whatever its case.
    pub fn named(name: &str) -> Option<Self> {
        LANGUAGES
            .iter()
            .find(|(known, _)| known.eq_ignore_ascii_case(name))
            .map(|&(_, language)| language)
    }
}

/// Gives what is built from here on styletto's words in `language`, as it
/// changes: a new one shows in place.
pub fn provide(language: impl Fn() -> Language + 'static) {
    provide_translations(move |key, args| match language() {
        Language::English => None,
        Language::Pseudo => pseudo(key, args),
    });
}

/// The words of `key` in the pseudo-language: each letter accented, about
/// 40% longer, in brackets, "[Ŀîĝĥţ··]"; its arguments, names, as they are.
fn pseudo(key: &str, args: &[(&str, &str)]) -> Option<String> {
    let english = KEYS.iter().find(|(known, _)| *known == key)?.1;
    // Accented but for an argument's `{ $name }`, which `fill` fills in.
    let mut within = false;
    let mut letters = 0;
    let accented: String = english
        .chars()
        .map(|c| {
            within = (within || c == '{') && c != '}';
            letters += usize::from(!within && c.is_alphabetic());
            if within { c } else { accented(c) }
        })
        .collect();
    // Rounded up, so even a short word grows.
    let longer = "·".repeat((letters * 2).div_ceil(5));
    Some(format!("[{}{longer}]", fill(&accented, args)))
}

/// Each letter of the English alphabet, lower case then upper, accented.
const ACCENTED: [char; 52] = [
    'á', 'ƀ', 'ç', 'ð', 'é', 'ƒ', 'ĝ', 'ĥ', 'î', 'ĵ', 'ķ', 'ļ', 'ɱ', 'ñ', 'ö', 'þ', 'ǫ', 'ŕ', 'š',
    'ţ', 'û', 'ṽ', 'ŵ', 'ẋ', 'ý', 'ž', 'Å', 'Ɓ', 'Ç', 'Đ', 'É', 'Ƒ', 'Ĝ', 'Ĥ', 'Î', 'Ĵ', 'Ķ', 'Ŀ',
    'Ṁ', 'Ñ', 'Ö', 'Þ', 'Ǫ', 'Ŕ', 'Š', 'Ţ', 'Û', 'Ṽ', 'Ŵ', 'Ẋ', 'Ý', 'Ž',
];

fn accented(c: char) -> char {
    match c {
        'a'..='z' => ACCENTED[(c as u8 - b'a') as usize],
        'A'..='Z' => ACCENTED[26 + (c as u8 - b'A') as usize],
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pseudo_word_is_longer_accented_and_bracketed() {
        assert_eq!(
            pseudo("styletto-mode-light", &[]).as_deref(),
            Some("[Ŀîĝĥţ··]")
        );
        // Names are filled in as they are.
        assert_eq!(
            pseudo(
                "styletto-note-showing-preset",
                &[("theme", "Nord"), ("flavour", "Polar")]
            )
            .as_deref(),
            Some("[Šĥöŵîñĝ Nord Polar···]")
        );
        assert_eq!(pseudo("not-a-key", &[]), None);
        // Every word has one.
        for (key, _) in KEYS {
            assert!(pseudo(key, &[]).is_some(), "{key}");
        }
    }
}
