import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { pinnedStore } from '$lib/stores/pinned.store';
import { processesStore } from '$lib/stores/processes.store';
import { pinWindow, unpinWindow } from '$lib/services/window.service';

let activeShortcut: string | null = null;

/** Convert "Alt+P" / "Ctrl+Shift+Delete" to the format Tauri's global-shortcut expects. */
function toAccelerator(combo: string): string {
  return combo
    .split('+')
    .map((p) => {
      const s = p.trim();
      const lower = s.toLowerCase();
      if (lower === 'ctrl') return 'Ctrl';
      if (lower === 'alt') return 'Alt';
      if (lower === 'shift') return 'Shift';
      if (lower === 'super' || lower === 'win' || lower === 'meta') return 'Super';
      return s.toUpperCase();
    })
    .join('+');
}

async function handleShortcut() {
  const hwnd = await invoke<number | null>('get_foreground_window');
  if (!hwnd) return;

  const pinned = get(pinnedStore);
  if (pinnedStore.isPinned(hwnd, pinned)) {
    await unpinWindow(hwnd);
    return;
  }

  // Find the process in the raw list (not filtered — search might hide it)
  const procs = get(processesStore);
  const proc = procs.find((p) => p.hwnd === hwnd);

  if (proc) {
    await pinWindow({
      hwnd: proc.hwnd,
      process_name: proc.name,
      window_title: proc.window_title,
      icon: proc.icon,
    });
  }
}

export async function registerHotkey(combo: string): Promise<void> {
  try {
    if (activeShortcut) {
      const wasRegistered = await isRegistered(activeShortcut);
      if (wasRegistered) await unregister(activeShortcut);
      activeShortcut = null;
    }

    if (!combo) return;

    const accelerator = toAccelerator(combo);
    await register(accelerator, (event) => {
      if (event.state === 'Pressed') handleShortcut();
    });
    activeShortcut = accelerator;
  } catch (err) {
    console.error('[hotkey] Failed to register shortcut:', err);
  }
}
