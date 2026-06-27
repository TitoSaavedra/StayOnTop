<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Button from './Button.svelte';
  import { processesStore } from '$stores/processes.store';
  import { pinWindow, unpinWindow } from '$services/window.service';
  import { excludeProcess } from '$services/settings.service';
  import { t } from '$lib/i18n';
  import type { ProcessInfo } from '$lib/types';

  interface Props {
    proc: ProcessInfo;
    alreadyPinned: boolean;
    isSelected: boolean;
  }

  let { proc, alreadyPinned, isSelected }: Props = $props();

  const { selectedHwnd } = processesStore;

  function select() {
    selectedHwnd.set(proc.hwnd);
  }

  function onmouseenter() {
    select();
    invoke('show_highlight', { hwnd: proc.hwnd }).catch(() => {});
  }

  function togglePin(e: MouseEvent) {
    e.stopPropagation();
    if (alreadyPinned) {
      unpinWindow(proc.hwnd);
    } else {
      pinWindow({ hwnd: proc.hwnd, process_name: proc.name, window_title: proc.window_title, icon: proc.icon });
    }
  }

  function onexclude(e: MouseEvent) {
    e.stopPropagation();
    excludeProcess(proc.name);
  }
</script>

<li
  class="process-item"
  class:process-item--pinned={alreadyPinned}
  class:process-item--selected={isSelected}
  role="option"
  tabindex="0"
  aria-selected={isSelected}
  {onmouseenter}
  onclick={select}
  onkeydown={(e) => { if (e.key === 'Enter') select(); }}
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
      onclick={togglePin}
      title={alreadyPinned ? $t.home.unpin : $t.home.pin}
    >
      {#if alreadyPinned}
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <circle cx="6" cy="4" r="2.5" fill="currentColor" stroke="currentColor" stroke-width="1.2"/>
          <line x1="6" y1="6.5" x2="6" y2="11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
        </svg>
      {:else}
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <circle cx="6" cy="4" r="2.5" stroke="currentColor" stroke-width="1.2"/>
          <line x1="6" y1="6.5" x2="6" y2="11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
        </svg>
      {/if}
    </Button>
    <Button
      variant="ghost"
      size="sm"
      onclick={onexclude}
      title="Exclude from list"
    >
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <circle cx="6" cy="6" r="4.5" stroke="currentColor" stroke-width="1.3"/>
        <path d="M3 3L9 9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
      </svg>
    </Button>
  </div>
</li>
