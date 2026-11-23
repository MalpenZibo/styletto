//! styletto, a design system for Guido: tokens, themes and components.
//!
//! - [`foundations`]: the scales every piece is built from (lengths,
//!   corners, type, control sizes), the motion and the icons.
//! - [`theme`]: the colours by meaning, the themes that fill them and,
//!   with the `serde` feature, the `[appearance]` table that picks one.
//! - [`primitives`]: plain functions returning a container, for an app to
//!   add to: the text styles, `centered_row`, the island.
//! - [`components`]: closed `#[component]`s, set through their props,
//!   slots and callbacks.
//!
//! A surface's choices among all of these are its [`Ui`], declared with
//! [`provide_ui`], read with [`ui()`] for what sizes a widget and with
//! [`look()`] for its colours, which follow a new palette in place.

pub mod components;
pub mod foundations;
pub mod primitives;
pub mod theme;
pub mod ui;

pub use foundations::icons::{Icon, icon};
pub use primitives::layout::centered_row;
pub use ui::{Look, Ui, look, provide_ui, ui};
