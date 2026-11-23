//! A surface's choices for its components: palette, density, shape, fonts,
//! material and motion. Set with [`provide_ui`], read with [`ui`].

use guido::prelude::*;
use guido::reactive::Prop;

use crate::foundations::motion;
use crate::theme::palette::Palette;

/// How much room things take: type, controls and space move one step, and
/// the lengths an app lays out with [`size::scaled`](crate::foundations::tokens::size::scaled)
/// grow or shrink by its [`scale`](Self::scale).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Density {
    Compact,
    #[default]
    Default,
    Comfortable,
}

impl Density {
    pub(crate) fn shift(self) -> i8 {
        match self {
            Density::Compact => -1,
            Density::Default => 0,
            Density::Comfortable => 1,
        }
    }

    /// How much a layout length grows: about as much as what a panel holds
    /// does when its type, controls and space move one step, so a panel
    /// keeps its proportions. A step itself is too coarse for this (padding
    /// goes 16 to 24, half again).
    pub fn scale(self) -> f32 {
        match self {
            Density::Compact => 0.875,
            Density::Default => 1.0,
            Density::Comfortable => 1.25,
        }
    }
}

/// How round corners are: every radius moves one step; pills stay pills.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Shape {
    Square,
    #[default]
    Soft,
    Round,
}

impl Shape {
    pub(crate) fn shift(self) -> i8 {
        match self {
            Shape::Square => -1,
            Shape::Soft => 0,
            Shape::Round => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fonts {
    pub sans: FontFamily,
    pub mono: FontFamily,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            sans: FontFamily::SansSerif,
            mono: FontFamily::Monospace,
        }
    }
}

/// What the island is made of: its fill's opacity and the blur behind it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Material {
    #[default]
    Solid,
    Translucent,
    Glass,
}

impl Material {
    /// Every material, from the most solid.
    pub const ALL: [Material; 3] = [Material::Solid, Material::Translucent, Material::Glass];

    /// The island's opacity. Glass stops at 0.75: past it, an island over a
    /// backdrop of its opposite turns so grey that its text can't keep its
    /// levels apart and still read.
    pub fn opacity(self) -> f32 {
        match self {
            Material::Solid => 1.0,
            Material::Translucent => 0.86,
            Material::Glass => 0.75,
        }
    }

    /// The compositor's blur behind it.
    pub fn blur(self) -> f32 {
        match self {
            Material::Solid => 0.0,
            Material::Translucent => 24.0,
            Material::Glass => 48.0,
        }
    }
}

/// How much things move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Motion {
    #[default]
    Full,
    /// Fades only: no spring, no growth, no shake.
    Reduced,
    /// Everything changes at once.
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ui {
    /// The palette, with the island's alpha set by the material and the
    /// text adjusted to read on it: what components paint with.
    pub colors: Palette,
    palette: Palette,
    pub density: Density,
    pub shape: Shape,
    pub fonts: Fonts,
    pub material: Material,
    pub motion: Motion,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new(Palette::default())
    }
}

impl Ui {
    pub fn new(palette: Palette) -> Self {
        Self {
            colors: palette,
            palette,
            density: Density::default(),
            shape: Shape::default(),
            fonts: Fonts::default(),
            material: Material::default(),
            motion: Motion::default(),
        }
        .settled()
    }

    pub fn with_density(self, density: Density) -> Self {
        Self { density, ..self }
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        Self { shape, ..self }
    }

    pub fn with_fonts(self, fonts: Fonts) -> Self {
        Self { fonts, ..self }
    }

    pub fn with_material(self, material: Material) -> Self {
        Self { material, ..self }.settled()
    }

    pub fn with_motion(self, motion: Motion) -> Self {
        Self { motion, ..self }
    }

    /// The island at the material's opacity, and its text as much darker or
    /// lighter as keeps it reading whatever shows through.
    fn settled(mut self) -> Self {
        let opacity = self.material.opacity();
        self.colors = self.palette.see_through(opacity);
        self
    }
}

/// Gives the components built from here on `ui`: a value, or a signal or a
/// closure that changes. A new palette, or a new material, repaints what is
/// built in place, easing into it, as every colour is read where it is
/// painted; density, shape, fonts and motion size and time what is built,
/// so a change of [`Ui::sizes`] still asks for the tree to be built again.
///
/// What it is given is worked out once per change, not once per colour
/// read, and a change that leaves the `Ui` as it was repaints nothing.
pub fn provide_ui<M>(ui: impl IntoSignal<Ui, M>) {
    provide_context(Provided(held(ui)));
}

