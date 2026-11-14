//! What a story is, and the context it declares its controls and actions on.

use std::cell::RefCell;
use std::ops::RangeInclusive;

use guido::prelude::*;

pub struct Story {
    pub group: &'static str,
    pub name: &'static str,
    pub render: fn(&Ctx) -> Container,
}

#[derive(Clone)]
pub enum Control {
    Bool {
        label: &'static str,
        value: RwSignal<bool>,
    },
    Number {
        label: &'static str,
        range: RangeInclusive<f32>,
        value: RwSignal<f32>,
    },
    Choice {
        label: &'static str,
        options: &'static [&'static str],
        value: RwSignal<usize>,
    },
    Text {
        label: &'static str,
        value: RwSignal<String>,
    },
}

/// Handed to a story while it renders. Every control it declares shows up in
/// the controls panel, bound to the signal the story gets back.
pub struct Ctx {
    controls: RefCell<Vec<Control>>,
    /// Bumped by every control declared, for a panel built before the story.
    declared: RwSignal<usize>,
    log: RwSignal<Vec<String>>,
}

impl Ctx {
    pub fn new(log: RwSignal<Vec<String>>) -> Self {
        Self {
            controls: RefCell::default(),
            declared: create_signal(0),
            log,
        }
    }

    pub fn bool(&self, label: &'static str, default: bool) -> RwSignal<bool> {
        let value = create_signal(default);
        self.declare(Control::Bool { label, value });
        value
    }

    pub fn number(
        &self,
        label: &'static str,
        range: RangeInclusive<f32>,
        default: f32,
    ) -> RwSignal<f32> {
        let value = create_signal(default);
        self.declare(Control::Number {
            label,
            range,
            value,
        });
        value
    }

    pub fn choice(
        &self,
        label: &'static str,
        options: &'static [&'static str],
        default: usize,
    ) -> RwSignal<usize> {
        let value = create_signal(default);
        self.declare(Control::Choice {
            label,
            options,
            value,
        });
        value
    }

    pub fn text(&self, label: &'static str, default: &str) -> RwSignal<String> {
        let value = create_signal(default.to_string());
        self.declare(Control::Text { label, value });
        value
    }

    /// A callback that writes `name` to the actions log each time it runs.
    pub fn action(&self, name: &'static str) -> impl Fn() + Clone + 'static {
        let log = self.log;
        move || push(log, name.to_string())
    }

    /// Like [`action`](Self::action), with the value the callback got.
    pub fn action_with<T: std::fmt::Debug>(
        &self,
        name: &'static str,
    ) -> impl Fn(T) + Clone + 'static {
        let log = self.log;
        move |value| push(log, format!("{name}({value:?})"))
    }

    fn declare(&self, control: Control) {
        self.controls.borrow_mut().push(control);
        self.declared.update(|n| *n += 1);
    }

    /// Forgets what a previous drawing of the story declared.
    pub fn reset(&self) {
        self.controls.borrow_mut().clear();
    }

    /// The controls declared so far; read in a reactive scope, it runs again
    /// as more are.
    pub fn controls(&self) -> Vec<Control> {
        self.declared.get();
        self.controls.borrow().clone()
    }
}

const LOG_LENGTH: usize = 50;

fn push(log: RwSignal<Vec<String>>, entry: String) {
    log.update(|entries| {
        entries.insert(0, entry);
        entries.truncate(LOG_LENGTH);
    });
}
