//! Raster and vector export from a pre-generated [`AsciiGrid`].
//!
//! Export resolution comes from [`ExportSettings`], not from the grid's generation
//! resolution. Layout is scaled via [`crate::ascii::renderer::export_layout`].

pub mod png;
pub mod jpeg;
pub mod svg;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub format: String,
    pub resolution: u32,
    pub aspect_ratio: String,
    /// Row spacing at reference scale (informational; grid cells come from generation).
    /// Serialized as `line_height` for API compatibility.
    #[serde(rename = "line_height")]
    pub line_spacing: f32,
    /// Extra horizontal spacing in pixels (informational; baked into grid cell width).
    pub char_spacing: f32,
    /// Reserved: space-character advance in pixels (not yet applied).
    pub space_spacing: f32,
    pub font_path: String,
    pub jpeg_quality: u8,
}
