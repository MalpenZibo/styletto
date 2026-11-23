//! Figures and cards coloured by their [`Level`].

use guido::prelude::*;

use crate::components::level::{Level, level_ink};
use crate::foundations::icons::{self, Icon};
use crate::foundations::motion;
use crate::foundations::tokens::{glow, icon as icon_box, radius, size, tint};
use crate::foundations::typography::{DISPLAY_XSMALL, LABEL_SMALL, Typeset};
use crate::primitives::layout::centered_row;
use crate::primitives::text::figure_text;
use crate::theme::palette::Palette;
use crate::ui::{Look, look};

/// The hue of a level other than normal, as a card's tint and glow take
/// it: the fill, not the text. None while normal.
fn level_fill(c: &Palette, level: Level) -> Option<Color> {
    level.hue().map(|hue| c.hue(hue).fill)
}

/// Quiet while normal, else the level's ink, as the palette is now.
fn quiet_ink(t: Look, level: Level) -> Color {
    match level {
        Level::Normal => t.colors().sub,
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
    let t = look();
    let quiet = move || quiet_ink(t, level.get());
    container()
        .layout(centered_row(size::S1.px()))
        .child(icons::icon(icon, icon_box::XSMALL.px(), quiet))
        .child(figure_text(figure, LABEL_SMALL, quiet))
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
    let t = look();
    let quiet = move || quiet_ink(t, level.get());
    container()
        .width(fill())
        .padding(size::S3.px())
        .corners(radius::MEDIUM.corners())
        .background(t.eased(move |c| {
            let level = level.get();
            let alpha = match level {
                Level::Alert => tint::MEDIUM,
                _ => tint::SUBTLE,
            };
            level_fill(c, level).map_or(c.surface, |fill| fill.with_alpha(alpha))
        }))
        .shadow(motion::eased_color(move || {
            glow::halo(level_fill(&t.colors(), level.get()), glow::MEDIUM)
        }))
        .layout(Flex::column().spacing(size::S1.px()))
        .child(
            container()
                .layout(centered_row(size::S1.px()))
                .child(icons::icon(icon, icon_box::XSMALL.px(), quiet))
                .child(
                    text(label)
                        .typeset(LABEL_SMALL)
                        .color(motion::eased_color(quiet))
                        .nowrap(),
                ),
        )
        .child(
            text(value)
                .typeset(DISPLAY_XSMALL.em().tabular())
                .color(t.eased(move |c| c.ink(level.get().hue())))
                .nowrap(),
        )
        .children(children)
}
