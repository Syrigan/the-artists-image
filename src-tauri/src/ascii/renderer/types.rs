//! Shared layout and sampling types for the ASCII renderer.

/// Resolved grid topology and reference-scale metrics from auto-fit.
pub struct ResolvedGrid {
    pub cols: u32,
    pub rows: u32,
    pub font_size_ref: f32,
    pub line_spacing: f32,
    pub cell_width_ref: f32,
    pub cell_height_ref: f32,
}

/// Layout metrics scaled for a given export canvas.
#[allow(dead_code)]
pub struct ExportLayout {
    pub cell_w: f32,
    pub cell_h: f32,
    pub font_size: f32,
    pub line_spacing: f32,
    pub start_x: f32,
    pub start_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMethod {
    Average,
    Absolute,
    Sharp,
}
