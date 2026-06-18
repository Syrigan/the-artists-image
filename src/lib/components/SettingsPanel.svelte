<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    settings,
    fonts,
    selectedFont,
    imagePath,
    lyricsText,
    asciiGrid,
    asciiPreview,
    isGenerating,
    statusMessage,
  } from '../stores/settings';
  import { buildExportSettings, gridGenerationKey } from '../exportSettings';
  import type { FontInfo, AsciiGrid, AsciiSettings } from '../types';
  import FontSelect from './FontSelect.svelte';

  const aspectRatios = ['1:1', '4:3', '3:4', '16:9', '9:16', '3:2', '2:3'];
  const methods = [
    { value: 'average', label: 'average' },
    { value: 'absolute', label: 'absolute' },
    { value: 'sharp', label: 'sharp' },
  ];

  let currentSettings = $state<AsciiSettings>({
    line_height: 2,
    char_spacing: 0,
    space_spacing: 8,
    font_path: '',
    font_style: 'Regular',
    aspect_ratio: '1:1',
    resolution: 1920,
    method: 'average',
  });

  let fontList = $state<FontInfo[]>([]);
  let currentImage = $state('');
  let currentLyrics = $state('');
  let currentGrid = $state<AsciiGrid | null>(null);

  settings.subscribe((v) => (currentSettings = v));
  fonts.subscribe((v) => (fontList = v));
  imagePath.subscribe((v) => (currentImage = v));
  lyricsText.subscribe((v) => (currentLyrics = v));
  asciiGrid.subscribe((v) => (currentGrid = v));

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let previewDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let generationId = 0;
  let previewId = 0;
  let lastGenerationKey = '';

  // Full grid regeneration when layout, sampling, or source inputs change.
  $effect(() => {
    const { font_path, aspect_ratio, method, line_height, char_spacing } = currentSettings;
    if (currentImage && currentLyrics && font_path) {
      void aspect_ratio;
      void method;
      void line_height;
      void char_spacing;
      void currentImage;
      void currentLyrics;
      triggerGenerate();
    }
  });

  // Re-rasterize cached grid when layout inputs or export resolution change.
  $effect(() => {
    const { font_path, resolution, aspect_ratio } = currentSettings;
    if (currentGrid && font_path) {
      void currentGrid;
      void resolution;
      void aspect_ratio;
      triggerPreviewRerender();
    }
  });

  function updateSetting(key: string, value: string | number) {
    if (key === 'resolution') {
      const n = Number(value);
      value = Math.min(7680, Math.max(320, Number.isFinite(n) ? n : 1920));
    } else if (key === 'line_height') {
      const n = Number(value);
      value = Math.min(48, Math.max(0, Number.isFinite(n) ? Math.round(n) : 2));
    } else if (key === 'char_spacing') {
      const n = Number(value);
      value = Math.min(32, Math.max(-8, Number.isFinite(n) ? Math.round(n) : 0));
    }
    settings.update((s) => ({ ...s, [key]: value }));
  }

  function onFontChange(path: string) {
    const font = fontList.find((f) => f.path === path);
    if (font) {
      selectedFont.set(font);
      updateSetting('font_path', font.path);
    }
  }

  function triggerGenerate() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => generate(), 300);
  }

  function triggerPreviewRerender() {
    if (previewDebounceTimer) clearTimeout(previewDebounceTimer);
    previewDebounceTimer = setTimeout(() => rerenderPreview(), 150);
  }

  async function rerenderPreview() {
    const grid = currentGrid;
    if (!grid || !currentSettings.font_path) return;

    const requestId = ++previewId;
    isGenerating.set(true);
    statusMessage.set('Rendering preview…');

    try {
      const pngBase64 = await invoke<string>('render_preview', {
        request: {
          grid,
          export_settings: buildExportSettings(currentSettings),
          font_path: currentSettings.font_path,
        },
      });

      if (requestId !== previewId) return;

      asciiPreview.set(`data:image/png;base64,${pngBase64}`);
      statusMessage.set('Preview ready');
    } catch (e) {
      if (requestId !== previewId) return;
      statusMessage.set(`Error: ${e}`);
    } finally {
      if (requestId === previewId) {
        isGenerating.set(false);
      }
    }
  }

  async function generate() {
    if (!currentImage || !currentLyrics) return;

    const generationKey = `${currentImage}\0${currentLyrics}\0${gridGenerationKey(currentSettings)}`;
    if (generationKey === lastGenerationKey && currentGrid) {
      return;
    }

    const requestId = ++generationId;
    previewId = requestId;
    isGenerating.set(true);
    statusMessage.set('Generating…');

    try {
      const grid = await invoke<AsciiGrid>('generate_ascii', {
        request: {
          image_path: currentImage,
          lyrics: currentLyrics,
          settings: currentSettings,
        },
      });

      if (requestId !== generationId) return;

      lastGenerationKey = generationKey;
      asciiGrid.set(grid);
    } catch (e) {
      if (requestId !== generationId) return;
      statusMessage.set(`Error: ${e}`);
      isGenerating.set(false);
    }
  }

  onMount(async () => {
    try {
      const systemFonts = await invoke<FontInfo[]>('get_system_fonts');
      fonts.set(systemFonts);
      if (systemFonts.length > 0) {
        const defaultFont = systemFonts.find((f) => f.is_monospace) ?? systemFonts[0];
        selectedFont.set(defaultFont);
        settings.update((s) => ({ ...s, font_path: defaultFont.path }));
      }
    } catch (e) {
      statusMessage.set(`Font error: ${e}`);
    }
  });

  onDestroy(() => {
    asciiPreview.set('');
  });
