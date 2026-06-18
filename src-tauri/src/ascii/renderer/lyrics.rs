//! Lyrics normalization and per-cell character placement.

/// Strip control characters; preserve visible text and spaces.
pub fn clean_lyrics(lyrics: &str) -> Vec<char> {
    lyrics.chars().filter(|c| !c.is_control()).collect()
}

/// Place lyrics in order: each character appears once before any repeat.
pub fn lyric_char_for_cell(clean: &[char], cell_index: usize) -> char {
    if clean.is_empty() {
        return ' ';
    }
    let len = clean.len();
    if cell_index < len {
        clean[cell_index]
    } else {
        clean[(cell_index - len) % len]
    }
}
