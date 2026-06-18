import type { AsciiSettings, ExportSettings, OutputFormat } from './types';

/** Grid layout reference width; matches backend GRID_REFERENCE_WIDTH. */
export const GRID_REFERENCE_WIDTH = 640;

export interface BuildExportSettingsOptions {
  format?: OutputFormat | string;
  jpegQuality?: number;
}

/** Build export settings shared by preview and export flows. */
export function buildExportSettings(
  settings: AsciiSettings,
  options: BuildExportSettingsOptions = {},
): ExportSettings {
  const {
    format = 'png',
    jpegQuality = 95,
  } = options;

  return {
    format,
    resolution: settings.resolution,
    aspect_ratio: settings.aspect_ratio,
    line_height: settings.line_height,
    char_spacing: settings.char_spacing,
    space_spacing: settings.space_spacing,
    font_path: settings.font_path,
    jpeg_quality: jpegQuality,
  };
}

/** Settings that require full grid regeneration (layout or color sampling). */
export function gridGenerationKey(settings: AsciiSettings): string {
  return [
    settings.aspect_ratio,
    settings.method,
    settings.line_height,
    settings.char_spacing,
    settings.font_path,
  ].join('\0');
}
