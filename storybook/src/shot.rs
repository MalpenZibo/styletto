//! `styletto-storybook shot`: one story drawn without a compositor, on the
//! storybook's canvas, and written to a PNG. The look is given with the keys
//! of the `[appearance]` table, so a picture can be taken in any mode,
//! density, shape, material and theme, and the same one again after a change.
//!
//! ```text
//! styletto-storybook shot --list
//! styletto-storybook shot <Group/Story> <out.png> [--size WxH] [--scale S]
//!     [--set "Control=value"]... [key=value]...
//! ```

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use guido::prelude::*;
use guido::testing::Headless;
use styletto::provide_ui;
use styletto::theme::config::UiConfig;

use crate::chrome::shell::canvas;
use crate::stories::STORIES;
use crate::stories::story::{Control, Ctx};

const USAGE: &str = "\
usage: styletto-storybook shot --list
       styletto-storybook shot <Group/Story> <out.png> [--size WxH] [--scale S]
           [--set \"Control=value\"]... [key=value]...

key=value are the keys of the [appearance] table: theme, dark_flavor,
light_flavor, accent, seed, wallpaper, variant, contrast, mode, density,
shape, material, motion, fonts.sans, fonts.mono, dark.<colour>,
light.<colour>. mode is dark unless given.";

/// The keys of the `[appearance]` table, and the tables whose keys are free.
const KEYS: &[&str] = &[
    "theme",
    "dark_flavor",
    "light_flavor",
    "accent",
    "seed",
    "wallpaper",
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

struct Shot {
    story: usize,
    out: PathBuf,
    size: (u32, u32),
    scale: f32,
    controls: Vec<(String, String)>,
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
        table.push_str(&format!("{key} = {}\n", toml_string(value)));
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
    let ui = config.ui(None);
    let backdrop = config.wallpaper.as_deref().map(expand);
    let story = &STORIES[shot.story];
    let drawn: Rc<RefCell<Option<Rc<Ctx>>>> = Rc::default();
    let seen = drawn.clone();
    let id = h.surface(SurfaceConfig::new(), move || {
        provide_ui(ui);
        let ctx = Rc::new(Ctx::new(create_signal(Vec::new())));
        *seen.borrow_mut() = Some(ctx.clone());
        let backdrop = create_memo(move || backdrop.clone());
        let ctx = ctx.clone();
        container().width(fill()).height(fill()).child(move || {
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
    settle(&mut h, start);

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

/// Steps through simulated time until nothing asks for another frame, or for
/// as long as the longest animation takes: a spinner never stops asking.
fn settle(h: &mut Headless, start: Instant) {
    let mut at = start + FRAME;
    while at - start < SETTLE {
        h.wait_for_image_decodes();
        h.step_at(at);
        if h.next_wait().is_none() {
            break;
        }
        at += FRAME;
    }
    h.wait_for_image_decodes();
    h.step_at(at + FRAME);
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

fn expand(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => std::env::home_dir().unwrap_or_default().join(rest),
        None => PathBuf::from(path),
    }
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
