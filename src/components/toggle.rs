//! An on/off switch whose knob springs across the track.

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{opacity, size};
use crate::theme::palette::{DIM_CONTRAST, Palette, contrast, over};
use crate::ui::{flip, look};

fn height() -> f32 {
    Size::Xxs.height()
}
fn width() -> f32 {
    height() + size::S4.px()
}
fn inset() -> f32 {
    size::S0_5.px()
}
fn knob() -> f32 {
    height() - 2.0 * inset()
}

/// The knob while off: `sub` where it keeps the 3:1 a control asks on the
/// track in a hovered or chosen row, which tints it again, with what shows
/// through worst; `text` where it doesn't, as on a see-through island,
/// where `sub` is held only to 3:1 on the bare track. `dim` never would.
fn knob_off(colors: &Palette) -> Color {
    let [_, _, track] = colors.grounds(colors.worst_backdrop());
    let in_row = over(colors.surface2, track);
    if contrast(colors.sub, track).min(contrast(colors.sub, in_row)) >= DIM_CONTRAST {
        colors.sub
    } else {
        colors.text
    }
}

/// On while `active`, which follows what the owner sets; a press calls
/// `on_toggle` with the opposite.
#[component]
pub fn toggle(
    #[prop(required)] active: bool,
    #[prop(default = true)] enabled: bool,
    /// Called with the state a press asks for.
    #[prop(callback)]
    on_toggle: fn(bool),
) -> impl Widget {
    let t = look();
    container()
        .width(width())
        .height(height())
        .corners(radius::pill(height()))
        .background(t.eased(move |c| {
            if active.get() {
                c.accent.fill
            } else {
                c.surface2
            }
        }))
        .opacity(1.0.transition(motion::hover()))
        .when_disabled(|s| s.opacity(opacity::DISABLED))
        .enabled(enabled)
        .on_click(flip(active, on_toggle))
        .child(
            container()
                .width(knob())
                .height(knob())
                .corners(radius::pill(knob()))
                // As Material 3's: what reads on the accent while on, and on
                // the track while off.
                .background(t.eased(move |c| {
                    if active.get() {
                        c.accent.on
                    } else {
                        knob_off(c)
                    }
                }))
                .translate(
                    (move || {
                        let x = if active.get() {
                            width() - knob() - inset()
                        } else {
                            inset()
                        };
                        Translate::new(x, inset())
                    })
                    .transition(motion::spring()),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::knob_off;
    use crate::theme::palette::{DIM_CONTRAST, contrast, over};
    use crate::theme::presets::THEMES;
    use crate::ui::{Material, Ui};

    #[test]
    fn the_knob_reads_on_the_accent_and_on_the_track() {
        // WCAG's 3:1 for a control, on and off, with every accent and on
        // every material, with what shows through worst: the off track as
        // it is drawn on the island, and in a chosen row, which tints it
        // again with `surface2`.
        for theme in THEMES {
            for flavor in theme.flavors {
                for (index, accent) in flavor.accents.iter().enumerate() {
                    let palette = flavor.scheme(index).palette();
                    for material in Material::ALL {
                        let ui = Ui::new(palette).with_material(material);
                        let colors = ui.colors;
                        let [_, _, track] = colors.grounds(palette.worst_backdrop());
                        let in_row = over(colors.surface2, track);
                        let on = contrast(colors.accent.on, colors.accent.fill);
                        let knob = knob_off(&colors);
                        let off = contrast(knob, track).min(contrast(knob, in_row));
                        assert!(
                            on >= DIM_CONTRAST && off >= DIM_CONTRAST,
                            "{} {} {} {material:?}: on {on:.2}, off {off:.2}",
                            theme.name,
                            flavor.name,
                            accent.name
                        );
                    }
                }
            }
        }
    }
}
