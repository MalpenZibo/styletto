//! The shell's motion: one spring for every fold and growth, so every
//! movement reads alike. Presets follow the current [`Motion`]: reduced
//! turns springs into fades, off makes changes instant.

use std::time::Duration;

use guido::prelude::*;

use crate::foundations::tokens::size;
use crate::ui::{Motion, with_ui};

pub const ISLAND_SPRING: SpringConfig = SpringConfig {
    mass: 1.0,
    stiffness: 230.0,
    damping: 21.0,
};

/// The quicker spring of a press or a pointer lean.
pub const TACTILE_SPRING: SpringConfig = SpringConfig {
    mass: 1.0,
    stiffness: 420.0,
    damping: 26.0,
};

pub const HOVER_MS: f32 = 150.0;
pub const COLOR_MS: f32 = 250.0;
/// How long presses are ignored after a view appears.
pub const ARM_MS: u64 = 450;

/// How far a growing shape's height trails its width.
pub const HEIGHT_LAG_MS: f32 = 45.0;

/// Instant, where nothing should move.
fn instant() -> Transition {
    Transition::new(1.0, TimingFunction::Linear)
}

/// A spring, or a fade's curve under reduced motion.
pub fn springing(config: SpringConfig) -> Transition {
    match with_ui(|u| u.motion) {
        Motion::Full => Transition::spring(config),
        Motion::Reduced => Transition::new(COLOR_MS, TimingFunction::EaseOut),
        Motion::None => instant(),
    }
}

/// An eased change, kept under reduced motion: it moves nothing.
pub fn easing(ms: f32) -> Transition {
    match with_ui(|u| u.motion) {
        Motion::Full | Motion::Reduced => Transition::new(ms, TimingFunction::EaseOut),
        Motion::None => instant(),
    }
}

pub fn spring() -> Transition {
    springing(ISLAND_SPRING)
}

pub fn tactile() -> Transition {
    springing(TACTILE_SPRING)
}

pub fn hover() -> Transition {
    easing(HOVER_MS)
}

pub fn color() -> Transition {
    easing(COLOR_MS)
}

/// How a value of a view coming and going moves: in with `enter`, out with
/// `leave`, and back with `enter` if the view returns while it leaves. For
/// any animated value (opacity, blur, translate, a length), as
/// `transition(fading(enter, leave))`.
pub fn fading<T>(
    enter: Transition,
    leave: Transition,
) -> impl Fn(Segment<T>) -> Transition + Clone + 'static {
    move |s| {
        if s.is_exit() {
            leave.clone()
        } else {
            enter.clone()
        }
    }
}

/// The view cross-fade: the new view fades in from a blur at a smaller
/// scale, the old one blurs out growing. Both scale about their centre so
/// neither runs past the island's bottom edge as it resizes.
pub fn arriving(content: impl Widget + 'static) -> Container {
    let content = match still(content) {
        Ok(still) => return still,
        Err(content) => content,
    };
    let leave = || Transition::new(160.0, TimingFunction::EaseOut);
    let fade = fading(
        Transition::new(300.0, TimingFunction::EaseOut).delay(70.0),
        leave(),
    );
    // The exit scales the outer box, the entry the inner one: each has its
    // own transition.
    let arrival = container()
        .scale(
            Scale::NONE
                .transition(spring())
                .entering_from(Scale::uniform(0.9)),
        )
        .child(content);
    container()
        .opacity(
            1.0.transition(fade.clone())
                .entering_from(0.0)
                .exiting_to(0.0),
        )
        .blur(0.0.transition(fade).entering_from(8.0).exiting_to(8.0))
        .scale(
            Scale::NONE
                .transition(leave())
                .exiting_to(Scale::uniform(1.06)),
        )
        .child(arrival)
}

/// A panel opening: it fades in from a blur, rising onto its place on the
/// island's spring, and leaves the same way, quickly. Unlike [`arriving`],
/// it does not scale.
pub fn entering(content: impl Widget + 'static) -> Container {
    let content = match still(content) {
        Ok(still) => return still,
        Err(content) => content,
    };
    let fade = fading(
        Transition::new(240.0, TimingFunction::EaseOut),
        Transition::new(160.0, TimingFunction::EaseOut),
    );
    let below = Translate::new(0.0, size::S2.px());
    container()
        .opacity(
            1.0.transition(fade.clone())
                .entering_from(0.0)
                .exiting_to(0.0),
        )
        .blur(0.0.transition(fade).entering_from(8.0).exiting_to(8.0))
        .translate(
            Translate::NONE
                .transition(spring())
                .entering_from(below)
                .exiting_to(below),
        )
        .child(content)
}

/// `content` as reduced or no motion shows a view coming and going: a fade,
/// or nothing. `Err` gives it back where motion is full.
fn still<W: Widget + 'static>(content: W) -> Result<Container, W> {
    match with_ui(|u| u.motion) {
        Motion::Full => Err(content),
        Motion::Reduced => Ok(container().child(content).opacity(
            1.0.transition(easing(COLOR_MS))
                .entering_from(0.0)
                .exiting_to(0.0),
        )),
        Motion::None => Ok(container().child(content)),
    }
}

/// A horizontal shake for a refused press or a wrong password: play
/// `shake().played_by(count)` on a translate, bumping `count`.
pub fn shake() -> Keyframes<Translate> {
    if with_ui(|u| u.motion) != Motion::Full {
        return Keyframes::new(420.0)
            .at(0.0, Translate::NONE)
            .at(1.0, Translate::NONE);
    }
    Keyframes::new(420.0)
        .at(0.0, Translate::NONE)
        .at(0.15, Translate::new(-9.0, 0.0))
        .at(0.35, Translate::new(8.0, 0.0))
        .at(0.55, Translate::new(-6.0, 0.0))
        .at(0.75, Translate::new(3.0, 0.0))
        .at(1.0, Translate::NONE)
}

/// False for [`ARM_MS`] after the calling view is built, so a click aimed
/// at the previous view lands on nothing.
pub fn armed() -> Signal<bool> {
    let armed = create_signal(false);
    set_timeout(Duration::from_millis(ARM_MS), move || armed.set(true));
    armed.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fading_moves_any_value() {
        // A function, not a closure: each property takes its own `T`.
        fn fade<T>() -> impl Fn(Segment<T>) -> Transition + Clone + 'static {
            fading(easing(COLOR_MS), easing(HOVER_MS))
        }
        container()
            .opacity(1.0.transition(fade()).entering_from(0.0))
            .translate(
                Translate::NONE
                    .transition(fade())
                    .exiting_to(Translate::new(0.0, size::S2.px())),
            );
    }
}
