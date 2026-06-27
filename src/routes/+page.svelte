<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Button, PinnedWindowItem, ProcessItem } from '$components';
  import { processesStore } from '$stores/processes.store';
  import { pinnedStore } from '$stores/pinned.store';
  import { settingsStore } from '$stores/settings.store';
  import { refreshProcesses } from '$services/process.service';
  import { unpinAll } from '$services/window.service';
  import { t } from '$lib/i18n';
  import type { PinnedWindow } from '$lib/types';

  const { filtered: filteredProcesses, loading, error, selectedHwnd } = processesStore;

  onMount(async () => {
    const name = await invoke<string>('get_app_name');
    processesStore.setAppName(name);
    await refreshProcesses();

    const persistedPins = await invoke<PinnedWindow[]>('get_pinned');
    for (const pin of persistedPins) {
      try {
        await invoke('pin_window', { hwnd: pin.hwnd, opacity: pin.opacity, clickThrough: pin.click_through });
        pinnedStore.add(pin);
      } catch {
        // window no longer exists, skip
      }
    }
  });

  $effect(() => {
    const intervalMs = $settingsStore.refresh_interval_ms;
    const id = setInterval(refreshProcesses, intervalMs);
    return () => clearInterval(id);
  });
</script>

<div class="home">
  {#if $pinnedStore.length > 0}
    <section class="home__section">
      <div class="home__section-header">
        <h2 class="home__section-title">{$t.home.pinned}</h2>
        <Button variant="ghost" size="sm" onclick={unpinAll} title={$t.home.unpinAll}>
          {$t.home.unpinAll}
        </Button>
      </div>
      <ul class="pinned-list">
        {#each $pinnedStore as win (win.hwnd)}
          <PinnedWindowItem {win} />
        {/each}
      </ul>
    </section>
  {/if}

  <section class="home__section">
    <div class="home__section-header">
      <h2 class="home__section-title">
        {$t.home.processes}
        {#if $loading}
          <span class="home__spinner" aria-label="Loading"></span>
        {/if}
      </h2>
    </div>

    {#if $error}
      <p class="home__error">{$error}</p>
    {:else}
      <ul
        class="process-list"
        role="listbox"
        aria-label={$t.home.processes}
        onmouseleave={() => { selectedHwnd.set(null); invoke('clear_highlight').catch(() => {}); }}
      >
        {#each $filteredProcesses as proc (proc.hwnd)}
          <ProcessItem
            {proc}
            alreadyPinned={pinnedStore.isPinned(proc.hwnd, $pinnedStore)}
            isSelected={$selectedHwnd === proc.hwnd}
          />
        {/each}
      </ul>
    {/if}
  </section>
</div>
