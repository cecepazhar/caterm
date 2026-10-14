<script lang="ts">
  import { Button } from '@cecepazhar/caui';

  let {
    totalItems = 0,
    page = $bindable(1),
    pageSize = $bindable(10),
    pageSizeOptions = [10, 25, 50],
    onPageChange
  }: {
    totalItems: number;
    page: number;
    pageSize: number;
    pageSizeOptions?: number[];
    onPageChange?: (newPage: number) => void;
  } = $props();

  const totalPages = $derived(Math.max(1, Math.ceil(totalItems / pageSize)));
  const startIndex = $derived((page - 1) * pageSize + 1);
  const endIndex = $derived(Math.min(page * pageSize, totalItems));

  function setPage(p: number) {
    if (p < 1 || p > totalPages) return;
    page = p;
    onPageChange?.(p);
  }
</script>

{#if totalItems > 0}
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-neutral-200 dark:border-neutral-800 text-xs text-neutral-500 select-none">
    <div class="flex items-center gap-2">
      <span>Tampilkan per halaman:</span>
      <select
        bind:value={pageSize}
        onchange={() => (page = 1)}
        class="px-2 py-1 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white font-medium focus:outline-none focus:border-sky-500"
      >
        {#each pageSizeOptions as opt}
          <option value={opt}>{opt}</option>
        {/each}
      </select>
      <span class="text-neutral-400">
        ({startIndex}-{endIndex} dari {totalItems} total)
      </span>
    </div>

    {#if totalPages > 1}
      <div class="flex items-center gap-1">
        <Button
          variant="secondary"
          size="sm"
          disabled={page <= 1}
          onclick={() => setPage(page - 1)}
        >
          ← Sebelumnya
        </Button>

        <span class="px-2 font-mono font-semibold text-neutral-900 dark:text-white">
          {page} / {totalPages}
        </span>

        <Button
          variant="secondary"
          size="sm"
          disabled={page >= totalPages}
          onclick={() => setPage(page + 1)}
        >
          Selanjutnya →
        </Button>
      </div>
    {/if}
  </div>
{/if}
