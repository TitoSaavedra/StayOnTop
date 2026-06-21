import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { PinnedWindow } from '$lib/types';
import { pinnedStore } from '$lib/stores/pinned.store';

export async function pinWindow(
  process: Pick<PinnedWindow, 'hwnd' | 'process_name' | 'window_title'> & { icon?: string },
  opacity = 1,
  click_through = false,
): Promise<void> {
  await invoke('pin_window', { hwnd: process.hwnd, opacity, clickThrough: click_through });
  pinnedStore.add({ ...process, opacity, click_through });
}

export async function unpinWindow(hwnd: number): Promise<void> {
  // Remove from store regardless — the window may already be closed (invalid HWND)
  pinnedStore.remove(hwnd);
  await invoke('unpin_window', { hwnd }).catch(() => {});
}

export async function unpinAll(): Promise<void> {
  const current = get(pinnedStore);
  pinnedStore.set([]);
  await Promise.allSettled(current.map((w) => invoke('unpin_window', { hwnd: w.hwnd })));
}

export async function updatePinnedOpacity(hwnd: number, opacity: number): Promise<void> {
  await invoke('set_window_opacity', { hwnd, opacity });
  pinnedStore.update(hwnd, { opacity });
}

export async function updatePinnedClickThrough(hwnd: number, click_through: boolean): Promise<void> {
  await invoke('set_window_click_through', { hwnd, clickThrough: click_through });
  pinnedStore.update(hwnd, { click_through });
}

