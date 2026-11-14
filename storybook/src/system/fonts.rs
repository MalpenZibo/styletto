//! The font families installed on the system, for the Theme page to offer,
//! as the font database the text renderer loads sees them: a name, whether
//! it is monospaced, the weights it has, and only families that write Latin
//! text, so a symbol or an emoji font is left out.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Family {
    pub name: String,
    /// Every face of it is monospaced.
    pub monospace: bool,
    /// The weights it draws, as ranges from the lightest: one weight for
    /// a static face, the span of its weight axis for a variable one.
    pub weights: Vec<(u16, u16)>,
}

impl Family {
    /// Whether it draws `weight`, or one within a step of it.
    pub fn has_weight(&self, weight: u16) -> bool {
        self.weights
            .iter()
            .any(|&(low, high)| weight + 50 >= low && weight <= high + 50)
    }
}

/// Every family that writes Latin text, by name.
pub fn families() -> Vec<Family> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let mut found: BTreeMap<String, Family> = BTreeMap::new();
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
        let span = axis.unwrap_or((face.weight.0, face.weight.0));
        let family = found.entry(name.clone()).or_insert_with(|| Family {
            name: name.clone(),
            monospace: true,
            weights: Vec::new(),
        });
        family.monospace &= face.monospaced;
        if !family.weights.contains(&span) {
            family.weights.push(span);
        }
    }
    found
        .into_values()
        .map(|mut family| {
            family.weights.sort_unstable();
            family
        })
        .collect()
}
