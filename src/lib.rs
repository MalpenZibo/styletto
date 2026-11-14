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
//! [`provide_ui`] and read with [`ui()`].

pub mod components;
pub mod foundations;
pub mod primitives;
pub mod theme;
pub mod ui;

pub use foundations::icons::{Icon, icon};
pub use primitives::layout::centered_row;
pub use ui::{Ui, provide_ui, ui};
