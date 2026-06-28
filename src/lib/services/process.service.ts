import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { ProcessInfo } from '$lib/types';
import { processesStore } from '$stores/processes.store';
import { pinnedStore } from '$stores/pinned.store';
import { unpinWindow } from '$services/window.service';

export async function refreshProcesses(): Promise<void> {
  processesStore.setLoading(true);
  processesStore.setError(null);
  try {
    const list = await invoke<ProcessInfo[]>('get_processes');
    processesStore.setProcesses(list);

    // Remove pinned windows that no longer exist
    const pinned = get(pinnedStore);
    await Promise.all(
      pinned.map(async (win) => {
        const valid = await invoke<boolean>('is_window_valid', { hwnd: win.hwnd }).catch(() => false);
        if (!valid) await unpinWindow(win.hwnd);
      }),
    );
  } catch (err) {
    processesStore.setError(String(err));
  } finally {
    processesStore.setLoading(false);
  }
}
