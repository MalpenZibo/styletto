# storybook

An interactive gallery of [`styletto`](..): every component with
controls to try it, the tokens it is built from, and a Theme page whose
choices every story is drawn with.

## Run

```bash
cargo run --release -p styletto-storybook
```

It opens as a layer-shell surface over the whole output. `Esc` or `−` folds
it into a pill in the bottom-right corner, a click on the pill brings it
back, `×` quits.

It needs a Wayland compositor with the layer-shell protocol. Following the
system's light or dark preference and choosing a wallpaper with the
system's file chooser go through `xdg-desktop-portal`; without it the
storybook still runs, dark and with a path to type.

## Shots

`shot` draws one story without a compositor, no window opened, and writes
it to a PNG; `tools/shot.sh` runs it from a release build:

```bash
tools/shot.sh --list
tools/shot.sh "Buttons/Button" button.png --scale 2 \
    mode=light density=compact shape=round theme=nord --set Enabled=false
```

`key=value` are the keys of the `[appearance]` table, dark unless `mode`
says otherwise, and `language=pseudo` shows styletto's words in the
pseudo-language below; `--set` sets one of the story's controls by its
label, and `--size WxH` the canvas (960x720 by default). The same arguments draw the
same pixels, so a shot taken before a change can be compared with one taken
after. It needs a GPU adapter.

## What is in it

- **Theme**, at the top of the sidebar: where the colours come from (a
  preset theme, a seed colour, a wallpaper, or each colour set by hand),
  the mode, then the density, the shape, the material, the motion and the
  fonts, picked among those installed. A preview on the right answers to
  clicks. Above them, the language styletto's words show in, everywhere:
  English, or a pseudo-language, every word accented, about 40% longer and
  in brackets (`[Ŀîĝĥţ··]`), to see a translation fit. Nothing is saved: a
  restart starts from the defaults.
- **Foundations**: the palette, the type scale, the space and radius
  scales, the control sizes, every icon, and a matrix of one panel in every
  density and shape.
- **Buttons**, **Inputs**, **Feedback**, **Composites**, **Motion**: one
  story per component or pattern, drawn on the island's fill over a
  wallpaper, so a translucent or glass material shows what is behind it.
  A wallpaper chosen on the Theme page replaces the shapes there.
- **Controls** and **Actions**, on the right of a story: the story's own
  controls, and a log of every callback it fired.

## Add a story

A story is a function from its context to a container. The context declares
the controls, which show in the panel and hand back their signals, and the
actions, which write to the log when called:

```rust
pub fn badge(ctx: &Ctx) -> Container {
    let urgent = ctx.bool("Urgent", false);
    let size = ctx.choice("Size", &["Small", "Medium"], 0);
    let count = ctx.number("Count", 0.0..=99.0, 3.0);
    let opened = ctx.action("open");
    // ...build it from the signals, and call `opened()` from a callback.
}
```

Register it in `src/stories/mod.rs`, under a group of the sidebar:

```rust
story("Feedback", "Badge", feedback::badge),
```

`ctx.text` declares a text control, and `ctx.action_with::<T>` an action
that logs the value it is called with. The story is built in the `Ui` the
Theme page chose, so `ui()` and every token read it.
