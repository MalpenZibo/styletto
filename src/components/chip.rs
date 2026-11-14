//! Pills that show a state: a count, a reading, the player chosen.

use guido::prelude::*;

use crate::components::button::hovered;
use crate::foundations::control::Size;
use crate::foundations::motion;
use crate::foundations::tokens::glow as glow_spread;
use crate::foundations::tokens::radius;
use crate::primitives::layout::centered_row;
use crate::ui::ui;

/// A pill holding its children in a centred row.
#[component]
pub fn chip(
    #[prop(default = Size::Xxs)] size: Size,
    /// Defaults to the surface.
    fill: Option<Color>,
    /// A halo around it.
    glow: Option<Color>,
    /// Hover and press feedback.
    #[prop(callback)]
    on_click: (),
    #[prop(children)] children: (),
) -> impl Widget {
    let t = ui();
    let size = size.get_untracked();
    let height = size.height();
    let paint = move || fill.get().unwrap_or(t.colors.surface);
    let base = container()
        .height(height)
        .padding([0.0, size.inline()])
        .corners(radius::pill(height))
        .background(paint.transition(motion::color()))
        .shadow(
            (move || {
                Shadow::new(
                    (0.0, 0.0),
                    glow_spread::SMALL,
                    0.0,
                    glow.get().unwrap_or(Color::TRANSPARENT),
                )
            })
            .transition(motion::color()),
        )
        .layout(centered_row(size.gap()))
        .children(children);
    if on_click.is_none() {
        return base;
    }
    base.when_hovered(move |s| s.background(move || hovered(paint())))
        .when_pressed(move |s| s.ripple_with_color(t.colors.ripple))
        .on_click(on_click)
}
