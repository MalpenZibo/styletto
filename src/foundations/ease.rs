//! A colour easing into each new value where Guido gives what paints it no
//! motion: a line a widget draws itself, a gradient. Guido moves a property
//! it declares, frame by frame; here the widget that holds the colour moves
//! it, asking Guido for each frame through a trigger it tracks.

use guido::prelude::*;
use guido::widget_prelude::*;

use crate::foundations::motion::COLOR_MS;
use crate::ui::{Motion, with_ui};

/// Where a colour stands on its way to the last value it was given, at
/// [`motion::color`](crate::foundations::motion::color)'s pace and curve.
pub(crate) struct ColorEase {
    from: Color,
    /// Where it goes; none until it is first given a value, which it takes
    /// at once.
    to: Option<Color>,
    since: Option<FrameInstant>,
    /// How long a change takes; none under [`Motion::None`].
    ms: Option<f32>,
    /// Tracked for the widget's animation job and notified to ask for the
    /// next frame.
    frame: Trigger,
}

impl ColorEase {
    /// At the pace of the nearest [`Ui`](crate::Ui)'s motion, as it is now.
    pub(crate) fn new() -> Self {
        Self {
            from: Color::TRANSPARENT,
            to: None,
            since: None,
            ms: (with_ui(|u| u.motion) != Motion::None).then_some(COLOR_MS),
            frame: create_trigger(),
        }
    }

    /// The colour at `now`.
    pub(crate) fn at(&self, now: FrameInstant) -> Color {
        let Some(to) = self.to else {
            return self.from;
        };
        match (self.ms, self.since) {
            (Some(ms), Some(since)) => {
                let t = (now.duration_since(since).as_secs_f32() * 1000.0 / ms).min(1.0);
                self.from.mix(to, TimingFunction::EaseOut.evaluate(t))
            }
            _ => to,
        }
    }

    /// Heads for `target` from where it is at `now`, and asks the frame
    /// that moves it, for widget `id`. The first value is taken at once.
    pub(crate) fn retarget(&mut self, target: Color, now: FrameInstant, id: WidgetId) {
        if self.to == Some(target) {
            return;
        }
        if self.to.is_some() && self.ms.is_some() {
            self.from = self.at(now);
            self.since = Some(now);
            self.ask_frame(id);
        } else {
            self.from = target;
            self.since = None;
        }
        self.to = Some(target);
    }

    /// Settles a change that has played out by `now`, or asks the next
    /// frame for widget `id`; true while it moves.
    pub(crate) fn advance(&mut self, now: FrameInstant, id: WidgetId) -> bool {
        let moving = match (self.to, self.ms, self.since) {
            (Some(_), Some(ms), Some(since)) => {
                now.duration_since(since).as_secs_f32() * 1000.0 < ms
            }
            _ => false,
        };
        if moving {
            self.ask_frame(id);
        } else if let Some(to) = self.to {
            self.from = to;
            self.since = None;
        }
        moving
    }

    /// Queues the widget's animation job: Guido runs a widget's job when
    /// something it tracked for that job changes.
    fn ask_frame(&self, id: WidgetId) {
        with_signal_tracking(id, JobType::Animation, || self.frame.track());
        self.frame.notify();
    }
}

/// A colour easing into each new value `target` gives, as a signal, for a
/// property Guido gives no motion. The signal is moved by `easer`, a widget
/// of no size the caller adds beside what reads it: laid out before
/// anything is painted, it holds `target` by the first frame.
pub(crate) fn eased_signal(target: impl Fn() -> Color + 'static) -> (Signal<Color>, Easer) {
    let shown = create_signal(Color::TRANSPARENT);
    let easer = Easer {
        target: Box::new(target),
        ease: ColorEase::new(),
        shown,
    };
    (shown.read_only(), easer)
}

/// Moves the colour [`eased_signal`] hands back.
pub(crate) struct Easer {
    target: Box<dyn Fn() -> Color>,
    ease: ColorEase,
    shown: RwSignal<Color>,
}

impl Widget for Easer {
    fn advance_animations(&mut self, tree: &mut Tree, id: WidgetId) -> bool {
        let now = tree.frame_instant();
        let moving = self.ease.advance(now, id);
        self.shown.set(self.ease.at(now));
        moving
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // Read while laying out, so a new target lays it out again.
        let target = (self.target)();
        let (now, id) = (ctx.tree_ref().frame_instant(), ctx.id());
        self.ease.retarget(target, now, id);
        self.shown.set(self.ease.at(now));
        constraints.constrain(Size::new(0.0, 0.0))
    }

    fn paint(&self, _ctx: &mut PaintContext) {}
}
