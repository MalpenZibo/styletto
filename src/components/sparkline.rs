//! A line through recent samples, newest at the right edge.

use guido::prelude::*;
use guido::widget_prelude::*;

use crate::foundations::ease::ColorEase;
use crate::foundations::tokens::{Space, size, stroke};
use crate::ui::{Look, look};

pub struct Sparkline {
    samples: Signal<Vec<f32>>,
    /// Width where the parent sets none.
    fallback_width: f32,
    slots: usize,
    top: Option<f32>,
    height: f32,
    /// Its colour is the accent's, read as it is laid out.
    look: Look,
    /// The colour it is drawn in, easing into a new accent: it draws its
    /// line itself, where Guido moves only a property it declares.
    ink: ColorEase,
    stroke: f32,
}

impl Sparkline {
    /// Scale to `top` instead of the highest sample.
    pub fn top(mut self, top: f32) -> Self {
        self.top = Some(top);
        self
    }

    pub fn height(mut self, height: Space) -> Self {
        self.height = height.px();
        self
    }
}

impl Widget for Sparkline {
    fn advance_animations(&mut self, tree: &mut Tree, id: WidgetId) -> bool {
        if self.ink.advance(tree.frame_instant(), id) {
            tree.mark_needs_paint(id);
        }
        self.ink.moving()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // Read while laying out, so a new accent lays it out again and
        // sets off the change.
        let accent = self.look.color(|c| c.accent.fill)();
        self.ink.retarget(ctx, accent);
        let width = if constraints.max_width.is_finite() {
            constraints.max_width
        } else {
            self.fallback_width
        };
        constraints.constrain(Size::new(width, self.height))
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let size = ctx.tree().cached_size(ctx.id()).unwrap_or_default();
        let line = self.samples.with(|samples| {
            let top = self
                .top
                .unwrap_or_else(|| samples.iter().copied().fold(0.0, f32::max))
                .max(f32::EPSILON);
            let step = size.width / (self.slots.max(2) - 1) as f32;
            let start = self.slots.saturating_sub(samples.len()) as f32 * step;
            let inset = self.stroke / 2.0;
            samples
                .iter()
                .enumerate()
                .fold(Path::new(), |path, (i, value)| {
                    let x = start + i as f32 * step;
                    let y =
                        inset + (size.height - self.stroke) * (1.0 - (value / top).clamp(0.0, 1.0));
                    if i == 0 {
                        path.move_to((x, y))
                    } else {
                        path.line_to((x, y))
                    }
                })
        });
        let stroke = Stroke::new(self.stroke, self.ink.color())
            .joins(Join::Round)
            .caps(Cap::Round);
        ctx.draw_path(&line, None, stroke);
    }
}

/// The last `slots` of `samples` as a full-width line, scaled to their
/// highest unless [`top`](Sparkline::top) is set.
pub fn sparkline(samples: Signal<Vec<f32>>, slots: usize) -> Sparkline {
    Sparkline {
        samples,
        fallback_width: size::S20.px(),
        slots,
        top: None,
        height: size::S8.px(),
        look: look(),
        ink: ColorEase::new(),
        stroke: stroke::RING,
    }
}
