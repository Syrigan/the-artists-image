<script lang="ts">
  import { tick } from 'svelte';
  import { asciiPreview, isGenerating, imagePath, lyricsText, previewZoom } from '../stores/settings';

  const MIN_ZOOM = 0.25;
  const MAX_ZOOM = 8;
  const MIN_DISPLAY_PX = 32;
  const VIEWPORT_PADDING = 16;
  const ZOOM_STEP = 1.25;
  const WHEEL_SENSITIVITY = 0.002;

  let previewUrl = $state('');
  let generating = $state(false);
  let hasImage = $state(false);
  let hasLyrics = $state(false);

  let viewportEl = $state<HTMLDivElement | null>(null);
  let zoom = $state(1);
  let fitScale = $state(1);
  let naturalWidth = $state(0);
  let naturalHeight = $state(0);
  let viewportWidth = $state(0);
  let viewportHeight = $state(0);

  // New PNG from SettingsPanel resets zoom; pan/zoom are UI-only.
  asciiPreview.subscribe(v => {
    previewUrl = v;
    zoom = 1;
    previewZoom.set(1);
    naturalWidth = 0;
    naturalHeight = 0;
  });
  isGenerating.subscribe(v => generating = v);
  imagePath.subscribe(v => hasImage = !!v);
  lyricsText.subscribe(v => hasLyrics = !!v);

  let emptyHint = $derived.by(() => {
    if (generating) return 'generating…';
    if (!hasImage) return 'open cover';
    if (!hasLyrics) return 'add lyrics';
    return 'adjust settings';
  });

  // fitScale maps the fixed-resolution preview image into the viewport at zoom=1.
  function updateFitScale() {
    if (!viewportEl || naturalWidth === 0 || naturalHeight === 0) return;
    viewportWidth = viewportEl.clientWidth;
    viewportHeight = viewportEl.clientHeight;
    const padding = VIEWPORT_PADDING * 2;
    const availableW = Math.max(1, viewportWidth - padding);
    const availableH = Math.max(1, viewportHeight - padding);
    fitScale = Math.min(availableW / naturalWidth, availableH / naturalHeight);
  }

  function onImageLoad(event: Event) {
    const img = event.currentTarget as HTMLImageElement;
    naturalWidth = img.naturalWidth;
    naturalHeight = img.naturalHeight;
    updateFitScale();
  }

  function displaySizeForZoom(z: number): { width: number; height: number } {
    if (naturalWidth === 0 || naturalHeight === 0) {
      return { width: 0, height: 0 };
    }
    const width = Math.max(MIN_DISPLAY_PX, Math.round(naturalWidth * fitScale * z));
    const scale = width / naturalWidth;
    const height = Math.max(MIN_DISPLAY_PX, Math.round(naturalHeight * scale));
    return { width, height };
  }

  function contentSizeForImage(imgW: number, imgH: number): { width: number; height: number } {
    const innerW = Math.max(1, viewportWidth - VIEWPORT_PADDING * 2);
    const innerH = Math.max(1, viewportHeight - VIEWPORT_PADDING * 2);
    return {
      width: Math.max(innerW, imgW),
      height: Math.max(innerH, imgH),
    };
  }

  function imageOffsetInContent(imgW: number, imgH: number): { x: number; y: number } {
    const content = contentSizeForImage(imgW, imgH);
    return {
      x: (content.width - imgW) / 2,
      y: (content.height - imgH) / 2,
    };
  }

  function clampZoom(next: number) {
    if (naturalWidth === 0 || naturalHeight === 0) {
      return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, next));
    }
    const minZoomForWidth = MIN_DISPLAY_PX / (naturalWidth * fitScale);
    const minZoomForHeight = MIN_DISPLAY_PX / (naturalHeight * fitScale);
    const minZoom = Math.max(MIN_ZOOM, minZoomForWidth, minZoomForHeight);
    return Math.min(MAX_ZOOM, Math.max(minZoom, next));
  }

  function setZoom(next: number) {
    zoom = clampZoom(next);
    previewZoom.set(zoom);
  }

  // Keep the pixel under the cursor fixed while zooming (wheel or toolbar).
  async function zoomAtPoint(clientX: number, clientY: number, nextZoom: number) {
    if (!viewportEl || naturalWidth === 0 || naturalHeight === 0) {
      setZoom(nextZoom);
      return;
    }

    const oldZoom = zoom;
    const clamped = clampZoom(nextZoom);
    if (clamped === oldZoom) return;

    const rect = viewportEl.getBoundingClientRect();
    const viewX = clientX - rect.left - VIEWPORT_PADDING;
    const viewY = clientY - rect.top - VIEWPORT_PADDING;

    const oldSize = displaySizeForZoom(oldZoom);
    const oldOffset = imageOffsetInContent(oldSize.width, oldSize.height);
    const contentX = viewportEl.scrollLeft + viewX;
    const contentY = viewportEl.scrollTop + viewY;
    const imageX = contentX - oldOffset.x;
    const imageY = contentY - oldOffset.y;
    const ratio = clamped / oldZoom;

    zoom = clamped;
    previewZoom.set(zoom);
    await tick();

    if (!viewportEl) return;

    const newSize = displaySizeForZoom(clamped);
    const newOffset = imageOffsetInContent(newSize.width, newSize.height);
    const newContentX = imageX * ratio + newOffset.x;
    const newContentY = imageY * ratio + newOffset.y;

    viewportEl.scrollLeft = Math.max(0, newContentX - viewX);
    viewportEl.scrollTop = Math.max(0, newContentY - viewY);
  }

  function viewportCenter(): { x: number; y: number } {
    if (!viewportEl) return { x: 0, y: 0 };
    const rect = viewportEl.getBoundingClientRect();
    return {
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2,
    };
  }

  function zoomIn() {
    const center = viewportCenter();
    void zoomAtPoint(center.x, center.y, zoom * ZOOM_STEP);
  }

  function zoomOut() {
    const center = viewportCenter();
    void zoomAtPoint(center.x, center.y, zoom / ZOOM_STEP);
  }

  function resetZoom() {
    zoom = 1;
    previewZoom.set(1);
    void tick().then(() => {
      viewportEl?.scrollTo({ left: 0, top: 0, behavior: 'instant' });
    });
  }

  function normalizeWheelDelta(event: WheelEvent): number {
    let delta = event.deltaY;
    if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
      delta *= 16;
    } else if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
      delta *= viewportEl?.clientHeight ?? 400;
    }
    return delta;
  }

  function onWheel(event: WheelEvent) {
    if (!previewUrl) return;
    event.preventDefault();
    const factor = Math.exp(-normalizeWheelDelta(event) * WHEEL_SENSITIVITY);
    void zoomAtPoint(event.clientX, event.clientY, zoom * factor);
  }

  $effect(() => {
    if (!viewportEl || !previewUrl) return;

    const observer = new ResizeObserver(() => updateFitScale());
    observer.observe(viewportEl);
    updateFitScale();

    return () => observer.disconnect();
  });

  $effect(() => {
    if (naturalWidth > 0 && naturalHeight > 0) {
      const clamped = clampZoom(zoom);
      if (clamped !== zoom) {
        zoom = clamped;
        previewZoom.set(zoom);
      }
    }
  });

  let displayWidth = $derived.by(() => {
    if (naturalWidth === 0) return undefined;
    return displaySizeForZoom(zoom).width;
  });
  let displayHeight = $derived.by(() => {
    if (naturalHeight === 0) return undefined;
    return displaySizeForZoom(zoom).height;
  });
  let contentWidth = $derived.by(() => {
    if (displayWidth === undefined) return undefined;
    return contentSizeForImage(displayWidth, displayHeight ?? displayWidth).width;
  });
  let contentHeight = $derived.by(() => {
    if (displayHeight === undefined) return undefined;
    return contentSizeForImage(displayWidth ?? displayHeight, displayHeight).height;
  });
  let zoomLabel = $derived(`${Math.round(zoom * 100)}%`);
