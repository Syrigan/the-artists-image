//! Full grid generation: layout topology + reference-scale color sampling.

use std::time::Instant;

use image::DynamicImage;

use super::color_sampling::sample_cell_color;
use super::layout::{find_optimal_grid, reference_dimensions, target_dimensions_from_settings};
use super::lyrics::{clean_lyrics, lyric_char_for_cell};
use super::types::ColorMethod;
use crate::ascii::{AsciiGrid, AsciiSettings};

/// Load and crop the source image to exact canvas dimensions (cover fit).
pub fn load_and_prepare_image(path: &str, width: u32, height: u32) -> Result<DynamicImage, String> {
    let img = image::open(path).map_err(|e| format!("Failed to load image: {}", e))?;

    let iw = img.width() as f32;
    let ih = img.height() as f32;
    let scale = (width as f32 / iw).max(height as f32 / ih);
    let mut scaled = img.resize(
        (iw * scale).round() as u32,
        (ih * scale).round() as u32,
        image::imageops::FilterType::Lanczos3,
    );

    let x = (scaled.width().saturating_sub(width)) / 2;
    let y = (scaled.height().saturating_sub(height)) / 2;
    Ok(scaled.crop(x, y, width, height))
}

pub fn generate_grid(
    image_path: &str,
    lyrics: &str,
    settings: &AsciiSettings,
    font_data: &[u8],
    color_method: ColorMethod,
) -> Result<AsciiGrid, String> {
    let t0 = Instant::now();
    let (sample_w, sample_h) = reference_dimensions(&settings.aspect_ratio);
    let img = load_and_prepare_image(image_path, sample_w, sample_h)?;
    eprintln!(
        "[generate] image prep @ {}x{}: {:.1}ms",
        sample_w,
        sample_h,
        t0.elapsed().as_secs_f64() * 1000.0
    );
    generate_grid_from_prepared(&img, lyrics, settings, font_data, color_method)
}

pub fn generate_grid_from_prepared(
    img: &DynamicImage,
    lyrics: &str,
    settings: &AsciiSettings,
    font_data: &[u8],
    color_method: ColorMethod,
) -> Result<AsciiGrid, String> {
    let t0 = Instant::now();

    let (sample_w, sample_h) = reference_dimensions(&settings.aspect_ratio);
    let (export_w, export_h) =
        target_dimensions_from_settings(settings.resolution, &settings.aspect_ratio);
    let clean = clean_lyrics(lyrics);
    let lyrics_total = clean.len() as u32;
    let lyric_count = clean.len().max(1);

    let grid = find_optimal_grid(
        &settings.aspect_ratio,
        font_data,
        settings.char_spacing,
        settings.line_spacing,
        lyric_count,
    )?;

    let t_layout = t0.elapsed();

    let cols = grid.cols;
    let rows = grid.rows;
    let cell_w = sample_w as f32 / cols as f32;
    let cell_h = sample_h as f32 / rows as f32;
    let lyrics_chars_placed = if clean.is_empty() {
        0
    } else {
        clean.len() as u32
    };

    let mut chars = Vec::with_capacity(rows as usize);
    let mut colors = Vec::with_capacity(rows as usize);
    let mut cell_index = 0usize;

    for row in 0..rows {
        let mut line = Vec::with_capacity(cols as usize);
        let mut color_line = Vec::with_capacity(cols as usize);
        for col in 0..cols {
            let x = (col as f32 * cell_w).floor() as u32;
            let y = (row as f32 * cell_h).floor() as u32;
            let x_end = ((col + 1) as f32 * cell_w).ceil() as u32;
            let y_end = ((row + 1) as f32 * cell_h).ceil() as u32;
            let w = x_end.saturating_sub(x).max(1);
            let h = y_end.saturating_sub(y).max(1);

            let color = sample_cell_color(img, x, y, w, h, color_method);
            let c = lyric_char_for_cell(&clean, cell_index);
            cell_index += 1;
            line.push(c);
            color_line.push(color);
        }
        chars.push(line);
        colors.push(color_line);
    }

    let t_sample = t0.elapsed() - t_layout;
    eprintln!(
        "[generate] layout: {:.1}ms, color sampling ({} cells @ ref): {:.1}ms",
        t_layout.as_secs_f64() * 1000.0,
        cols * rows,
        t_sample.as_secs_f64() * 1000.0
    );

    Ok(AsciiGrid {
        chars,
        colors,
        width: cols,
        height: rows,
        char_aspect_ratio: cell_w / cell_h,
        target_width: export_w,
        target_height: export_h,
        cell_width: export_w as f32 / cols as f32,
        cell_height: export_h as f32 / rows as f32,
        cell_width_ref: grid.cell_width_ref,
        cell_height_ref: grid.cell_height_ref,
        font_size_ref: grid.font_size_ref,
        line_spacing: grid.line_spacing,
        lyrics_chars_placed,
        lyrics_total,
    })
}
