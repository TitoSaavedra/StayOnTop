import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { pinnedStore } from '$stores/pinned.store';
import { processesStore } from '$stores/processes.store';
import { pinWindow, unpinWindow } from '$services/window.service';
import type { ProcessInfo } from '$lib/types';

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

  // Find in the current process list first
  const procs = get(processesStore);
  let proc: ProcessInfo | undefined = procs.find((p) => p.hwnd === hwnd);

  // Fallback: ask Rust to identify the window directly
  if (!proc) {
    proc = await invoke<ProcessInfo | null>('get_process_by_hwnd', { hwnd }) ?? undefined;
  }

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
      if (event.state === 'Pressed') handleShortcut().catch((err) => console.error('[hotkey]', err));
    });
    activeShortcut = accelerator;
  } catch (err) {
    console.error('[hotkey] Failed to register shortcut:', err);
  }
}
