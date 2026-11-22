//! The font families installed, for [`font_picker`](super::font_picker) to
//! offer: Guido's own list of the fonts it draws with, the system's and those
//! the app loaded. Only families that write Latin text are kept, so a symbol
//! or an emoji font is left out.

use std::ops::RangeInclusive;

use guido::prelude::*;

use super::Family;
use crate::foundations::typography::LABEL_MEDIUM;

/// The families installed, by name whatever its case, empty until Guido has
/// read them.
///
/// Guido reads its fonts once, on a thread, and every call shares what it
/// found; the signal is the caller's, a memo over Guido's list, which the
/// app owns. A family is kept if it draws `a` and `A`, is monospaced if every
/// face of it is, and semibold if it draws the weight of the emphasised type
/// (`.em()`), or one within a step of it.
pub fn installed_families() -> Signal<Vec<Family>> {
    let families = font_families();
    let semibold = LABEL_MEDIUM.em().resolve().weight;
    create_memo(move || {
        families.with(|families| {
            families
                .iter()
                .filter(|family| family.covers('a') && family.covers('A'))
                .map(|family| Family {
                    name: family.name().to_string(),
                    monospace: family.monospaced(),
                    semibold: draws(family.weights(), semibold),
                })
                .collect()
        })
    })
    .into_signal()
}

/// Whether one of `weights` reaches `weight`, or one within a step of it: a
/// static face is one weight, a variable one the span of its `wght` axis.
fn draws(weights: &[RangeInclusive<FontWeight>], weight: FontWeight) -> bool {
    // Saturating: a broken face may say its axis ends at the top of `u16`.
    weights.iter().any(|w| {
        weight.0.saturating_add(50) >= w.start().0 && weight.0 <= w.end().0.saturating_add(50)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_family_is_semibold_within_a_step_of_the_weight() {
        let semibold = FontWeight::SEMI_BOLD;
        let one = |w: u16| FontWeight(w)..=FontWeight(w);
        assert!(draws(&[one(400), one(600)], semibold));
        // 50 either side is near enough...
        assert!(draws(&[one(550)], semibold));
        assert!(draws(&[one(650)], semibold));
        // ...more than that is not.
        assert!(!draws(&[one(400), one(700)], semibold));
        assert!(!draws(&[], semibold));
        // A variable face anywhere along its axis.
        assert!(draws(&[FontWeight(100)..=FontWeight(900)], semibold));
        assert!(draws(&[FontWeight(300)..=FontWeight(550)], semibold));
        assert!(!draws(&[FontWeight(300)..=FontWeight(500)], semibold));
        assert!(!draws(
            &[FontWeight(u16::MAX)..=FontWeight(u16::MAX)],
            semibold
        ));
    }
}
