<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { app } from "$lib/stores/app.svelte.ts";

  let { onClose, onSuccess, initialEmail = "", message }: {
    onClose: () => void;
    onSuccess: (email: string) => void | Promise<void>;
    initialEmail?: string;
    message?: string;
  } = $props();

  let email = $state("");
  let password = $state("");
  let loading = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    if (initialEmail) {
      email = initialEmail;
      return;
    }
    try {
      const saved = await invoke<string | null>("get_cloud_credentials");
      if (typeof saved === "string") email = saved;
    } catch {
      // non-fatal — email field stays empty
    }
  });

  async function handleOpen() {
    loading = true;
    error = null;
    try {
      await app.openCloud(email, password);
      await onSuccess(email);
    } catch (e) {
      error = typeof e === "string" ? e : "An unexpected error occurred.";
    } finally {
      loading = false;
    }
  }
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label="Open Cloud Notebook">
  <div class="dialog">
    <h2 class="title">Open Cloud Notebook</h2>
    {#if message}
      <div class="notice" role="status">{message}</div>
    {/if}
    <div class="field">
      <label for="cloud-email">Email</label>
      <input id="cloud-email" type="email" bind:value={email} autocomplete="username" />
    </div>
    <div class="field">
      <label for="cloud-password">Password</label>
      <input id="cloud-password" type="password" bind:value={password} autocomplete="current-password" />
    </div>
    {#if error}
      <div class="error" role="alert">{error}</div>
    {/if}
    <div class="actions">
      <button class="btn" onclick={onClose}>Cancel</button>
      <button class="btn primary" onclick={handleOpen} disabled={loading}>
        {loading ? "Opening…" : "Open Cloud"}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .dialog { background: var(--panel); border: 1px solid var(--hair); border-radius: var(--r-panel, 8px); padding: 24px; width: 360px; display: flex; flex-direction: column; gap: 16px; }
  .title { margin: 0; font-size: 16px; font-weight: 600; color: var(--txt); }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .field label { font-size: 12.5px; color: var(--txt-2); }
  .field input { height: 32px; padding: 0 10px; background: var(--panel-2); border: 1px solid var(--hair); border-radius: var(--r-control); color: var(--txt); font: inherit; font-size: 13px; outline: none; }
  .field input:focus { border-color: var(--blue); }
  .notice { font-size: 12.5px; color: var(--txt-2); background: var(--panel-2); border: 1px solid var(--hair); border-radius: var(--r-control); padding: 8px 10px; }
  .error { font-size: 12.5px; color: var(--red, #c0392b); padding: 6px 0; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  .btn { height: 30px; padding: 0 14px; border-radius: var(--r-control); border: 1px solid var(--hair); background: var(--elev); color: var(--txt); font: inherit; cursor: pointer; }
  .btn.primary { background: var(--blue); border-color: transparent; color: #fff; font-weight: 560; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
