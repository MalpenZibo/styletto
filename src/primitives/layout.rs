//! Layout shorthands.

use guido::prelude::*;

/// A row whose children are centred across it, `spacing` apart.
pub fn centered_row(spacing: f32) -> Flex {
    Flex::row()
        .spacing(spacing)
        .cross_alignment(CrossAlignment::Center)
}
