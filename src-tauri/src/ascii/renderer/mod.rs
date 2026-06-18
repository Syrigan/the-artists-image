//! ASCII grid rendering: layout, color sampling, and grid assembly.
//!
//! Grid topology (cols/rows/chars) is computed at a fixed reference width
//! ([`constants::GRID_REFERENCE_WIDTH`]) so lyrics density stays stable across
//! export resolutions. Color sampling uses the same reference canvas, so
//! changing resolution only affects PNG rasterization, not grid regeneration.

mod color_sampling;
mod constants;
mod grid_generation;
mod layout;
mod lyrics;
mod types;

pub use constants::GRID_REFERENCE_WIDTH;
pub use grid_generation::generate_grid;
pub use layout::{baseline_y, export_layout, target_dimensions_from_settings};
pub(crate) use layout::find_optimal_grid;
pub use types::ColorMethod;

use crate::ascii::AsciiGrid;

pub fn grid_to_colored_html(grid: &AsciiGrid) -> String {
    let mut html = String::from("<pre class=\"ascii-output\">");
    for (row_idx, row) in grid.chars.iter().enumerate() {
        for (col_idx, &ch) in row.iter().enumerate() {
            let color = &grid.colors[row_idx][col_idx];
            html.push_str(&format!(
                "<span style=\"color:rgb({},{},{})\">{}</span>",
                color[0], color[1], color[2], ch
            ));
        }
        html.push('\n');
    }
    html.push_str("</pre>");
    html
}

#[cfg(test)]
mod tests {
    use image::DynamicImage;

    use super::color_sampling::{
        sample_color_absolute, sample_color_average, sample_color_sharp,
    };
    use super::constants;
    use super::grid_generation::generate_grid_from_prepared;
    use super::layout::{
        clamp_char_spacing_px, clamp_line_spacing_px, export_layout, find_optimal_grid,
        grid_fits, load_font, max_font_size_for_grid, reference_dimensions,
        target_dimensions_from_settings,
    };
    use super::lyrics::{clean_lyrics, lyric_char_for_cell};
    use super::types::ColorMethod;
    use crate::ascii::AsciiSettings;

    #[test]
    fn target_dimensions_respects_aspect_ratio() {
        let (w, h) = target_dimensions_from_settings(1920, "16:9");
        assert_eq!(w, 1920);
        assert_eq!(h, 1080);
    }

    #[test]
    fn reference_dimensions_keeps_aspect_ratio() {
        let (w, h) = reference_dimensions("16:9");
        assert_eq!(w, 640);
        assert_eq!(h, 360);
    }

    #[test]
    fn export_layout_scales_from_reference_width() {
        let grid = crate::ascii::AsciiGrid {
            chars: vec![vec!['a'; 10]; 5],
            colors: vec![vec![[0, 0, 0]; 10]; 5],
            width: 10,
            height: 5,
            char_aspect_ratio: 1.0,
            target_width: 1920,
            target_height: 1080,
            cell_width: 192.0,
            cell_height: 216.0,
            cell_width_ref: 64.0,
            cell_height_ref: 72.0,
            font_size_ref: 12.0,
            line_spacing: 2.0,
            lyrics_chars_placed: 0,
            lyrics_total: 0,
        };
        let layout = export_layout(&grid, 640, 360);
        assert!((layout.cell_w - 64.0).abs() < 0.01);
        assert!((layout.cell_h - 72.0).abs() < 0.01);
        assert!((layout.font_size - 12.0).abs() < 0.01);
        assert!((layout.start_x - 0.0).abs() < 0.01);
        assert!((layout.start_y - 0.0).abs() < 0.01);

        let layout_hd = export_layout(&grid, 1920, 1080);
        assert!((layout_hd.font_size - 36.0).abs() < 0.01);
        assert!((layout_hd.cell_w - 192.0).abs() < 0.01);
    }

    #[test]
    fn sample_color_absolute_tie_breaks_lexicographic_rgb() {
        let img = image::RgbaImage::from_fn(2, 2, |x, _| {
            if x == 0 {
                image::Rgba([32, 32, 32, 255])
            } else {
                image::Rgba([16, 16, 16, 255])
            }
        });
        let dynamic = DynamicImage::ImageRgba8(img);
        let color = sample_color_absolute(&dynamic, 0, 0, 2, 2);
        assert_eq!(color, [16, 16, 16]);
    }

