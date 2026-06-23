<script lang="ts">
  import '../styles/main.scss';
  import { onMount } from 'svelte';
  import { TitleBar, StatusBar } from '$components';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { loadSettings } from '$services/settings.service';
  import { t } from '$lib/i18n';
  import type { Snippet } from 'svelte';

  interface Props { children: Snippet; }
  let { children }: Props = $props();

  const tabs = $derived([
    { path: '/',         label: $t.nav.windows },
    { path: '/images',   label: $t.nav.images },
    { path: '/settings', label: $t.nav.settings },
  ]);

  onMount(() => { loadSettings(); });
</script>

<div id="app">
  <TitleBar />

  <nav class="nav">
    {#each tabs as tab (tab.path)}
      <button
        class="nav__tab"
        class:nav__tab--active={$page.url.pathname === tab.path}
        onclick={() => goto(tab.path)}
      >
        {tab.label}
      </button>
    {/each}
  </nav>

  <main class="content">
    {@render children()}
  </main>

  <StatusBar />
</div>
