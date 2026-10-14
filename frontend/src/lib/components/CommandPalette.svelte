<script lang="ts">
  // Ctrl+K / Ctrl+Shift+P: jump to a host, an open session, a page or an action by typing.
  // Ctrl+Shift+T opens it in "hosts" mode to start a new session.
  import { goto } from '$app/navigation';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { openSession } from '$lib/nav';
  import { navItems, settingsNavItem } from '$lib/navItems';
  import { getPalette, closePalette } from '$lib/stores/commandPalette.svelte';
  import { getTabs, tabLabel, LOCAL_HOST_ID } from '$lib/stores/sessionTabs.svelte';
  import { setSelectedTabId, setShowFiles } from '$lib/stores/sessionView.svelte';
  import { isFavorite, lastUsedAt } from '$lib/stores/hostPrefs.svelte';
  import { toggleTheme } from '$lib/stores/theme.svelte';
  import { toggleAiChat } from '$lib/stores/aiChat.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { Button, Input, Kbd, Icon, focusTrap } from '@cecepazhar/caui';

  let { onLock }: { onLock: () => void } = $props();

  type Group = 'hosts' | 'sessions' | 'pages' | 'actions';
  interface Item {
    id: string;
    group: Group;
    label: string;
    hint?: string;
    run: () => void;
  }

  const palette = getPalette();
  let query = $state('');
  let active = $state(0);
  let hosts = $state<HostRecord[]>([]);
  let inputRef = $state<HTMLInputElement | undefined>(undefined);
  let list: HTMLUListElement | undefined = $state();

  // Fresh host list and a clean query every time it opens.
  $effect(() => {
    if (!palette.open) return;
    query = '';
    active = 0;
    listHosts()
      .then((all) => (hosts = all))
      .catch(() => (hosts = []));
    requestAnimationFrame(() => inputRef?.focus());
  });

  function hostRank(host: HostRecord): number {
    return (isFavorite(host.id) ? 1e15 : 0) + (lastUsedAt(host.id) ?? 0);
  }

  function showTab(id: string) {
    setSelectedTabId(id);
    void goto('/session');
  }

  const items = $derived.by((): Item[] => {
    const result: Item[] = [
      { id: 'local', group: 'hosts', label: t('palette.localTerminal'), hint: 'localhost', run: () => void openSession(LOCAL_HOST_ID) },
      ...[...hosts]
        .sort((a, b) => hostRank(b) - hostRank(a))
        .map((host) => ({
          id: `host:${host.id}`,
          group: 'hosts' as const,
          label: host.label,
          hint: `${host.username}@${host.address}${host.port && host.port !== 22 ? `:${host.port}` : ''}`,
          run: () => void openSession(host.id)
        }))
    ];
    if (palette.mode === 'hosts') return result;

    for (const tab of getTabs()) {
      result.push({ id: `tab:${tab.id}`, group: 'sessions', label: tabLabel(tab), hint: tab.host.address, run: () => showTab(tab.id) });
    }
    for (const page of [...navItems(), settingsNavItem()]) {
      result.push({ id: `page:${page.href}`, group: 'pages', label: page.label, run: () => void goto(page.href) });
    }
    result.push({ id: 'page:/design-system', group: 'pages', label: 'Design System Lab (CAUI v1.1)', run: () => void goto('/design-system') });
    result.push(
      { id: 'action:theme', group: 'actions', label: t('palette.toggleTheme'), run: toggleTheme },
      {
        id: 'action:ai',
        group: 'actions',
        label: t('palette.toggleAi'),
        run: () => {
          setShowFiles(false);
          toggleAiChat();
        }
      },
      { id: 'action:lock', group: 'actions', label: t('palette.lockVault'), run: onLock }
    );
    return result;
  });

  /** Every word of the query must appear in the label or hint; label-prefix matches rank first. */
  const filtered = $derived.by(() => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length === 0) return items;
    return items
      .filter((item) => {
        const haystack = `${item.label} ${item.hint ?? ''}`.toLowerCase();
        return words.every((word) => haystack.includes(word));
      })
      .sort((a, b) => Number(b.label.toLowerCase().startsWith(words[0])) - Number(a.label.toLowerCase().startsWith(words[0])));
  });

  $effect(() => {
    // Keep the highlight on a real row as the list shrinks.
    if (active >= filtered.length) active = Math.max(0, filtered.length - 1);
  });

  function choose(item: Item | undefined) {
    if (!item) return;
    closePalette();
    item.run();
  }

  function move(step: number) {
    if (filtered.length === 0) return;
    active = (active + step + filtered.length) % filtered.length;
    requestAnimationFrame(() => list?.querySelector(`[data-index="${active}"]`)?.scrollIntoView({ block: 'nearest' }));
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      move(1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      move(-1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      choose(filtered[active]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      closePalette();
    }
  }

  const GROUP_LABEL: Record<Group, string> = {
    hosts: 'palette.groupHosts',
    sessions: 'palette.groupSessions',
    pages: 'palette.groupPages',
    actions: 'palette.groupActions'
  };
</script>

{#if palette.open}
  <div
    class="fixed inset-0 z-[300] bg-black/40 dark:bg-black/70 backdrop-blur-xs flex items-start justify-center px-4 pt-[12vh]"
    role="presentation"
    onpointerdown={(e) => {
      if (e.target === e.currentTarget) closePalette();
    }}
  >
    <div
      class="w-full max-w-xl rounded-xl border border-[var(--ca-border)] bg-[var(--ca-surface-elevated)] shadow-2xl overflow-hidden font-sans"
      role="dialog"
      aria-modal="true"
      aria-label={t('palette.title')}
      use:focusTrap={{ onEscape: closePalette, returnFocus: true }}
    >
      <div class="p-2 border-b border-[var(--ca-border)]">
        <Input
          variant="ghost"
          size="md"
          bind:value={query}
          onkeydown={onKeydown}
          oninput={() => (active = 0)}
          placeholder={palette.mode === 'hosts' ? t('palette.placeholderHosts') : t('palette.placeholder')}
          ariaLabel={t('palette.title')}
          bind:ref={inputRef}
          class="w-full"
        >
          {#snippet leadingIcon()}
            <Icon name="search" size={16} class="text-[var(--ca-text-muted)] shrink-0" />
          {/snippet}
          {#snippet trailingAction()}
            <Button
              variant="ghost"
              size="icon"
              onclick={closePalette}
              tabindex={-1}
              aria-label="Close"
              class="text-[var(--ca-text-muted)] hover:text-[var(--ca-text-primary)]"
            >
              <Kbd size="sm">Esc</Kbd>
            </Button>
          {/snippet}
        </Input>
      </div>

      <ul bind:this={list} id="palette-list" role="listbox" class="max-h-[50vh] overflow-y-auto py-1.5 px-1.5 space-y-0.5">
        {#each filtered as item, i (item.id)}
          {#if i === 0 || filtered[i - 1].group !== item.group}
            <li role="presentation" class="px-2.5 pt-2.5 pb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--ca-text-muted)]">
              {t(GROUP_LABEL[item.group])}
            </li>
          {/if}
          <li role="presentation">
            <Button
              variant="ghost"
              id="palette-item-{i}"
              data-index={i}
              role="option"
              aria-selected={i === active}
              onpointermove={() => (active = i)}
              onclick={() => choose(item)}
              class="w-full !justify-between gap-3 text-left text-xs !font-normal px-2.5 py-1.5 {i === active
                ? '!bg-sky-600 !text-white dark:!bg-sky-600'
                : 'text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)]'}"
            >
              <span class="truncate font-medium">{item.label}</span>
              {#if item.hint}
                <span class="truncate text-[11px] font-mono {i === active ? 'text-sky-100' : 'text-[var(--ca-text-muted)]'}">{item.hint}</span>
              {/if}
            </Button>
          </li>
        {:else}
          <li class="px-4 py-8 text-center text-xs text-[var(--ca-text-muted)] font-sans">{t('palette.empty')}</li>
        {/each}
      </ul>
    </div>
  </div>
{/if}