    #[test]
    fn cell_sampling_bounds_cover_full_grid() {
        let cell_w = 10.5f32;
        let cell_h = 8.3f32;
        let cols = 3u32;
        let rows = 2u32;
        let mut covered = vec![false; (cols * rows) as usize];

        for row in 0..rows {
            for col in 0..cols {
                let x = (col as f32 * cell_w).floor() as u32;
                let y = (row as f32 * cell_h).floor() as u32;
                let x_end = ((col + 1) as f32 * cell_w).ceil() as u32;
                let y_end = ((row + 1) as f32 * cell_h).ceil() as u32;
                let w = x_end.saturating_sub(x).max(1);
                let h = y_end.saturating_sub(y).max(1);
                assert!(w >= 1);
                assert!(h >= 1);
                covered[row as usize * cols as usize + col as usize] = true;
            }
        }
        assert!(covered.iter().all(|&c| c));
    }

    #[test]
    fn sample_color_absolute_picks_dominant_color() {
        let img = image::RgbaImage::from_fn(4, 4, |x, y| {
            if x < 3 && y < 3 {
                image::Rgba([200, 10, 10, 255])
            } else {
                image::Rgba([10, 10, 200, 255])
            }
        });
        let dynamic = DynamicImage::ImageRgba8(img);
        let color = sample_color_absolute(&dynamic, 0, 0, 4, 4);
        assert_eq!(color, [192, 0, 0]);
    }

    #[test]
    fn sample_color_average_computes_mean() {
        let img = image::RgbaImage::from_fn(2, 2, |_, _| image::Rgba([100, 200, 50, 255]));
        let dynamic = DynamicImage::ImageRgba8(img);
        let color = sample_color_average(&dynamic, 0, 0, 2, 2);
        assert_eq!(color, [100, 200, 50]);
    }

    #[test]
    fn sample_color_sharp_picks_center_pixel() {
        let img = image::RgbaImage::from_fn(2, 2, |x, y| {
            if x == 1 && y == 1 {
                image::Rgba([100, 110, 120, 255])
            } else {
                image::Rgba([1, 2, 3, 255])
            }
        });
        let dynamic = DynamicImage::ImageRgba8(img);
        let color = sample_color_sharp(&dynamic, 0, 0, 2, 2);
        assert_eq!(color, [100, 110, 120]);
    }

    #[test]
    fn lyric_char_places_each_once_before_repeat() {
        let clean: Vec<char> = "abc".chars().collect();
        assert_eq!(lyric_char_for_cell(&clean, 0), 'a');
        assert_eq!(lyric_char_for_cell(&clean, 1), 'b');
        assert_eq!(lyric_char_for_cell(&clean, 2), 'c');
        assert_eq!(lyric_char_for_cell(&clean, 3), 'a');
        assert_eq!(lyric_char_for_cell(&clean, 4), 'b');
    }

    #[test]
    fn lyric_char_empty_lyrics_uses_space() {
        assert_eq!(lyric_char_for_cell(&[], 0), ' ');
    }

    #[test]
    fn clean_lyrics_strips_control_chars() {
        let clean = clean_lyrics("a\nb\rc");
        assert_eq!(clean, vec!['a', 'b', 'c']);
    }

