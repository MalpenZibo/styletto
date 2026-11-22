//! Colours by meaning, the preset themes and the ones worked out from a
//! colour, and the `[appearance]` table that chooses among them.

#[cfg(feature = "serde")]
pub mod config;
pub mod material;
pub mod palette;
pub mod presets;
#[cfg(feature = "wallpaper")]
pub mod wallpaper;
