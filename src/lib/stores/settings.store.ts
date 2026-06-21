import { writable } from 'svelte/store';
import type { AppSettings } from '$lib/types';
import { DEFAULT_SETTINGS } from '$lib/types';

const settings = writable<AppSettings>({ ...DEFAULT_SETTINGS });

export const settingsStore = {
  subscribe: settings.subscribe,
  set: settings.set,
  update: settings.update,
  patch: (partial: Partial<AppSettings>) =>
    settings.update((s) => ({ ...s, ...partial })),
};
