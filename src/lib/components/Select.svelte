<script lang="ts">
  interface SelectOption {
    value: string;
    label: string;
  }

  let {
    value = $bindable(''),
    label = '',
    options = [],
    disabled = false,
    id = `select-${Math.random().toString(36).slice(2, 7)}`,
    onchange,
  }: {
    value?: string;
    label?: string;
    options?: SelectOption[];
    disabled?: boolean;
    id?: string;
    onchange?: () => void;
  } = $props();
</script>

<div class="select-field" class:select-field--disabled={disabled}>
  {#if label}
    <label class="select-field__label" for={id}>{label}</label>
  {/if}

  <div class="select-field__wrapper">
    <select {id} {disabled} bind:value {onchange} class="select-field__select">
      {#each options as opt (opt.value)}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
    <span class="select-field__chevron" aria-hidden="true">
      <svg width="10" height="6" viewBox="0 0 10 6" fill="none">
        <path d="M1 1L5 5L9 1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </span>
  </div>
</div>
