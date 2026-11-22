//! Wallpapers read for their colours: once per path, kept for the whole
//! process until [`reread`],
//! each on a thread of its own, whatever reads it and however often. A
//! `UiConfig` whose source is a wallpaper reads it here, and so do the
//! appearance controls, which show what the reading came to.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use guido::prelude::{Color, RwSignal};
use guido::reactive::GlobalSignal;

use crate::theme::material;

/// A wallpaper as read for the colours a theme could be made of.
#[derive(Clone, Debug, PartialEq)]
pub enum Picture {
    /// Being read.
    Reading,
    /// Read: the colours it yields, best first, as [`material::seeds`]
    /// gives them; never none.
    Read(Vec<Color>),
    /// Not read, and why: it can't be opened, or yields no colour.
    Failed(String),
}

impl Picture {
    /// The colours it yields, best first; none until it is read, or if it
    /// can't be.
    pub fn colors(&self) -> &[Color] {
        match self {
            Picture::Read(colors) => colors,
            _ => &[],
        }
    }
}

/// Every picture whose read has ended, by path.
type Pictures = HashMap<PathBuf, Picture>;

/// The store. A signal, so what reads a picture runs again when its read
/// ends; owned by the `App`, so no surface takes it away with it.
static PICTURES: GlobalSignal<Pictures> = GlobalSignal::new(HashMap::new);

thread_local! {
    /// The reads started on this thread, for the store they will write to:
    /// a new `App` has a new store, and reads them again. Not a signal:
    /// [`picture`] records a read where the store is read, often in a memo
    /// or an effect, where writing a signal it reads would run it again.
    static STARTED: RefCell<Option<Started>> = const { RefCell::new(None) };
}

struct Started {
    store: RwSignal<Pictures>,
    /// The paths being read, by the number of their last read: the one
    /// whose picture the store takes. A read that has ended is in the store.
    reads: HashMap<PathBuf, u64>,
    /// Reads started so far, to number the next.
    count: u64,
}

impl Started {
    /// Numbers a new read of `path`, which replaces any before it.
    fn begin(&mut self, path: &Path) -> u64 {
        self.count += 1;
        self.reads.insert(path.to_path_buf(), self.count);
        self.count
    }
}

/// Runs `f` on the reads started for `store`; those of a store gone are
/// forgotten.
fn started<R>(store: RwSignal<Pictures>, f: impl FnOnce(&mut Started) -> R) -> R {
    STARTED.with_borrow_mut(|started| {
        if started.as_ref().is_some_and(|s| s.store != store) {
            *started = None;
        }
        f(started.get_or_insert_with(|| Started {
            store,
            reads: HashMap::new(),
            count: 0,
        }))
    })
}

/// The wallpaper at `path`, as far as it is read. A path not seen before
/// starts its read, on a thread of its own: until it ends, it is
/// [`Picture::Reading`], and a reactive closure that asked runs again when
/// it does, as every surface that asked for the same path shares that one
/// read.
///
/// Call it on the `App`'s thread: the store is that thread's. A program
/// with no running `App`, as a one-off render, never sees a read end: it
/// reads the picture itself with [`material::seeds`].
pub fn picture(path: &Path) -> Picture {
    let pictures = PICTURES.get();
    if let Some(picture) = pictures.with(|read| read.get(path).cloned()) {
        return picture;
    }
    let first = started(pictures, |s| {
        (!s.reads.contains_key(path)).then(|| s.begin(path))
    });
    if let Some(number) = first {
        start(pictures, path.to_path_buf(), number);
    }
    Picture::Reading
}

/// Reads the picture at `path` again, as when the file has changed: every
/// `Ui` made from it follows once the new read ends, and until then keeps
/// the colours read before, rather than show the theme in between. A read
/// already under way has nothing left to say.
///
/// `path` is as `UiConfig::wallpaper_path` gives it, the key the store
/// keeps it by. styletto doesn't watch the file: the app calls this from
/// its own watcher, handing the change to the `App`'s thread first, as a
/// signal's writer does: called on another thread, it reads into a store no
/// surface reads. `UiConfig::reread_wallpaper` calls it for the config's
/// wallpaper, and starts again from the best colour; this keeps the colour
/// chosen.
pub fn reread(path: &Path) {
    let pictures = PICTURES.get();
    let number = started(pictures, |s| s.begin(path));
    start(pictures, path.to_path_buf(), number);
}

/// Whether a picture asked for on this thread is still being read: for a
/// test, or a one-off render, that waits for the wallpaper before it draws.
/// The reads end on the `App`'s next frames.
pub fn reading() -> bool {
    let pictures = PICTURES.get();
    STARTED.with_borrow(|started| {
        started
            .as_ref()
            .is_some_and(|s| s.store == pictures && !s.reads.is_empty())
    })
}

