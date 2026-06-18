<script lang="ts">
  import type { FontInfo } from '../types';

  interface Props {
    fonts: FontInfo[];
    value: string;
    onchange: (path: string) => void;
  }

  let { fonts, value, onchange }: Props = $props();

  let open = $state(false);
  let filterQuery = $state('');
  let rootEl = $state<HTMLDivElement | null>(null);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let filterInputEl = $state<HTMLInputElement | null>(null);
  let listStyle = $state('');

  let selected = $derived(fonts.find((f) => f.path === value) ?? fonts[0] ?? null);
  let label = $derived(selected?.name ?? '—');
  let filteredFonts = $derived(
    filterQuery.trim()
      ? fonts.filter((f) => f.name.toLowerCase().includes(filterQuery.trim().toLowerCase()))
      : fonts,
  );

  function updateListPosition() {
    if (!triggerEl) return;
    const rect = triggerEl.getBoundingClientRect();
    const maxHeight = Math.min(180, window.innerHeight - rect.bottom - 8);
    listStyle = `top:${rect.bottom + 2}px;left:${rect.left}px;width:${rect.width}px;max-height:${maxHeight}px;`;
  }

  function select(font: FontInfo) {
    onchange(font.path);
    open = false;
    filterQuery = '';
  }

  function toggleOpen() {
    open = !open;
    if (open) {
      filterQuery = '';
      updateListPosition();
    }
  }

  function onDocClick(e: MouseEvent) {
    if (!open || !rootEl) return;
    if (!rootEl.contains(e.target as Node)) {
      open = false;
      filterQuery = '';
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
      filterQuery = '';
    }
  }

  $effect(() => {
    if (!open) return;
    filterInputEl?.focus();
    updateListPosition();
    document.addEventListener('click', onDocClick);
    document.addEventListener('keydown', onKeydown);
    window.addEventListener('scroll', updateListPosition, true);
    window.addEventListener('resize', updateListPosition);
    return () => {
      document.removeEventListener('click', onDocClick);
      document.removeEventListener('keydown', onKeydown);
      window.removeEventListener('scroll', updateListPosition, true);
      window.removeEventListener('resize', updateListPosition);
    };
  });
</script>

<div class="font-select" bind:this={rootEl}>
  <button
    type="button"
    class="font-trigger"
    bind:this={triggerEl}
    aria-haspopup="listbox"
    aria-expanded={open}
    onclick={toggleOpen}
  >
    <span class="font-label">{label}</span>
    <span class="font-caret" aria-hidden="true">_</span>
  </button>

  {#if open}
    <div class="font-dropdown" style={listStyle}>
      <input
        type="text"
        class="font-filter"
        bind:this={filterInputEl}
        bind:value={filterQuery}
        placeholder="filter…"
        aria-label="Filter fonts"
        onclick={(e) => e.stopPropagation()}
      />
      <ul class="font-list" role="listbox" aria-label="font">
        {#each filteredFonts as font (font.path)}
          <li>
            <button
              type="button"
              role="option"
              class:selected={font.path === value}
              aria-selected={font.path === value}
              onclick={() => select(font)}
            >
              {font.name}
              {#if font.is_monospace}
                <span class="mono-badge">m</span>
              {/if}
            </button>
          </li>
        {:else}
          <li class="font-empty">no match</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>

<style>
  .font-select {
    position: relative;
    flex: 1;
    min-width: 0;
  }

  .font-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-sm);
    width: 100%;
    padding: 2px 6px;
    font-size: 11px;
    text-align: left;
    background: transparent;
    border: 1px solid var(--border-subtle);
    color: var(--fg-primary);
  }

  .font-trigger:hover {
    border-color: var(--fg-dim);
  }

  .font-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .font-caret {
    flex-shrink: 0;
    font-size: 9px;
    color: var(--fg-dim);
  }

  .font-dropdown {
    position: fixed;
    z-index: 100;
    display: flex;
    flex-direction: column;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .font-filter {
    flex-shrink: 0;
    width: 100%;
    padding: 4px 8px;
    font-size: 11px;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    border-radius: 0;
    background: var(--bg-input);
    color: var(--fg-primary);
    outline: none;
  }

  .font-filter:focus {
    border-bottom-color: var(--border-focus);
  }

  .font-list {
    margin: 0;
    padding: 0;
    list-style: none;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .font-list button {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 3px 8px;
    font-size: 11px;
    text-align: left;
    border: none;
    border-radius: 0;
    background: transparent;
    color: var(--fg-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mono-badge {
    flex-shrink: 0;
    font-size: 9px;
    color: var(--fg-dim);
  }

  .font-empty {
    padding: 6px 8px;
    font-size: 11px;
    color: var(--fg-dim);
  }

  .font-list button:hover {
    background: var(--bg-hover);
    color: var(--fg-primary);
  }

  .font-list button.selected {
    color: var(--fg-primary);
  }
</style>
