use image::DynamicImage;
use super::png;
use super::ExportSettings;
use crate::ascii::AsciiGrid;

pub fn export(
    grid: &AsciiGrid,
    settings: &ExportSettings,
    font_data: &[u8],
    output_path: &str,
) -> Result<(), String> {
    let rgba = png::render_rgba(grid, settings, font_data)?;
    let rgb = DynamicImage::ImageRgba8(rgba).into_rgb8();

    let quality = settings.jpeg_quality.max(1).min(100);
    let file = std::fs::File::create(output_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = std::io::BufWriter::new(file);
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
    rgb.write_with_encoder(encoder)
        .map_err(|e| format!("Failed to save JPEG: {}", e))?;
    Ok(())
}
