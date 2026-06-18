import { writable } from 'svelte/store';
import type { AsciiSettings, FontInfo, AsciiGrid, LyricsData } from '../types';

// Shared app state — components subscribe locally; SettingsPanel owns the Tauri invoke pipeline.

export const fonts = writable<FontInfo[]>([]);
export const selectedFont = writable<FontInfo | null>(null);

export const settings = writable<AsciiSettings>({
  line_height: 2,
  char_spacing: 0,
  space_spacing: 8,
  font_path: '',
  font_style: 'Regular',
  aspect_ratio: '1:1',
  resolution: 1920,
  method: 'average',
});

export const imagePath = writable<string>('');

export const lyricsText = writable<string>('');
export const lyricsData = writable<LyricsData | null>(null);
export const removeSpaces = writable<boolean>(false);

// asciiGrid: character/color layout from generate_ascii (resolution-independent).
// asciiPreview: base64 PNG from render_preview at current resolution — what AsciiPreview displays.
export const asciiGrid = writable<AsciiGrid | null>(null);
export const asciiPreview = writable<string>('');

export const isGenerating = writable<boolean>(false);
export const isExporting = writable<boolean>(false);
export const isFetchingLyrics = writable<boolean>(false);
export const fetchProgress = writable<string>('');

export const statusMessage = writable<string>('READY');

export const previewZoom = writable<number>(1);
