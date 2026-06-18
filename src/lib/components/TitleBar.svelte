<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = getCurrentWindow();
  let maximized = $state(false);

  onMount(() => {
    let unlistenResize: (() => void) | undefined;

    void (async () => {
      maximized = await appWindow.isMaximized();
      unlistenResize = await appWindow.onResized(async () => {
        maximized = await appWindow.isMaximized();
      });
    })();

    return () => {
      unlistenResize?.();
    };
  });

  function minimize() {
    void appWindow.minimize();
  }

  function toggleMaximize() {
    void appWindow.toggleMaximize();
  }

  function close() {
    void appWindow.close();
  }
</script>

<header class="title-bar">
  <div class="drag-region" data-tauri-drag-region>
    <span class="title-name">the-artists-image</span>
  </div>

  <div class="window-controls">
    <button type="button" class="control-btn" aria-label="Minimize" onclick={minimize}>
      <span aria-hidden="true">─</span>
    </button>
    <button
      type="button"
      class="control-btn"
      aria-label={maximized ? 'Restore' : 'Maximize'}
      onclick={toggleMaximize}
    >
      <span aria-hidden="true">{maximized ? '❐' : '□'}</span>
    </button>
    <button type="button" class="control-btn close-btn" aria-label="Close" onclick={close}>
      <span aria-hidden="true">×</span>
    </button>
  </div>
</header>

<style>
  .title-bar {
    display: flex;
    align-items: stretch;
    height: var(--title-bar-height);
    flex-shrink: 0;
    background: var(--bg-bar);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 10px;
    color: var(--fg-dim);
    user-select: none;
  }

  .drag-region {
    flex: 1;
    display: flex;
    align-items: center;
    min-width: 0;
    padding: 0 12px;
    cursor: default;
  }

  .title-name {
    color: var(--fg-muted);
    letter-spacing: 0.04em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .window-controls {
    display: flex;
    flex-shrink: 0;
    height: 100%;
  }

  .control-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    padding: 0;
    border: none;
    border-radius: 0;
    border-left: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--fg-muted);
    font-size: 11px;
    line-height: 1;
    transition: background-color 0.1s, color 0.1s;
  }

  .control-btn:hover {
    background: var(--bg-hover);
    color: var(--fg-primary);
    border-color: var(--border-subtle);
  }

  .control-btn:active {
    background: var(--bg-panel-elevated);
    color: var(--accent);
  }

  .close-btn:hover {
    background: var(--accent-red);
    color: var(--bg-base);
  }

  .close-btn:active {
    background: #a85858;
    color: var(--bg-base);
  }
</style>
