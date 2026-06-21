<script lang="ts">
  let {
    value = $bindable(''),
    label = '',
    placeholder = '',
    error = '',
    disabled = false,
    type = 'text',
    id = `input-${Math.random().toString(36).slice(2, 7)}`,
  }: {
    value?: string;
    label?: string;
    placeholder?: string;
    error?: string;
    disabled?: boolean;
    type?: 'text' | 'password' | 'search';
    id?: string;
  } = $props();
</script>

<div class="field" class:field--error={!!error} class:field--disabled={disabled}>
  {#if label}
    <label class="field__label" for={id}>{label}</label>
  {/if}

  <input
    {id}
    {type}
    {placeholder}
    {disabled}
    bind:value
    class="field__input"
  />

  {#if error}
    <span class="field__hint field__hint--error">{error}</span>
  {/if}
</div>
