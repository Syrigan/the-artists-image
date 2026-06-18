//! Canvas dimensions, auto-fit grid search, and export layout scaling.

use ab_glyph::{Font, FontRef, ScaleFont};

use super::constants::{
    GRID_REFERENCE_WIDTH, MAX_CHAR_SPACING, MAX_FONT_SIZE_REF, MAX_LINE_SPACING, MIN_CHAR_SPACING,
    MIN_FONT_SIZE_REF, MIN_LINE_SPACING,
};
use super::types::{ExportLayout, ResolvedGrid};

pub fn parse_aspect_ratio(ratio: &str) -> (f32, f32) {
    let parts: Vec<&str> = ratio.split(':').collect();
    if parts.len() == 2 {
        let w = parts[0].trim().parse::<f32>().unwrap_or(1.0);
        let h = parts[1].trim().parse::<f32>().unwrap_or(1.0);
        (w, h)
    } else {
        (1.0, 1.0)
    }
}

pub fn target_dimensions_from_settings(resolution: u32, aspect_ratio: &str) -> (u32, u32) {
    let (ar_w, ar_h) = parse_aspect_ratio(aspect_ratio);
    let w = resolution;
    let h = ((w as f32) * ar_h / ar_w).round() as u32;
    (w.max(1), h.max(1))
}

pub fn reference_dimensions(aspect_ratio: &str) -> (u32, u32) {
    target_dimensions_from_settings(GRID_REFERENCE_WIDTH, aspect_ratio)
}

pub fn clamp_line_spacing_px(value: f32) -> f32 {
    value.round().clamp(MIN_LINE_SPACING, MAX_LINE_SPACING)
}

pub fn clamp_char_spacing_px(value: f32) -> f32 {
    value.round().clamp(MIN_CHAR_SPACING, MAX_CHAR_SPACING)
}

pub fn load_font(font_data: &[u8]) -> Result<FontRef<'_>, String> {
    FontRef::try_from_slice(font_data).map_err(|e| format!("Failed to load font: {}", e))
}

fn glyph_width(font: &FontRef<'_>, font_size: f32, char_spacing: f32) -> f32 {
    let glyph = font.glyph_id('M').with_scale(font_size);
    let bounds = font.glyph_bounds(&glyph);
    (bounds.max.x - bounds.min.x) + char_spacing
}

fn glyph_row_pitch(font: &FontRef<'_>, font_size: f32, line_spacing: f32) -> f32 {
    let scaled = font.as_scaled(font_size);
    let ascent = scaled.ascent();
    let descent = scaled.descent();
    (ascent - descent) + line_spacing
}

pub fn grid_fits(
    font: &FontRef<'_>,
    ref_w: f32,
    ref_h: f32,
    cols: u32,
    rows: u32,
    font_size: f32,
    char_spacing: f32,
    line_spacing: f32,
) -> bool {
    if cols == 0 || rows == 0 {
        return false;
    }
    let total_w = cols as f32 * glyph_width(font, font_size, char_spacing);
    let total_h = rows as f32 * glyph_row_pitch(font, font_size, line_spacing);
    total_w <= ref_w + 0.01 && total_h <= ref_h + 0.01
}

