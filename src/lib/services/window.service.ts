import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { PinnedWindow } from '$lib/types';
import { pinnedStore } from '$stores/pinned.store';
import { settingsStore } from '$stores/settings.store';

async function savePinnedState(): Promise<void> {
  await invoke('save_pinned', { pins: get(pinnedStore) }).catch(() => {});
}

export async function pinWindow(
  process: Pick<PinnedWindow, 'hwnd' | 'process_name' | 'window_title'> & { icon?: string },
  opacity?: number,
  click_through = false,
): Promise<void> {
  const resolvedOpacity = opacity ?? get(settingsStore).default_opacity;
  await invoke('pin_window', { hwnd: process.hwnd, opacity: resolvedOpacity, clickThrough: click_through });
  pinnedStore.add({ ...process, opacity: resolvedOpacity, click_through });
  await savePinnedState();
}

export async function unpinWindow(hwnd: number): Promise<void> {
  pinnedStore.remove(hwnd);
  await invoke('unpin_window', { hwnd }).catch(() => {});
  await savePinnedState();
}

export async function unpinAll(): Promise<void> {
  const current = get(pinnedStore);
  pinnedStore.set([]);
  await Promise.allSettled(current.map((w) => invoke('unpin_window', { hwnd: w.hwnd })));
  await savePinnedState();
}

export async function updatePinnedOpacity(hwnd: number, opacity: number): Promise<void> {
  await invoke('set_window_opacity', { hwnd, opacity });
  pinnedStore.update(hwnd, { opacity });
  await savePinnedState();
}

export async function updatePinnedClickThrough(hwnd: number, click_through: boolean): Promise<void> {
  await invoke('set_window_click_through', { hwnd, clickThrough: click_through });
  pinnedStore.update(hwnd, { click_through });
  await savePinnedState();
}
