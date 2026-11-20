//! The storybook's own chrome: the story list, the canvas the selected story
//! renders on, and the panel with its controls and the actions it fired.

use std::path::PathBuf;
use std::rc::Rc;

use guido::prelude::*;
use styletto::components::button::{self, Fill, icon_button};
use styletto::components::list::row;
use styletto::components::slider::slider;
use styletto::components::toggle::toggle;
use styletto::foundations::control::Size;
use styletto::foundations::tokens;
use styletto::foundations::tokens::{icon as icon_box, size as space, stroke};
use styletto::foundations::typography as ty;
use styletto::foundations::typography::Typeset;
use styletto::primitives::text::{caption, heading, label, typed};
use styletto::{Icon, Ui, centered_row, icon, provide_ui, ui};

use crate::chrome::appearance::{self, Appearance};
use crate::chrome::widgets::{choices, field, island, section, wallpaper};
use crate::stories::story::{Control, Ctx};
use crate::stories::{STORIES, group_icon};

const CHROME: Color = Color::from_hex(0x1a1b26);
const SIDEBAR: Color = Color::from_hex(0x16161e);

pub const MARGIN: i32 = 32;
const PILL: (u32, u32) = (260, 48);

/// Folds the storybook into a pill in the bottom-right corner that takes no
/// keyboard, so the rest of the desktop stays usable, and back.
#[derive(Clone)]
struct Window {
    surface: SurfaceHandle,
    collapsed: RwSignal<bool>,
}

impl Window {
    fn collapse(&self) {
        self.surface
            .set_keyboard_interactivity(KeyboardInteractivity::None);
        self.surface.set_anchor(Anchor::BOTTOM | Anchor::RIGHT);
        self.surface.set_margin(Margin::all(16));
        self.surface.set_size(PILL.0, PILL.1);
        self.collapsed.set(true);
    }

    fn expand(&self) {
        self.surface
            .set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        self.surface.set_margin(Margin::all(MARGIN));
        self.surface
            .set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        self.collapsed.set(false);
    }
}

pub fn view() -> Container {
    provide_ui(Ui::default());
    let selected = create_signal(0usize);
    // The theme page shows in place of the selected story.
    let theming = create_signal(false);
    let appearance = Appearance::new();
    let log = create_signal(Vec::<String>::new());
    let window = Window {
        surface: expect_context::<SurfaceHandle>(),
        collapsed: create_signal(false),
    };
    let collapsed = window.collapsed;

    container()
        .width(fill())
        .height(fill())
        .layout(ZStack::new())
        .child(
            full(selected, theming, appearance, log, window.clone())
                .visible(move || !collapsed.get()),
        )
        .child(pill(selected, window).visible(move || collapsed.get()))
}

fn pill(selected: RwSignal<usize>, window: Window) -> Container {
    let t = ui();
    button::pressable(
        container()
            .width(fill())
            .height(fill())
            .padding([0.0, space::S2.px(), 0.0, space::S4.px()])
            .corners(tokens::radius::pill(PILL.1 as f32))
            .background(CHROME.transition(styletto::foundations::motion::hover()))
            .border(stroke::HAIRLINE, Color::WHITE.with_alpha(0.12))
            .when_hovered(|s| s.background(SIDEBAR)),
        0.96,
    )
    .on_click(move || window.expand())
    .layout(centered_row(space::S2.px()))
    .child(
        container().width(fill()).child(
            text(move || format!("Storybook · {}", STORIES[selected.get()].name))
                .typeset(ty::LABEL_MEDIUM.em())
                .color(t.colors.text)
                .nowrap(),
        ),
    )
    .child(icon(Icon::ChevronUp, icon_box::MEDIUM.px(), t.colors.sub))
}

