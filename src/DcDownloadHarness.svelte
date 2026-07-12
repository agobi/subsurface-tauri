<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount } from "svelte";
  import { applyTheme } from "$lib/prefs.ts";
  import DcDownloadDialog from "$lib/components/DcDownloadDialog.svelte";

  onMount(() => {
    applyTheme("auto");
  });
</script>

<!--
  No wrapper sizing here: DcDownloadDialog's backdrop is `position: fixed;
  inset: 0`, so it already spans the real viewport (desktop or android
  project) on its own. A fixed pixel width here would overflow narrow
  mobile viewports and desync Playwright's click coordinates from the
  rendered layout.
-->
<DcDownloadDialog open={true} onClose={() => {}} />
