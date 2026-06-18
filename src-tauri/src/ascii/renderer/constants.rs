//! Reference-scale constants for grid layout and spacing clamps.

/// Grid density (cols × rows) is computed at this width so lyrics usage
/// does not change when export resolution changes.
pub const GRID_REFERENCE_WIDTH: u32 = 640;

/// Minimum auto-fit font size at reference scale.
pub const MIN_FONT_SIZE_REF: f32 = 4.0;

/// Maximum font size searched during auto-fit at reference scale.
pub const MAX_FONT_SIZE_REF: f32 = 256.0;

/// Extra vertical gap between text rows, in reference pixels.
pub const MIN_LINE_SPACING: f32 = -16.0;
pub const MAX_LINE_SPACING: f32 = 48.0;

/// Extra horizontal gap added to each glyph's advance width, in pixels.
pub const MIN_CHAR_SPACING: f32 = -16.0;
pub const MAX_CHAR_SPACING: f32 = 32.0;