fn full(
    selected: RwSignal<usize>,
    theming: RwSignal<bool>,
    appearance: Appearance,
    log: RwSignal<Vec<String>>,
    window: Window,
) -> Container {
    let escape = window.clone();
    let drawn = create_memo(move || appearance.ui());
    container()
        .width(fill())
        .height(fill())
        .corners(tokens::radius::MEDIUM.corners())
        .overflow(Overflow::Hidden)
        .background(CHROME)
        .border(stroke::HAIRLINE, Color::WHITE.with_alpha(0.08))
        .layout(Flex::row())
        .on_key_down(move |key, _, _| {
            if key == Key::Escape {
                escape.collapse();
                EventResponse::Handled
            } else {
                EventResponse::Ignored
            }
        })
        .child(sidebar(selected, theming, log))
        .child(
            container()
                .width(fill())
                .height(fill())
                .layout(Flex::column())
                .child(toolbar(selected, theming, window))
                .child(container().width(fill()).height(fill()).child(move || {
                    if theming.get() {
                        return appearance::page(appearance);
                    }
                    let story = &STORIES[selected.get()];
                    let drawn = drawn.get();
                    let ctx = Rc::new(Ctx::new(log));
                    // The story and its canvas get the chosen look in a scope
                    // of their own; the panel beside them keeps the chrome's.
                    let canvas = container().width(fill()).height(fill()).child({
                        let ctx = ctx.clone();
                        move || {
                            ctx.reset();
                            provide_ui(drawn);
                            canvas(appearance.backdrop(), (story.render)(&ctx))
                        }
                    });
                    container()
                        .width(fill())
                        .height(fill())
                        .layout(Flex::row())
                        .child(canvas)
                        .child(panel(ctx, log))
                })),
        )
}

fn sidebar(
    selected: RwSignal<usize>,
    theming: RwSignal<bool>,
    log: RwSignal<Vec<String>>,
) -> Container {
    let mut rows = vec![
        row("Theme")
            .icon(Icon::Sun)
            .selected(theming)
            .on_click(move || theming.set(true))
            .into_any(),
    ];
    // Each group is a heading, its stories nested under a guide line, so a
    // group reads as one and not as one more story.
    let t = ui();
    let mut start = 0;
    while start < STORIES.len() {
        let group = STORIES[start].group;
        let end = STORIES[start..]
            .iter()
            .position(|story| story.group != group)
            .map_or(STORIES.len(), |len| start + len);
        rows.push(
            container()
                .padding([
                    space::S3.px(),
                    space::S3.px(),
                    space::S1.px(),
                    space::S3.px(),
                ])
                .layout(centered_row(space::S2.px()))
                .child(icon(
                    group_icon(group),
                    icon_box::SMALL.px(),
                    t.colors.accent_text,
                ))
                .child(
                    typed(group, ty::TITLE_XSMALL.em())
                        .color(t.colors.text)
                        .nowrap(),
                )
                .into_any(),
        );
        let stories = (start..end).map(|index| {
            row(STORIES[index].name)
                .selected(move || !theming.get() && selected.get() == index)
                .on_click(move || {
                    log.set(Vec::new());
                    theming.set(false);
                    selected.set(index);
                })
                .into_any()
        });
        rows.push(
            container()
                .width(fill())
                .padding([0.0, 0.0, 0.0, space::S4.px()])
                .layout(Flex::row().spacing(space::S1.px()))
                .child(
                    container()
                        .width(stroke::HAIRLINE)
                        .height(fill())
                        .background(t.colors.line),
                )
                .child(
                    container()
                        .width(fill())
                        .layout(Flex::column().spacing(space::S0_5.px()))
                        .children(stories.collect::<Vec<_>>()),
                )
                .into_any(),
        );
        start = end;
    }

    container()
        .width(space::S60.px())
        .height(fill())
        .background(SIDEBAR)
        .layout(Flex::column())
        .child(
            container()
                .padding([
                    space::S4.px(),
                    space::S4.px(),
                    space::S1.px(),
                    space::S4.px(),
                ])
                .child(heading("styletto")),
        )
        .child(
            container()
                .width(fill())
                .height(fill())
                .padding([0.0, space::S2.px(), space::S4.px(), space::S2.px()])
                .scroll(Scroll::vertical())
                .layout(Flex::column().spacing(space::S0_5.px()))
                .children(rows),
        )
}

