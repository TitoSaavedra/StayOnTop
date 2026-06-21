<script lang="ts">
  let {
    checked = $bindable(false),
    label = '',
    disabled = false,
    id = `checkbox-${Math.random().toString(36).slice(2, 7)}`,
    onchange,
  }: {
    checked?: boolean;
    label?: string;
    disabled?: boolean;
    id?: string;
    onchange?: () => void;
  } = $props();
</script>

<label class="toggle" class:toggle--disabled={disabled} for={id}>
  <input
    {id}
    type="checkbox"
    {disabled}
    bind:checked
    {onchange}
    class="toggle__input"
  />
  <span class="toggle__track" aria-hidden="true">
    <span class="toggle__thumb"></span>
  </span>
  {#if label}
    <span class="toggle__label">{label}</span>
  {/if}
</label>
