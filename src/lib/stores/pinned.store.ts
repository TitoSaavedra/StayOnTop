import { writable } from 'svelte/store';
import type { PinnedWindow } from '$lib/types';

const pinned = writable<PinnedWindow[]>([]);

function add(window: PinnedWindow) {
  pinned.update((list) => {
    const exists = list.some((w) => w.hwnd === window.hwnd);
    return exists ? list : [...list, window];
  });
}

function remove(hwnd: number) {
  pinned.update((list) => list.filter((w) => w.hwnd !== hwnd));
}

function update(hwnd: number, patch: Partial<Omit<PinnedWindow, 'hwnd'>>) {
  pinned.update((list) =>
    list.map((w) => (w.hwnd === hwnd ? { ...w, ...patch } : w)),
  );
}

function isPinned(hwnd: number, list: PinnedWindow[]) {
  return list.some((w) => w.hwnd === hwnd);
}

export const pinnedStore = {
  subscribe: pinned.subscribe,
  set: pinned.set,
  add,
  remove,
  update,
  isPinned,
};
