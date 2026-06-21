<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount } from "svelte";
  import { applyTheme } from "$lib/prefs.ts";
  import Toolbar from "$lib/components/Toolbar.svelte";

  const fixtures = (window as any).__playwright_fixtures__ ?? {};
  const isCloud: boolean = fixtures.isCloud ?? false;

  // Never resolves — lets Playwright screenshot the syncing state mid-flight.
  const onSync = isCloud ? (): Promise<void> => new Promise(() => {}) : undefined;

  onMount(() => {
    applyTheme("auto");
  });
</script>

<div class="harness">
  <Toolbar onSearch={() => {}} {isCloud} {onSync} />
</div>

<style>
  .harness { width: 800px; }
</style>
