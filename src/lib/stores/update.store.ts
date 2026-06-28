import { writable } from 'svelte/store';
import type { Update } from '@tauri-apps/plugin-updater';

export interface UpdateState {
  available: boolean;
  version?: string;
  update?: Update;
}

export const updateStore = writable<UpdateState>({ available: false });
