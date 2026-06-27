<script lang="ts">
  import { onMount } from 'svelte';
  import { Button, ImageItem } from '$components';
  import { imagesStore } from '$stores/images.store';
  import { addImage, unpinAllImages, restoreImages } from '$services/image.service';
  import { t } from '$lib/i18n';

  onMount(async () => {
    if ($imagesStore.length > 0) return;
    await restoreImages();
  });
</script>

<div class="images">
  <section class="images__section">
    <div class="images__section-header">
      <h2 class="images__section-title">{$t.images.tab}</h2>
      {#if $imagesStore.length > 0}
        <Button variant="ghost" size="sm" onclick={unpinAllImages}>
          {$t.images.unpinAll}
        </Button>
      {/if}
    </div>

    {#if $imagesStore.length === 0}
      <button type="button" class="images__drop-zone" onclick={addImage}>
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <rect x="3" y="3" width="18" height="18" rx="3" stroke="currentColor" stroke-width="1.4" stroke-dasharray="3 2"/>
          <path d="M12 8v8M8 12h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
        <span>{$t.images.addImage}</span>
      </button>
    {:else}
      <ul class="img-list">
        {#each $imagesStore as img (img.id)}
          <ImageItem {img} />
        {/each}
      </ul>

      <button type="button" class="images__add-more" onclick={addImage}>
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
          <path d="M5 1v8M1 5h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
        {$t.images.addImage}
      </button>
    {/if}
  </section>
</div>
