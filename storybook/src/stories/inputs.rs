use guido::prelude::*;
use styletto::centered_row;
use styletto::components::button::{Fill, icon_button};
use styletto::components::input::{field as frame, password_field as secret};
use styletto::components::slider::slider as quick_slider;
use styletto::components::toggle::toggle as switch;
use styletto::foundations::control::Size;
use styletto::foundations::tokens::{icon as icon_box, size as space};
use styletto::foundations::typography as ty;
use styletto::foundations::typography::Typeset;
use styletto::primitives::text::label;
use styletto::{Icon, icon, ui};

use crate::stories::story::Ctx;

pub fn toggle(ctx: &Ctx) -> Container {
    let on = ctx.bool("On", true);
    let enabled = ctx.bool("Enabled", true);
    let toggled = ctx.action_with::<bool>("on_toggle");
    container()
        .layout(centered_row(space::S3.px()))
        .child(label("Bluetooth"))
        .child(switch(on).enabled(enabled).on_toggle(move |value| {
            on.set(value);
            toggled(value);
        }))
}

pub fn slider(ctx: &Ctx) -> Container {
    let value = ctx.number("Value", 0.0..=100.0, 60.0);
    let width = ctx.number("Width", 120.0..=480.0, 280.0);
    let changed = ctx.action_with::<f32>("on_change");
    container()
        .width(move || width.get())
        .child(quick_slider(value).on_change(move |v| {
            value.set(v);
            changed(v);
        }))
}

/// The frame a text field sits in, ringed in the accent or, refused, in red,
/// in the launcher's size or the Wi-Fi password's.
pub fn field(ctx: &Ctx) -> Container {
    let refused = ctx.bool("Refused", false);
    let small = ctx.bool("Small", false);
    let typed = create_signal(String::new());
    let submitted = ctx.action_with::<String>("on_submit");
    let t = ui();
    container().width(space::S80.px()).child(move || {
        let submitted = submitted.clone();
        frame()
            .size(if small.get() { Size::Sm } else { Size::Md })
            .ring(move || {
                if refused.get() {
                    t.colors.danger
                } else {
                    t.colors.accent.with_alpha(0.7)
                }
            })
            .child(icon(Icon::Search, icon_box::SMALL.px(), t.colors.dim))
            .child(
                container().width(fill()).child(
                    text_input(typed)
                        .typeset(ty::TITLE_XSMALL)
                        .color(t.colors.text)
                        .placeholder("Search apps".to_string())
                        .placeholder_color(t.colors.sub)
                        .cursor_color(t.colors.accent)
                        .on_submit(move |value| submitted(value.to_string())),
                ),
            )
    })
}

/// The password's field as a polkit request or a lock screen asks for it:
/// the ring turns red on a refusal, the end spins while the answer is
/// checked, and a reveal button can go before it.
pub fn password_field(ctx: &Ctx) -> Container {
    let refused = ctx.bool("Refused", false);
    let busy = ctx.bool("Busy", false);
    let held = ctx.bool("Held", false);
    let revealer = ctx.bool("Reveal button", true);
    let submitted = ctx.action("on_submit");
    let revealed = ctx.action("reveal");
    let password = create_password();
    let t = ui();
    container().width(space::S80.px()).child(move || {
        let reveal = revealer.get().then(|| {
            icon_button(Icon::Eye)
                .size(Size::Sm)
                .fill(Fill::Ghost)
                .on_click(revealed.clone())
        });
        let mut field = secret()
            .refused(refused)
            .busy(busy)
            .held(held)
            .on_submit(submitted.clone())
            .child(icon(Icon::Lock, icon_box::SMALL.px(), t.colors.sub))
            .child(
                container().width(fill()).child(
                    password_input(password)
                        .typeset(ty::LABEL_MEDIUM)
                        .color(t.colors.text)
                        .placeholder("Password".to_string())
                        .placeholder_color(t.colors.sub)
                        .cursor_color(t.colors.accent)
                        .readonly(held)
                        .on_submit(submitted.clone()),
                ),
            );
        if let Some(reveal) = reveal {
            field = field.extra(reveal);
        }
        field
    })
}
