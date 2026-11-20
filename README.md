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
| `theme` | `palette`, `presets`, `material`, `config`: colours by meaning, the preset themes, themes from a colour, the `[appearance]` table |
| `primitives` | `text`, `layout`, `island`, `satellite`: plain functions an app can add to |
| `components` | the components, below |

`Icon`, `icon`, `centered_row`, `Ui`, `provide_ui` and `ui` are also at the
root. Below, a path such as `tokens::size` is short for
`foundations::tokens::size`.

## Use

```toml
[dependencies]
styletto = { git = "https://github.com/MalpenZibo/styletto", features = ["serde", "wallpaper"] }
```

```rust
use styletto::{Icon, provide_ui, ui};
use styletto::components::list::row;
use styletto::components::toggle::toggle;

provide_ui(config.appearance.ui(system_dark)); // or Ui::default()

row("Wi-Fi")
    .icon(Icon::Wifi)
    .subtitle("Home 5G")
    .trailing(toggle(true))
```

| Feature | Adds |
|---|---|
| `serde` | `theme::config::UiConfig`, read from an `[appearance]` table |
| `material` | themes worked out from a seed colour |
| `wallpaper` | themes worked out from an image (implies `material`) |

## The Ui

`provide_ui(Ui)` declares it for a subtree and `ui()` reads it. Each choice
moves the scales rather than setting numbers:

| Choice | Values | What it does |
|---|---|---|
| `Density` | `Compact`, `Default`, `Comfortable` | moves type, controls, icons, tiles and space up to `S8` one step |
| `Shape` | `Square`, `Soft`, `Round` | moves every radius one step; a pill stays a pill |
| `Material` | `Solid`, `Translucent`, `Glass` | the island's opacity (1, 0.86, 0.75) and blur (0, 24, 48); the text on it darkens or lightens as far as it must to read with the island's opposite behind |
| `Motion` | `Full`, `Reduced`, `None` | springs, fades only, or instant changes |
| `Fonts` | `sans`, `mono` | the families, the system's by default |

## Tokens

A step reads its value with `.px()` in the current `Ui`.

| Scale | Steps | px |
|---|---|---|
| `tokens::size` | `S0` `S0_5` `S1` `S2` `S3` `S4` `S6` `S8` `S10` `S15` `S20` `S30` `S40` `S60` `S80` `S100` | 0 2 4 8 12 16 24 32 40 60 80 120 160 240 320 400 |
| `tokens::radius` | `NONE` `XSMALL` `SMALL` `MEDIUM` `LARGE` `XLARGE` | 0 4 8 16 32 64 |
| `tokens::icon` | `XSMALL` `SMALL` `MEDIUM` `LARGE` `XLARGE` | 12 16 20 24 32 |
| `tokens::tile` | `XSMALL` `SMALL` `MEDIUM` `LARGE` `XLARGE` `XXLARGE` | 16 24 32 40 48 64 |

Up to `S8` a size step is space and follows the density; past it, it is a
layout length and stays. A tile step also sets its corners, the icon on it
and the letter of an initial; an icon step sets the stroke of a ring.

Corners: a box is a squircle (`radius::shape`, or `Radius::corners`), a pill
or a circle is round (`radius::pill(height)`). A box around another takes
`radius::around(inner, padding)`, so the curves stay concentric.

Fixed values: `stroke::HAIRLINE` 1, `RING` 1.5, `BOLD` 2, `HEAVY` 3;
`opacity::DISABLED` 0.4; `glow::SMALL` 4, `MEDIUM` 8; `scrollbar::WIDTH` 4,
`HOVER_WIDTH` 8, `MARGIN` 2, `MIN_HANDLE` 24, the same in every density.

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

A theme sets a `Scheme`: `background`, `text`, `sub`, `dim`, and four fills
(`accent`, `success`, `warning`, `danger`), each with the colour that reads
on it and its hue as text. An on-colour left unset is worked out to reach a
4.5:1 contrast.

Components read the `Palette` derived from it: `island`, `surface`,
`surface2`, `line`, `text`, `sub`, `dim`, each fill with its `on_*` and its
`*_text`, and `ripple`.

A fill is what paints a surface, a bar, a ring or a border; `on_*` is the
text on that fill; `accent_text`, `success_text`, `warning_text` and
`danger_text` are the hue as text or an icon on the island, `surface` and
`surface2`, at 4.5:1 on all three (3:1 on a see-through island, below).
Unless the theme or the config sets one,
it is the fill moved in lightness only, in Oklab, with its hue and chroma
kept (as far as sRGB holds them), just as far as it must to read: a fill
that already reads is its own text, and a pale one turns deeper, not grey.

