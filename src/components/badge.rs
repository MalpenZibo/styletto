//! Marks that something wants attention.

use guido::prelude::*;

use crate::components::chip::chip;
use crate::foundations::control::Size;
use crate::foundations::icons::{self, Icon};
use crate::foundations::tokens::{icon, radius, size, stroke};

#[component]
pub fn dot(
    #[prop(required)] color: Color,
    /// A ring around it.
    ring: Option<Color>,
) -> impl Widget {
    container()
        .width(size::S2.px())
        .height(size::S2.px())
        .corners(radius::pill(size::S2.px()))
        .background(color)
        .shadow(move || {
            Shadow::new(
                (0.0, 0.0),
                0.0,
                stroke::BOLD,
                ring.get().unwrap_or(Color::TRANSPARENT),
            )
        })
}

/// A tinted, glowing capsule holding the icons of what is in use.
#[component]
pub fn capsule(#[prop(required)] color: Color, #[prop(children)] children: ()) -> impl Widget {
    chip()
        .size(Size::Xxs)
        .fill(move || color.get().with_alpha(0.2))
        .glow(move || color.get().with_alpha(0.15))
        .children(children)
}

/// An icon on a tinted squircle, heading an alert. `color` draws the icon
/// and tints the squircle: a hue as text, such as `danger_text`, so the
/// icon reads.
#[component]
pub fn icon_tile(#[prop(required)] icon: Icon, #[prop(required)] color: Color) -> impl Widget {
    container()
        .width(size::S10.px())
        .height(size::S10.px())
        .corners(radius::SMALL.corners())
        .background(move || color.get().with_alpha(0.22))
        .layout(Flex::column().center())
        .child(icons::icon(icon, icon::MEDIUM.px(), color))
}
