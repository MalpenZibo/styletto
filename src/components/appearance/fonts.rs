//! The font families installed, for [`font_picker`](super::font_picker) to
//! offer, as the font database the text renderer loads sees them: read once
//! per process, on a thread of their own, since that reads every font file.
//! Only families that write Latin text are kept, so a symbol or an emoji font
//! is left out.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use guido::prelude::*;

use super::Family;
use crate::foundations::typography::LABEL_MEDIUM;

/// The families installed, by name, empty until they have been read.
///
/// The first call starts the reading, on a thread; every later one shares
/// what it finds, and none reads again. Each call gets a signal in its own
/// scope, so a picker that goes away takes only its own.
///
/// They are the system's fonts as fontdb finds them, read apart from the
/// renderer's; the source may become Guido's own font database once it
/// shares it (MalpenZibo/guido#948), which also knows the fonts an app
/// loaded itself.
pub fn installed_families() -> Signal<Vec<Family>> {
    // The list is the process's, the signal the caller's: a signal lives in
    // the scope that made it, and the first caller's may go before the
    // others'. A `GlobalSignal`, which the app owns, will take its place
    // once Guido makes it public (MalpenZibo/guido#947).
    static FOUND: OnceLock<Vec<Family>> = OnceLock::new();
    if let Some(found) = FOUND.get() {
        return create_stored(found.clone());
    }
    let families = create_signal(Vec::new());
    let writer = families.writer();
    // The first thread reads; one started while it does waits for it. Set
    // from there, the write waits for this thread's next frame, and one to a
    // signal whose scope has gone since is dropped.
    std::thread::spawn(move || writer.set(FOUND.get_or_init(system_families).clone()));
    families.read_only()
}

/// The families of the system's fonts.
fn system_families() -> Vec<Family> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    families(&db)
}

/// Every family of `db` that writes Latin text, by name: monospaced if every
/// face of it is, and semibold if it draws the weight of the emphasised type
/// (`.em()`), or one within a step of it.
fn families(db: &fontdb::Database) -> Vec<Family> {
    let semibold = LABEL_MEDIUM.em().resolve().weight.0;
    let mut found: BTreeMap<&str, Family> = BTreeMap::new();
    for face in db.faces() {
        let Some((name, _)) = face.families.first() else {
            continue;
        };
        // Whether it writes Latin text, and the span of its weight axis if
        // it is a variable face.
        let read = db.with_face_data(face.id, |data, index| {
            let parsed = ttf_parser::Face::parse(data, index).ok()?;
            let latin = parsed.glyph_index('a').is_some() && parsed.glyph_index('A').is_some();
            let axis = parsed
                .variation_axes()
                .into_iter()
                .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wght"))
                .map(|axis| (axis.min_value as u16, axis.max_value as u16));
            Some((latin, axis))
        });
        let Some(Some((true, axis))) = read else {
            continue;
        };
        // The weights it draws: one for a static face, the span of its
        // weight axis for a variable one.
        let (low, high) = axis.unwrap_or((face.weight.0, face.weight.0));
        let family = found.entry(name).or_insert_with(|| Family {
            name: name.clone(),
            monospace: true,
            semibold: false,
        });
        family.monospace &= face.monospaced;
        family.semibold |= semibold + 50 >= low && semibold <= high + 50;
    }
    found.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_family_is_read_as_the_picker_needs_it() {
        let mut db = fontdb::Database::new();
        db.load_font_data(include_bytes!("../../../tests/fonts/families.ttc").to_vec());
        let family = |name: &str, monospace: bool, semibold: bool| Family {
            name: name.to_string(),
            monospace,
            semibold,
        };
        // Fixture Caps, which has no a, is left out.
        assert_eq!(
            families(&db),
            vec![
                // One face of it is not monospaced.
                family("Fixture Mixed", false, false),
                family("Fixture Mono", true, true),
                family("Fixture Sans", false, false),
                // Its axis spans the semibold weight.
                family("Fixture Variable", false, true),
            ]
        );
    }
}