fn toolbar(selected: RwSignal<usize>, theming: RwSignal<bool>, window: Window) -> Container {
    let t = ui();
    let place = create_memo(move || {
        if theming.get() {
            ("Settings", "Theme")
        } else {
            let story = &STORIES[selected.get()];
            (story.group, story.name)
        }
    });
    let place = move || place.get();

    container()
        .width(fill())
        .height(space::S15.px())
        .padding([0.0, space::S4.px(), 0.0, space::S4.px()])
        .layout(centered_row(space::S4.px()))
        .child(
            container()
                .width(fill())
                .layout(centered_row(space::S2.px()))
                .child(caption(move || place().0.to_string()))
                .child(caption("/"))
                .child(
                    text(move || place().1.to_string())
                        .typeset(ty::LABEL_MEDIUM.em())
                        .color(t.colors.text),
                ),
        )
        .child(
            container()
                .layout(Flex::row().spacing(space::S0_5.px()))
                .child(
                    icon_button(Icon::Minus)
                        .size(Size::Sm)
                        .fill(Fill::Ghost)
                        .on_click(move || window.collapse()),
                )
                .child(
                    icon_button(Icon::X)
                        .size(Size::Sm)
                        .fill(Fill::Ghost)
                        .on_click(quit_app),
                ),
        )
}

/// The story on the island's fill, over a wallpaper that shows through a
/// translucent island and blurs behind a glass one.
pub fn canvas(backdrop: Memo<Option<PathBuf>>, rendered: Container) -> Container {
    wallpaper(backdrop, island(space::S4.px()).child(rendered))
}

fn panel(ctx: Rc<Ctx>, log: RwSignal<Vec<String>>) -> Container {
    let t = ui();

    container()
        .width(space::S80.px())
        .height(fill())
        .padding([
            space::S1.px(),
            space::S4.px(),
            space::S4.px(),
            space::S2.px(),
        ])
        .layout(Flex::column().spacing(space::S2.px()))
        .child(section("Controls"))
        // Built after the story, in the scope it was drawn in, so its
        // controls are declared by then.
        .child(move || {
            let controls = ctx.controls();
            if controls.is_empty() {
                return container().child(caption("This story has no controls."));
            }
            container()
                .width(fill())
                .layout(Flex::column().spacing(space::S3.px()))
                .children(controls.iter().map(control_row).collect::<Vec<_>>())
        })
        .child(container().height(space::S2.px()))
        .child(
            container()
                .width(fill())
                .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
                .child(section("Actions"))
                .child(
                    container()
                        .visible(move || log.with(|l| !l.is_empty()))
                        .child(
                            icon_button(Icon::X)
                                .size(Size::Sm)
                                .fill(Fill::Ghost)
                                .on_click(move || log.set(Vec::new())),
                        ),
                ),
        )
        .child(
            container()
                .width(fill())
                .height(fill())
                .padding(space::S2.px())
                .corners(tokens::radius::SMALL.corners())
                .background(t.colors.surface)
                .scroll(Scroll::vertical())
                .child(move || {
                    let entries = log.get();
                    if entries.is_empty() {
                        return container().child(caption("Nothing fired yet."));
                    }
                    container()
                        .layout(Flex::column().spacing(space::S1.px()))
                        .children(entries.into_iter().map(|entry| {
                            container()
                                .child(text(entry).typeset(ty::MONO_SMALL).color(t.colors.sub))
                        }))
                }),
        )
}

fn control_row(control: &Control) -> Container {
    let t = ui();
    match control {
        Control::Bool { label: name, value } => {
            let value = *value;
            container()
                .width(fill())
                .layout(centered_row(0.0).main_alignment(MainAlignment::SpaceBetween))
                .child(label(*name))
                .child(toggle(value).on_toggle(move |on| value.set(on)))
        }
        Control::Number {
            label: name,
            range,
            value,
        } => {
            let value = *value;
            let (min, max) = (*range.start(), *range.end());
            let span = (max - min).max(f32::EPSILON);
            field(
                name,
                Some(
                    text(move || format!("{:.0}", value.get()))
                        .typeset(ty::LABEL_MEDIUM.tabular())
                        .color(t.colors.sub),
                ),
                slider(move || (value.get() - min) / span * 100.0)
                    .on_change(move |percent| value.set(min + percent / 100.0 * span)),
            )
        }
        Control::Choice {
            label: name,
            options,
            value,
        } => {
            let value = *value;
            field(
                name,
                None,
                choices(
                    options.to_vec(),
                    move |index| value.get() == index,
                    move |index| value.set(index),
                ),
            )
        }
        Control::Text { label: name, value } => field(
            name,
            None,
            container()
                .width(fill())
                .padding([space::S2.px(), space::S2.px()])
                .corners(tokens::radius::SMALL.corners())
                .background(t.colors.surface)
                .child(
                    text_input(*value)
                        .typeset(ty::LABEL_MEDIUM)
                        .color(t.colors.text),
                ),
        ),
    }
}
