import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { AppSettings } from '$lib/types';
import { settingsStore } from '$stores/settings.store';
import { registerHotkey } from '$services/hotkey.service';

export async function loadSettings(): Promise<void> {
  const settings = await invoke<AppSettings>('get_settings');
  settingsStore.set(settings);
  if (settings.keep_app_on_top) {
    await getCurrentWindow().setAlwaysOnTop(true);
  }
  await registerHotkey(settings.hotkey_pin_toggle);
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  await invoke('save_settings', { settings });
  settingsStore.set(settings);
  await getCurrentWindow().setAlwaysOnTop(settings.keep_app_on_top);
  await registerHotkey(settings.hotkey_pin_toggle);
}

export async function setStartWithWindows(enabled: boolean): Promise<void> {
  await invoke('register_startup', { enabled });
  settingsStore.patch({ start_with_windows: enabled });
}

export async function excludeProcess(name: string): Promise<void> {
  const current = get(settingsStore);
  if (current.excluded_processes.includes(name)) return;
  const updated: AppSettings = {
    ...current,
    excluded_processes: [...current.excluded_processes, name],
  };
  settingsStore.set(updated);
  await invoke('save_settings', { settings: updated });
}
