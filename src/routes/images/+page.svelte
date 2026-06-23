<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { Button, Badge, Slider, Checkbox } from '$components';
  import { imagesStore } from '$stores/images.store';
  import { t } from '$lib/i18n';
  import type { PinnedImage } from '$lib/types';

  let imageList = $state<PinnedImage[]>([]);
  const unsub = imagesStore.subscribe((v) => (imageList = v));

  async function persistImages() {
    // Capture current positions from native windows before saving
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

  async function addImage() {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'gif', 'webp'] }],
    });
    if (!selected) return;

    const path = typeof selected === 'string' ? selected : selected.path;
    const id = crypto.randomUUID();
    const filename = path.split(/[\\/]/).pop() ?? path;

    // Place near center of a typical 1080p screen
    const x = Math.round((1920 - 400) / 2);
    const y = Math.round((1080 - 300) / 2);

    try {
      await invoke('pin_image', { id, path, x, y, opacity: 1.0, scale: 1.0 });
      imagesStore.add({ id, path, x, y, opacity: 1.0, scale: 1.0, click_through: false, filename });
      await persistImages();
    } catch (err) {
      console.error('pin_image failed:', err);
    }
  }

  async function unpinImage(id: string) {
    await invoke('unpin_image', { id }).catch(() => {});
    imagesStore.remove(id);
    await persistImages();
  }

  async function unpinAll() {
    for (const img of imageList) {
      await invoke('unpin_image', { id: img.id }).catch(() => {});
    }
    imagesStore.clear();
    await invoke('save_pinned_images', { images: [] }).catch(() => {});
  }

  // oninput: fire-and-forget — keeps the native window in sync while dragging
  // without blocking the slider or triggering store re-renders mid-drag
  function liveOpacity(id: string, opacity: number) {
    invoke('set_image_opacity', { id, opacity }).catch(() => {});
  }
  function liveScale(id: string, scale: number) {
    invoke('set_image_scale', { id, scale }).catch(() => {});
  }

  // onchange: called once on release — update store and persist final value
  async function saveOpacity(id: string, opacity: number) {
    imagesStore.update(id, { opacity });
    await persistImages();
  }
  async function saveScale(id: string, scale: number) {
    imagesStore.update(id, { scale });
    await persistImages();
  }

  async function resetScale(id: string) {
    invoke('set_image_scale', { id, scale: 1.0 }).catch(() => {});
    imagesStore.update(id, { scale: 1.0 });
    await persistImages();
  }

  async function updateClickThrough(id: string, clickThrough: boolean) {
    await invoke('set_image_click_through', { id, clickThrough }).catch(() => {});
    imagesStore.update(id, { click_through: clickThrough });
    await persistImages();
  }

  onMount(async () => {
    // If images are already in the store (user returned to this tab), skip —
    // calling pin_image again would create duplicate native windows and orphan the originals
    if (imageList.length > 0) return;

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
        // Restore click-through if it was set
        if (img.click_through) {
          await invoke('set_image_click_through', { id: img.id, clickThrough: true }).catch(() => {});
        }
        imagesStore.add({ ...img, filename });
      } catch {
        // File unreadable or window creation failed — skip silently
      }
    }
  });

  onDestroy(() => unsub());
</script>

<div class="images">
  <section class="images__section">
    <div class="images__section-header">
      <h2 class="images__section-title">{$t.images.tab}</h2>
      {#if imageList.length > 0}
        <Button variant="ghost" size="sm" onclick={unpinAll}>
          {$t.images.unpinAll}
        </Button>
      {/if}
    </div>

    {#if imageList.length === 0}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
      <div class="images__drop-zone" role="button" tabindex="0" onclick={addImage}>
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <rect x="3" y="3" width="18" height="18" rx="3" stroke="currentColor" stroke-width="1.4" stroke-dasharray="3 2"/>
          <path d="M12 8v8M8 12h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
        <span>{$t.images.addImage}</span>
      </div>
    {:else}
      <ul class="img-list">
        {#each imageList as img (img.id)}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <li class="img-item">
            <div class="img-item__header">
              <span class="img-item__name" title={img.path}>{img.filename ?? img.path}</span>
              <Badge color="accent">{$t.images.onTop}</Badge>
              <Button
                variant="ghost"
                size="sm"
                onclick={() => unpinImage(img.id)}
                title={$t.images.unpin}
              >
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                  <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                </svg>
              </Button>
            </div>
            <div class="img-item__controls">
              <Slider
                bind:value={img.opacity}
                min={0.1}
                max={1}
                step={0.01}
                label={$t.images.opacity}
                oninput={() => liveOpacity(img.id, img.opacity)}
                onchange={() => saveOpacity(img.id, img.opacity)}
              />
              <div class="scale-row">
                <Slider
                  bind:value={img.scale}
                  min={0.1}
                  max={4}
                  step={0.05}
                  label={$t.images.scale}
                  formatValue={(v) => `${v.toFixed(2)}×`}
                  oninput={() => liveScale(img.id, img.scale)}
                  onchange={() => saveScale(img.id, img.scale)}
                />
                {#if Math.abs(img.scale - 1.0) > 0.01}
                  <button
                    class="btn btn--ghost btn--sm scale-reset"
                    onclick={() => resetScale(img.id)}
                    title="Reset to 1×"
                  >↺</button>
                {/if}
              </div>
              <Checkbox
                bind:checked={img.click_through}
                label={$t.images.clickThrough}
                onchange={() => updateClickThrough(img.id, img.click_through)}
              />
            </div>
          </li>
        {/each}
      </ul>

      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
      <div class="images__add-more" role="button" tabindex="0" onclick={addImage}>
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
          <path d="M5 1v8M1 5h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
        {$t.images.addImage}
      </div>
    {/if}
  </section>
</div>
