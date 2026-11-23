//! `styletto-storybook shot`: one story drawn without a compositor, on the
//! storybook's canvas, and written to a PNG. The look is given with the keys
//! of the `[appearance]` table, so a picture can be taken in any mode,
//! density, shape, material and theme, and the same one again after a change.
//!
//! ```text
//! styletto-storybook shot --list
//! styletto-storybook shot <Group/Story> <out.png> [--size WxH] [--scale S]
//!     [--set "Control=value"]... [--select TEXT] [key=value]...
//! ```

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use guido::prelude::*;
use guido::testing::Headless;
use styletto::provide_ui;
use styletto::theme::config::UiConfig;
use styletto::theme::wallpaper;

use crate::chrome::shell::canvas;
use crate::stories::STORIES;
use crate::stories::story::{Control, Ctx};

const USAGE: &str = "\
usage: styletto-storybook shot --list
       styletto-storybook shot <Group/Story> <out.png> [--size WxH] [--scale S]
           [--set \"Control=value\"]... [--select TEXT] [key=value]...

--select types TEXT into the story's first field, which a Tab focuses, and
selects it all.

key=value are the keys of the [appearance] table: theme, dark_flavor,
light_flavor, accent, dark_accent, light_accent, seed, wallpaper,
wallpaper_color, variant, contrast, mode, density,
shape, material, motion, fonts.sans, fonts.mono, dark.<colour>,
light.<colour>. mode is dark unless given.";

/// The keys of the `[appearance]` table, and the tables whose keys are free.
const KEYS: &[&str] = &[
    "theme",
    "dark_flavor",
    "light_flavor",
    "accent",
    "dark_accent",
    "light_accent",
    "seed",
    "wallpaper",
    "wallpaper_color",
    "variant",
    "contrast",
    "mode",
    "density",
    "shape",
    "material",
    "motion",
    "fonts.sans",
    "fonts.mono",
];
const TABLES: &[&str] = &["dark.", "light."];

/// The canvas as a story sees it in the storybook, less the panel.
const SIZE: (u32, u32) = (960, 720);
/// Simulated time an animation gets to settle, a frame at a time.
const SETTLE: Duration = Duration::from_secs(4);
const FRAME: Duration = Duration::from_millis(16);
/// Real time a picture the story reads gets to be read.
const READING: Duration = Duration::from_secs(10);

struct Shot {
    story: usize,
    out: PathBuf,
    size: (u32, u32),
    scale: f32,
    controls: Vec<(String, String)>,
    /// Typed into the first field and selected, to show a selection.
    select: Option<String>,
    appearance: Vec<(String, String)>,
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.iter().any(|a| a == "--list") {
        for story in STORIES {
            println!("{}/{}", story.group, story.name);
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    let shot = parse(args)?;
    let config = appearance(&shot.appearance)?;
    take(&shot, &config)
}

fn parse(args: &[String]) -> Result<Shot, String> {
    let mut positional = Vec::new();
    let mut size = SIZE;
    let mut scale = 1.0;
    let mut controls = Vec::new();
    let mut select = None;
    let mut appearance = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| {
            args.next()
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value\n\n{USAGE}"))
        };
        match arg.as_str() {
            "--size" => {
                let v = value("--size")?;
                size = v
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                    .filter(|&(w, h)| w > 0 && h > 0)
                    .ok_or_else(|| format!("--size {v}: expected WIDTHxHEIGHT"))?;
            }
            "--scale" => {
                let v = value("--scale")?;
                scale = v
                    .parse()
                    .ok()
                    .filter(|s: &f32| *s > 0.0)
                    .ok_or_else(|| format!("--scale {v}: expected a positive number"))?;
            }
            "--set" => {
                let v = value("--set")?;
                let (name, to) = v
                    .split_once('=')
                    .ok_or_else(|| format!("--set {v}: expected Control=value"))?;
                controls.push((name.trim().to_string(), to.trim().to_string()));
            }
            "--select" => select = Some(value("--select")?),
            flag if flag.starts_with("--") => {
                return Err(format!("unknown flag {flag}\n\n{USAGE}"));
            }
            pair if pair.contains('=') && positional.len() >= 2 => {
                let (key, value) = pair.split_once('=').unwrap();
                appearance.push((key.trim().to_string(), value.trim().to_string()));
            }
            other => positional.push(other.to_string()),
        }
    }
    let [name, out] = <[String; 2]>::try_from(positional)
        .map_err(|_| format!("expected a story and an output file\n\n{USAGE}"))?;
    let story = STORIES
        .iter()
        .position(|s| format!("{}/{}", s.group, s.name).eq_ignore_ascii_case(&name))
        .ok_or_else(|| format!("no story {name:?}: `shot --list` names them"))?;
    Ok(Shot {
        story,
        out: out.into(),
        size,
        scale,
        controls,
        select,
        appearance,
    })
}

/// The `[appearance]` table the pairs make, read as a config file would be.
fn appearance(pairs: &[(String, String)]) -> Result<UiConfig, String> {
    let mut table = String::new();
    for (key, value) in pairs {
        if !KEYS.contains(&key.as_str()) && !TABLES.iter().any(|t| key.starts_with(t)) {
            return Err(format!("unknown appearance key {key:?}\n\n{USAGE}"));
        }
        // The one number among them; the rest are names.
        let value = if key == "wallpaper_color" {
            value.to_string()
        } else {
            toml_string(value)
        };
        table.push_str(&format!("{key} = {value}\n"));
    }
    if !pairs.iter().any(|(key, _)| key == "mode") {
        table.push_str("mode = \"dark\"\n");
    }
    toml::from_str(&table).map_err(|err| format!("appearance: {err}"))
}

fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn take(shot: &Shot, config: &UiConfig) -> Result<(), String> {
    // A name that matches nothing is logged by styletto and left out; here
    // it is said, or the picture would quietly show the default.
    log::set_logger(&STDERR).ok();
    log::set_max_level(log::LevelFilter::Warn);

    let mut h = Headless::new().ok_or("no GPU adapter: a frame has nowhere to land")?;
    // The wallpaper is read here, into the store of the App `Headless` has
    // just made, before the story is built: it is built once, already in
    // the wallpaper's colours. Nothing it reads later changes this `Ui`, so
    // a picture the story reads of its own doesn't build it again, which
    // would reset the controls.
    let ui = config.ui_blocking(None);
    let backdrop = config.wallpaper_path();
    let story = &STORIES[shot.story];
    let drawn: Rc<RefCell<Option<Rc<Ctx>>>> = Rc::default();
    let seen = drawn.clone();
    let id = h.surface(SurfaceConfig::new(), move || {
        let ctx = Rc::new(Ctx::new(create_signal(Vec::new())));
        *seen.borrow_mut() = Some(ctx.clone());
        let backdrop = create_memo(move || backdrop.clone());
        let ctx = ctx.clone();
        container().width(fill()).height(fill()).child(move || {
            provide_ui(ui);
            ctx.reset();
            canvas(backdrop, (story.render)(&ctx))
        })
    });
    h.configure(id, shot.size.0, shot.size.1, shot.scale);
    // Every frame at a moment of simulated time, from the first: a frame at
    // the real time would give the first step of an animation however long
    // building the story took, and a spring would come to rest a fraction
    // of a pixel elsewhere from one run to the next.
    let start = Instant::now();
    h.step_at(start);

    let ctx = drawn.borrow().clone().ok_or("the story was not drawn")?;
    let controls = ctx.controls();
    for (name, value) in &shot.controls {
        set(&controls, name, value)?;
    }
    // A picture the story reads of its own, as the Appearance story does
    // with Colours=Wallpaper, is read on a thread: it is waited for here.
    // The story reads it in a child of its own, so its landing doesn't
    // build the whole story again, which would reset the controls.
    h.step_at(start);
    read(&mut h, start)?;
    let start = match &shot.select {
        Some(text) => select(&mut h, id, text, start),
        None => start,
    };
    settle(&mut h, start, SETTLE);

    let (width, height) = h.physical_size(id);
    let frame = h.read_frame(id);
    ::image::save_buffer(
        &shot.out,
        &frame,
        width,
        height,
        ::image::ExtendedColorType::Rgba8,
    )
    .map_err(|err| format!("{}: {err}", shot.out.display()))?;
    println!("{} ({width}x{height})", shot.out.display());
    Ok(())
}

/// Steps, at `at`, until every picture the story asked for is read: on a
/// thread, in real time, while the frames are in simulated time.
fn read(h: &mut Headless, at: Instant) -> Result<(), String> {
    let deadline = Instant::now() + READING;
    while wallpaper::reading() {
        if Instant::now() > deadline {
            return Err("a picture the story reads took too long to read".to_string());
        }
        std::thread::sleep(Duration::from_millis(2));
        h.step_at(at);
    }
    Ok(())
}

/// Tabs into the first field, types `text` and selects it all, a key a
/// frame from `at`; gives the moment of the last.
fn select(h: &mut Headless, id: SurfaceId, text: &str, mut at: Instant) -> Instant {
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let keys = std::iter::once((Key::Tab, Modifiers::default()))
        .chain(text.chars().map(|c| (Key::Char(c), Modifiers::default())))
        .chain([(Key::Char('a'), ctrl)]);
    for (key, modifiers) in keys {
        at += FRAME;
        let down = Event::KeyDown {
            key,
            modifiers,
            repeat: false,
        };
        h.event_at(id, down, at);
        h.step_at(at);
    }
    at
}

/// Steps a frame at a time through `time` of simulated time after `start`,
/// waiting each frame for the pictures being decoded, which arrive in real
/// time; gives the moment of the last frame. It doesn't stop when
/// `next_wait` says nothing is pending: that asks at the real time, behind
/// the simulated one, and says so while animations are still under way.
pub(crate) fn settle(h: &mut Headless, start: Instant, time: Duration) -> Instant {
    let mut at = start + FRAME;
    while at - start < time {
        h.wait_for_image_decodes();
        h.step_at(at);
        at += FRAME;
    }
    h.wait_for_image_decodes();
    h.step_at(at);
    at
}

fn set(controls: &[Control], name: &str, value: &str) -> Result<(), String> {
    let label = |c: &Control| match c {
        Control::Bool { label, .. }
        | Control::Number { label, .. }
        | Control::Choice { label, .. }
        | Control::Text { label, .. } => *label,
    };
    let control = controls
        .iter()
        .find(|c| label(c).eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            let names: Vec<_> = controls.iter().map(label).collect();
            format!("no control {name:?}; this story has {names:?}")
        })?;
    let bad = |expected: &str| format!("--set {name}={value}: expected {expected}");
    match control {
        Control::Bool { value: signal, .. } => {
            signal.set(value.parse().map_err(|_| bad("true or false"))?);
        }
        Control::Number {
            range,
            value: signal,
            ..
        } => {
            let n: f32 = value.parse().map_err(|_| bad("a number"))?;
            if !range.contains(&n) {
                return Err(bad(&format!("a number in {range:?}")));
            }
            signal.set(n);
        }
        Control::Choice {
            options,
            value: signal,
            ..
        } => {
            let index = options
                .iter()
                .position(|o| o.eq_ignore_ascii_case(value))
                .ok_or_else(|| bad(&format!("one of {options:?}")))?;
            signal.set(index);
        }
        Control::Text { value: signal, .. } => signal.set(value.to_string()),
    }
    Ok(())
}

struct Stderr;

static STDERR: Stderr = Stderr;

impl log::Log for Stderr {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Warn
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
