//! A colour easing into each new value where Guido gives what paints it no
//! motion: a line a widget draws itself, a gradient. Guido moves a property
//! it declares, frame by frame; here the widget that holds the colour moves
//! it, asking Guido for each frame through a trigger it tracks.

use guido::prelude::*;
use guido::widget_prelude::*;
use guido::widgets::container::AnimationState;

use crate::foundations::motion;

/// A colour on its way to the last value it was given, at
/// [`motion::color`]'s pace and curve: Guido's own state for a property,
/// held by a widget that paints it.
pub(crate) struct ColorEase {
    state: AnimationState<Color>,
    /// False under [`Motion::None`]: a new value is taken at once.
    moves: bool,
    /// Tracked for the widget's animation job and notified to ask for the
    /// next frame.
    frame: Trigger,
}

impl ColorEase {
    /// At the pace of the nearest [`Ui`](crate::Ui)'s motion, as it is now.
    pub(crate) fn new() -> Self {
        Self {
            state: AnimationState::new(Color::TRANSPARENT, motion::color()),
            moves: motion::colors_ease(),
            frame: create_trigger(),
        }
    }

    /// The colour as the last frame left it.
    pub(crate) fn color(&self) -> Color {
        self.state.displayed()
    }

    /// Heads for `target`, as the widget laid out in `ctx` reads it, and asks
    /// the frame that moves it. The first value is taken at once. A measure
    /// pass leaves it alone, as Guido's own properties do: the layout after
    /// it is where a change starts, and a colour sizes nothing.
    pub(crate) fn retarget(&mut self, ctx: &mut LayoutCtx, target: Color) {
        if ctx.measuring() {
            return;
        }
        if self.state.is_initial() || !self.moves {
            self.state.set_immediate(ctx, target);
            return;
        }
        self.state
            .animate_to(target, ctx.tree_ref().frame_instant());
        if self.state.is_animating() {
            self.ask_frame(ctx.id());
        }
    }

    /// Moves it on to `now`, and asks the next frame for widget `id` while it
    /// is still on its way; true if it moved.
    pub(crate) fn advance(&mut self, now: FrameInstant, id: WidgetId) -> bool {
        let moved = self.state.advance(now).is_changed();
        if self.moving() {
            self.ask_frame(id);
        }
        moved
    }

    /// Whether it is still on its way.
    pub(crate) fn moving(&self) -> bool {
        self.state.is_animating()
    }

    /// Queues the widget's animation job: Guido runs a widget's job when
    /// something it tracked for that job changes, and a widget outside Guido
    /// has no other way to ask.
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
    /// Out of flow: laid out, but its parent's layout never sees it, so it
    /// adds no gap and moves nothing.
    fn layout_hints(&self) -> LayoutHints {
        LayoutHints {
            out_of_flow: true,
            ..LayoutHints::default()
        }
    }

    fn advance_animations(&mut self, tree: &mut Tree, id: WidgetId) -> bool {
        if self.ease.advance(tree.frame_instant(), id) {
            self.shown.set(self.ease.color());
        }
        self.ease.moving()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // Read while laying out, so a new target lays it out again.
        let target = (self.target)();
        self.ease.retarget(ctx, target);
        self.shown.set(self.ease.color());
        constraints.constrain(Size::new(0.0, 0.0))
    }

    fn paint(&self, _ctx: &mut PaintContext) {}
}
