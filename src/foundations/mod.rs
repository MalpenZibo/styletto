//! The scales every component is built from, the motion and the icons.
//! Components hold no numbers of their own: they read them here, in the
//! current [`Ui`](crate::Ui).

pub mod control;
pub(crate) mod ease;
pub mod icons;
pub mod motion;
pub mod tokens;
pub mod typography;
