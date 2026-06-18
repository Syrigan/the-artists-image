<script lang="ts">
  import { save } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import {
    settings,
    asciiGrid,
    isExporting,
    isGenerating,
    statusMessage,
    imagePath,
    lyricsText,
  } from '../stores/settings';
  import { buildExportSettings } from '../exportSettings';
  import type { AsciiGrid } from '../types';

  let format = $state<'png' | 'jpeg' | 'svg'>('png');
  let currentSettings = $state({
    line_height: 2,
    char_spacing: 0,
    space_spacing: 8,
    font_path: '',
    font_style: 'Regular',
    aspect_ratio: '1:1',
    resolution: 1920,
    method: 'average',
  });
  let currentImage = $state('');
  let currentLyrics = $state('');
  let cachedGrid = $state<AsciiGrid | null>(null);
  let exporting = $state(false);
  let generating = $state(false);

  settings.subscribe((v) => (currentSettings = v));
  asciiGrid.subscribe((v) => (cachedGrid = v));
  isExporting.subscribe((v) => (exporting = v));
  isGenerating.subscribe((v) => (generating = v));
  imagePath.subscribe((v) => (currentImage = v));
  lyricsText.subscribe((v) => (currentLyrics = v));

  let canExport = $derived(
    !!currentImage && !!currentLyrics && !!currentSettings.font_path && !exporting && !generating,
  );

  async function exportImage() {
    if (!canExport) return;

    const extensions =
      format === 'svg' ? ['svg'] : format === 'jpeg' ? ['jpg', 'jpeg'] : ['png'];
    const outputPath = await save({
      filters: [{ name: format.toUpperCase(), extensions }],
    });

    if (!outputPath) return;

    isExporting.set(true);
    statusMessage.set('Exporting…');

    try {
      const grid =
        cachedGrid ??
        (await invoke<AsciiGrid>('generate_ascii', {
          request: {
            image_path: currentImage,
            lyrics: currentLyrics,
            settings: currentSettings,
          },
        }));

      await invoke('export_ascii', {
        request: {
          grid,
          export_settings: buildExportSettings(currentSettings, { format }),
          font_path: currentSettings.font_path,
          output_path: outputPath,
        },
      });

      statusMessage.set('Exported successfully');
    } catch (e) {
      statusMessage.set(`Export error: ${e}`);
    } finally {
      isExporting.set(false);
    }
  }
</script>

<div class="workflow-step export-step">
  <h2 class="section-header">export</h2>

  <div class="export-row">
    <select id="export-format" class="kv-control" bind:value={format}>
      <option value="png">png</option>
      <option value="jpeg">jpeg</option>
      <option value="svg">svg</option>
    </select>

    <button class="primary export-btn" onclick={exportImage} disabled={!canExport}>
      {exporting ? 'saving…' : 'save'}
    </button>
  </div>
</div>

<style>
  .section-header {
    margin-bottom: var(--space-md);
  }

  .export-step {
    margin-top: auto;
  }

  .export-row {
    display: flex;
    gap: var(--space-sm);
    align-items: center;
  }

  .kv-control {
    flex: 1;
    min-width: 0;
    font-size: 11px;
    padding: 4px 8px;
    background: transparent;
    border: 1px solid var(--border-subtle);
  }

  .export-btn {
    flex-shrink: 0;
    padding: 4px 14px;
    font-size: 11px;
  }
</style>