/// `ui` as a signal worked out once per change: a memo, so a closure runs
/// once per change rather than per read, and a change that leaves the `Ui`
/// as it was repaints nothing.
fn held<M>(ui: impl IntoSignal<Ui, M>) -> Signal<Ui> {
    match ui.into_prop() {
        Prop::Reactive(ui) => create_memo(move || ui.get()).into_signal(),
        Prop::Const(ui) => create_stored(ui),
        Prop::Unset => create_stored(Ui::default()),
    }
}

/// What `build` makes, given `ui` (a value, or a signal or a closure that
/// changes) as [`provide_ui`] gives it, and built again only when what
/// sizes it changes ([`Ui::sizes`]): a new palette or material repaints it
/// in place, and whatever is open, typed or scrolled in it stays so.
pub fn themed<M, W: Widget + 'static>(
    ui: impl IntoSignal<Ui, M>,
    build: impl Fn() -> W + 'static,
) -> Container {
    // Worked out once per change, for the sizes and for what is built.
    let ui = held(ui);
    let sizes = create_memo(move || ui.with(Ui::sizes));
    container().child(move || {
        sizes.get();
        provide_context(Provided(ui));
        build()
    })
}

/// The `Ui` provided, as a signal: a fixed one is a signal that never
/// changes.
#[derive(Clone, Copy)]
struct Provided(Signal<Ui>);

/// The nearest [`Ui`], or the default, as it is now: for what sizes a
/// widget while it is built. Read while building, not in event handlers.
pub fn ui() -> Ui {
    with_ui(|ui| *ui)
}

/// The nearest [`Ui`] as it changes. Read it while building, then read its
/// colours inside the closures a widget's properties take: the widget
/// follows a new palette without being built again.
pub fn look() -> Look {
    Look(use_context::<Provided>().map(|provided| provided.0))
}

/// A handle on the nearest [`Ui`] (see [`look`]), whose reads are tracked.
#[derive(Clone, Copy)]
pub struct Look(Option<Signal<Ui>>);

impl Look {
    /// The colours components paint with, as they are now; tracked, so a
    /// property's closure that reads them runs again when they change.
    pub fn colors(self) -> Palette {
        self.with(|ui| ui.colors)
    }

    /// A colour of the palette as a closure a property takes, so it follows
    /// the palette: `.color(t.color(|c| c.text))`. `role` picks it, and may
    /// read what it captures, a hue or a signal:
    /// `t.color(move |c| c.hue(hue).fill)`. For a property with a motion of
    /// its own, or a state layer's override, which takes none; elsewhere
    /// [`eased`](Self::eased).
    pub fn color(
        self,
        role: impl Fn(&Palette) -> Color + Copy + 'static,
    ) -> impl Fn() -> Color + Copy + 'static {
        move || self.with(|ui| role(&ui.colors))
    }

    /// A colour of the palette, as [`color`](Self::color) picks it, easing
    /// into a new palette or a new pick at [`motion::color`]: for a property
    /// with no motion of its own. Under [`Motion::None`] it changes at once.
    pub fn eased(self, role: impl Fn(&Palette) -> Color + Copy + 'static) -> Animated<Color> {
        // Its own Ui's motion, not the nearest one where it is read: a Look
        // taken in one subtree may be read in another.
        let motion = match self.0 {
            Some(ui) => ui.with_untracked(|ui| ui.motion),
            None => DEFAULT.with(|ui| ui.motion),
        };
        motion::eased_color_in(motion, self.color(role))
    }

    /// The whole `Ui` as it is now, tracked: for a subtree that provides a
    /// `Ui` of its own made from this one, and follows it,
    /// `provide_ui(move || t.ui().with_density(Density::Compact))`.
    pub fn ui(self) -> Ui {
        self.with(|ui| *ui)
    }

    /// What the island is made of, tracked.
    pub fn material(self) -> Material {
        self.with(|ui| ui.material)
    }

    fn with<R>(self, f: impl FnOnce(&Ui) -> R) -> R {
        match self.0 {
            Some(ui) => ui.with(f),
            None => DEFAULT.with(f),
        }
    }
}

impl Ui {
    /// What sizes and times a widget, fixed when it is built: a change here
    /// asks for the tree to be built again, where a new palette or material
    /// is followed in place.
    pub fn sizes(&self) -> (Density, Shape, Fonts, Motion) {
        (self.density, self.shape, self.fonts, self.motion)
    }
}

thread_local! {
    /// The `Ui` read where none is provided, worked out once: its palette
    /// takes a search per hue.
    static DEFAULT: Ui = Ui::default();
}

