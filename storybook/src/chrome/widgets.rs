//! The storybook's own small pieces, shared by its chrome and its pages.

use std::path::PathBuf;
use styletto::components::button::{self, chip_button};
use styletto::foundations::tokens::{radius, size as space, stroke};
use styletto::primitives::text::{label, overline};
use styletto::ui;

use guido::prelude::*;

/// A dot of `color`, ringed while `on`.
pub fn swatch(color: Color, on: impl Fn() -> bool + 'static) -> Container {
    button::pressable(
        container()
            .width(space::S4.px())
            .height(space::S4.px())
            .corners(radius::pill(space::S4.px()))
            .background(color)
            .border(
                stroke::BOLD,
                (move || {
                    if on() {
                        Color::WHITE
                    } else {
                        Color::TRANSPARENT
                    }
                })
                .transition(styletto::foundations::motion::hover()),
            ),
        0.85,
    )
}

/// A named choice among a few options, as chips.
pub fn choice<T: Copy + PartialEq + 'static>(
    name: &'static str,
    options: &'static [(&'static str, T)],
    value: RwSignal<T>,
) -> Container {
    field(
        name,
        None,
        chips(
            options.iter().map(|(name, _)| *name).collect(),
            move |index| value.get() == options[index].1,
            move |index| value.set(options[index].1),
        ),
    )
}

/// A setting: its name, a note at the other end of the line, and what sets
/// it underneath.
pub fn field(name: &'static str, note: Option<Text>, body: impl Widget + 'static) -> Container {
    container()
        .width(fill())
        .layout(Flex::column().spacing(space::S2.px()))
        .child(
            container()
                .width(fill())
                .layout(
                    Flex::row()
                        .spacing(space::S3.px())
                        .cross_alignment(CrossAlignment::Center),
                )
                .child(label(name))
                .child(
                    container()
                        .width(fill())
                        .layout(Flex::row().main_alignment(MainAlignment::End))
                        .children(note.map(|note| note.overflow(TextOverflow::Ellipsis))),
                ),
        )
        .child(body)
}

/// The options as chips, as many to a line as fit.
pub fn chips(
    names: Vec<&'static str>,
    active: impl Fn(usize) -> bool + Copy + 'static,
    pick: impl Fn(usize) + Copy + 'static,
) -> Container {
    container()
        .width(fill())
        .layout(Flex::row().spacing(space::S1.px()).wrap(true))
        .children(names.into_iter().enumerate().map(move |(index, name)| {
            chip_button(name)
                .active(move || active(index))
                .on_click(move || pick(index))
        }))
}

pub fn section(title: &str) -> Container {
    container()
        .padding([space::S3.px(), 0.0, 0.0, 0.0])
        .child(overline(title.to_string()))
}

/// The island's fill and material, its corners concentric with what sits
/// `padding` inside it at MEDIUM.
pub fn island(padding: f32) -> Container {
    let t = ui();
    container()
        .padding(padding)
        .corners(radius::shape(radius::around(radius::MEDIUM.px(), padding)))
        .background(t.colors.island)
        .backdrop_blur(t.material.blur())
}

/// Shapes of muted colour, lightly blurred, scattered over the dark
/// gradient: enough for a translucent island to show what is behind it and
/// a glass one to blur it, without shouting.
const BLOBS: [(u32, f32, f32, bool); 12] = [
    (0x7c6fd6, -320.0, -260.0, true),
    (0x3fa7a0, 40.0, -300.0, false),
    (0xd98a6c, 360.0, -220.0, true),
    (0xc7a35a, -160.0, -100.0, false),
    (0x6f9fd8, 200.0, -60.0, true),
    (0xb86f9a, -380.0, 60.0, false),
    (0x5fae7c, -40.0, 60.0, true),
    (0x7c6fd6, 380.0, 120.0, false),
    (0xd98a6c, -240.0, 240.0, true),
    (0x6f9fd8, 120.0, 220.0, false),
    (0x3fa7a0, 320.0, 320.0, true),
    (0xc7a35a, -60.0, 360.0, false),
];

/// `content` centred over the picture in `backdrop`, or a soft wallpaper
/// where there is none, scrolling where it is taller.
pub fn wallpaper<M>(
    backdrop: Memo<Option<PathBuf>>,
    content: impl guido::widgets::IntoChild<M>,
) -> Container {
    let behind = container()
        .width(fill())
        .height(fill())
        .child(move || match backdrop.get() {
            Some(path) => container()
                .width(fill())
                .height(fill())
                .child(image(ImageSource::Path(path)).content_fit(ContentFit::Cover)),
            None => blobs(),
        });
    container()
        .width(fill())
        .height(fill())
        .corners(radius::MEDIUM.corners())
        .overflow(Overflow::Hidden)
        .layout(ZStack::new())
        .child(behind)
        .child(
            container()
                .width(fill())
                .height(fill())
                .padding(space::S4.px())
                .scroll(Scroll::vertical())
                .layout(Flex::column().center())
                .child(
                    container()
                        .width(fill())
                        .layout(Flex::column().cross_alignment(CrossAlignment::Center))
                        .child(content),
                ),
        )
}

fn blobs() -> Container {
    container()
        .width(fill())
        .height(fill())
        .gradient(LinearGradient::new(
            Color::from_hex(0x414868),
            Color::from_hex(0x1a1b26),
            GradientDirection::Diagonal,
        ))
        .layout(ZStack::new().align(Alignment::CENTER))
        .children(BLOBS.into_iter().map(|(hex, x, y, large)| {
            let side = if large {
                space::S30.px()
            } else {
                space::S20.px()
            };
            container()
                .width(side)
                .height(side)
                .corners(radius::pill(side))
                .background(Color::from_hex(hex).with_alpha(0.6))
                .translate(Translate::new(x, y))
                .blur(space::S2.px())
        }))
}
