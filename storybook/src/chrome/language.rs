//! The language styletto's words show in: English, or a pseudo-language that
//! shows what a translation does to a layout, every word longer, accented
//! and bracketed, while it still reads.

use styletto::provide_translations;
use styletto::words::KEYS;

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
    let mut out = String::from("[");
    let mut rest = english;
    let mut letters = 0;
    while !rest.is_empty() {
        // An argument, `{ $name }`, is filled in, not accented.
        if let Some(after) = rest.strip_prefix("{ $")
            && let Some((name, after)) = after.split_once(" }")
        {
            let value = args.iter().find(|(n, _)| *n == name).map_or("", |a| a.1);
            out.push_str(value);
            rest = after;
            continue;
        }
        let c = rest.chars().next()?;
        out.push(accented(c));
        letters += usize::from(c.is_alphabetic());
        rest = &rest[c.len_utf8()..];
    }
    // Rounded up, so even a short word grows.
    out.extend(std::iter::repeat_n('·', (letters * 2).div_ceil(5)));
    out.push(']');
    Some(out)
}

fn accented(c: char) -> char {
    const PLAIN: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const ACCENTED: &str = "áƀçðéƒĝĥîĵķļɱñöþǫŕšţûṽŵẋýžÅƁÇĐÉƑĜĤÎĴĶĿṀÑÖÞǪŔŠŢÛṼŴẊÝŽ";
    PLAIN
        .chars()
        .position(|p| p == c)
        .and_then(|i| ACCENTED.chars().nth(i))
        .unwrap_or(c)
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
            pseudo("styletto-note-showing", &[("shown", "Nord Polar")]).as_deref(),
            Some("[Šĥöŵîñĝ Nord Polar···]")
        );
        assert_eq!(pseudo("not-a-key", &[]), None);
        // Every word has one.
        for (key, _) in KEYS {
            assert!(pseudo(key, &[]).is_some(), "{key}");
        }
    }
}
