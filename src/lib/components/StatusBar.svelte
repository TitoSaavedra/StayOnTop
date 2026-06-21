<script lang="ts">
  import { derived } from 'svelte/store';
  import { t } from '$lib/i18n';
  import { processesStore } from '$stores/processes.store';
  import { pinnedStore } from '$stores/pinned.store';
  import { settingsStore } from '$stores/settings.store';

  const { filtered, selectedHwnd } = processesStore;

  const processCount = derived(filtered, ($f) => $f.length);
  const pinnedCount = derived(pinnedStore, ($p) => $p.length);

  const selectedName = derived(
    [selectedHwnd, filtered, pinnedStore],
    ([$hwnd, $filtered, $pinned]) => {
      if (!$hwnd) return null;
      const inFiltered = $filtered.find((p) => p.hwnd === $hwnd);
      if (inFiltered) return inFiltered.name;
      const inPinned = $pinned.find((p) => p.hwnd === $hwnd);
      return inPinned ? inPinned.process_name : null;
    },
  );
</script>

<div class="status-bar">
  <div class="status-bar__left">
    <span class="status-bar__item">{$processCount} {$t.status.processes}</span>
    {#if $pinnedCount > 0}
      <span class="status-bar__sep">·</span>
      <span class="status-bar__item status-bar__item--accent">{$pinnedCount} {$t.status.pinned}</span>
    {/if}
    {#if $selectedName}
      <span class="status-bar__sep">·</span>
      <span class="status-bar__item status-bar__item--selected">{$selectedName}</span>
    {/if}
  </div>
  <div class="status-bar__right">
    <span class="status-bar__item status-bar__item--hotkey" title={$t.settings.pinToggle}>
      {$settingsStore.hotkey_pin_toggle}
    </span>
  </div>
</div>
