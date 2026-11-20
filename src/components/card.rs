//! Figures and cards coloured by their [`Level`].

use guido::prelude::*;

use crate::components::level::{Level, level_ink};
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::{glow, icon as icon_box, radius, size, tint};
use crate::foundations::typography::{DISPLAY_XSMALL, LABEL_SMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::ui::ui;

/// The hue of a level other than normal, as a card's tint and glow take
/// it: the fill, not the text.
fn level_fill(level: Level) -> Color {
    let t = ui();
    match level {
        Level::Normal => t.colors.surface,
        Level::Good => t.colors.success,
        Level::Warn => t.colors.warning,
        Level::Alert => t.colors.danger,
    }
}

/// Quiet while normal, else the level's ink.
fn quiet_ink(level: Level) -> Color {
    match level {
        Level::Normal => ui().colors.sub,
        hot => level_ink(hot),
    }
}

/// A figure with its icon: quiet while normal, else in its level's colour,
/// as the island shows what runs hot.
#[component]
pub fn level_figure(
    #[prop(required)] icon: Icon,
    #[prop(required)] figure: String,
    #[prop(required)] level: Level,
) -> impl Widget {
    let quiet = move || quiet_ink(level.get());
    container()
        .layout(centered_row(size::S1.px()))
        .child(icons::icon(icon, icon_box::XSMALL.px(), quiet))
        .child(
            text(figure)
                .typeset(LABEL_SMALL.em().tabular())
                .text_box(TextEdge::Cap, TextEdge::Alphabetic)
                .color(quiet.transition(motion::color()))
                .nowrap(),
        )
}

/// A card that takes its level's colour, fill, glow and label while not
/// normal, as the island's pills do.
#[component]
pub fn stat_card(
    #[prop(required)] icon: Icon,
    #[prop(required)] label: String,
    #[prop(required)] value: String,
    level: Level,
    #[prop(children)] children: (),
) -> impl Widget {
    let t = ui();
    let quiet = move || quiet_ink(level.get());
    container()
        .width(fill())
        .padding(size::S3.px())
        .corners(radius::MEDIUM.corners())
        .background(
            (move || match level.get() {
                Level::Normal => t.colors.surface,
                Level::Alert => level_fill(Level::Alert).with_alpha(tint::MEDIUM),
                calmer => level_fill(calmer).with_alpha(tint::SUBTLE),
            })
            .transition(motion::color()),
        )
        .shadow(
            (move || {
                let halo = match level.get() {
                    Level::Normal => Color::TRANSPARENT,
                    hot => level_fill(hot).with_alpha(tint::MEDIUM),
                };
                Shadow::new((0.0, 0.0), glow::MEDIUM, 0.0, halo)
            })
            .transition(motion::color()),
        )
        .layout(Flex::column().spacing(size::S1.px()))
        .child(
            container()
                .layout(centered_row(size::S1.px()))
                .child(icons::icon(icon, icon_box::XSMALL.px(), quiet))
                .child(
                    text(label)
                        .typeset(LABEL_SMALL)
                        .color(quiet.transition(motion::color()))
                        .nowrap(),
                ),
        )
        .child(
            text(value)
                .typeset(DISPLAY_XSMALL.em().tabular())
                .color((move || level_ink(level.get())).transition(motion::color()))
                .nowrap(),
        )
        .children(children)
}
