<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { Button, Badge, Slider, Checkbox } from '$components';
  import { processesStore } from '$stores/processes.store';
  import { pinnedStore } from '$stores/pinned.store';
  import { settingsStore } from '$stores/settings.store';
  import { refreshProcesses } from '$services/process.service';
  import {
    pinWindow,
    unpinWindow,
    unpinAll,
    updatePinnedOpacity,
    updatePinnedClickThrough,
  } from '$services/window.service';
  import { excludeProcess } from '$services/settings.service';
  import { t } from '$lib/i18n';
  import type { PinnedWindow } from '$lib/types';

  const { filtered: filteredProcesses, loading, error, selectedHwnd } = processesStore;

  let pinnedList = $state<PinnedWindow[]>([]);

  const unsubPinned = pinnedStore.subscribe((v) => (pinnedList = v));

  let intervalId: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    const name = await invoke<string>('get_app_name');
    processesStore.setAppName(name);
    await refreshProcesses();
    intervalId = setInterval(refreshProcesses, get(settingsStore).refresh_interval_ms);
  });

  onDestroy(() => {
    unsubPinned();
    if (intervalId) clearInterval(intervalId);
  });
</script>

<div class="home">
  <!-- Pinned windows -->
  {#if pinnedList.length > 0}
    <section class="home__section">
      <div class="home__section-header">
        <h2 class="home__section-title">{$t.home.pinned}</h2>
        <Button variant="ghost" size="sm" onclick={unpinAll} title={$t.home.unpinAll}>
          {$t.home.unpinAll}
        </Button>
      </div>
      <ul class="pinned-list">
        {#each pinnedList as win (win.hwnd)}
          <li class="pinned-item">
            <div class="pinned-item__header">
              {#if win.icon}
                <img class="pinned-item__icon" src={win.icon} alt="" width="16" height="16" aria-hidden="true" />
              {/if}
              <span class="pinned-item__name">{win.process_name}</span>
              <Badge color="accent">{$t.home.onTop}</Badge>
              <Button
                variant="ghost"
                size="sm"
                onclick={() => unpinWindow(win.hwnd)}
                title={$t.home.unpin}
              >
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                  <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                </svg>
              </Button>
            </div>
            {#if win.window_title}
              <p class="pinned-item__subtitle">{win.window_title}</p>
            {/if}
            <div class="pinned-item__controls">
              <Slider
                bind:value={win.opacity}
                min={0.1}
                max={1}
                step={0.01}
                label={$t.home.opacity}
                oninput={() => updatePinnedOpacity(win.hwnd, win.opacity)}
              />
              <Checkbox
                bind:checked={win.click_through}
                label={$t.home.clickThrough}
                onchange={() => updatePinnedClickThrough(win.hwnd, win.click_through)}
              />
            </div>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <!-- Process list -->
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
      <ul class="process-list" onmouseleave={() => { selectedHwnd.set(null); invoke('clear_highlight').catch(() => {}); }}>
        {#each $filteredProcesses as proc (proc.hwnd)}
          {@const alreadyPinned = pinnedStore.isPinned(proc.hwnd, pinnedList)}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
          <li
            class="process-item"
            class:process-item--pinned={alreadyPinned}
            class:process-item--selected={$selectedHwnd === proc.hwnd}
            onmouseenter={() => { selectedHwnd.set(proc.hwnd); if (pinnedList.length === 0) invoke('show_highlight', { hwnd: proc.hwnd }).catch(() => {}); }}
            onclick={() => selectedHwnd.set(proc.hwnd)}
          >
            <div class="process-item__info">
              {#if proc.icon}
                <img class="process-item__icon" src={proc.icon} alt="" width="16" height="16" aria-hidden="true" />
              {/if}
              <div class="process-item__text">
                <span class="process-item__name">{proc.name}</span>
                {#if proc.window_title}
                  <span class="process-item__window">{proc.window_title}</span>
                {/if}
              </div>
            </div>
            <div class="process-item__actions">
              <Button
                variant="ghost"
                size="sm"
                onclick={() => (alreadyPinned ? unpinWindow(proc.hwnd) : pinWindow({ hwnd: proc.hwnd, process_name: proc.name, window_title: proc.window_title, icon: proc.icon }))}
                title={alreadyPinned ? $t.home.unpin : $t.home.pin}
              >
                {#if alreadyPinned}
                  <!-- Pinned: filled thumbtack -->
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                    <circle cx="6" cy="4" r="2.5" fill="currentColor" stroke="currentColor" stroke-width="1.2"/>
                    <line x1="6" y1="6.5" x2="6" y2="11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                  </svg>
                {:else}
                  <!-- Not pinned: outline thumbtack -->
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                    <circle cx="6" cy="4" r="2.5" stroke="currentColor" stroke-width="1.2"/>
                    <line x1="6" y1="6.5" x2="6" y2="11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                  </svg>
                {/if}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onclick={() => excludeProcess(proc.name)}
                title="Exclude from list"
              >
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                  <circle cx="6" cy="6" r="4.5" stroke="currentColor" stroke-width="1.3"/>
                  <path d="M3 3L9 9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                </svg>
              </Button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>
