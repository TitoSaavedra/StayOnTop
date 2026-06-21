import { writable, derived } from 'svelte/store';
import type { ProcessInfo } from '$lib/types';
import { settingsStore } from './settings.store';

const selectedHwnd = writable<number | null>(null);

const processes = writable<ProcessInfo[]>([]);
const loading = writable(false);
const error = writable<string | null>(null);
const appName = writable('');

const filtered = derived(
  [processes, settingsStore, appName],
  ([$processes, $settings, $appName]) => {
    const excluded = new Set($settings.excluded_processes.map((p) => p.toLowerCase()));

    return $processes.filter((p) => {
      const name = p.name.toLowerCase();
      if ($appName && name === $appName.toLowerCase()) return false;
      if (excluded.has(name)) return false;
      return true;
    });
  },
);

export const processesStore = {
  subscribe: processes.subscribe,
  filtered,
  loading: { subscribe: loading.subscribe },
  error: { subscribe: error.subscribe },
  selectedHwnd,
  setProcesses: (list: ProcessInfo[]) => processes.set(list),
  setLoading: (v: boolean) => loading.set(v),
  setError: (msg: string | null) => error.set(msg),
  setAppName: (name: string) => appName.set(name),
};
