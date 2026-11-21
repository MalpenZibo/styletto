//! An on/off switch whose knob springs across the track.

use guido::prelude::*;

use crate::foundations::control::Size;
use crate::foundations::motion;
use crate::foundations::tokens::radius;
use crate::foundations::tokens::{opacity, size};
use crate::ui::{active_fill, active_ink, flip, ui};

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
    let t = ui();
    container()
        .width(width())
        .height(height())
        .corners(radius::pill(height()))
        .background(active_fill(active, t.colors.surface2).transition(motion::color()))
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
                // the track while off. `sub`, not `dim`: a switch often sits
                // in a row, whose hover or selection tints the track again,
                // and only `sub` keeps the 3:1 a control asks there too.
                .background(active_ink(active, t.colors.sub).transition(motion::color()))
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
                    for material in [Material::Solid, Material::Translucent, Material::Glass] {
                        let colors = Ui::new(palette).with_material(material).colors;
                        let [_, _, track] = colors.grounds(palette.worst_backdrop());
                        let in_row = over(colors.surface2, track);
                        let on = contrast(colors.accent.on, colors.accent.fill);
                        let off = contrast(colors.sub, track).min(contrast(colors.sub, in_row));
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
