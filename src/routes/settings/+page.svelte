<script lang="ts">
  import { Select, Checkbox, Slider, Button, Input } from '$components';
  import { settingsStore } from '$stores/settings.store';
  import { saveSettings, setStartWithWindows } from '$services/settings.service';
  import { t } from '$lib/i18n';
  import type { AppSettings } from '$lib/types';
  import { getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';

  type Section = 'general' | 'hotkeys' | 'excluded' | 'about';

  type UpdateStatus =
    | { type: 'up-to-date' }
    | { type: 'available'; version: string; update: Update }
    | { type: 'installing'; progress: number }
    | { type: 'error' };

  const LANGUAGE_OPTIONS = [
    { value: 'en',    label: 'English' },
    { value: 'es',    label: 'Español' },
    { value: 'pt-br', label: 'Português (BR)' },
  ];

  let settings        = $state<AppSettings | null>(null);
  let saving          = $state(false);
  let newExcluded     = $state('');
  let capturingHotkey = $state(false);
  let activeSection   = $state<Section>('general');
  let appVersion      = $state('');
  let checking        = $state(false);
  let updateStatus    = $state<UpdateStatus | null>(null);

  getVersion().then(v => { appVersion = v; });

  $effect(() => { settings = { ...$settingsStore }; });

  const isDirty = $derived(
    settings !== null &&
    JSON.stringify(settings) !== JSON.stringify($settingsStore)
  );

  const filteredExcluded = $derived(
    settings?.excluded_processes.filter(p =>
      !newExcluded.trim() || p.toLowerCase().includes(newExcluded.toLowerCase().trim())
    ) ?? []
  );

  function handleLanguageChange() {
    if (settings?.language) settingsStore.patch({ language: settings.language });
  }

  async function handleSave() {
    if (!settings) return;
    saving = true;
    try { await saveSettings(settings); } finally { saving = false; }
  }

  async function handleStartupToggle() {
    if (!settings) return;
    await setStartWithWindows(settings.start_with_windows);
  }

  function addExcluded() {
    const name = newExcluded.trim();
    if (!name || !settings) return;
    if (!settings.excluded_processes.includes(name)) {
      settings.excluded_processes = [...settings.excluded_processes, name];
    }
    newExcluded = '';
  }

  function removeExcluded(name: string) {
    if (!settings) return;
    settings.excluded_processes = settings.excluded_processes.filter(p => p !== name);
  }

  function formatMs(ms: number) {
    return ms >= 1000 ? `${ms / 1000}s` : `${ms}ms`;
  }

  function startCaptureHotkey() { capturingHotkey = true; }

  function captureHotkey(e: KeyboardEvent) {
    if (!capturingHotkey || !settings) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape') { capturingHotkey = false; return; }
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;

    const parts: string[] = [];
    if (e.ctrlKey)  parts.push('Ctrl');
    if (e.altKey)   parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);

    settings.hotkey_pin_toggle = parts.join('+');
    capturingHotkey = false;
  }

  function discardChanges() {
    settings = { ...$settingsStore };
  }

  async function checkForUpdates() {
    checking = true;
    updateStatus = null;
    try {
      const update = await check();
      if (update?.available) {
        updateStatus = { type: 'available', version: update.version, update };
      } else {
        updateStatus = { type: 'up-to-date' };
      }
    } catch {
      updateStatus = { type: 'error' };
    } finally {
      checking = false;
    }
  }

  async function installUpdate() {
    if (updateStatus?.type !== 'available') return;
    const { update } = updateStatus;
    let contentLength = 0;
    let downloaded = 0;
    updateStatus = { type: 'installing', progress: 0 };
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          contentLength = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          updateStatus = {
            type: 'installing',
            progress: contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0,
          };
        }
      });
      await relaunch();
    } catch {
      updateStatus = { type: 'error' };
    }
  }

  function onExcludedKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') addExcluded();
  }
</script>

