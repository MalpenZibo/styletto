# styletto: notes for coding agents

A design system for [Guido](https://github.com/MalpenZibo/guido): tokens, themes and
components, in Rust. The crate is at the root; `storybook/` is an interactive gallery of every
piece of it (`styletto-storybook`). [README.md](README.md) is the documentation of the system:
read it before changing anything, and keep it true.

## Layout

- `src/foundations`: the scales (`tokens`, `typography`, `control`), `motion`, `icons`.
- `src/theme`: `palette`, the preset themes (`presets/`, one file each), `material` (themes from
  a seed or a wallpaper), `config` (the `[appearance]` table).
- `src/primitives`: plain functions returning a container, that an app can add to.
- `src/components`: the `#[component]`s.
- `storybook/src/stories`: one story per component or pattern, registered in `mod.rs`.
- `icons/`: unmodified Lucide SVGs (ISC, see `icons/LICENSE`); `assets/`: logos and wordmarks.

## Rules of the system

- Every length, corner, icon and type size is a step on a scale, every colour a name in the
  palette. A component holds no numbers of its own; if a value is missing, the scale is what
  changes.
- What a surface chose (palette, density, shape, fonts, material, motion) is its `Ui`, read with
  `ui()` where the tree is built. A component must look right in every `Density`, `Shape`,
  `Material` and `Motion`, in light and dark, with every preset theme.
- Components are closed `#[component]`s, set through props, slots and callbacks. Primitives stay
  plain functions.
- A text on a fill keeps a 4.5:1 contrast (the palette tests check the presets).
- Every component has a story; a new one is registered in `storybook/src/stories/mod.rs`.
- A change to what the README describes (tokens, components, features, the config table)
  updates the README in the same change.
- Other projects depend on the public API: say so plainly when a change breaks it.

## Style

Follow the code around you: comments say why, in plain English; `cargo fmt` decides the
layout. Prefer a small, direct change over a new abstraction. The library's `rust-version`
is 1.97: no newer language or library features.

## Checks

What CI runs (`.github/workflows/ci.yml`); all must pass:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy -p styletto --all-targets -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc -p styletto --all-features --no-deps
```

## Seeing a change

Never open a window: draw a story headless into a PNG and look at it.

```sh
tools/shot.sh --list                                   # every story, as Group/Story
tools/shot.sh "Buttons/Button" /tmp/b.png              # dark, defaults
tools/shot.sh "Buttons/Button" /tmp/b.png --scale 2 --size 480x240 \
    mode=light density=compact shape=round material=glass theme=nord \
    --set Variant=Danger --set Enabled=false
```

`key=value` are the keys of the `[appearance]` table (see the README); `--set` sets a story's
control by its label. The same arguments give the same pixels, so a before/after pair can be
compared exactly: `magick compare -metric AE before.png after.png null:` prints how many pixels
differ (0 means none). It needs a GPU adapter; the first run builds in release. The headless
tests need one too: they skip without it, or fail when `STYLETTO_REQUIRE_GPU` is set (CI sets it).

## Git

- `main` changes only through pull requests with green CI; never push to it.
- Branches are `<issue>-<short-slug>`. Commits and PRs are in English; a commit subject says
  what changed, the body explains why. No trailers (no `Co-Authored-By`), no "Generated with"
  footers.
- PRs are squash-merged.
