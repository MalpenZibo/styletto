//! A tree mounted headless for the tests, with what it asked its owner.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use guido::prelude::*;
use guido::testing::Headless;

/// What a tree asks its owner, in order.
pub type Asked<T> = Rc<RefCell<Vec<T>>>;

/// Long enough for a colour to ease into a new palette, and a spring to
/// come to rest.
const SECOND: Duration = Duration::from_secs(1);

/// One headless app, one GPU context, in which pairs of surfaces are
/// compared one after another, on one clock that only moves forward.
pub struct Bench {
    h: Headless,
    at: Instant,
}

impl Bench {
    /// None without a GPU, or a panic where `GUIDO_GPU_REQUIRED` is set
    /// (`Headless::new_or_skip`).
    pub fn new() -> Option<Self> {
        Some(Self {
            h: Headless::new_or_skip()?,
            at: Instant::now(),
        })
    }

    /// Two surfaces `size` drawing what `build` makes of a value: one given
    /// `from` and then `to`, one given `to` from the start. Both are built
    /// in the same first frame and stepped at the same moments after it, so
    /// what moves on its own, a spring or a timer a story sets, moves alike
    /// in both: a timer is armed by the frame that builds it, and ticks on
    /// the simulated clock. They close when the pair is dropped.
    pub fn pair<T: Clone + PartialEq + Send + 'static>(
        &mut self,
        (width, height): (u32, u32),
        from: T,
        to: T,
        build: impl Fn(RwSignal<T>) -> Container + Clone + 'static,
    ) -> Pair<'_, T> {
        let handles = Rc::new(RefCell::new(Vec::new()));
        let mut mount = |value: RwSignal<T>| {
            let (build, handles) = (build.clone(), handles.clone());
            let id = self.h.surface(SurfaceConfig::new(), move || {
                handles.borrow_mut().push(expect_context::<SurfaceHandle>());
                build(value)
            });
            self.h.configure(id, width, height, 1.0);
            id
        };
        let given = create_signal(from);
        let changed = mount(given);
        let built = mount(create_signal(to.clone()));
        self.at += FRAME;
        self.h.step_at(self.at);
        Pair {
            bench: self,
            given,
            to,
            surfaces: [changed, built],
            handles,
            width,
        }
    }
}

/// A frame of the bench's clock, a vsync's.
const FRAME: Duration = Duration::from_millis(16);

/// Two surfaces of a [`Bench`], to compare.
pub struct Pair<'a, T: 'static> {
    bench: &'a mut Bench,
    given: RwSignal<T>,
    to: T,
    /// The one given `from` then `to`, and the one built with `to`.
    surfaces: [SurfaceId; 2],
    handles: Rc<RefCell<Vec<SurfaceHandle>>>,
    width: u32,
}

impl<T: Clone + PartialEq + Send + 'static> Pair<'_, T> {
    /// Gives the first surface `to`, and lets both settle.
    pub fn change(&mut self) {
        self.given.set(self.to.clone());
        let bench = &mut *self.bench;
        bench.at = crate::shot::settle(&mut bench.h, bench.at, SECOND);
    }

    /// Where the two frames differ (see [`unlike`]); none where they draw
    /// the same.
    pub fn unlike(&self) -> Option<String> {
        let [a, b] = self.surfaces.map(|id| self.bench.h.read_frame(id));
        unlike(&a, &b, self.width as usize)
    }
}

impl<T: 'static> Drop for Pair<'_, T> {
    /// Closes both surfaces, so the next pair is drawn alone.
    fn drop(&mut self) {
        for handle in self.handles.borrow_mut().drain(..) {
            handle.close();
        }
        self.bench.at += FRAME;
        self.bench.h.step_at(self.bench.at);
    }
}

/// A picture to draw behind what two frames compare, in place of the
/// blurred shapes: the software renderer CI draws with gives a blur drawn
/// while every test starts at once a few tiles of its own.
pub fn still_backdrop() -> Memo<Option<std::path::PathBuf>> {
    let logo = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/logo.svg");
    create_memo(move || Some(logo.into()))
}

/// How far a channel may differ between two frames that draw the same: a
/// blur drawn on two surfaces at once comes out a few levels apart on the
/// software renderer CI draws with. A colour of another palette is tens of
/// levels away.
const NOISE: u8 = 16;

/// Where two frames of the same surface size, `width` pixels wide, differ
/// by more than [`NOISE`] in a channel: how many pixels, and the first and
/// the last with what each frame has there. None where they draw the same.
fn unlike(a: &[u8], b: &[u8], width: usize) -> Option<String> {
    let far = |i: usize| (0..4).any(|c| a[i * 4 + c].abs_diff(b[i * 4 + c]) > NOISE);
    let differ: Vec<usize> = (0..a.len().min(b.len()) / 4).filter(|&i| far(i)).collect();
    let (&first, &last) = (differ.first()?, differ.last()?);
    let at = |i: usize| (i % width, i / width);
    Some(format!(
        "{} pixels, the first at {:?}: {:?} and {:?}; the last at {:?}",
        differ.len(),
        at(first),
        &a[first * 4..first * 4 + 4],
        &b[first * 4..first * 4 + 4],
        at(last),
    ))
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
