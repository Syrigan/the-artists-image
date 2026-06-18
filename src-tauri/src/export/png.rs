use std::io::Cursor;
use std::time::Instant;

use ab_glyph::{Font, FontRef, ScaleFont, point};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageBuffer, ImageEncoder, Rgba, RgbaImage};

use super::ExportSettings;
use crate::ascii::{AsciiGrid, renderer};

fn blend_glyph_pixel(pixel: &mut Rgba<u8>, color: [u8; 3], coverage: f32) {
    let src_a = (coverage.clamp(0.0, 1.0) * 255.0).round() as u32;
    if src_a == 0 {
        return;
    }
    let dst = pixel.0;
    let dst_a = dst[3] as u32;
    let inv_src_a = 255u32.saturating_sub(src_a);
    let out_a = src_a + (dst_a * inv_src_a) / 255;
    if out_a == 0 {
        return;
    }
    let cr = color[0] as u32;
    let cg = color[1] as u32;
    let cb = color[2] as u32;
    let dr = dst[0] as u32;
    let dg = dst[1] as u32;
    let db = dst[2] as u32;
    pixel.0 = [
        ((cr * src_a + dr * dst_a * inv_src_a / 255) / out_a) as u8,
        ((cg * src_a + dg * dst_a * inv_src_a / 255) / out_a) as u8,
        ((cb * src_a + db * dst_a * inv_src_a / 255) / out_a) as u8,
        out_a as u8,
    ];
}

pub fn render_rgba(
    grid: &AsciiGrid,
    settings: &ExportSettings,
    font_data: &[u8],
) -> Result<RgbaImage, String> {
    let t0 = Instant::now();
    let font = FontRef::try_from_slice(font_data)
        .map_err(|e| format!("Failed to load font: {}", e))?;

    let (img_width, img_height) =
        renderer::target_dimensions_from_settings(settings.resolution, &settings.aspect_ratio);

    let layout = renderer::export_layout(grid, img_width, img_height);

    let mut img: RgbaImage = ImageBuffer::from_pixel(img_width, img_height, Rgba([0, 0, 0, 0]));

    let scaled = font.as_scaled(layout.font_size);

    for (row_idx, row) in grid.chars.iter().enumerate() {
        let mut x_pos = layout.start_x;
        let cell_y = layout.start_y + (row_idx as f32 * layout.cell_h);
        let y_pos = renderer::baseline_y(&font, cell_y, layout.font_size);

        for (col_idx, &ch) in row.iter().enumerate() {
            if ch != ' ' {
                let color = grid.colors[row_idx][col_idx];
                let glyph_id = font.glyph_id(ch);
                let outline = scaled.outline_glyph(glyph_id.with_scale_and_position(
                    layout.font_size,
                    point(x_pos, y_pos),
                ));

                if let Some(outlined) = outline {
                    let b = outlined.px_bounds();
                    outlined.draw(|gx, gy, gv| {
                        let px = gx as i32 + b.min.x as i32;
                        let py = gy as i32 + b.min.y as i32;
                        if px >= 0 && py >= 0 && (px as u32) < img_width && (py as u32) < img_height
                        {
                            let pixel = img.get_pixel_mut(px as u32, py as u32);
                            blend_glyph_pixel(pixel, color, gv);
                        }
                    });
                }
            }

            x_pos += layout.cell_w;
        }
    }

    eprintln!(
        "[render] rasterize {}x{} ({} cells): {:.1}ms",
        img_width,
        img_height,
        grid.width * grid.height,
        t0.elapsed().as_secs_f64() * 1000.0
    );

    Ok(img)
}

fn encode_png(img: &RgbaImage, fast: bool) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    if fast {
        let encoder = PngEncoder::new_with_quality(
            &mut buf,
            CompressionType::Fast,
            FilterType::NoFilter,
        );
        encoder
            .write_image(img.as_raw(), img.width(), img.height(), ExtendedColorType::Rgba8)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    } else {
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }
    Ok(buf)
}