pub(crate) fn max_font_size_for_grid(
    font: &FontRef<'_>,
    ref_w: f32,
    ref_h: f32,
    cols: u32,
    rows: u32,
    char_spacing: f32,
    line_spacing: f32,
) -> f32 {
    let mut lo = MIN_FONT_SIZE_REF;
    let mut hi = MAX_FONT_SIZE_REF;
    if !grid_fits(font, ref_w, ref_h, cols, rows, lo, char_spacing, line_spacing) {
        return 0.0;
    }
    while hi - lo > 0.05 {
        let mid = (lo + hi) / 2.0;
        if grid_fits(font, ref_w, ref_h, cols, rows, mid, char_spacing, line_spacing) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

#[derive(Clone, Copy)]
struct GridCandidate {
    cols: u32,
    rows: u32,
    font_size_ref: f32,
    waste: usize,
    aspect_penalty: f32,
}

fn candidate_score(c: GridCandidate) -> (i64, i64, i64) {
    let font_bits = (c.font_size_ref * 1000.0) as i64;
    let waste_bits = -(c.waste as i64);
    let aspect_bits = -(c.aspect_penalty * 1000.0) as i64;
    (font_bits, waste_bits, aspect_bits)
}

fn row_search_bounds(lyric_count: usize, ref_w: u32, ref_h: u32) -> (u32, u32) {
    if lyric_count <= 1 {
        return (1, 1);
    }
    let n = lyric_count as f32;
    let ideal = ((n * ref_h as f32) / ref_w as f32).sqrt().ceil().max(1.0) as u32;
    let min_rows = (ideal / 2).max(1);
    let max_rows = (ideal * 2).min(lyric_count as u32).max(min_rows);
    (min_rows, max_rows)
}

fn evaluate_row_count(
    font: &FontRef<'_>,
    ref_w: u32,
    ref_h: u32,
    lyric_count: usize,
    rows: u32,
    char_spacing: f32,
    line_spacing: f32,
) -> Option<GridCandidate> {
    let cols = lyric_count.div_ceil(rows as usize) as u32;
    let font_size_ref = max_font_size_for_grid(
        font,
        ref_w as f32,
        ref_h as f32,
        cols,
        rows,
        char_spacing,
        line_spacing,
    );
    if font_size_ref < MIN_FONT_SIZE_REF {
        return None;
    }
    let waste = cols as usize * rows as usize - lyric_count;
    let grid_aspect = cols as f32 / rows as f32;
    let canvas_aspect = ref_w as f32 / ref_h as f32;
    let aspect_penalty = (grid_aspect - canvas_aspect).abs();
    Some(GridCandidate {
        cols,
        rows,
        font_size_ref,
        waste,
        aspect_penalty,
    })
}

/// Auto-fit grid: maximize font size for lyrics, minimize wasted cells.
pub fn find_optimal_grid(
    aspect_ratio: &str,
    font_data: &[u8],
    char_spacing: f32,
    line_spacing: f32,
    lyric_count: usize,
) -> Result<ResolvedGrid, String> {
    let (ref_w, ref_h) = reference_dimensions(aspect_ratio);
    let char_spacing = clamp_char_spacing_px(char_spacing);
    let line_spacing = clamp_line_spacing_px(line_spacing);
    let lyric_count = lyric_count.max(1);
    let font = load_font(font_data)?;

    let (row_lo, row_hi) = row_search_bounds(lyric_count, ref_w, ref_h);
    let mut best: Option<GridCandidate> = None;

    for rows in row_lo..=row_hi {
        if let Some(candidate) = evaluate_row_count(
            &font,
            ref_w,
            ref_h,
            lyric_count,
            rows,
            char_spacing,
            line_spacing,
        ) {
            if best.map_or(true, |b| candidate_score(candidate) > candidate_score(b)) {
                best = Some(candidate);
            }
        }
    }

    // Fallback: scan all row counts if the window missed a better fit.
    if lyric_count as u32 <= 512 {
        for rows in 1..=lyric_count as u32 {
            if rows >= row_lo && rows <= row_hi {
                continue;
            }
            if let Some(candidate) = evaluate_row_count(
                &font,
                ref_w,
                ref_h,
                lyric_count,
                rows,
                char_spacing,
                line_spacing,
            ) {
                if best.map_or(true, |b| candidate_score(candidate) > candidate_score(b)) {
                    best = Some(candidate);
                }
            }
        }
    }

    let best = best.ok_or_else(|| {
        format!(
            "Could not fit {} lyric characters on canvas (try reducing char spacing)",
            lyric_count
        )
    })?;

    Ok(ResolvedGrid {
        cols: best.cols,
        rows: best.rows,
        font_size_ref: best.font_size_ref,
        line_spacing,
        cell_width_ref: ref_w as f32 / best.cols as f32,
        cell_height_ref: ref_h as f32 / best.rows as f32,
    })
}

/// Scale grid cell metrics from reference layout to export canvas size.
pub fn export_layout(
    grid: &crate::ascii::AsciiGrid,
    canvas_width: u32,
    canvas_height: u32,
) -> ExportLayout {
    let scale = canvas_width as f32 / GRID_REFERENCE_WIDTH as f32;
    let cell_w = grid.cell_width_ref * scale;
    let cell_h = grid.cell_height_ref * scale;
    let font_size = grid.font_size_ref * scale;
    let line_spacing = grid.line_spacing * scale;
    let content_w = grid.width as f32 * cell_w;
    let content_h = grid.height as f32 * cell_h;
    let start_x = ((canvas_width as f32 - content_w) / 2.0).max(0.0);
    let start_y = ((canvas_height as f32 - content_h) / 2.0).max(0.0);
    ExportLayout {
        cell_w,
        cell_h,
        font_size,
        line_spacing,
        start_x,
        start_y,
    }
}

pub fn baseline_y(font: &FontRef<'_>, cell_y: f32, font_size: f32) -> f32 {
    let ascent = font.as_scaled(font_size).ascent();
    cell_y + ascent
}
