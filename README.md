<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/wordmark-dark.svg">
    <img alt="styletto" src="assets/wordmark-light.svg" height="140">
  </picture>
</p>

# styletto

A design system for [Guido](https://github.com/MalpenZibo/guido): tokens,
themes and components. Every piece of it is on show in the
[storybook](storybook).

## Principles

- Every length, corner, icon and type size is a step on a scale, and every
  colour a name in the palette. Components hold no numbers of their own.
- What a surface chose (palette, density, shape, fonts, material, motion)
  is its `Ui`, declared once and read where the tree is built.
- Components are `#[component]`s: closed, set through their props, slots
  and callbacks. Primitives stay plain functions returning a container, so
  an app can add to them: `icon`, the text styles, `centered_row`,
  `pressable`, `island`.

## Layout

| Module | Holds |
|---|---|
| `foundations` | `tokens`, `typography`, `control`, `motion`, `icons`: the scales, the motion and the icons |
| `theme` | `palette`, `presets`, `material`, `wallpaper`, `config`: colours by meaning, the preset themes, themes from a colour, the wallpapers read for theirs, the `[appearance]` table |
| `primitives` | `text`, `layout`, `island`, `satellite`: plain functions an app can add to |
| `components` | the components, below |

`Icon`, `icon`, `centered_row`, `Ui`, `provide_ui`, `ui`, `look` and `Look`
are also at the root. Below, a path such as `tokens::size` is short for
`foundations::tokens::size`.

## Use

```toml
[dependencies]
styletto = { git = "https://github.com/MalpenZibo/styletto", features = ["serde", "wallpaper"] }
```

```rust
use styletto::{Icon, themed};
use styletto::components::list::row;
use styletto::components::toggle::toggle;

// The Ui as the config asks, and a wallpaper once it is read: worked out
// once per change.
let drawn = create_memo(move || config.with(|c| c.appearance.ui(system_dark.get())));

// Built again only when what sizes it changes; repainted in place on a new
// palette. Or a fixed Ui, as Ui::default().
themed(drawn, || {
    row("Wi-Fi")
        .icon(Icon::Wifi)
        .subtitle("Home 5G")
        .trailing(toggle(true))
})
```

| Feature | Adds |
|---|---|
| `serde` | `theme::config::UiConfig`, read from an `[appearance]` table and written back, and the controls that edit it (`components::appearance`) |
| `material` | themes worked out from a seed colour |
| `wallpaper` | themes worked out from an image, read on a thread of its own, once per path until read again (`theme::wallpaper`; implies `material`) |

## The Ui

`provide_ui` declares it for a subtree: a `Ui`, or a signal or a closure of
one, worked out once per change. `themed(ui, build)` is what an app usually
wants: it provides `ui` to what `build` makes and builds it again only when
`Ui::sizes()` changes, so a new palette or material repaints it in place.
Each choice moves the scales rather than setting numbers:

| Choice | Values | What it does |
|---|---|---|
| `Density` | `Compact`, `Default`, `Comfortable` | moves type, controls, icons, tiles and space up to `S8` one step |
| `Shape` | `Square`, `Soft`, `Round` | moves every radius one step; a pill stays a pill |
| `Material` | `Solid`, `Translucent`, `Glass` | the island's opacity (1, 0.86, 0.75) and blur (0, 24, 48); the text on it darkens or lightens as far as it must to read with the island's opposite behind |
| `Motion` | `Full`, `Reduced`, `None` | springs, fades only, or instant changes |
| `Fonts` | `sans`, `mono` | the families, the system's by default |

A component reads it two ways, both while it is built:

- `ui()` is the `Ui` as it is now, for what sizes and times what is built:
  density, shape, fonts and motion (`Ui::sizes()`). A change of these
  builds the tree again: `themed` reads `Ui::sizes()` and nothing else of
  the `Ui`.
- `look()` is a `Copy` handle on the `Ui` as it changes, for its colours
  and material. Read inside the closures a widget's properties take
  (`t.colors()`, `t.color(|c| c.text)`, `t.material()`), they follow a new
  palette in place: a wallpaper that changes, or the system turning dark,
  repaints the tree without building it again, and whatever is open,
  typed or scrolled stays so. The role `t.color` and `t.eased` take is a
  closure that picks a colour from the palette, and may read what it
  captures: `t.color(move |c| c.hue(hue).fill)`, or a signal, as
  `t.eased(move |c| if on.get() { c.accent.fill } else { c.surface })`.
  `t.ui()` is the whole `Ui` as it changes, for a subtree that provides
  its own from it and follows it:
  `provide_ui(move || t.ui().with_density(Density::Compact))`.
- `t.eased(|c| c.surface)` is a colour that eases at `motion::color()`,
  into a new palette as into a new state, for a property with no motion of
  its own; one with a motion (a hover at `motion::hover()`, a toggling
  fill) moves with that. Under `Motion::None` it changes at once, and keeps
  no motion. `motion::eased_color(value)` does the same for a colour, or a
  shadow of one, worked out of more than the palette (`colors_ease()` says
  whether colours ease at all). Every colour of the components eases so, those an app gives
  them too (a dot's, a ring's, an app icon's, rolling digits'), a text
  field's text, placeholder, cursor and selection, an app icon's gradient
  and a sparkline's line. Nothing changes at once but under
  `Motion::None`.

A colour copied while building (`ui().colors.text`) stays as it was: an app
that no longer builds its tree again on a new palette reads its own colours
through `look()` too.

## Tokens

A step reads its value with `.px()` in the current `Ui`.

| Scale | Steps | px |
|---|---|---|
| `tokens::size` | `S0` `S0_5` `S1` `S2` `S3` `S4` `S6` `S8` `S10` `S15` `S20` `S30` `S40` `S60` `S80` `S100` | 0 2 4 8 12 16 24 32 40 60 80 120 160 240 320 400 |
| `tokens::radius` | `NONE` `XSMALL` `SMALL` `MEDIUM` `LARGE` `XLARGE` | 0 4 8 16 32 64 |
| `tokens::icon` | `XSMALL` `SMALL` `MEDIUM` `LARGE` `XLARGE` | 12 16 20 24 32 |
| `tokens::tile` | `XSMALL` `SMALL` `MEDIUM` `LARGE` `XLARGE` `XXLARGE` | 16 24 32 40 48 64 |

Up to `S8` a size step is space and follows the density; past it, it is a
layout length and stays. A length an app lays out with, a panel's width or
a list's height before it scrolls, follows the density through
`size::scaled(length)`: ×0.875 compact, ×1.25 comfortable
(`Density::scale`), about as much as what a panel holds grows, so it keeps
its proportions. A tile step also sets its corners, the icon on it
and the letter of an initial; an icon step sets the stroke of a ring.

Corners: a box is a squircle (`radius::shape`, or `Radius::corners`), a pill
or a circle is round (`radius::pill(height)`). A box around another takes
`radius::around(inner, padding)`, so the curves stay concentric.

Fixed values: `stroke::HAIRLINE` 1, `RING` 1.5, `BOLD` 2, `HEAVY` 3;
`opacity::DISABLED` 0.4, `RING` 0.7 (a field's ring in the accent),
`SELECTION` 0.4 (the most of the accent a field's selection takes; each
palette's `selection` takes as much of it as keeps its text at 3:1, for
each material),
`STACKED` 0.6 (a share of `surface`'s alpha, for the edge below a folded
group of notifications), `HALO` 0.5 (a share of the glow's alpha, for the
island's halo); `glow::SMALL` 4, `MEDIUM` 8, `LARGE` 16 (the island's
halo); `scrollbar::WIDTH` 4, `HOVER_WIDTH` 8, `MARGIN` 2,
`MIN_HANDLE` 24, the same in every density; `indicator::DOT` 8 (a page
indicator's dot, the gap between two, and the least a badge's dot shrinks
to) and `CURRENT` 24 (the current
page's pill), the same in every density too, as a dot that shrank with the
space would all but vanish in compact.

A fill lights up by a `lift` step. Under the pointer, an opaque one moves
`HOVER` 0.12 toward white and a translucent one gains `HOVER_ALPHA` 0.06 of
alpha. An app icon's tile runs from `SHEEN` 0.15 lighter than its colour to
as much darker.

A `shade` step is the alpha of the black (`palette::SHADE`) that sinks a
box into a fill, as the box behind an active quick tile's icon: `REST`
0.13, `HOVER` 0.2.

The island's `shadow` is that black at `ALPHA` 0.3, `DROP` 4 below it,
blurred by `BLUR` 12. Around a state worth seeing its halo starts
`HALO_INSET` 4 inside the edge.

A hue tints what it fills or lights at a `tint` step, the alpha of its fill
over the island: `SUBTLE` 0.12 a card's wash while it warns or does well,
`MEDIUM` 0.15 a card's while it alerts and a glow, `STRONG` 0.2 behind a
small mark's ink (a tag, an alert's tile). What sits on a tint is drawn in
the hue as text.

## Type

Six categories in four sizes, `LARGE` to `XSMALL` (`typography::TITLE_SMALL`
and so on). Sizes are even and line heights multiples of 4; the density may
move a size within its category.

| Category | Use | Sizes / line heights |
|---|---|---|
| `DISPLAY` | big numbers: a clock, a level | 48/48 40/40 32/32 24/24 |
| `TITLE` | a page's or a group's title | 24/28 20/24 16/20 14/20 |
| `LABEL` | one line of a control | 16/20 14/16 12/16 10/12 |
| `BODY` | running text | 16/24 14/20 12/16 10/16 |
| `MICRO` | uppercase, tracked: section names | as `LABEL` |
| `MONO` | code, logs | as `LABEL` |

`.em()` is the semibold weight, `.tabular()` keeps figures one width.
`.typeset(kind)` applies a type to a text or an input; a `Resolved` style
off the scale works too. `text::{label, caption, heading, overline, figure}`
are the common ones, already coloured.

## Controls

`control::Size` is the height of a control and what follows from it:

| Size | `Xxs` | `Xs` | `Sm` | `Md` | `Lg` | `Xl` |
|---|---|---|---|---|---|---|
| Height | 20 | 24 | 28 | 36 | 40 | 48 |

`icon()`, `inline()` (the padding from a pill's ends), `gap()` and
`label()` give the rest, already moved for the density.

## Colours

A theme sets a `Scheme`: `background`, `text`, `sub`, `dim`, and four hues
(`accent`, `success`, `warning`, `danger`), each a `SchemeHue`: its fill
`color`, the colour that reads `on` it and its `text`. An on-colour left
unset is worked out to reach a 4.5:1 contrast. `Scheme::named()` lists its
colours by the names of the config file (`background`, `on_accent`,
`accent_text`, ...) and `Scheme::with(name, colour)` sets one by that name.

Components read the `Palette` derived from it: `island`, `surface`,
`surface2`, `line`, `text`, `sub`, `dim`, the four hues `accent`, `success`,
`warning` and `danger`, `ripple`, and `selection`, a field's selection: the
accent's fill at as much of `opacity::SELECTION` (0.4) as keeps `text` at
3:1 on it, on a field, worked out when the palette is built and again for
each material, as its text is (0.39 to 0.4 on a solid island, from 0.26
with the palest accents to 0.4 on a see-through one). Each hue is a `HueColors`: its `fill`,
the colour `on` it and its `text`; `palette.hue(Hue::Danger)` reads one by
its `Hue` (`Scheme::fill(hue)` its theme's colours), `Hue::ALL` lists the
four and `hue.name()` names one. `palette.ink(hue)` is the hue as text, or
`text` without one (`None`), what a component toned or left neutral writes
in. One colour is the same in every theme: `palette::SHADE`, the black of a
shadow or of a box sunk into a fill.

Two inks are worked out where they are drawn, as Material 3 does. A
switch's knob is `accent.on` while on, and `sub` while off: as `sub` reads
on `surface2`, the track, it keeps the 3:1 WCAG asks of a control even in a
hovered or chosen row, which tints the track again. Where it would not, as
on a see-through island, where `sub` keeps only 3:1 on the bare track
(below), the knob is `text` while off. An app's icon
(`app_icon`) draws its glyph in black or white, whichever has the more
contrast on the app's colour.

A hue's `fill` is what paints a surface, a bar, a ring or a border; `on` is
the text on that fill; `text` is the hue as text or an icon on the island,
`surface` and `surface2`, at 4.5:1 on all three (3:1 on a see-through
island, below).
Unless the theme or the config sets one,
it is the fill moved in lightness only, in Oklab, with its hue and chroma
kept (as far as sRGB holds them), just as far as it must to read: a fill
that already reads is its own text, and a pale one turns deeper, not grey.

`text` is body text and `sub` secondary text (captions, placeholders, small
labels): both reach 4.5:1 on the island and on `surface` and `surface2`,
and `text` 7:1 on the island besides (WCAG's AAA), so that it reads at a
glance and leaves `sub` room to be a second, lighter level. `dim` is for
icons, dividers and large text, never small text, and reaches 3:1. A preset
takes these from the theme's own colours for text, not from its borders or
overlays, and a darker text tone the theme declares where its own only just
reads: Tokyo Night Day's text is Tokyo Night Light's foreground, and Day's
blue its `sub`; Kanagawa Lotus's is its darker ink, `lotusInk2`. Where none
reads well enough, `Scheme::readable` works one out, and does the same for a
theme from a seed: `text` moved in lightness, in Oklab, keeping its hue as a
hue's text does, `sub` and `dim` mixed toward `text`, each as little as
gets it there (Day's blue is darkened a little so). On a background where
not even black or white reaches 7:1, `text` goes all the way to the one it
moves toward. Colours set in `[appearance.dark]` and `[appearance.light]` are
used as given, and the text they leave to the source is made to read again
on a `background` set there; a hue's text left to the source follows its
fill.

On a see-through island the text is adjusted again: `Ui::colors` holds it
as dark or as light as it must be to read with the island's opposite showing
through, black behind a light island or white behind a dark one, to the
4.5:1 of every ground there, not to the 7:1 it has on a solid one. `sub` is
held there to 3:1 with that worst behind it, the floor `dim` has (WCAG's for
icons and large text), not 4.5:1: the island turns grey, and at 4.5:1 `sub`
is pushed as far as `text`, to the same colour, and the two levels of text
are one. On a solid island it keeps 4.5:1. Each hue's `text` is moved the
same way, in lightness only, to 3:1 too: with white behind dark glass, a
hue that reads at 4.5:1 is so light it is all but white, as light as the
text, and a warning no longer looks like one. Where not even black or white
would read, the island is drawn denser. A colour set below its floor keeps
the contrast it was set with. Paint text with the `Ui`'s colours
(`look().colors()`), not with a `Scheme`'s or a theme's own colours, or it
won't follow.

The preset themes, in `presets::THEMES`, each with dark and light flavours
and its own accents: Tokyo Night, Catppuccin, Nord, Gruvbox, Dracula, Rosé
Pine, Everforest, Kanagawa, one file each in `src/theme/presets/`.
`material::schemes(seed, variant, contrast)` makes both sides from one
colour with Material Color Utilities.

## Appearance config

With the `serde` feature, `UiConfig` reads this table and `ui(system_dark)`
resolves it. Names match in any case; a wrong one is logged and left out.

```toml
[appearance]
# One source of colours: a theme...
theme = "catppuccin"
dark_flavor = "mocha"
light_flavor = "latte"
accent = "mauve"         # on each side whose flavour has it
light_accent = "peach"   # a side's own (dark_accent, light_accent) wins over it
# ...or a seed colour...
# seed = "#6750a4"
# ...or a wallpaper (it wins over a seed, which wins over a theme).
# wallpaper = "~/Pictures/wallpaper.jpg"
# wallpaper_color = 1    # which of the colours it yields, best first; 0, the best, by default
variant = "tonal-spot"   # for a seed or a wallpaper: vibrant, expressive, content, ...
contrast = "standard"    # medium, high

mode = "auto"            # light, dark, or as the system prefers
density = "default"      # compact, comfortable
shape = "soft"           # square, round
material = "translucent" # solid, glass
motion = "full"          # reduced, none

[appearance.fonts]
sans = "Inter"
mono = "JetBrains Mono"

# Any colour of either side, by its Scheme::named() name, set over the source.
# A hue's are its name, on_ before it and _text after it: danger, on_danger,
# danger_text.
[appearance.dark]
accent = "#ff79c6"
on_accent = "#1a1b26"
accent_text = "#ff92d0" # the accent as text; worked out from accent if unset

[appearance.light]
background = "#fdf6e3"
```

`UiConfig` is also `Serialize`, so an app that lets the user change these can
write the choice back with its own format crate (`toml::to_string(&config)`).
It writes only what differs from the defaults, with its keys at the top level
so the app can put them under any table:

```toml
theme = "catppuccin"
dark_flavor = "macchiato"
accent = "peach"
density = "compact"
material = "glass"

[dark]
accent = "#ff79c6"

[fonts]
sans = "Inter"
```

## Motion

`motion::spring()` is the island's spring for every fold and growth,
`tactile()` the quicker one of a press, `hover()` and `color()` eased
changes, `easing(ms)` and `springing(config)` custom ones. All follow the
`Motion` of the `Ui`. `toggling(rest, hovered)` moves a fill that turns on
and off, as a toggling button's or a tile's: a hover at `hover()`, a change
of state at `color()`, told apart by the fill it leaves. `fading(enter, leave)` gives a value one transition
for a view coming in and another for it going out. `arriving` cross-fades a
view in, `entering` opens a panel, `shake()` plays a refusal and `armed()`
ignores presses just after a view appears.

A press (`pressable`) shrinks what it presses to one of three steps: a
button or a small tile to `PRESS_SCALE` (0.95); a wide surface, as a wide
tile, to `PRESS_SCALE_WIDE` (0.97), less, as a little already moves its ends
a long way; a small target, as the icon box in a wide tile, to
`PRESS_SCALE_INNER` (0.9), more, so its press shows.

## The island

`island::island` is one shape springing around whatever view it shows. Its
look comes in pieces, for every shape that should match it: `paint` fills
and blurs it as the material asks, `border` rims it, `shadow(glow)` is what
it casts, and `edge` is the border and the shadow together. With a solid
material `paint` still declares the shape to the compositor, so a
compositor rule that blurs a surface with no blur of its own (niri's
`layer-rule` `blur`) leaves the space around it alone. Merging an island
with what comes out of it, the holder takes the edge,
`edge(holder.merge_children(reach))`, what comes out takes the paint, and
the island leaves its shadow to the holder with `.shadow(Shadows::none())`.
A `satellite` is such a shape: it splits off the end of its host and merges
back.

## Icons

[Lucide](https://lucide.dev) icons, the unmodified SVG files of
`lucide-static` 1.61.0 in [`icons/`](icons), under Lucide's licence. To add
one, copy its file there and add a line to the `icons!` table in
`src/foundations/icons.rs`. The module also draws spinners, initials,
pictures and artwork; the island and its satellites are in `primitives`.

## Components

The modules of `components`:

| Module | What |
|---|---|
| `button` | buttons, with a label, an icon or both, an action or a toggle as a tile is; the bar's pills; `pressable` |
| `toggle`, `slider`, `input` | a switch, a slider and a slider row (to 100%, or past it to a max; a wheel notch moves it by a step, 5% unless set), text and password fields |
| `list` | rows (a checked one marks the choice in use; one in a tag's `Tone`, as warning or danger, has its title, icon, hover and press in that hue), sections, empty states, page headers with a subtitle, pages that slide and a page indicator for where a flow of them stands, a list that scrolls past a height with the theme's scrollbar |
| `tile` | quick-settings tiles, wide and small; a wide one's subtitle takes a second line when it needs one, and tiles side by side in a `Flex::row()` are as tall as the tallest |
| `tag` | read-only pills for a state, tinted by a `Tone` (neutral, accent, success, warning, danger), glowing if asked |
| `badge` | dots, icon tiles |
| `level` | `Level`, how a value stands, and its ink; the level indicator: the icon a value picks from a set, its percentage beside it if asked |
| `card` | level figures and stat cards coloured by their `Level` |
| `progress`, `sparkline`, `roll` | bars, rings, recent samples, rolling digits |
| `notification` | notifications, alone and grouped |
| `settings` | the pieces of a settings panel: a titled group of settings, a setting with its name and a note over what sets it |
| `choices` | one of several values to choose: a row of buttons that wraps, or a window's tabs on one line |
| `appearance` | controls that edit a `UiConfig` (with the `serde` feature): the theme, the mode, a side's colours, the fonts, the layout, and a panel of them |

A button is one component, with a label, an icon before it, or the icon
alone in a circle (`button().icon(icon)` with no label).
Its `Variant` (`Neutral`, `Accent`, `Danger`) sets its colour and its `Fill`
how it rests: `Solid` in the variant's fill (`surface` for a neutral one)
with the ink on it, `Ghost` clear until hovered, its ink the hue as text.
An action takes `on_click`. A toggle takes `active` and `on_toggle`: while
`active` it is on, as a tile: filled with the accent, its ink `accent.on`,
whatever its variant and fill; a toggle is a neutral or ghost button (a
solid accent one is filled with the accent already, and looks the same on
and off). Given both, a press calls `on_click`, then `on_toggle`. Hovered,
a neutral fill steps to the next surface and a hue brightens; disabled, on
or off, it fades and takes no press.

Whatever turns on and off (a toggling button, a tile, a small tile, the
switch) is controlled, as GTK's toggle button: the app passes `active`, a
press calls `on_toggle` with the opposite, and the app stores what it
keeps, or not. `ui::active_fill(active, off)` and `active_ink(active, off)`
are its fill and its ink: the accent and `accent.on` while on, `off` while
not. A bar's `pill` is not a toggle: its `active` marks the one chosen (a
workspace shown) and a press calls `on_click`, as a list row's `selected`.

A tag shows and takes no press. Its `Tone` tints it at `tint::STRONG`
(`surface` while neutral), and `Tone::ink()` is the colour for what it
holds: `text` while neutral, else the hue as text. Both read the palette as
it is now: in a property's closure, they follow a new one. `Tone::hue()` is its
`Hue`, none while neutral; a list row takes the same `Tone`. With `glow` a toned tag
lights around it, as the island's privacy tag does.

```rust
use styletto::components::button::{Fill, Variant, button};
use styletto::components::tag::{Tone, tag};

button().label("Auto").active(auto).on_toggle(move |on| auto.set(on));
button().label("Forget").variant(Variant::Danger);
button().icon(Icon::X).fill(Fill::Ghost).size(Size::Sm);
tag().tone(Tone::Warning).glow(true).child(icon(Icon::Mic, size, Tone::Warning.ink()));
```

A text field, `input::field`, is a pill on the surface holding the app's
`text_input` and a trailing button if given. Its ring is the accent at
`opacity::RING`, `danger` while `refused` (the last answer was refused and
nothing new is typed); a ring of the app's own, `.ring(color)`, takes the
place of both, and a closure giving `None` hands it back to that rule. The
`text_input` is the app's; `text::PaletteColors` gives it (or a
`password_input`) the palette's colours, `text`, `sub` for the placeholder,
`accent.text` for the cursor and `selection`, easing into a new palette:
`text_input(typed).palette_colors()`. `password_field` is a field whose end submits, or spins while
`busy`; `held`, it fades.

```rust
use styletto::components::input::field;

field().refused(refused).child(text_input(typed));
// Clear while it sits in a grid, the rule's ring otherwise.
field().refused(refused).ring(move || in_grid.get().then_some(Color::TRANSPARENT));
```

The app decides a level, with its own thresholds, and styletto draws it:
`Level::of(value, warn, alert)` when high is bad, `Level::below(value, warn,
alert)` when low is, and `Level::Good` for better than normal, a battery
charging. `Level::hue()` is its `Hue`: success, warning or danger, none
while normal. `level_ink` is `text` while normal, else that hue's `text`
(in a property's closure, it follows a new palette); a card's tint and glow
take its `fill`.

The indicator's sets of icons are data, each icon from the value it starts
at: `Levels::BATTERY` rounds its bars up, a bar staying while there is
charge in its third: empty below 15%, one bar from 15%, two past a third,
three past two thirds, so one bar and the empty one fall where a battery is
a warning and an alert; `Levels::WIFI` takes the nearest number of arcs,
none below 17%, then one, two and three from 17, 50 and 83%; `Levels::VOLUME` has no waves below 1%, one from 1% and two from 50%. A
state off the scale takes its own icon in place of the set's:

```rust
use styletto::Icon;
use styletto::components::level::{Level, Levels, level_indicator};

level_indicator(Levels::BATTERY, 12.0).level(Level::below(12.0, 30.0, 15.0)).percent(true);
level_indicator(Levels::BATTERY, 50.0).icon(Icon::BatteryCharging).level(Level::Good);
level_indicator(Levels::VOLUME, 0.0).icon(Icon::VolumeOff);
level_indicator(Levels::WIFI, 70.0);
```

A settings panel is made of three pieces, which the appearance controls
are built from too, so an app's own settings sit beside styletto's spaced
the same. `setting_group(title)` is a section's title with its settings
under it, set in from the edge; `setting(name)` is a name, a `note` at the
other end of the line if given, and what sets it underneath; `choices(options,
chosen)` is a row of buttons that wraps, one per option, the one whose value
is `chosen` `active`. It is generic over the value, any `Copy + PartialEq`
type, and controlled as a toggle is: a press calls `on_choose` with the
option's value, and the app sets `chosen`. With `enabled` false, its
options fade and take no press, as a disabled button does.

```rust
use styletto::components::appearance::layout_picker; // with the `serde` feature
use styletto::components::choices::choices;
use styletto::components::settings::{setting, setting_group};

#[derive(Clone, Copy, PartialEq)]
enum Edge { Top, Bottom }

let edge = create_signal(Edge::Top);
setting_group("Bar").child(
    setting("Edge").note("Moves at once").child(
        choices(vec![("Top".into(), Edge::Top), ("Bottom".into(), Edge::Bottom)], edge)
            .on_choose(move |chosen| edge.set(chosen)),
    ),
);
// Beside one of styletto's, editing the app's config.
let config = create_signal(UiConfig::default());
setting_group("Layout").child(layout_picker(config));
```

`tabs(options, chosen)` is the same choice as a window's tabs: ghost
buttons on one line, the current one `active`, as tall as a header's
buttons, to sit between its title and its close button; it takes the same
values, `enabled` and `on_choose`. Where a flow of pages stands, as a
first start's steps, is a `page_indicator(count, current)`: a dot for each
page in `dim`, the current one stretched into a short pill in the accent
as text, which reads on the island on every material (the fill may not).
It takes no press: the flow's own buttons move it, Back and Next at the
footer's ends.

```rust
use styletto::components::choices::tabs;
use styletto::components::list::page_indicator;

let tab = create_signal(Tab::Look);
tabs(vec![("Bar".into(), Tab::Bar), ("Look".into(), Tab::Look)], tab)
    .on_choose(move |chosen| tab.set(chosen));
page_indicator(4, step); // step: the page showing, from 0
```

## Appearance controls

With the `serde` feature, `components::appearance` lets the user choose
what the `[appearance]` table holds. Each control edits an
`RwSignal<UiConfig>` the app holds, and each change is written to it at
once; the app turns it into its `Ui` (and saves it) as it changes.

| Component | Edits |
|---|---|
| `theme_picker` | where the colours come from (`Source`): a preset with, on a card for each side, its flavour and accent; a seed or a wallpaper, with its variant and contrast; or every colour by hand, in a `color_editor` for each side |
| `mode_picker` | light, dark or as the system prefers, and which side of which theme that shows |
| `color_editor` | one side's colours, by name, as hexes; one left empty is worked out |
| `font_picker` | the sans and the mono family, among those installed, or the system's |
| `layout_picker` | the density, the shape, the material and the motion |
| `appearance` | all of them, in sections (theme and mode, layout, fonts) |

What reaches the system stays the app's, and is handed in, so styletto
needs no portal: what the system prefers, and a callback that opens a file
chooser for a wallpaper and sets the path chosen. The font families
installed (`Family`: a name, whether it is monospaced, whether it draws the
semibold weight) are handed in as `families` too, or left out: the font
picker then reads them itself with `installed_families()`, from Guido's
`font_families()`, the fonts it draws with: the system's and those the app
gave to `load_font`. Guido reads them once, on a thread of its own, and
every picker shares what it found; a family is offered if it writes Latin
(a face of it draws `a` and one draws `A`), is monospaced if every face of
it is, and is semibold if a face draws the weight of the emphasised type,
within a step (a variable face, anywhere along its `wght` axis).

A wallpaper is read by styletto, once per path, on a thread of its own, and
kept for the whole process until it is read again: `wallpaper::picture(path)` is how far it is
read (`Reading`, `Read` with the colours it yields best first, as
`material::seeds` gives them, or `Failed` with why). `UiConfig::ui` never
reads it on the thread that calls it: until it is read, the theme shows, and
the closure that called it runs again once it is, as every surface that
asked shares the one read. The theme picker reads the same store: it notes
"Reading…" or why the picture can't be read, and shows the colours found to
choose the one the sides are made of (`wallpaper_color`). Call them on the
`App`'s thread. An app whose first frame must already be in the wallpaper's
colours, as a lock screen or a greeter, or a program with no running `App`,
as a one-off render, calls `config.ui_blocking(system_dark)` once, before it
draws and not in a reactive closure: it reads a picture not read yet on the
calling thread, which waits for it, and puts it in the same store, so the
surfaces after it take it at once and nothing reads it again. A picture that
can't be read is logged and the theme shows, as with `ui`; without the
`wallpaper` feature it is `ui`.
A picture changed at the same path is read again with
`config.update(UiConfig::reread_wallpaper)`, from the app's own file
watcher (styletto watches no file) on the `App`'s thread, or on Enter over
the same path in the picker's field: it starts again from the best colour,
the old picture's until the new one lands. `wallpaper::reread(path)` reads
again and keeps the colour chosen.
`UiConfig::wallpaper_path()` is the path read (`~/` expanded);
`set_wallpaper(path)` sets a new one, starting from its best colour, as the
app's chooser does, and what the chooser says, as that there is none, is the
app's `chooser_note`. Chosen as the source, a
wallpaper is an empty path until one is set, which `UiConfig` takes as none
chosen yet: the theme shows meanwhile. A seed is offered with the `material` feature, a
wallpaper with `wallpaper`. The panel fills the width it is given, wrapping
its options, as in a quick-settings panel; the app scrolls it.

```rust
use styletto::components::appearance::{appearance, mode_picker, theme_picker};

let config = create_signal(UiConfig::default()); // or as the app loaded it
appearance(config)
    .system_dark(system_dark)
    .families(families)            // or left out: installed_families()
    .on_pick_wallpaper(move || chooser.send(()))
    .chooser_note(note);           // what the app's chooser said, if anything

// A first-start step: only the theme and its mode.
container()
    .child(mode_picker(config).system_dark(system_dark))
    .child(theme_picker(config).system_dark(system_dark));
```

A preset's card sets its side's accent with `UiConfig::set_accent(light,
name)`, which writes as little as keeps both sides as they show: no key
while each shows its flavour's own, `accent` while both show the same one,
and `dark_accent` or `light_accent` for a side whose accent differs;
`set_theme(name)` picks a theme in its own flavours and accents. A field to type in (a colour, the seed) follows a
change made to the config elsewhere, but not while it has the focus, so the
text never moves under the cursor; it catches up once it is left, over an
edit not finished.

`Source::of(&config)` tells where a config's colours come from and
`source.apply(&mut config, &wallpaper_colors)` switches it, leaving out
what the source before had set but the theme chosen, to go back to; it
starts from what shows, a wallpaper as the colours read from it (the theme
picker hands it those of `wallpaper::picture`). `MODES`, `DENSITIES`, `SHAPES`, `MATERIALS`, `MOTIONS`,
`VARIANTS`, `CONTRASTS` and `SOURCES` list each choice by the name a
control shows.

## Licence

MIT, see [LICENSE](LICENSE). The icons in [`icons/`](icons) are Lucide's,
under its own licence: see [`icons/LICENSE`](icons/LICENSE).
