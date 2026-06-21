<script lang="ts">
  let {
    value = $bindable(1),
    min = 0,
    max = 1,
    step = 0.05,
    label = '',
    disabled = false,
    formatValue = (v: number) => `${Math.round(v * 100)}%`,
    onchange,
    oninput,
  }: {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    disabled?: boolean;
    formatValue?: (v: number) => string;
    onchange?: () => void;
    oninput?: () => void;
  } = $props();

  const percent = $derived(((value - min) / (max - min)) * 100);
</script>

<div class="slider-field" class:slider-field--disabled={disabled}>
  {#if label}
    <div class="slider-field__header">
      <span class="slider-field__label">{label}</span>
      <span class="slider-field__value">{formatValue(value)}</span>
    </div>
  {/if}

  <div class="slider-field__track-wrapper">
    <input
      type="range"
      {min}
      {max}
      {step}
      {disabled}
      bind:value
      {onchange}
      {oninput}
      class="slider-field__input"
      style="--percent: {percent}%"
    />
  </div>
</div>
