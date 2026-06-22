<script lang="ts">
  import { onDestroy } from 'svelte';
  import { Select, Checkbox, Slider, Button, Input } from '$components';
  import { settingsStore } from '$stores/settings.store';
  import { saveSettings, setStartWithWindows } from '$services/settings.service';
  import { t } from '$lib/i18n';
  import type { AppSettings } from '$lib/types';

  const LANGUAGE_OPTIONS = [
    { value: 'en',    label: 'English' },
    { value: 'es',    label: 'Español' },
    { value: 'pt-br', label: 'Português (BR)' },
  ];

  let settings = $state<AppSettings | null>(null);
  let saving = $state(false);
  let newExcluded = $state('');
  let capturingHotkey = $state(false);

  const unsub = settingsStore.subscribe((v) => {
    settings = { ...v };
  });

  onDestroy(unsub);

  function handleLanguageChange() {
    if (settings?.language) {
      settingsStore.patch({ language: settings.language });
    }
  }

  async function handleSave() {
    if (!settings) return;
    saving = true;
    try {
      await saveSettings(settings);
    } finally {
      saving = false;
    }
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
    settings.excluded_processes = settings.excluded_processes.filter((p) => p !== name);
  }

  function formatMs(ms: number) {
    return ms >= 1000 ? `${ms / 1000}s` : `${ms}ms`;
  }

  function startCaptureHotkey() {
    capturingHotkey = true;
  }

  function captureHotkey(e: KeyboardEvent) {
    if (!capturingHotkey || !settings) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.key === 'Escape') {
      capturingHotkey = false;
      return;
    }
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;

    const parts: string[] = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);

    settings.hotkey_pin_toggle = parts.join('+');
    capturingHotkey = false;
  }
</script>

<div class="settings">
  {#if settings}
    <section class="settings__section">
      <h2 class="settings__section-title">{$t.settings.general}</h2>

      <div class="settings__row">
        <Select
          label={$t.settings.language}
          bind:value={settings.language}
          options={LANGUAGE_OPTIONS}
          onchange={handleLanguageChange}
        />
      </div>

      <div class="settings__row">
        <Checkbox
          label={$t.settings.startWithWindows}
          bind:checked={settings.start_with_windows}
          onchange={handleStartupToggle}
        />
      </div>

      <div class="settings__row">
        <Checkbox
          label={$t.settings.keepOnTop}
          bind:checked={settings.keep_app_on_top}
          onchange={handleSave}
        />
      </div>

      <div class="settings__row">
        <Slider
          label={$t.settings.defaultOpacity}
          bind:value={settings.default_opacity}
          min={0.1}
          max={1}
          step={0.05}
          onchange={handleSave}
        />
      </div>
    </section>

    <section class="settings__section">
      <h2 class="settings__section-title">{$t.settings.processRefresh}</h2>

      <div class="settings__row">
        <Slider
          label={$t.settings.refreshInterval}
          bind:value={settings.refresh_interval_ms}
          min={1000}
          max={30000}
          step={1000}
          formatValue={formatMs}
        />
      </div>
    </section>

    <section class="settings__section">
      <h2 class="settings__section-title">{$t.settings.hotkeys}</h2>

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
    </section>

    <section class="settings__section settings__section--grow">
      <h2 class="settings__section-title">{$t.settings.excludedProcesses}</h2>

      <div class="settings__add-row">
        <Input
          bind:value={newExcluded}
          placeholder={$t.settings.addPlaceholder}
        />
        <Button variant="secondary" size="sm" onclick={addExcluded} disabled={!newExcluded.trim()}>
          {$t.settings.add}
        </Button>
      </div>

      <ul class="excluded-list">
        {#each settings.excluded_processes as name (name)}
          <li class="excluded-item">
            <span class="excluded-item__name">{name}</span>
            <Button variant="ghost" size="sm" onclick={() => removeExcluded(name)} title="Remove">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </Button>
          </li>
        {/each}
      </ul>
    </section>

    <div class="settings__footer">
      <Button variant="primary" onclick={handleSave} disabled={saving}>
        {saving ? $t.settings.saving : $t.settings.save}
      </Button>
    </div>
  {:else}
    <div class="settings__loading">
      <span class="settings__spinner" aria-label="Loading settings"></span>
    </div>
  {/if}
</div>
