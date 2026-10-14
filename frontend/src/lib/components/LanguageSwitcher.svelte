<script lang="ts">
  import { LanguageSwitcher as CauiLanguageSwitcher } from '@cecepazhar/caui';
  import { getLocale, setLocale, LOCALES, type Locale } from '$lib/i18n/index.svelte';
  import type { ComponentProps } from 'svelte';

  type Props = Omit<ComponentProps<typeof CauiLanguageSwitcher>, 'value' | 'onchange'> & {
    value?: string;
    onchange?: (locale: string) => void;
  };

  let {
    compact = true,
    variant = 'outline',
    size = 'sm',
    locales = LOCALES.map((l) => ({ code: l.code, label: l.code.toUpperCase() })),
    value,
    onchange,
    class: customClass = '',
    ...restProps
  }: Props = $props();

  const current = $derived(value ?? getLocale());

  function handleChange(next: string) {
    setLocale(next as Locale);
    onchange?.(next);
  }
</script>

<CauiLanguageSwitcher
  {compact}
  {variant}
  {size}
  {locales}
  value={current}
  onchange={handleChange}
  class={customClass}
  {...restProps}
/>
