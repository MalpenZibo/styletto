//! Marks that something wants attention.

use guido::prelude::*;

use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::{icon, indicator, radius, size, stroke, tint};

/// A small round mark, as of something unread. It grows with the density,
/// but never below a page indicator's dot: shrunk with the space in compact
/// (4 px), it would all but vanish.
#[component]
pub fn dot(
    #[prop(required)] color: Color,
    /// A ring around it.
    ring: Option<Color>,
) -> impl Widget {
    let side = size::S2.px().max(indicator::DOT);
    container()
        .width(side)
        .height(side)
        .corners(radius::pill(side))
        .background(motion::eased_color(color))
        .shadow(
            (move || {
                Shadow::new(
                    (0.0, 0.0),
                    0.0,
                    stroke::BOLD,
                    ring.get().unwrap_or(Color::TRANSPARENT),
                )
            })
            .transition(motion::color()),
        )
}

/// An icon on a tinted squircle, heading an alert. `color` draws the icon
/// and tints the squircle: a hue as text, such as `colors.danger.text`, so
/// the icon reads.
#[component]
pub fn icon_tile(#[prop(required)] icon: Icon, #[prop(required)] color: Color) -> impl Widget {
    container()
        .width(size::S10.px())
        .height(size::S10.px())
        .corners(radius::SMALL.corners())
        .background(motion::eased_color(move || {
            color.get().with_alpha(tint::STRONG)
        }))
        .layout(Flex::column().center())
        .child(icons::icon(icon, icon::MEDIUM.px(), color))
}
