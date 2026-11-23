//! A tree mounted headless for the tests, with what it asked its owner.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use guido::prelude::*;
use guido::testing::Headless;

/// What a tree asks its owner, in order.
pub type Asked<T> = Rc<RefCell<Vec<T>>>;

/// Steps every surface of `h` a frame every 32 ms for a second after `at`,
/// long enough for a colour to ease into a new palette, and moves `at` to
/// the last frame. Surfaces stepped together this way move alike.
pub fn settle(h: &mut Headless, at: &mut Instant) {
    for _ in 0..32 {
        *at += Duration::from_millis(32);
        h.step_at(*at);
    }
}

/// A tree mounted headless, 200 by 100, with what it asked.
pub struct Mounted<T> {
    h: Headless,
    id: SurfaceId,
    pub asked: Asked<T>,
}

impl<T: 'static> Mounted<T> {
    /// The tree `build` makes from the log of what is asked, and the
    /// signals it hands back for the test to drive; none without a GPU, or a
    /// panic where `GUIDO_GPU_REQUIRED` is set (`Headless::new_or_skip`).
    pub fn new<P: 'static>(
        build: impl Fn(Asked<T>) -> (Container, P) + 'static,
    ) -> Option<(Self, P)> {
        let mut h = Headless::new_or_skip()?;
        let asked = Asked::default();
        let probe = Rc::new(Cell::new(None));
        let (log, seen) = (asked.clone(), probe.clone());
        let id = h.surface(SurfaceConfig::new(), move || {
            let (tree, handed) = build(log.clone());
            seen.set(Some(handed));
            tree
        });
        h.configure(id, 200, 100, 1.0);
        h.step();
        // Only a missing GPU skips a test; a tree that was not built fails.
        let handed = probe.take().expect("the tree was built");
        Some((Self { h, id, asked }, handed))
    }

    /// Sends `event` at `at`, steps there, and gives what it asked.
    pub fn send_at(&mut self, event: Event, at: Instant) -> Vec<T> {
        self.h.event_at(self.id, event, at);
        self.h.step_at(at);
        self.asked.borrow_mut().drain(..).collect()
    }

    /// Sends `event` now, steps, and gives what it asked.
    pub fn send(&mut self, event: Event) -> Vec<T> {
        self.send_at(event, Instant::now())
    }

    /// A wheel notch, up or down.
    pub fn notch(&mut self, up: bool) -> Vec<T> {
        let dy = if up { -1.0 } else { 1.0 };
        self.send(Event::scroll(100.0, 8.0, 0.0, dy, ScrollSource::Wheel))
    }

    /// A press and a release at `x`, giving what the press asked.
    pub fn press(&mut self, x: f32) -> Vec<T> {
        let asked = self.send(Event::mouse_down(x, 8.0, MouseButton::Left));
        self.send(Event::mouse_up(x, 8.0, MouseButton::Left));
        asked
    }

    /// A press and a release at a point, in one frame, giving what they
    /// asked: a click is answered on the release.
    pub fn click(&mut self, x: f32, y: f32) -> Vec<T> {
        self.h.click(self.id, x, y);
        self.h.step();
        self.asked.borrow_mut().drain(..).collect()
    }

    pub fn step(&mut self) {
        self.h.step();
    }

    pub fn step_at(&mut self, at: Instant) {
        self.h.step_at(at);
    }

    /// The colour drawn at a point of the last frame.
    pub fn pixel(&self, x: f32, y: f32) -> [u8; 4] {
        self.h.read_pixel(self.id, x as u32, y as u32)
    }
}
