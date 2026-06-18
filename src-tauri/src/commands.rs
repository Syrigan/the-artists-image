//! Tauri command handlers bridging the frontend to Rust backends.
//!
//! **Generate** builds an [`AsciiGrid`] (layout + sampled colors) from image + lyrics.
//! **Preview** and **export** reuse that grid: they call `export_layout` to scale
//! reference metrics to the requested canvas, then rasterize (PNG/JPEG) or emit SVG.
//! Preview returns base64 PNG bytes; export writes to disk.

use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use crate::ascii::{self, AsciiGrid, AsciiSettings, RenderMethod};
use crate::export::{self, ExportSettings};
use crate::fonts::{self, FontInfo};
use crate::scraper;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub image_path: String,
    pub lyrics: String,
    pub settings: AsciiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub grid: AsciiGrid,
    pub export_settings: ExportSettings,
    pub font_path: String,
    pub output_path: String,
}

#[tauri::command]
pub fn get_system_fonts() -> Result<Vec<FontInfo>, String> {
    fonts::get_system_fonts()
}

#[tauri::command]
/// Build grid at generation settings; does not render glyphs to pixels.
pub fn generate_ascii(request: GenerateRequest) -> Result<AsciiGrid, String> {
    let font_data = fonts::load_font_data(&request.settings.font_path)?;
    let method = RenderMethod::from_str(&request.settings.method);

    match method {
        RenderMethod::Average => {
            ascii::average::generate(&request.image_path, &request.lyrics, &request.settings, &font_data)
        }
        RenderMethod::Absolute => {
            ascii::absolute::generate(&request.image_path, &request.lyrics, &request.settings, &font_data)
        }
        RenderMethod::Sharp => {
            ascii::sharp::generate(&request.image_path, &request.lyrics, &request.settings, &font_data)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewRequest {
    pub grid: AsciiGrid,
    pub export_settings: ExportSettings,
    pub font_path: String,
}

#[tauri::command]
/// Raster preview at export settings (typically 640px); same path as PNG export.
pub fn render_preview(request: PreviewRequest) -> Result<String, String> {
    let font_data = fonts::load_font_data(&request.font_path)?;
    let bytes = export::png::export_to_png_bytes(&request.grid, &request.export_settings, &font_data)?;
    Ok(STANDARD.encode(bytes))
}

#[tauri::command]
pub fn export_ascii(request: ExportRequest) -> Result<(), String> {
    let font_data = fonts::load_font_data(&request.font_path)?;

    match request.export_settings.format.to_lowercase().as_str() {
        "png" => export::png::export(&request.grid, &request.export_settings, &font_data, &request.output_path),
        "jpeg" | "jpg" => export::jpeg::export(&request.grid, &request.export_settings, &font_data, &request.output_path),
        "svg" => export::svg::export(&request.grid, &request.export_settings, &font_data, &request.output_path),
        _ => Err(format!("Unsupported format: {}", request.export_settings.format)),
    }
}

#[tauri::command]
pub fn grid_to_colored_html(grid: AsciiGrid) -> String {
    ascii::renderer::grid_to_colored_html(&grid)
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

fn parse_scraper_json_error(stdout: &str) -> Option<String> {
    let result = serde_json::from_str::<serde_json::Value>(stdout.trim()).ok()?;
    result
        .get("error")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn scraper_failure_message(stdout: &str, stderr: &str) -> String {
    if !stderr.trim().is_empty() {
        return stderr.trim().to_string();
    }

    stdout.trim().to_string()
}

#[tauri::command]
pub async fn fetch_lyrics(
    app: tauri::AppHandle,
    artist: String,
    album: String,
) -> Result<serde_json::Value, String> {
    let scraper_path = scraper::resolve_scraper_path(&app)?;

    let output = tauri::async_runtime::spawn_blocking(move || {
        scraper::run_scraper(&scraper_path, &artist, &album)
    })
    .await
    .map_err(|e| format!("Scraper task failed: {e}"))??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        if let Some(error) = parse_scraper_json_error(&stdout) {
            return Err(error);
        }
        let message = scraper_failure_message(&stdout, &stderr);
        return Err(format!("Scraper failed: {message}"));
    }

    let result: serde_json::Value = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("Failed to parse scraper output: {e}. Output: {stdout}"))?;

    if let Some(error) = result.get("error").and_then(|v| v.as_str()) {
        return Err(error.to_string());
    }

    Ok(result)
}