</script>

<div class="preview-area">
  {#if previewUrl}
    <div class="preview-toolbar" aria-label="Preview zoom controls">
      <button type="button" onclick={zoomOut} title="Zoom out" aria-label="Zoom out">−</button>
      <span class="zoom-label">{zoomLabel}</span>
      <button type="button" onclick={zoomIn} title="Zoom in" aria-label="Zoom in">+</button>
      <button type="button" class="fit-btn" onclick={resetZoom} title="Fit to view" aria-label="Fit to view">fit</button>
    </div>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="preview-viewport"
      bind:this={viewportEl}
      onwheel={onWheel}
    >
      <div
        class="preview-content"
        style:width={contentWidth !== undefined ? `${contentWidth}px` : undefined}
        style:height={contentHeight !== undefined ? `${contentHeight}px` : undefined}
      >
        <img
          class="preview-image"
          src={previewUrl}
          alt="ASCII preview"
          width={displayWidth}
          height={displayHeight}
          onload={onImageLoad}
        />
      </div>
    </div>
  {:else}
    <div class="empty-buffer">
      <p class="empty-hint">{emptyHint}</p>
    </div>
  {/if}
</div>

<style>
  .preview-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    background: #000000;
    position: relative;
    overflow: hidden;
  }

  .preview-toolbar {
    position: absolute;
    top: var(--space-sm);
    right: var(--space-sm);
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 1px;
    padding: 1px 2px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-bar);
    opacity: 0.5;
    transition: opacity 0.15s;
  }

  .preview-area:hover .preview-toolbar,
  .preview-toolbar:focus-within {
    opacity: 1;
  }

  .preview-toolbar button {
    min-width: 18px;
    padding: 1px 4px;
    font-size: 10px;
    border: none;
    background: transparent;
    color: var(--fg-dim);
  }

  .preview-toolbar button:hover {
    color: var(--fg-primary);
  }

  .fit-btn {
    font-size: 9px !important;
    color: var(--fg-dim) !important;
    margin-left: 2px;
    border-left: 1px solid var(--border-subtle) !important;
    padding-left: 6px !important;
  }

  .zoom-label {
    min-width: 32px;
    text-align: center;
    font-size: 9px;
    color: var(--fg-dim);
    font-variant-numeric: tabular-nums;
  }

  .preview-viewport {
    flex: 1;
    overflow: auto;
    padding: 16px;
    min-height: 0;
    width: 100%;
    height: 100%;
    background: #000000;
  }

  .preview-content {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    background: transparent;
  }

  .preview-image {
    display: block;
    flex-shrink: 0;
    image-rendering: pixelated;
    min-width: 32px;
    min-height: 32px;
    background: transparent;
  }

  .empty-buffer {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 0;
    overflow: hidden;
  }

  .empty-hint {
    font-size: 11px;
    color: var(--fg-dim);
    white-space: nowrap;
    letter-spacing: 0.02em;
  }
</style>
