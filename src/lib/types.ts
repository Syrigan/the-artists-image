export interface FontInfo {
  name: string;
  path: string;
  is_monospace: boolean;
}

export interface AsciiSettings {
  line_height: number;
  char_spacing: number;
  space_spacing: number;
  font_path: string;
  font_style: string;
  aspect_ratio: string;
  resolution: number;
  method: string;
}

export interface AsciiGrid {
  chars: string[][];
  colors: number[][][];
  width: number;
  height: number;
  char_aspect_ratio: number;
  target_width: number;
  target_height: number;
  cell_width: number;
  cell_height: number;
  // *_ref fields are measured at GRID_REFERENCE_WIDTH (640px); export_layout scales them to resolution.
  cell_width_ref: number;
  cell_height_ref: number;
  font_size_ref: number;
  line_spacing: number;
  lyrics_chars_placed: number;
  lyrics_total: number;
}

// Passed to render_preview and export_ascii on the Rust side.
export interface ExportSettings {
  format: string;
  resolution: number;
  aspect_ratio: string;
  line_height: number;
  char_spacing: number;
  space_spacing: number;
  font_path: string;
  jpeg_quality: number;
}

export interface SongLyrics {
  name: string;
  lyrics: string;
  error?: string | null;
}

export interface LyricsData {
  artist: string;
  album: string;
  songs: SongLyrics[];
}

export type RenderMethod = 'average' | 'absolute' | 'sharp';
export type OutputFormat = 'png' | 'jpeg' | 'svg';
