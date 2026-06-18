use super::renderer::{self, ColorMethod};
use super::{AsciiGrid, AsciiSettings};

/// Average RGB color per grid cell.
pub fn generate(
    image_path: &str,
    lyrics: &str,
    settings: &AsciiSettings,
    font_data: &[u8],
) -> Result<AsciiGrid, String> {
    renderer::generate_grid(image_path, lyrics, settings, font_data, ColorMethod::Average)
}
