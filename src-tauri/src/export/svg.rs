//! SVG export: same `export_layout` positioning as PNG, with `<text>` elements.

use ab_glyph::FontRef;
use super::ExportSettings;
use crate::ascii::{AsciiGrid, renderer};

/// Write SVG using scaled layout from the grid's reference metrics.
pub fn export(grid: &AsciiGrid, settings: &ExportSettings, font_data: &[u8], output_path: &str) -> Result<(), String> {
    let font = FontRef::try_from_slice(font_data)
        .map_err(|e| format!("Failed to load font: {}", e))?;

    let (svg_w, svg_h) = renderer::target_dimensions_from_settings(
        settings.resolution, &settings.aspect_ratio
    );

    let layout = renderer::export_layout(grid, svg_w, svg_h);

    let mut svg_content = String::new();
    svg_content.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
"#,
        svg_w, svg_h, svg_w, svg_h
    ));

    for (row_idx, row) in grid.chars.iter().enumerate() {
        let mut x_pos = layout.start_x;
        let cell_y = layout.start_y + (row_idx as f32 * layout.cell_h);
        let y_pos = renderer::baseline_y(&font, cell_y, layout.font_size);

        for (col_idx, &ch) in row.iter().enumerate() {
            if ch != ' ' {
                let c = &grid.colors[row_idx][col_idx];
                let escaped = match ch {
                    '<' => "&lt;".to_string(),
                    '>' => "&gt;".to_string(),
                    '&' => "&amp;".to_string(),
                    '"' => "&quot;".to_string(),
                    _ => ch.to_string(),
                };
                svg_content.push_str(&format!(
                    r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="{}" fill="rgb({},{},{})">{}</text>"##,
                    x_pos, y_pos, layout.font_size, c[0], c[1], c[2], escaped
                ));
            }

            x_pos += layout.cell_w;
        }
    }

    svg_content.push_str("\n</svg>");

    std::fs::write(output_path, svg_content)
        .map_err(|e| format!("Failed to save SVG: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ascii::AsciiGrid;

    fn test_grid() -> AsciiGrid {
        AsciiGrid {
            chars: vec![vec![' ', 'X']],
            colors: vec![vec![[255, 0, 0], [0, 255, 0]]],
            width: 2,
            height: 1,
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
            format: "svg".to_string(),
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
    fn spaces_emit_no_filled_rects_and_no_background() {
        let font_data = crate::fonts::load_font_data("").expect("monospace font");
        let dir = std::env::temp_dir();
        let path = dir.join(format!("test_export_{}.svg", std::process::id()));
        let path_str = path.to_string_lossy().to_string();

        export(&test_grid(), &test_settings(), &font_data, &path_str).expect("export svg");
        let content = std::fs::read_to_string(&path).expect("read svg");

        assert!(!content.contains("#1a1b26"), "SVG should not have opaque background fill");
        assert!(!content.contains("<rect"), "SVG should not emit filled rects for spaces");
        assert!(content.contains("<text"), "SVG should still emit text for non-space chars");

        let _ = std::fs::remove_file(path);
    }
}
