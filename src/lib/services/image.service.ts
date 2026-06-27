import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { get } from 'svelte/store';
import { imagesStore } from '$stores/images.store';
import type { PinnedImage } from '$lib/types';

async function persistImages(): Promise<void> {
  const imageList = get(imagesStore);
  const updated = await Promise.all(
    imageList.map(async (img) => {
      try {
        const pos = await invoke<[number, number] | null>('get_image_position', { id: img.id });
        if (pos) return { ...img, x: pos[0], y: pos[1] };
      } catch {
        // ignore
      }
      return img;
    }),
  );
  await invoke('save_pinned_images', { images: updated }).catch(() => {});
}

export async function addImage(): Promise<void> {
  const selected = await openDialog({
    multiple: false,
    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'gif', 'webp'] }],
  });
  if (!selected) return;

  const path = typeof selected === 'string' ? selected : selected.path;
  const id = crypto.randomUUID();
  const filename = path.split(/[\\/]/).pop() ?? path;
  const x = Math.round((1920 - 400) / 2);
  const y = Math.round((1080 - 300) / 2);

  await invoke('pin_image', { id, path, x, y, opacity: 1.0, scale: 1.0 });
  imagesStore.add({ id, path, x, y, opacity: 1.0, scale: 1.0, click_through: false, filename });
  await persistImages();
}

export async function unpinImage(id: string): Promise<void> {
  await invoke('unpin_image', { id }).catch(() => {});
  imagesStore.remove(id);
  await persistImages();
}

export async function unpinAllImages(): Promise<void> {
  const current = get(imagesStore);
  for (const img of current) {
    await invoke('unpin_image', { id: img.id }).catch(() => {});
  }
  imagesStore.clear();
  await invoke('save_pinned_images', { images: [] }).catch(() => {});
}

export function liveImageOpacity(id: string, opacity: number): void {
  invoke('set_image_opacity', { id, opacity }).catch(() => {});
}

export async function saveImageOpacity(id: string, opacity: number): Promise<void> {
  imagesStore.update(id, { opacity });
  await persistImages();
}

export function liveImageScale(id: string, scale: number): void {
  invoke('set_image_scale', { id, scale }).catch(() => {});
}

export async function saveImageScale(id: string, scale: number): Promise<void> {
  imagesStore.update(id, { scale });
  await persistImages();
}

export async function resetImageScale(id: string): Promise<void> {
  invoke('set_image_scale', { id, scale: 1.0 }).catch(() => {});
  imagesStore.update(id, { scale: 1.0 });
  await persistImages();
}

export async function setImageClickThrough(id: string, clickThrough: boolean): Promise<void> {
  await invoke('set_image_click_through', { id, clickThrough }).catch(() => {});
  imagesStore.update(id, { click_through: clickThrough });
  await persistImages();
}

export async function restoreImages(): Promise<void> {
  const persisted = await invoke<PinnedImage[]>('get_pinned_images');
  for (const img of persisted) {
    const filename = img.path.split(/[\\/]/).pop() ?? img.path;
    try {
      await invoke('pin_image', {
        id: img.id,
        path: img.path,
        x: img.x,
        y: img.y,
        opacity: img.opacity,
        scale: img.scale,
      });
      if (img.click_through) {
        await invoke('set_image_click_through', { id: img.id, clickThrough: true }).catch(() => {});
      }
      imagesStore.add({ ...img, filename });
    } catch {
      // File unreadable or window creation failed — skip silently
    }
  }
}