/// Reads one field of the nearest [`Ui`], untracked, without cloning it.
pub(crate) fn with_ui<R>(f: impl Fn(&Ui) -> R) -> R {
    with_context(|provided: &Provided| provided.0.with_untracked(&f))
        .unwrap_or_else(|| DEFAULT.with(|ui| f(ui)))
}

/// The accent while `active`, `off` (a colour, a signal or a closure) while
/// not: the fill of anything that turns on, as a tile or a toggling button.
pub fn active_fill<M>(
    active: Signal<bool>,
    off: impl IntoSignal<Color, M>,
) -> impl Fn() -> Color + Copy + 'static {
    // `into_prop` keeps a constant as it is, with no signal made for it,
    // and always carries a value: the fallback is never read.
    let (on, off) = (look().color(|c| c.accent.fill), off.into_prop());
    move || {
        if active.get() {
            on()
        } else {
            off.get_or(Color::TRANSPARENT)
        }
    }
}

/// The ink that reads on [`active_fill`]: `accent.on` while `active`, `off`
/// while not.
pub fn active_ink<M>(
    active: Signal<bool>,
    off: impl IntoSignal<Color, M>,
) -> impl Fn() -> Color + Copy + 'static {
    let (on, off) = (look().color(|c| c.accent.on), off.into_prop());
    move || {
        if active.get() {
            on()
        } else {
            off.get_or(Color::TRANSPARENT)
        }
    }
}

/// Asks `on_toggle` for the opposite of `active`.
pub(crate) fn flip(
    active: Signal<bool>,
    on_toggle: Option<Callback<(bool,)>>,
) -> impl Fn() + Copy + 'static {
    move || {
        if let Some(on_toggle) = on_toggle {
            on_toggle.run(!active.get_untracked());
        }
    }
}

