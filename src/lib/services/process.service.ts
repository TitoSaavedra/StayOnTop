import { invoke } from '@tauri-apps/api/core';
import type { ProcessInfo } from '$lib/types';
import { processesStore } from '$stores/processes.store';

export async function refreshProcesses(): Promise<void> {
  processesStore.setLoading(true);
  processesStore.setError(null);
  try {
    const list = await invoke<ProcessInfo[]>('get_processes');
    processesStore.setProcesses(list);
  } catch (err) {
    processesStore.setError(String(err));
  } finally {
    processesStore.setLoading(false);
  }
}
