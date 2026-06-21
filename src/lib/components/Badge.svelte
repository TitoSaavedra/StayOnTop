<script lang="ts">
  import type { Snippet } from 'svelte';

  type Color = 'default' | 'success' | 'warning' | 'error' | 'info' | 'accent';

  interface Props {
    color?: Color;
    children: Snippet;
  }

  let { color = 'default', children }: Props = $props();
</script>

<span class="badge badge--{color}">
  {@render children()}
</span>