/// Tabular figures, so a number keeps its width.
pub fn tabular() -> FontFeatures {
    FontFeatures::new().enable(b"tnum")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::palette::{Hue, contrast};
    use crate::theme::presets::{THEMES, ThemeChoice};

    #[test]
    fn a_denser_layout_scales_its_lengths_down() {
        let [compact, default, comfortable] =
            [Density::Compact, Density::Default, Density::Comfortable].map(Density::scale);
        assert!(compact < default && default == 1.0 && default < comfortable);
        assert_eq!(392.0 * comfortable, 490.0);
    }

    #[test]
    fn text_reads_on_every_material_over_the_worst_backdrop() {
        // The text, and every hue as text with each of the accents; `sub`
        // to 4.5:1 on a solid island and to 3:1 on a see-through one. And
        // the text on a field's selection at the 3:1 a control keeps, the
        // selection no more of the accent than `opacity::SELECTION` and
        // never so little it doesn't show.
        use crate::foundations::tokens::opacity;
        use crate::theme::palette::{DIM_CONTRAST, over};
        for theme in THEMES {
            for flavor in theme.flavors {
                for (index, accent) in flavor.accents.iter().enumerate() {
                    let palette = flavor.scheme(index).palette();
                    for material in Material::ALL {
                        let colors = Ui::new(palette).with_material(material).colors;
                        let named = || {
                            format!(
                                "{} {} {} {material:?}",
                                theme.name, flavor.name, accent.name
                            )
                        };
                        assert_eq!(
                            colors.unreadable(palette.worst_backdrop()),
                            None,
                            "{}",
                            named()
                        );
                        let field = colors.grounds(palette.worst_backdrop())[1];
                        let selected = contrast(colors.text, over(colors.selection, field));
                        let alpha = colors.selection.a;
                        assert!(
                            selected >= DIM_CONTRAST
                                && (opacity::SELECTION_MIN..=opacity::SELECTION).contains(&alpha),
                            "{}: selection at {alpha:.2}, its text at {selected:.2}",
                            named()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_selection_shows_where_no_opacity_keeps_its_text_at_three_to_one() {
        // Text set to the island's own colour, under an accent of the same:
        // no band keeps it at 3:1 on a solid island, and the selection still
        // shows, at the floor; never less on a see-through one, whose text
        // is pushed away from the island.
        use crate::foundations::tokens::opacity;
        let mut palette = ThemeChoice::default().palette_of(false);
        palette.text = palette.island;
        palette.accent.fill = palette.island;
        let solid = Ui::new(palette).colors.selection;
        assert_eq!(solid.a, opacity::SELECTION_MIN);
        for material in Material::ALL {
            let selection = Ui::new(palette).with_material(material).colors.selection;
            assert!(
                selection.a >= opacity::SELECTION_MIN,
                "{material:?}: {}",
                selection.a
            );
        }
    }

    #[test]
    fn themed_works_its_ui_out_once_per_change() {
        use std::cell::Cell;
        use std::rc::Rc;

        use guido::testing::Headless;

        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let choice = ThemeChoice::default();
        let [dark, light] = [false, true].map(|light| Ui::new(choice.palette_of(light)));
        let given = create_signal(dark);
        let runs = Rc::new(Cell::new(0));
        let counted = runs.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let counted = counted.clone();
            let ui = move || {
                counted.set(counted.get() + 1);
                given.get()
            };
            themed(ui, || container().child(look_at_text()))
        });
        h.configure(id, 64, 64, 1.0);
        h.step();
        for ui in [light, light.with_density(Density::Compact)] {
            let before = runs.get();
            given.set(ui);
            h.step();
            assert_eq!(
                runs.get(),
                before + 1,
                "worked out again for {:?}",
                ui.sizes()
            );
        }
    }

    /// A box in the text's colour, read through `look`.
    fn look_at_text() -> Container {
        container()
            .width(8.0)
            .height(8.0)
            .background(look().color(|c| c.text))
    }

    #[test]
    fn text_and_sub_stay_two_levels_on_every_material() {
        // With the island's opposite behind it, where both are pushed
        // furthest, `text` still reads at least 1:1 more than `sub` on the
        // island, the measure of `a_light_theme_keeps_two_levels_of_text`.
        for theme in THEMES {
            for flavor in theme.flavors {
                for (index, accent) in flavor.accents.iter().enumerate() {
                    let palette = flavor.scheme(index).palette();
                    for material in Material::ALL {
                        let colors = Ui::new(palette).with_material(material).colors;
                        let island = colors.grounds(palette.worst_backdrop())[0];
                        let [text, sub] =
                            [colors.text, colors.sub].map(|ink| contrast(ink, island));
                        assert!(
                            text >= sub + 1.0,
                            "{} {} {} {material:?}: {text:.2} and {sub:.2}",
                            theme.name,
                            flavor.name,
                            accent.name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn on_glass_the_text_moves_away_from_the_backdrop() {
        let choice = ThemeChoice::default();
        let (dark, light) = (choice.palette_of(false), choice.palette_of(true));
        let glass = |palette| Ui::new(palette).with_material(Material::Glass).colors;
        assert_eq!(glass(dark).island.a, Material::Glass.opacity());
        assert!(glass(dark).sub.luminance() > dark.sub.luminance());
        assert!(glass(light).text.luminance() < light.text.luminance());
    }

    #[test]
    fn a_colour_set_keeps_its_contrast() {
        // A dim and a warning's text set paler than their floors (the
        // warning's at 2.5:1, under even the 3:1 a hue keeps on glass)
        // stay as set on a solid island, and on glass darken only as far
        // as they read when solid.
        let mut palette = ThemeChoice::default().palette_of(true);
        palette.dim = Color::from_hex(0xa0a0a0);
        palette.warning.text = palette.danger.fill;
        assert_eq!(Ui::new(palette).colors, palette);
        let glass = Ui::new(palette).with_material(Material::Glass).colors;
        assert!(glass.dim.luminance() < palette.dim.luminance());
        assert!(glass.warning.text.luminance() < palette.warning.text.luminance());
        assert_eq!(
            glass.unreadable(Color::BLACK).map(|(role, _)| role),
            Some("dim")
        );
        let dim_reading = Palette {
            dim: glass.text,
            ..glass
        };
        assert_eq!(
            dim_reading.unreadable(Color::BLACK).map(|(role, _)| role),
            Some("warning_text")
        );
    }

    /// Two points of a surface given the default dark `Ui`, then the light
    /// one in place: the island, and a point of what it holds.
    struct Repainted {
        before: [[u8; 4]; 2],
        /// 100 ms into the change.
        midway: [[u8; 4]; 2],
        /// Once it has settled.
        after: [[u8; 4]; 2],
        /// As a surface built light draws them.
        light: [[u8; 4]; 2],
        /// How many times the surface was built.
        builds: u8,
    }

    /// `content` on an island 16 px in from the edges of a 120 by 64
    /// surface, read at (4, 4) and at `at`.
    fn repainted(motion: Motion, content: fn() -> Container, at: (u32, u32)) -> Option<Repainted> {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::{Duration, Instant};

        use guido::testing::Headless;

        use crate::primitives::island;

        let mut h = Headless::new_or_skip()?;
        let choice = ThemeChoice::default();
        let [dark, light] =
            [false, true].map(|light| Ui::new(choice.palette_of(light)).with_motion(motion));
        let mount = |h: &mut Headless, given: RwSignal<Ui>, builds: Rc<Cell<u8>>| {
            let id = h.surface(SurfaceConfig::new(), move || {
                builds.set(builds.get() + 1);
                provide_ui(given);
                island::paint(container().width(fill()).height(fill()))
                    .padding(16.0)
                    .child(content())
            });
            h.configure(id, 120, 64, 1.0);
            h.step();
            id
        };
        let read = |h: &Headless, id| [(4, 4), at].map(|(x, y)| h.read_pixel(id, x, y));

        let look = create_signal(dark);
        let builds = Rc::new(Cell::new(0));
        let id = mount(&mut h, look, builds.clone());
        let before = read(&h, id);
        let start = Instant::now() + Duration::from_millis(50);
        look.set(light);
        h.step_at(start);
        h.step_at(start + Duration::from_millis(100));
        let midway = read(&h, id);
        let settled = start + Duration::from_secs(2);
        h.step_at(settled);
        h.step_at(settled);
        let after = read(&h, id);
        let fixed = mount(&mut h, create_signal(light), Rc::default());
        Some(Repainted {
            before,
            midway,
            after,
            light: read(&h, fixed),
            builds: builds.get(),
        })
    }

    /// `content`, read at `at`, follows a new palette in place, easing into
    /// it as the island does.
    fn eases_in_place(content: fn() -> Container, at: (u32, u32)) {
        let Some(Repainted {
            before,
            midway,
            after,
            light,
            builds,
        }) = repainted(Motion::Full, content, at)
        else {
            return;
        };
        assert_ne!(before, after, "the palette changed nothing");
        assert_eq!(after, light, "not as a light Ui draws it");
        assert_eq!(builds, 1, "built again");
        // Both on their way: neither the old colour nor the new.
        for (i, point) in ["the island", "the content"].into_iter().enumerate() {
            assert!(
                midway[i] != before[i] && midway[i] != after[i],
                "{point} didn't ease: {:?} {:?} {:?}",
                before[i],
                midway[i],
                after[i]
            );
        }
    }

    /// A button's middle, before its label.
    fn button() -> Container {
        use crate::components::button::button;
        container().child(button().label("Go"))
    }

    fn inside_the_button() -> (u32, u32) {
        use crate::foundations::control::Size;
        (19, 16 + Size::default().height() as u32 / 2)
    }

    #[test]
    fn a_new_palette_repaints_in_place_easing_into_it() {
        eases_in_place(button, inside_the_button());
    }

    #[test]
    fn a_colour_picked_by_a_role_that_captures_eases() {
        eases_in_place(
            || {
                let hue = Hue::Danger;
                container()
                    .width(40.0)
                    .height(24.0)
                    .background(look().eased(move |c| c.hue(hue).fill))
            },
            (36, 28),
        );
    }

    #[test]
    fn a_components_colour_prop_eases() {
        use crate::components::notification::app_icon;
        use crate::foundations::icons::Icon;
        // The tile's gradient, above its icon.
        eases_in_place(
            || container().child(app_icon(Icon::Bell, look().color(|c| c.accent.fill))),
            (36, 19),
        );
    }

    #[test]
    fn the_sparklines_line_eases() {
        use crate::components::sparkline::sparkline;
        eases_in_place(
            || {
                // At the top, where the line covers the first row of
                // pixels whole: none of the island shows through.
                let samples = create_signal(vec![100.0; 8]).read_only();
                container()
                    .width(88.0)
                    .child(sparkline(samples, 8).top(100.0))
            },
            (60, 16),
        );
    }

    #[test]
    fn a_ui_made_from_the_look_follows_it() {
        eases_in_place(
            || {
                let t = look();
                container().child(move || {
                    provide_ui(move || t.ui().with_density(Density::Compact));
                    container()
                        .width(40.0)
                        .height(24.0)
                        .background(look().eased(|c| c.accent.fill))
                })
            },
            (36, 28),
        );
    }

    #[test]
    fn without_motion_a_new_palette_shows_at_once() {
        let Some(Repainted {
            before,
            midway,
            after,
            light,
            builds,
        }) = repainted(Motion::None, button, inside_the_button())
        else {
            return;
        };
        assert_ne!(before, after);
        assert_eq!(after, light);
        assert_eq!(midway, after, "eased with no motion");
        assert_eq!(builds, 1, "built again");
    }
}
