//! ASCII art generation: album image + lyrics → colored character grid.
//!
//! Three render methods ([`RenderMethod`]) differ only in per-cell color sampling;
//! layout (grid size, font auto-fit) is shared via [`renderer`].

pub mod absolute;
pub mod average;
pub mod renderer;
pub mod sharp;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiSettings {
    /// Extra vertical gap between text rows, in reference pixels (not font size).
    /// Serialized as `line_height` for API compatibility.
    #[serde(rename = "line_height")]
    pub line_spacing: f32,
    /// Extra horizontal spacing in pixels added to each glyph's advance width.
    pub char_spacing: f32,
    /// Reserved: horizontal advance for space characters in pixels (not yet applied).
    pub space_spacing: f32,
    pub font_path: String,
    pub font_style: String,
    pub aspect_ratio: String,
    pub resolution: u32,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiGrid {
    pub chars: Vec<Vec<char>>,
    pub colors: Vec<Vec<[u8; 3]>>,
    pub width: u32,
    pub height: u32,
    pub char_aspect_ratio: f32,
    /// Pixel dimensions the source image was cropped to during generation.
    pub target_width: u32,
    pub target_height: u32,
    /// Color-sampling cell size at generation resolution.
    pub cell_width: f32,
    pub cell_height: f32,
    /// Reference cell width at GRID_REFERENCE_WIDTH (640px).
    pub cell_width_ref: f32,
    /// Reference cell height at reference canvas height.
    pub cell_height_ref: f32,
    /// Auto-computed font size at GRID_REFERENCE_WIDTH.
    pub font_size_ref: f32,
    /// User line spacing at reference scale.
    pub line_spacing: f32,
    /// Lyrics characters placed on visible cells.
    pub lyrics_chars_placed: u32,
    /// Total lyrics characters available.
    pub lyrics_total: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMethod {
    Average,
    Absolute,
    Sharp,
}

impl RenderMethod {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "absolute" => RenderMethod::Absolute,
            "sharp" => RenderMethod::Sharp,
            _ => RenderMethod::Average,
        }
    }
}