</script>

<div class="workflow-step">
  <h2 class="section-header">settings</h2>

  <div class="settings-grid">
    <div class="kv-row">
      <span class="kv-key">ratio</span>
      <select
        id="aspect-ratio"
        class="kv-control"
        value={currentSettings.aspect_ratio}
        onchange={(e) => updateSetting('aspect_ratio', e.currentTarget.value)}
      >
        {#each aspectRatios as ar}
          <option value={ar}>{ar}</option>
        {/each}
      </select>
    </div>

    <div class="kv-row">
      <span class="kv-key">res</span>
      <input
        id="resolution"
        class="kv-control kv-narrow"
        type="number"
        value={currentSettings.resolution}
        onchange={(e) => updateSetting('resolution', Number(e.currentTarget.value))}
        min="320"
        max="7680"
      />
    </div>

    <div class="kv-row">
      <span class="kv-key">style</span>
      <select
        id="method"
        class="kv-control"
        value={currentSettings.method}
        onchange={(e) => updateSetting('method', e.currentTarget.value)}
      >
        {#each methods as m}
          <option value={m.value}>{m.label}</option>
        {/each}
      </select>
    </div>

    <div class="kv-row">
      <span class="kv-key">font</span>
      <FontSelect fonts={fontList} value={currentSettings.font_path} onchange={onFontChange} />
    </div>

    <div class="kv-row">
      <span class="kv-key" title="Extra space between lines">line</span>
      <div class="kv-value">
        <input
          id="line-height"
          class="kv-control kv-narrow"
          type="number"
          value={currentSettings.line_height}
          onchange={(e) => updateSetting('line_height', Number(e.currentTarget.value))}
          min="-16"
          max="48"
          step="1"
        />
        <span class="kv-unit">px</span>
      </div>
    </div>

    <div class="kv-row">
      <span class="kv-key">char</span>
      <div class="kv-value">
        <input
          id="char-spacing"
          class="kv-control kv-narrow"
          type="number"
          value={currentSettings.char_spacing}
          onchange={(e) => updateSetting('char_spacing', Number(e.currentTarget.value))}
          min="-16"
          max="32"
          step="1"
        />
        <span class="kv-unit">px</span>
      </div>
    </div>
  </div>
</div>

<style>
  .section-header {
    margin-bottom: var(--space-md);
  }

  .settings-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .kv-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    min-height: 22px;
  }

  .kv-key {
    flex-shrink: 0;
    width: 3.25em;
    font-size: 10px;
    color: var(--fg-dim);
    text-align: right;
  }

  .kv-key::after {
    content: ':';
  }

  .kv-value {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .kv-unit {
    flex-shrink: 0;
    font-size: 10px;
    color: var(--fg-dim);
  }

  .kv-control {
    flex: 1;
    min-width: 0;
    font-size: 11px;
    padding: 2px 6px;
    background: transparent;
    border: 1px solid var(--border-subtle);
  }

  .kv-control:focus {
    border-color: var(--border-focus);
  }

  .kv-narrow {
    flex: 0 1 5em;
    max-width: 5em;
  }
</style>
