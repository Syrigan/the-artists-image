<script lang="ts">
  // Layout shell: sidebar inputs/settings drive stores; SettingsPanel invokes Tauri; preview reads asciiPreview.
  import '../styles/global.css';

  import ImageInput from '$lib/components/ImageInput.svelte';
  import LyricsInput from '$lib/components/LyricsInput.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import AsciiPreview from '$lib/components/AsciiPreview.svelte';
  import ExportPanel from '$lib/components/ExportPanel.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
</script>

<div class="app">
  <TitleBar />

  <div class="main-content">
    <aside class="workflow-sidebar">
      <ImageInput />
      <LyricsInput />
      <SettingsPanel />
      <ExportPanel />
    </aside>
    <AsciiPreview />
  </div>

  <StatusBar />
</div>
