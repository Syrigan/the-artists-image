use std::collections::HashMap;
use std::sync::Mutex;

use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use serde::{Deserialize, Serialize};

const MONO_HINTS: &[&str] = &[
    "mono",
    "courier",
    "consolas",
    "menlo",
    "lucida console",
    "sf mono",
    "jetbrains",
    "fira code",
    "source code",
    "cascadia",
    "inconsolata",
    "ubuntu mono",
    "dejavu sans mono",
    "liberation mono",
    "ibm plex mono",
    "roboto mono",
    "dm mono",
    "space mono",
];

static FONT_DATA_CACHE: Mutex<Option<HashMap<String, Vec<u8>>>> = Mutex::new(None);

fn looks_monospace(family: &str, postscript: Option<&str>) -> bool {
    let family_lower = family.to_lowercase();
    let ps_lower = postscript.unwrap_or("").to_lowercase();
    MONO_HINTS
        .iter()
        .any(|hint| family_lower.contains(hint) || ps_lower.contains(hint))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub name: String,
    pub path: String,
    pub is_monospace: bool,
}

pub fn get_system_fonts() -> Result<Vec<FontInfo>, String> {
    let source = SystemSource::new();
    let mut fonts = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let all_fonts = source.all_fonts().map_err(|e| format!("Failed to enumerate fonts: {}", e))?;

    for handle in all_fonts {
        let font = match handle.load() {
            Ok(f) => f,
            Err(_) => continue,
        };

        let postscript = font.postscript_name();
        let is_mono = looks_monospace(&font.family_name(), postscript.as_deref());

        let name = font.family_name();
        if seen.contains(&name) {
            continue;
        }
        seen.insert(name.clone());

        let path = match handle {
            font_kit::handle::Handle::Path { ref path, .. } => {
                path.to_string_lossy().to_string()
            }
            font_kit::handle::Handle::Memory { .. } => continue,
        };

        fonts.push(FontInfo {
            name,
            path,
            is_monospace: is_mono,
        });
    }

    fonts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(fonts)
}

pub fn load_font_data(font_path: &str) -> Result<Vec<u8>, String> {
    let cache_key = if font_path.is_empty() {
        "__default__".to_string()
    } else {
        font_path.to_string()
    };

    let mut guard = FONT_DATA_CACHE
        .lock()
        .map_err(|_| "Font cache lock poisoned".to_string())?;

    if guard.is_none() {
        *guard = Some(HashMap::new());
    }

    let cache = guard.as_mut().expect("font cache initialized");
    if let Some(data) = cache.get(&cache_key) {
        return Ok(data.clone());
    }

    let data = if font_path.is_empty() {
        load_default_font()?
    } else {
        std::fs::read(font_path).map_err(|e| format!("Failed to read font file: {}", e))?
    };

    cache.insert(cache_key, data.clone());
    Ok(data)
}

fn load_default_font() -> Result<Vec<u8>, String> {
    let source = SystemSource::new();
    let families = &[FamilyName::Monospace];
    let font = source
        .select_best_match(families, &Properties::new())
        .map_err(|e| format!("Failed to find monospace font: {}", e))?;

    let loaded = font.load().map_err(|e| format!("Failed to load font: {}", e))?;
    match loaded.copy_font_data() {
        Some(data) => Ok(data.to_vec()),
        None => Err("Font has no data".to_string()),
    }
}
