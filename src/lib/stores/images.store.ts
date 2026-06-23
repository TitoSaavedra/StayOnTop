import { writable } from 'svelte/store';
import type { PinnedImage } from '$lib/types';

const images = writable<PinnedImage[]>([]);

function add(image: PinnedImage) {
  images.update((list) => {
    const exists = list.some((i) => i.id === image.id);
    return exists ? list : [...list, image];
  });
}

function remove(id: string) {
  images.update((list) => list.filter((i) => i.id !== id));
}

function update(id: string, patch: Partial<Omit<PinnedImage, 'id'>>) {
  images.update((list) => list.map((i) => (i.id === id ? { ...i, ...patch } : i)));
}

function clear() {
  images.set([]);
}

export const imagesStore = {
  subscribe: images.subscribe,
  add,
  remove,
  update,
  clear,
};
