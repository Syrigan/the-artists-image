<script lang="ts">
  import {
    statusMessage,
    lyricsText,
    asciiGrid,
    settings,
    previewZoom,
    asciiPreview,
  } from '../stores/settings';
  import { GRID_REFERENCE_WIDTH } from '../exportSettings';
  import type { AsciiGrid } from '../types';

  let message = $state('READY');
  let lyrics = $state('');
  let grid = $state<AsciiGrid | null>(null);
  let resolution = $state(1920);
  let zoom = $state(1);
  let hasPreview = $state(false);

  statusMessage.subscribe(v => message = v);
  lyricsText.subscribe(v => lyrics = v);
  asciiGrid.subscribe(v => grid = v);
  settings.subscribe(v => resolution = v.resolution);
  previewZoom.subscribe(v => zoom = v);
  asciiPreview.subscribe(v => hasPreview = !!v);

  let messageClass = $derived.by(() => {
    const lower = message.toLowerCase();
    if (lower.startsWith('error') || lower.includes('error:')) return 'error';
    if (lower.includes('…') || lower.includes('ing')) return 'busy';
    return '';
  });

  let centerStats = $derived.by(() => {
    const parts: string[] = [];
    if (lyrics) parts.push(`${lyrics.length.toLocaleString()}c`);
    if (grid) {
      const scale = resolution / GRID_REFERENCE_WIDTH;
      const fontPx = Math.round(grid.font_size_ref * scale);
      parts.push(`font=${fontPx}px`);
      parts.push(`${grid.width}x${grid.height}`);
      parts.push(`${grid.lyrics_chars_placed}/${grid.lyrics_total}`);
    }
    return parts.join('  ');
  });

  // Trailing slot: zoom % when preview visible, otherwise raw grid dimensions.
  let trailingCell = $derived.by(() => {
    if (hasPreview) return `${Math.round(zoom * 100)}%`;
    if (grid) return `${grid.width}x${grid.height}`;
    return '—';
  });
</script>

<div class="status-bar">
  <span class="status-message" class:error={messageClass === 'error'} class:busy={messageClass === 'busy'}>
    {message}
  </span>

  {#if centerStats}
    <span class="status-stats">{centerStats}</span>
  {/if}

  <span class="status-trailing">{trailingCell}</span>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    height: 20px;
    background: var(--status-bar-bg);
    border-top: 1px solid var(--border-subtle);
    font-size: 10px;
    flex-shrink: 0;
    padding: 0 12px;
    gap: var(--space-md);
    color: var(--fg-dim);
  }

  .status-message {
    color: var(--fg-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    max-width: 40%;
  }

  .status-message.error {
    color: var(--accent-red);
  }

  .status-message.busy {
    color: var(--fg-secondary);
  }

  .status-stats {
    flex: 1;
    text-align: center;
    color: var(--fg-dim);
    font-size: 10px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .status-trailing {
    flex-shrink: 0;
    color: var(--fg-muted);
    font-variant-numeric: tabular-nums;
  }
</style>
