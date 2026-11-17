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
`opacity::DISABLED` 0.4; `glow::SMALL` 4, `MEDIUM` 8.

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
on it. An on-colour left unset is worked out to reach a 4.5:1 contrast.

Components read the `Palette` derived from it: `island`, `surface`,
`surface2`, `line`, `text`, `sub`, `dim`, each fill with its `on_*`, and
`ripple`.

`text` is body text and `sub` secondary text (captions, placeholders, small
labels): both reach 4.5:1 on the island and on `surface` and `surface2`.
`dim` is for icons, dividers and large text, never small text, and reaches
3:1. A preset takes these from the theme's own colours for text, not from
its borders or overlays; where none reads well enough, `Scheme::readable`
mixes the nearest toward `text` until it does, and does the same for a theme
from a seed. Colours set in `[appearance.dark]` and `[appearance.light]` are
used as given, and the text they leave to the source is made to read again
on a `background` set there.

On a see-through island the text is adjusted again: `Ui::colors` holds it
as dark or as light as it must be to read with the island's opposite showing
through, black behind a light island or white behind a dark one. Where not
even black or white would read, the island is drawn denser. A colour set
below its floor keeps the contrast it was set with. Paint text with `ui().colors`, not with a
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
| `button` | pill buttons, icon buttons, toggle chips, `pressable` |
| `toggle`, `slider`, `input` | a switch, a slider and a slider row, text and password fields |
| `list` | rows, sections, empty states, page headers, pages that slide |
| `tile` | quick-settings tiles, wide and small |
| `chip`, `badge` | pills for a state, dots, capsules, icon tiles |
| `card` | level figures and stat cards coloured past a threshold |
| `progress`, `sparkline`, `roll` | bars, rings, a battery's colour, recent samples, rolling digits |
| `notification` | notifications, alone and grouped |

## Licence

MIT, see [LICENSE](LICENSE). The icons in [`icons/`](icons) are Lucide's,
under its own licence: see [`icons/LICENSE`](icons/LICENSE).