<div class="settings">
  {#if settings}
    <div class="settings__body">

      <!-- ── Sidebar ── -->
      <nav class="settings__sidebar">

        <button
          class="settings__nav-item"
          class:settings__nav-item--active={activeSection === 'general'}
          onclick={() => (activeSection = 'general')}
        >
          <!-- sliders / tune icon -->
          <svg class="settings__nav-icon" width="15" height="15" viewBox="0 0 15 15" fill="none" aria-hidden="true">
            <line x1="2"   y1="3.5" x2="4.5" y2="3.5"  stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <circle cx="6.5" cy="3.5" r="1.5"            stroke="currentColor" stroke-width="1.3"/>
            <line x1="8"   y1="3.5" x2="13"  y2="3.5"  stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <line x1="2"   y1="7.5" x2="9"   y2="7.5"  stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <circle cx="11"  cy="7.5" r="1.5"            stroke="currentColor" stroke-width="1.3"/>
            <line x1="12.5" y1="7.5" x2="13" y2="7.5"  stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <line x1="2"   y1="11.5" x2="4.5" y2="11.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <circle cx="6.5" cy="11.5" r="1.5"           stroke="currentColor" stroke-width="1.3"/>
            <line x1="8"   y1="11.5" x2="13" y2="11.5"  stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
          <span class="settings__nav-label">{$t.settings.general}</span>
        </button>

        <button
          class="settings__nav-item"
          class:settings__nav-item--active={activeSection === 'hotkeys'}
          onclick={() => (activeSection = 'hotkeys')}
        >
          <!-- keyboard icon -->
          <svg class="settings__nav-icon" width="15" height="15" viewBox="0 0 15 15" fill="none" aria-hidden="true">
            <rect x="1.5" y="4" width="12" height="7" rx="1.5" stroke="currentColor" stroke-width="1.3"/>
            <line x1="4"   y1="6.5" x2="4"   y2="6.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
            <line x1="7.5" y1="6.5" x2="7.5" y2="6.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
            <line x1="11"  y1="6.5" x2="11"  y2="6.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
            <line x1="5.5" y1="9"   x2="9.5" y2="9"   stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <span class="settings__nav-label">{$t.settings.shortHotkeys}</span>
        </button>

        <button
          class="settings__nav-item"
          class:settings__nav-item--active={activeSection === 'excluded'}
          onclick={() => (activeSection = 'excluded')}
        >
          {#if settings.excluded_processes.length > 0}
            <span class="settings__nav-badge">{settings.excluded_processes.length}</span>
          {/if}
          <!-- block / no-entry icon -->
          <svg class="settings__nav-icon" width="15" height="15" viewBox="0 0 15 15" fill="none" aria-hidden="true">
            <circle cx="7.5" cy="7.5" r="5.5" stroke="currentColor" stroke-width="1.3"/>
            <line x1="3.6" y1="3.6" x2="11.4" y2="11.4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
          <span class="settings__nav-label">{$t.settings.shortExcluded}</span>
        </button>

        <button
          class="settings__nav-item"
          class:settings__nav-item--active={activeSection === 'about'}
          onclick={() => (activeSection = 'about')}
        >
          <!-- info icon -->
          <svg class="settings__nav-icon" width="15" height="15" viewBox="0 0 15 15" fill="none" aria-hidden="true">
            <circle cx="7.5" cy="7.5" r="5.5" stroke="currentColor" stroke-width="1.3"/>
            <line x1="7.5" y1="6.5" x2="7.5" y2="10.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <circle cx="7.5" cy="4.5" r="0.7" fill="currentColor"/>
          </svg>
          <span class="settings__nav-label">{$t.settings.about}</span>
        </button>

      </nav>

      <!-- ── Content ── -->
      <div class="settings__content">
        {#if isDirty}
          <div class="settings__banner">
            <span class="settings__banner-dot"></span>
            <span class="settings__banner-text">{$t.settings.unsavedChanges}</span>
            <button
              class="settings__banner-discard"
              onclick={discardChanges}
              title="Discard changes"
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
            <button
              class="settings__banner-save"
              onclick={handleSave}
              disabled={saving}
              title={$t.settings.save}
            >
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
                <path d="M2 7L5.5 10.5L12 3.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
          </div>
        {/if}

        {#if activeSection === 'general'}
          <div class="settings__panel">
            <h2 class="settings__panel-title">{$t.settings.general}</h2>
            <div class="settings__group">
              <Select
                label={$t.settings.language}
                bind:value={settings.language}
                options={LANGUAGE_OPTIONS}
                onchange={handleLanguageChange}
              />
              <Checkbox
                label={$t.settings.startWithWindows}
                bind:checked={settings.start_with_windows}
                onchange={handleStartupToggle}
              />
              <Checkbox
                label={$t.settings.keepOnTop}
                bind:checked={settings.keep_app_on_top}
                onchange={handleSave}
              />
              <Slider
                label={$t.settings.defaultOpacity}
                bind:value={settings.default_opacity}
                min={0.1}
                max={1}
                step={0.05}
              />
              <Slider
                label={$t.settings.refreshInterval}
                bind:value={settings.refresh_interval_ms}
                min={1000}
                max={30000}
                step={1000}
                formatValue={formatMs}
              />
            </div>
          </div>

        {:else if activeSection === 'hotkeys'}
          <div class="settings__panel">
            <h2 class="settings__panel-title">{$t.settings.hotkeys}</h2>
            <div class="settings__hotkey-row">
              <span class="settings__hotkey-label">{$t.settings.pinToggle}</span>
              <!-- svelte-ignore a11y_interactive_supports_focus -->
              <button
                class="hotkey-input"
                class:hotkey-input--capturing={capturingHotkey}
                onclick={startCaptureHotkey}
                onblur={() => (capturingHotkey = false)}
                onkeydown={captureHotkey}
              >
                {capturingHotkey ? $t.settings.pressKeys : settings.hotkey_pin_toggle}
              </button>
            </div>
          </div>

        {:else if activeSection === 'excluded'}
          <div class="settings__panel settings__panel--fill">
            <h2 class="settings__panel-title">{$t.settings.excludedProcesses}</h2>
            <div class="excluded">
              {#if filteredExcluded.length > 0}
                <ul class="excluded__list">
                  {#each filteredExcluded as name (name)}
                    <li class="excluded-item">
                      <span class="excluded-item__name">{name}</span>
                      <button
                        class="excluded-item__remove"
                        onclick={() => removeExcluded(name)}
                        title="Remove"
                      >
                        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
                          <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                        </svg>
                      </button>
                    </li>
                  {/each}
                </ul>
              {:else}
                <p class="excluded__empty">
                  {newExcluded.trim() ? $t.settings.noMatches : $t.settings.noExcluded}
                </p>
              {/if}

              <div class="excluded__divider"></div>

              <div class="excluded__add-row">
                <input
                  class="excluded__add-input"
                  bind:value={newExcluded}
                  placeholder={$t.settings.addPlaceholder}
                  onkeydown={onExcludedKeydown}
                />
                <button
                  class="excluded__add-btn"
                  onclick={addExcluded}
                  disabled={!newExcluded.trim()}
                  title={$t.settings.add}
                >
                  <svg width="13" height="13" viewBox="0 0 13 13" fill="none" aria-hidden="true">
                    <path d="M11 3v4a2 2 0 01-2 2H2M2 9l3-3M2 9l3 3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>

        {:else if activeSection === 'about'}
          <div class="settings__panel">
            <h2 class="settings__panel-title">{$t.settings.about}</h2>

            <div class="about">
              <div class="about__app">
                <span class="about__name">StayOnTop</span>
                <span class="about__version">v{appVersion}</span>
              </div>

              <div class="about__divider"></div>

              <div class="about__author">
                <span class="about__made-by">{$t.settings.madeBy}</span>
                <button class="about__link" onclick={() => openUrl('https://github.com/TitoSaavedra/StayOnTop')}>
                  Tito Saavedra
                </button>
              </div>

              <div class="about__divider"></div>

              <div class="about__updates">
                {#if updateStatus?.type === 'installing'}
                  <div class="about__progress">
                    <div class="about__progress-bar" style="width: {updateStatus.progress}%"></div>
                  </div>
                  <span class="about__status about__status--new">
                    {$t.settings.installing} {updateStatus.progress}%
                  </span>
                {:else}
                  <Button variant="secondary" size="sm" onclick={checkForUpdates} disabled={checking}>
                    {checking ? $t.settings.checking : $t.settings.checkUpdates}
                  </Button>
                  {#if updateStatus}
                    {#if updateStatus.type === 'up-to-date'}
                      <span class="about__status about__status--ok">
                        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                          <path d="M2 6L4.5 8.5L10 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                        {$t.settings.upToDate}
                      </span>
                    {:else if updateStatus.type === 'available'}
                      <div class="about__update-row">
                        <span class="about__status about__status--new">
                          {$t.settings.updateAvailable}: v{updateStatus.version}
                        </span>
                        <Button variant="primary" size="sm" onclick={installUpdate}>
                          {$t.settings.install}
                        </Button>
                      </div>
                    {:else}
                      <span class="about__status about__status--error">{$t.settings.updateError}</span>
                    {/if}
                  {/if}
                {/if}
              </div>
            </div>
          </div>
        {/if}

      </div>
    </div>

  {:else}
    <div class="settings__loading">
      <span class="settings__spinner" aria-label="Loading settings"></span>
    </div>
  {/if}
</div>
