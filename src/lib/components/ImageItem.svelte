<script lang="ts">
  import Button from './Button.svelte';
  import Slider from './Slider.svelte';
  import Checkbox from './Checkbox.svelte';
  import Badge from './Badge.svelte';
  import {
    unpinImage,
    liveImageOpacity,
    saveImageOpacity,
    liveImageScale,
    saveImageScale,
    resetImageScale,
    setImageClickThrough,
  } from '$services/image.service';
  import { t } from '$lib/i18n';
  import type { PinnedImage } from '$lib/types';

  let { img }: { img: PinnedImage } = $props();

  let opacity = $state(img.opacity);
  let scale = $state(img.scale);
  let clickThrough = $state(img.click_through);

  $effect(() => {
    opacity = img.opacity;
    scale = img.scale;
    clickThrough = img.click_through;
  });
</script>

<li class="img-item">
  <div class="img-item__header">
    <span class="img-item__name" title={img.path}>{img.filename ?? img.path}</span>
    <Badge color="accent">{$t.images.onTop}</Badge>
    <Button variant="ghost" size="sm" onclick={() => unpinImage(img.id)} title={$t.images.unpin}>
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
    </Button>
  </div>
  <div class="img-item__controls">
    <Slider
      bind:value={opacity}
      min={0.1}
      max={1}
      step={0.01}
      label={$t.images.opacity}
      oninput={() => liveImageOpacity(img.id, opacity)}
      onchange={() => saveImageOpacity(img.id, opacity)}
    />
    <div class="scale-row">
      <Slider
        bind:value={scale}
        min={0.1}
        max={4}
        step={0.05}
        label={$t.images.scale}
        formatValue={(v) => `${v.toFixed(2)}×`}
        oninput={() => liveImageScale(img.id, scale)}
        onchange={() => saveImageScale(img.id, scale)}
      />
      {#if Math.abs(scale - 1.0) > 0.01}
        <button
          type="button"
          class="btn btn--ghost btn--sm scale-reset"
          onclick={() => resetImageScale(img.id)}
          title="Reset to 1×"
        >↺</button>
      {/if}
    </div>
    <Checkbox
      bind:checked={clickThrough}
      label={$t.images.clickThrough}
      onchange={() => setImageClickThrough(img.id, clickThrough)}
    />
  </div>
</li>
