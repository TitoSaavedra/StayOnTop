<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n';

  const appWindow = getCurrentWindow();

  function minimizeToTray() {
    appWindow.hide();
  }

  function quit() {
    invoke('quit_app');
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar__identity" data-tauri-drag-region>
    <svg class="titlebar__icon" width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <rect x="1" y="1" width="12" height="12" rx="3" fill="#e6b450" opacity="0.9"/>
      <path d="M4 7h6M7 4v6" stroke="#765b24" stroke-width="1.5" stroke-linecap="round"/>
    </svg>
    <span class="titlebar__title" data-tauri-drag-region>StayOnTop</span>
  </div>

  <div class="titlebar__controls">
    <button class="titlebar__btn titlebar__btn--minimize" onclick={minimizeToTray} title={$t.titlebar.minimize} aria-label={$t.titlebar.minimize}>
      <svg width="10" height="2" viewBox="0 0 10 2" fill="none" aria-hidden="true">
        <path d="M1 1H9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
    </button>
    <button class="titlebar__btn titlebar__btn--close" onclick={quit} title={$t.titlebar.close} aria-label={$t.titlebar.close}>
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
        <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</header>
