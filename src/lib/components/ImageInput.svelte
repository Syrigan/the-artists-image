<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { imagePath, statusMessage } from '../stores/settings';

  async function selectImage() {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] }],
    });

    if (selected) {
      imagePath.set(selected);
      statusMessage.set('Image loaded');
    }
  }

  let currentPath = $state('');
  imagePath.subscribe((v) => (currentPath = v));

  let fileName = $derived(currentPath ? currentPath.split(/[\\/]/).pop() : '');
</script>

<div class="workflow-step">
  <h2 class="section-header">image</h2>

  <button class="full-width" onclick={selectImage}>
    {currentPath ? 'change' : 'choose'}
  </button>

  {#if fileName}
    <div class="file-name">{fileName}</div>
  {/if}
</div>

<style>
  .section-header {
    margin-bottom: var(--space-md);
  }

  .full-width {
    width: 100%;
  }
</style>
