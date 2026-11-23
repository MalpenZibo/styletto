//! Digits that roll to their next value like a mechanical counter; other
//! characters are drawn as they are.

use guido::prelude::*;

use crate::foundations::motion;
use crate::foundations::typography::{LABEL_MEDIUM, Resolved, Typeset};

#[component]
pub fn rolling(
    #[prop(required)] value: String,
    /// A [`Type`](crate::foundations::typography::Type), or a style off the scale. Its
    /// line height is the clip and the step between digits.
    #[prop(default = LABEL_MEDIUM.em().into())]
    look: Resolved,
    color: Color,
) -> impl Widget {
    let look = Resolved {
        tabular: true,
        ..look.get_untracked()
    };
    // Rebuilt only when the text's shape changes ("9:59" to "10:00"), so a
    // ticking clock's columns keep rolling.
    let pattern = create_memo(move || {
        value
            .get()
            .chars()
            .map(|c| if c.is_ascii_digit() { '0' } else { c })
            .collect::<String>()
    });

    container().child(move || {
        let children: Vec<Container> = pattern.with(|p| {
            p.chars()
                .enumerate()
                .map(|(index, c)| {
                    if c == '0' {
                        digit(value, index, look, color)
                    } else {
                        container().child(glyph(c.to_string(), look, color))
                    }
                })
                .collect()
        });
        container()
            .layout(Flex::row().cross_alignment(CrossAlignment::Start))
            .children(children)
    })
}

fn glyph(content: String, look: Resolved, color: Signal<Color>) -> Text {
    text(content)
        .typeset(look)
        .color(motion::eased_color(color))
        .nowrap()
}

fn digit(value: Signal<String>, index: usize, look: Resolved, color: Signal<Color>) -> Container {
    let line = look.line;
    let shown = create_memo(move || {
        value.with(|v| {
            v.chars()
                .nth(index)
                .and_then(|c| c.to_digit(10))
                .unwrap_or(0)
        }) as f32
    });
    let column = container()
        .layout(Flex::column())
        .translate((move || Translate::new(0.0, -shown.get() * line)).transition(motion::spring()))
        .children((0..10).map(move |n| container().child(glyph(n.to_string(), look, color))));
    container()
        .height(line)
        .overflow(Overflow::Hidden)
        .child(column)
}
