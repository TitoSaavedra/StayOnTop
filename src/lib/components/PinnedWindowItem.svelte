<script lang="ts">
  import Button from './Button.svelte';
  import Slider from './Slider.svelte';
  import Checkbox from './Checkbox.svelte';
  import Badge from './Badge.svelte';
  import { unpinWindow, updatePinnedOpacity, updatePinnedClickThrough, getWindowRect, setWindowPosSize } from '$services/window.service';
  import { t } from '$lib/i18n';
  import type { PinnedWindow } from '$lib/types';

  let { win }: { win: PinnedWindow } = $props();

  let opacity = $state(win.opacity);
  let clickThrough = $state(win.click_through);
  let showPosition = $state(false);
  let posX = $state(0);
  let posY = $state(0);
  let posW = $state(800);
  let posH = $state(600);

  $effect(() => {
    opacity = win.opacity;
    clickThrough = win.click_through;
  });

  async function togglePosition() {
    showPosition = !showPosition;
    if (showPosition) {
      const rect = await getWindowRect(win.hwnd);
      if (rect) {
        posX = rect.x;
        posY = rect.y;
        posW = rect.width;
        posH = rect.height;
      }
    }
  }

  async function applyPos() {
    await setWindowPosSize(win.hwnd, posX, posY, posW, posH);
  }

  function onPosKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') applyPos();
  }
</script>

<li class="pinned-item">
  <div class="pinned-item__header">
    {#if win.icon}
      <img class="pinned-item__icon" src={win.icon} alt="" width="16" height="16" aria-hidden="true" />
    {/if}
    <span class="pinned-item__name">{win.process_name}</span>
    <Badge color="accent">{$t.home.onTop}</Badge>
    <Button variant="ghost" size="sm" onclick={() => unpinWindow(win.hwnd)} title={$t.home.unpin}>
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
    </Button>
  </div>
  {#if win.window_title}
    <p class="pinned-item__subtitle">{win.window_title}</p>
  {/if}
  <div class="pinned-item__controls">
    <Slider
      bind:value={opacity}
      min={0.1}
      max={1}
      step={0.01}
      label={$t.home.opacity}
      oninput={() => updatePinnedOpacity(win.hwnd, opacity)}
    />
    <Checkbox
      bind:checked={clickThrough}
      label={$t.home.clickThrough}
      onchange={() => updatePinnedClickThrough(win.hwnd, clickThrough)}
    />
  </div>

  <button class="pinned-item__pos-toggle" onclick={togglePosition}>
    <svg
      width="10" height="10" viewBox="0 0 10 10" fill="none"
      aria-hidden="true"
      class:pinned-item__pos-arrow--open={showPosition}
    >
      <path d="M2 4l3 3 3-3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    {$t.home.positionSize}
  </button>

  {#if showPosition}
    <div class="pinned-item__pos">
      <div class="pinned-item__pos-row">
        <label class="pinned-item__pos-field">
          <span>X</span>
          <input type="number" bind:value={posX} onblur={applyPos} onkeydown={onPosKeydown} />
        </label>
        <label class="pinned-item__pos-field">
          <span>Y</span>
          <input type="number" bind:value={posY} onblur={applyPos} onkeydown={onPosKeydown} />
        </label>
      </div>
      <div class="pinned-item__pos-row">
        <label class="pinned-item__pos-field">
          <span>W</span>
          <input type="number" bind:value={posW} onblur={applyPos} onkeydown={onPosKeydown} />
        </label>
        <label class="pinned-item__pos-field">
          <span>H</span>
          <input type="number" bind:value={posH} onblur={applyPos} onkeydown={onPosKeydown} />
        </label>
      </div>
    </div>
  {/if}
</li>
