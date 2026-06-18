<script lang="ts">
  import { get } from 'svelte/store';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import {
    lyricsText,
    lyricsData,
    removeSpaces,
    isFetchingLyrics,
    fetchProgress,
    statusMessage,
  } from '../stores/settings';
  import type { LyricsData } from '../types';

  let artist = $state('');
  let album = $state('');
  let manualLyrics = $state('');
  let rawLyrics = $state('');
  let inputMode = $state<'manual' | 'file' | 'fetch'>('manual');

  function normalizeLyrics(raw: string, stripSpaces: boolean): string {
    let text = raw.replace(/\n/g, '');
    if (stripSpaces) text = text.replace(/ /g, '');
    return text;
  }

  function applyLyrics(raw: string) {
    rawLyrics = raw;
    lyricsText.set(normalizeLyrics(raw, get(removeSpaces)));
    lyricsData.set(null);
  }

  function applyManualLyrics() {
    applyLyrics(manualLyrics);
    statusMessage.set('Lyrics applied');
  }

  async function loadFromFile() {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Text', extensions: ['txt'] }],
    });

    if (selected) {
      try {
        const content = await invoke<string>('read_text_file', { path: selected });
        manualLyrics = content;
        applyLyrics(content);
        statusMessage.set('Lyrics loaded from file');
      } catch (e) {
        statusMessage.set(`Error: ${e}`);
      }
    }
  }

  async function fetchLyrics() {
    if (!artist || !album) return;

    isFetchingLyrics.set(true);
    fetchProgress.set('Starting...');
    statusMessage.set('Fetching lyrics…');

    try {
      const result = await invoke<LyricsData>('fetch_lyrics', { artist, album });
      lyricsData.set(result);

      const combined = result.songs.map((s) => s.lyrics).join('');
      applyLyrics(combined);
      const failed = result.songs.filter((s) => s.error && !s.lyrics).length;
      statusMessage.set(
        failed > 0
          ? `Lyrics fetched (${failed} song${failed === 1 ? '' : 's'} missing lyrics)`
          : 'Lyrics fetched',
      );
    } catch (e) {
      statusMessage.set(`Error: ${e}`);
    } finally {
      isFetchingLyrics.set(false);
      fetchProgress.set('');
    }
  }

  function toggleRemoveSpaces() {
    removeSpaces.update((v) => {
      const next = !v;
      if (rawLyrics) {
        lyricsText.set(normalizeLyrics(rawLyrics, next));
      }
      return next;
    });
  }

  let fetching = $state(false);
  isFetchingLyrics.subscribe((v) => (fetching = v));

  let progress = $state('');
  fetchProgress.subscribe((v) => (progress = v));

  let rmSpaces = $state(false);
  removeSpaces.subscribe((v) => (rmSpaces = v));
</script>

<div class="workflow-step">
  <h2 class="section-header">lyrics</h2>

  <div class="mode-row">
    <button type="button" class:active={inputMode === 'manual'} onclick={() => (inputMode = 'manual')}>
      paste
    </button>
    <button type="button" class:active={inputMode === 'file'} onclick={() => (inputMode = 'file')}>
      file
    </button>
    <button type="button" class:active={inputMode === 'fetch'} onclick={() => (inputMode = 'fetch')}>
      fetch
    </button>
  </div>

  {#if inputMode === 'manual'}
    <textarea bind:value={manualLyrics} placeholder="paste lyrics…" rows="4"></textarea>
    <button class="full-width apply-btn" onclick={applyManualLyrics} disabled={!manualLyrics.trim()}>
      apply
    </button>
  {:else if inputMode === 'file'}
    <button class="full-width" onclick={loadFromFile}>open file</button>
  {:else}
    <div class="fetch-form">
      <input type="text" bind:value={artist} placeholder="artist" />
      <input type="text" bind:value={album} placeholder="album" />
      <button onclick={fetchLyrics} disabled={fetching || !artist || !album}>
        {fetching ? 'fetching…' : 'fetch'}
      </button>
      {#if progress}
        <div class="progress-text">{progress}</div>
      {/if}
    </div>
  {/if}

  <label class="checkbox-row">
    <input type="checkbox" checked={rmSpaces} onchange={toggleRemoveSpaces} />
    <span>remove spaces</span>
  </label>
</div>

<style>
  .section-header {
    margin-bottom: var(--space-md);
  }

  .mode-row {
    display: flex;
    gap: 0;
    margin-bottom: var(--space-sm);
    border: 1px solid var(--border-subtle);
  }

  .mode-row button {
    flex: 1;
    padding: 3px 6px;
    font-size: 10px;
    color: var(--fg-dim);
    border: none;
    border-right: 1px solid var(--border-subtle);
    border-radius: 0;
  }

  .mode-row button:last-child {
    border-right: none;
  }

  .mode-row button:hover:not(:disabled) {
    color: var(--fg-secondary);
    background: var(--bg-hover);
  }

  .mode-row button.active {
    color: var(--fg-primary);
    background: var(--bg-hover);
  }

  textarea {
    width: 100%;
    resize: vertical;
    min-height: 64px;
    font-size: 11px;
    line-height: 1.4;
    user-select: text;
    border: 1px solid var(--border-subtle);
    background: transparent;
  }

  .fetch-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .fetch-form input {
    user-select: text;
    border: 1px solid var(--border-subtle);
    background: transparent;
  }

  .full-width {
    width: 100%;
  }

  .apply-btn {
    margin-top: var(--space-sm);
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-top: var(--space-sm);
    font-size: 10px;
    color: var(--fg-dim);
    cursor: pointer;
    user-select: none;
  }

  .checkbox-row input[type='checkbox'] {
    width: 11px;
    height: 11px;
    accent-color: var(--fg-muted);
    padding: 0;
    cursor: pointer;
    border: none;
  }
</style>