/// Reads the picture at `path` on a thread, which writes what it found to
/// the store if it is still the last read of it (`number`): the only write,
/// queued for the `App`'s thread, never made while something reads the
/// store.
fn start(pictures: RwSignal<Pictures>, path: PathBuf, number: u64) {
    #[cfg(all(test, feature = "serde"))]
    tests::READS.lock().unwrap().push(path.clone());
    let write = pictures.writer();
    let key = path.clone();
    let spawned = std::thread::Builder::new()
        .name("styletto-wallpaper".to_string())
        .spawn(move || {
            // A decoder that panics on a picture fails it, rather than
            // leaving it being read for ever.
            let seeds = std::panic::catch_unwind(|| material::seeds(&path))
                .unwrap_or_else(|_| Err("Can't be read".to_string()));
            let picture = match seeds {
                Ok(seeds) if seeds.is_empty() => Picture::Failed("No colour found".to_string()),
                Ok(seeds) => Picture::Read(seeds),
                Err(err) => Picture::Failed(err),
            };
            if let Picture::Failed(why) = &picture {
                log::warn!("appearance.wallpaper {}: {why}", path.display());
            }
            // Run on the App's thread, where the reads were started.
            write.update(move |read| {
                let last = STARTED.with_borrow_mut(|started| {
                    let reads = &mut started.as_mut()?.reads;
                    (reads.get(&path) == Some(&number)).then(|| reads.remove(&path))
                });
                if last.is_some() {
                    read.insert(path, picture);
                }
            });
        });
    if let Err(err) = spawned {
        log::warn!("appearance.wallpaper: no thread to read it on: {err}");
        // Not being read: the next to ask starts it again.
        started(pictures, |s| {
            if s.reads.get(&key) == Some(&number) {
                s.reads.remove(&key);
            }
        });
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use std::rc::Rc;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    use guido::prelude::*;
    use guido::testing::Headless;

    use super::*;
    use crate::theme::config::UiConfig;
    use crate::ui::Ui;

    /// Every read started, by path, in every test.
    pub(super) static READS: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

    fn reads(path: &Path) -> usize {
        READS.lock().unwrap().iter().filter(|p| *p == path).count()
    }

    /// A path of its own in the temporary directory.
    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "styletto-wallpaper-{name}-{}.png",
            std::process::id()
        ))
    }

    /// A picture of two colours, the blue the larger.
    fn write_picture(name: &str) -> PathBuf {
        let path = temp(name);
        ::image::RgbaImage::from_fn(32, 16, |x, _| {
            if x < 20 {
                ::image::Rgba([30, 90, 200, 255])
            } else {
                ::image::Rgba([220, 120, 40, 255])
            }
        })
        .save(&path)
        .unwrap();
        path
    }

    /// The `Ui` of `config` with its wallpaper as it is on the disk now.
    fn read_now(config: &UiConfig) -> Ui {
        let seeds = material::seeds(&config.wallpaper_path().unwrap()).unwrap();
        config.with_wallpaper_colors(&seeds).ui(None)
    }

    fn config(path: &Path) -> UiConfig {
        UiConfig {
            wallpaper: Some(path.display().to_string()),
            ..UiConfig::default()
        }
    }

    /// Every `Ui` a surface was given, as an app's is given the config's in a
    /// reactive closure.
    type Seen = Rc<RefCell<Vec<Ui>>>;

    fn mount(h: &mut Headless, config: UiConfig) -> Seen {
        let seen = Seen::default();
        let log = seen.clone();
        let id = h.surface(SurfaceConfig::new(), move || {
            let (config, log) = (config.clone(), log.clone());
            create_effect(move || log.borrow_mut().push(config.ui(None)));
            container()
        });
        h.configure(id, 10, 10, 1.0);
        h.step();
        seen
    }

    fn last(seen: &Seen) -> Ui {
        seen.borrow()
            .last()
            .cloned()
            .expect("the surface was given a Ui")
    }

    /// Steps until `done`, in real time, as a read is on a thread.
    fn wait(h: &mut Headless, done: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !done() {
            assert!(Instant::now() < deadline, "the read never landed");
            std::thread::sleep(Duration::from_millis(2));
            h.step();
        }
    }

    #[test]
    fn a_read_lands_and_the_ui_follows() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let path = write_picture("lands");
        let config = config(&path);
        let seen = mount(&mut h, config.clone());
        let seeds = material::seeds(&path).unwrap();
        let read = config.with_wallpaper_colors(&seeds).ui(None);
        wait(&mut h, || last(&seen) == read);
        // The theme first: nothing was read where the Ui was asked for.
        assert_eq!(
            *seen.borrow(),
            [config.with_wallpaper_colors(&[]).ui(None), read]
        );
        assert_eq!(picture(&path), Picture::Read(seeds));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn every_surface_shares_one_read() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let path = write_picture("shared");
        let config = config(&path);
        let first = mount(&mut h, config.clone());
        let second = mount(&mut h, config.clone());
        let read = read_now(&config);
        wait(&mut h, || last(&first) == read && last(&second) == read);
        // Gone from the disk, it is still read: a surface made now takes
        // what was found at once.
        std::fs::remove_file(&path).unwrap();
        assert_eq!(*mount(&mut h, config).borrow(), [read]);
        assert_eq!(reads(&path), 1, "read more than once");
    }

    #[test]
    fn a_picture_that_cannot_be_read_says_why() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let missing = temp("missing");
        let seen = mount(&mut h, config(&missing));
        wait(&mut h, || picture(&missing) != Picture::Reading);
        let Picture::Failed(why) = picture(&missing) else {
            panic!("a missing picture was read");
        };
        assert!(!why.is_empty());
        // The theme shows, as it did while reading.
        assert_eq!(last(&seen), Ui::default());
    }

    #[test]
    fn a_picture_read_again_replaces_its_colours() {
        let Some(mut h) = Headless::new_or_skip() else {
            return;
        };
        let path = write_picture("again");
        let config = config(&path);
        let seen = mount(&mut h, config.clone());
        let before = read_now(&config);
        wait(&mut h, || last(&seen) == before);
        // Rewritten in place, as a wallpaper manager does: green now.
        ::image::RgbaImage::from_pixel(16, 16, ::image::Rgba([40, 160, 60, 255]))
            .save(&path)
            .unwrap();
        let after = read_now(&config);
        assert_ne!(after, before);
        reread(&path);
        wait(&mut h, || last(&seen) == after);
        // Straight from one to the other: the colours read before stay
        // until the new ones land, not the theme.
        let seen = seen.borrow();
        let from = seen.iter().position(|ui| *ui == before).unwrap();
        assert_eq!(seen[from..], [before, after]);
        std::fs::remove_file(&path).ok();
    }
}