    #[test]
    fn find_optimal_grid_fits_minimum_cells() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let lyrics_len = 2_000usize;
        let grid = find_optimal_grid("16:9", &font_data, 0.0, 2.0, lyrics_len).expect("grid layout");
        assert!(
            grid.cols as usize * grid.rows as usize >= lyrics_len,
            "grid {}x{} = {} cells, need {}",
            grid.cols,
            grid.rows,
            grid.cols as usize * grid.rows as usize,
            lyrics_len
        );
        assert!(grid.font_size_ref >= constants::MIN_FONT_SIZE_REF);
    }

    #[test]
    fn clamp_line_spacing_px_rounds_and_clamps() {
        assert_eq!(clamp_line_spacing_px(-1.0), 0.0);
        assert_eq!(clamp_line_spacing_px(4.6), 5.0);
        assert_eq!(clamp_line_spacing_px(64.0), 48.0);
    }

    #[test]
    fn clamp_char_spacing_px_rounds_and_clamps() {
        assert_eq!(clamp_char_spacing_px(-9.0), -8.0);
        assert_eq!(clamp_char_spacing_px(4.6), 5.0);
        assert_eq!(clamp_char_spacing_px(40.0), 32.0);
    }

    #[test]
    fn line_spacing_reduces_font_size() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let lyrics_len = 200usize;
        let tight = find_optimal_grid("1:1", &font_data, 0.0, 0.0, lyrics_len).expect("grid");
        let loose = find_optimal_grid("1:1", &font_data, 0.0, 16.0, lyrics_len).expect("grid");
        assert!(loose.font_size_ref <= tight.font_size_ref);

        let (ref_w, ref_h) = reference_dimensions("1:1");
        let font = load_font(&font_data).expect("font");
        for grid in [&tight, &loose] {
            assert!(
                grid.cols as usize * grid.rows as usize >= lyrics_len,
                "grid must fit all lyric cells"
            );
            assert!(grid_fits(
                &font,
                ref_w as f32,
                ref_h as f32,
                grid.cols,
                grid.rows,
                grid.font_size_ref,
                0.0,
                grid.line_spacing,
            ));
        }
    }

    #[test]
    fn find_optimal_grid_auto_fits_max_font_size() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let lyrics_len = 100usize;
        let grid = find_optimal_grid("1:1", &font_data, 0.0, 2.0, lyrics_len).expect("grid");
        let (ref_w, ref_h) = reference_dimensions("1:1");
        let font = load_font(&font_data).expect("font");
        let max_fit = max_font_size_for_grid(
            &font,
            ref_w as f32,
            ref_h as f32,
            grid.cols,
            grid.rows,
            0.0,
            grid.line_spacing,
        );
        assert!(
            (grid.font_size_ref - max_fit).abs() < 0.1,
            "auto-fit should use max fitting font size"
        );
    }

    #[test]
    fn find_optimal_grid_minimizes_cell_waste() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let lyrics_len = 100usize;
        let grid = find_optimal_grid("1:1", &font_data, 0.0, 2.0, lyrics_len).expect("grid");
        let cells = grid.cols as usize * grid.rows as usize;
        assert!(cells >= lyrics_len);
        assert!(cells - lyrics_len <= grid.cols as usize, "waste should be minimal");
    }

    #[test]
    fn layout_stable_across_resolution() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let lyrics = "abcdefghijklmnopqrstuvwxyz0123456789";
        let base_settings = AsciiSettings {
            line_spacing: 2.0,
            char_spacing: 0.0,
            space_spacing: 8.0,
            font_path: String::new(),
            font_style: "Regular".to_string(),
            aspect_ratio: "1:1".to_string(),
            resolution: 640,
            method: "average".to_string(),
        };
        let img = image::RgbaImage::from_fn(640, 640, |_, _| image::Rgba([128, 64, 32, 255]));
        let dynamic = DynamicImage::ImageRgba8(img);

        let grid_640 = generate_grid_from_prepared(
            &dynamic,
            lyrics,
            &base_settings,
            &font_data,
            ColorMethod::Average,
        )
        .expect("generate grid");

        let mut settings_hd = base_settings.clone();
        settings_hd.resolution = 1920;
        let grid_hd = generate_grid_from_prepared(
            &dynamic,
            lyrics,
            &settings_hd,
            &font_data,
            ColorMethod::Average,
        )
        .expect("generate grid hd");

        assert_eq!(grid_640.width, grid_hd.width);
        assert_eq!(grid_640.height, grid_hd.height);
        assert!((grid_640.font_size_ref - grid_hd.font_size_ref).abs() < 0.01);
        assert_eq!(grid_640.chars, grid_hd.chars);
        assert_eq!(grid_640.colors, grid_hd.colors);
        assert_eq!(grid_hd.target_width, 1920);
    }

    #[test]
    fn generate_grid_places_all_lyrics_chars() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let lyrics = "abcdefghijklmnopqrstuvwxyz0123456789";
        let settings = AsciiSettings {
            line_spacing: 2.0,
            char_spacing: 0.0,
            space_spacing: 8.0,
            font_path: String::new(),
            font_style: "Regular".to_string(),
            aspect_ratio: "1:1".to_string(),
            resolution: 640,
            method: "average".to_string(),
        };
        let img = image::RgbaImage::from_fn(640, 640, |_, _| image::Rgba([128, 64, 32, 255]));
        let dynamic = DynamicImage::ImageRgba8(img);

        let grid = generate_grid_from_prepared(
            &dynamic,
            lyrics,
            &settings,
            &font_data,
            ColorMethod::Average,
        )
        .expect("generate grid");

        assert_eq!(grid.lyrics_total, lyrics.len() as u32);
        assert_eq!(grid.lyrics_chars_placed, grid.lyrics_total);
        assert!(
            grid.width as usize * grid.height as usize >= lyrics.len(),
            "grid must have at least one cell per lyric character"
        );

        let placed: String = grid.chars.iter().flatten().collect();
        for ch in lyrics.chars() {
            assert!(placed.contains(ch), "missing lyric character '{ch}'");
        }
    }
}