`text` is body text and `sub` secondary text (captions, placeholders, small
labels): both reach 4.5:1 on the island and on `surface` and `surface2`.
`dim` is for icons, dividers and large text, never small text, and reaches
3:1. A preset takes these from the theme's own colours for text, not from
its borders or overlays; where none reads well enough, `Scheme::readable`
mixes the nearest toward `text` until it does, and does the same for a theme
from a seed. Colours set in `[appearance.dark]` and `[appearance.light]` are
used as given, and the text they leave to the source is made to read again
on a `background` set there; a hue's text left to the source follows its
fill.

On a see-through island the text is adjusted again: `Ui::colors` holds it
as dark or as light as it must be to read with the island's opposite showing
through, black behind a light island or white behind a dark one. Each
`*_text` is moved the same way, in lightness only, but to 3:1, the floor
`dim` has (WCAG's for icons and large text), not 4.5:1: with white behind
dark glass, a hue that reads at 4.5:1 is so light it is all but white, as
light as the text, and a warning no longer looks like one. Where not even
black or white would read, the island is drawn denser. A colour set below
its floor keeps the contrast it was set with. Paint text with `ui().colors`, not with a
`Scheme`'s or a theme's own colours, or it won't follow.

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
accent = "mauve"
# ...or a seed colour...
# seed = "#6750a4"
# ...or a wallpaper (it wins over a seed, which wins over a theme).
# wallpaper = "~/Pictures/wallpaper.jpg"
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

# Any colour of either side, set over the source; "_" for a space.
[appearance.dark]
accent = "#ff79c6"
on_accent = "#1a1b26"
accent_text = "#ff92d0" # the accent as text; worked out from accent if unset

[appearance.light]
background = "#fdf6e3"
```

## Motion

`motion::spring()` is the island's spring for every fold and growth,
`tactile()` the quicker one of a press, `hover()` and `color()` eased
changes, `easing(ms)` and `springing(config)` custom ones. All follow the
`Motion` of the `Ui`. `fading(enter, leave)` gives a value one transition
for a view coming in and another for it going out. `arriving` cross-fades a
view in, `entering` opens a panel, `shake()` plays a refusal and `armed()`
ignores presses just after a view appears.

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
| `button` | pill buttons, icon buttons (on and off, as a tile), toggle chips, `pressable` |
| `toggle`, `slider`, `input` | a switch, a slider and a slider row (to 100%, or past it to a max), text and password fields |
| `list` | rows (a checked one marks the choice in use; one in a `Tone`, warning or danger, has its title, icon, hover and press in that hue), sections, empty states, page headers with a subtitle, pages that slide, a list that scrolls past a height with the theme's scrollbar |
| `tile` | quick-settings tiles, wide and small; a wide one's subtitle takes a second line when it needs one, and tiles side by side in a `Flex::row()` are as tall as the tallest |
| `chip`, `badge` | pills for a state, dots, capsules, icon tiles |
| `level` | `Level`, how a value stands, and its ink; the level indicator: the icon a value picks from a set, its percentage beside it if asked |
| `card` | level figures and stat cards coloured by their `Level` |
| `progress`, `sparkline`, `roll` | bars, rings, recent samples, rolling digits |
| `notification` | notifications, alone and grouped |

The app decides a level, with its own thresholds, and styletto draws it:
`Level::of(value, warn, alert)` when high is bad, `Level::below(value, warn,
alert)` when low is, and `Level::Good` for better than normal, a battery
charging. `level_ink` is `text` while normal, else `success_text`,
`warning_text` or `danger_text`; a card's tint and glow take the fill.

The indicator's sets of icons are data, each icon from the value it starts
at: `Levels::BATTERY` is empty below 17%, then one, two and three bars from
17, 50 and 83% (the nearest number of bars); `Levels::WIFI` is the same in
arcs; `Levels::VOLUME` has no waves below 1%, one from 1% and two from 50%. A
state off the scale takes its own icon in place of the set's:

```rust
use styletto::Icon;
use styletto::components::level::{Level, Levels, level_indicator};

level_indicator(Levels::BATTERY, 12.0).level(Level::below(12.0, 30.0, 15.0)).percent(true);
level_indicator(Levels::BATTERY, 50.0).icon(Icon::BatteryCharging).level(Level::Good);
level_indicator(Levels::VOLUME, 0.0).icon(Icon::VolumeOff);
level_indicator(Levels::WIFI, 70.0);
```

## Licence

MIT, see [LICENSE](LICENSE). The icons in [`icons/`](icons) are Lucide's,
under its own licence: see [`icons/LICENSE`](icons/LICENSE).