pub fn export_to_png_bytes(
    grid: &AsciiGrid,
    settings: &ExportSettings,
    font_data: &[u8],
) -> Result<Vec<u8>, String> {
    let t0 = Instant::now();
    let img = render_rgba(grid, settings, font_data)?;
    let fast = settings.resolution <= renderer::GRID_REFERENCE_WIDTH;
    let bytes = encode_png(&img, fast)?;
    eprintln!(
        "[render] PNG encode (fast={}): {:.1}ms",
        fast,
        t0.elapsed().as_secs_f64() * 1000.0
    );
    Ok(bytes)
}

pub fn export(
    grid: &AsciiGrid,
    settings: &ExportSettings,
    font_data: &[u8],
    output_path: &str,
) -> Result<(), String> {
    let img = render_rgba(grid, settings, font_data)?;
    img.save(output_path)
        .map_err(|e| format!("Failed to save PNG: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ascii::AsciiGrid;

    fn test_grid(chars: Vec<Vec<char>>, colors: Vec<Vec<[u8; 3]>>) -> AsciiGrid {
        let height = chars.len() as u32;
        let width = chars.first().map(|r| r.len()).unwrap_or(0) as u32;
        AsciiGrid {
            chars,
            colors,
            width,
            height,
            char_aspect_ratio: 1.0,
            target_width: 64,
            target_height: 64,
            cell_width: 32.0,
            cell_height: 32.0,
            cell_width_ref: 32.0,
            cell_height_ref: 32.0,
            font_size_ref: 16.0,
            line_spacing: 2.0,
            lyrics_chars_placed: 0,
            lyrics_total: 0,
        }
    }

    fn test_settings() -> ExportSettings {
        ExportSettings {
            format: "png".to_string(),
            resolution: 64,
            aspect_ratio: "1:1".to_string(),
            line_spacing: 2.0,
            char_spacing: 0.0,
            space_spacing: 0.0,
            font_path: String::new(),
            jpeg_quality: 90,
        }
    }

    #[test]
    fn canvas_and_spaces_are_transparent() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let grid = test_grid(
            vec![vec![' ', ' ']],
            vec![vec![[255, 0, 0], [0, 255, 0]]],
        );
        let img = render_rgba(&grid, &test_settings(), &font_data).expect("render");

        for pixel in img.pixels() {
            assert_eq!(pixel.0[3], 0, "expected fully transparent canvas");
        }
    }

    #[test]
    fn non_space_chars_have_opaque_pixels() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let grid = test_grid(vec![vec!['A']], vec![vec![[200, 100, 50]]]);
        let img = render_rgba(&grid, &test_settings(), &font_data).expect("render");

        let drawn_count = img.pixels().filter(|p| p.0[3] > 0).count();
        assert!(drawn_count > 0, "glyph should produce visible pixels");
        let transparent_count = img.pixels().filter(|p| p.0[3] == 0).count();
        assert!(transparent_count > 0, "canvas background should stay transparent");
    }

    #[test]
    fn large_grid_at_export_resolution_does_not_panic() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let lyrics_len = 8000usize;
        let resolved = renderer::find_optimal_grid("1:1", &font_data, 0.0, 2.0, lyrics_len)
            .expect("grid layout");

        let mut chars = Vec::new();
        let mut colors = Vec::new();
        for _ in 0..resolved.rows {
            chars.push(vec!['M'; resolved.cols as usize]);
            colors.push(vec![[128, 64, 32]; resolved.cols as usize]);
        }

        let grid = AsciiGrid {
            chars,
            colors,
            width: resolved.cols,
            height: resolved.rows,
            char_aspect_ratio: 1.0,
            target_width: 1920,
            target_height: 1920,
            cell_width: 1920.0 / resolved.cols as f32,
            cell_height: 1920.0 / resolved.rows as f32,
            cell_width_ref: resolved.cell_width_ref,
            cell_height_ref: resolved.cell_height_ref,
            font_size_ref: resolved.font_size_ref,
            line_spacing: resolved.line_spacing,
            lyrics_chars_placed: lyrics_len as u32,
            lyrics_total: lyrics_len as u32,
        };

        let mut settings = test_settings();
        settings.resolution = 1920;
        settings.aspect_ratio = "1:1".to_string();

        let img = render_rgba(&grid, &settings, &font_data).expect("render large export");
        assert_eq!(img.width(), 1920);
        assert_eq!(img.height(), 1920);
    }
}
