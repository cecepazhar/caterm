<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { page } from '$app/state';
  import { getTabs } from '$lib/stores/sessionTabs.svelte';
  import { BottomNav as CauiBottomNav } from '@cecepazhar/caui';

  let { onOpenAll }: { onOpenAll: () => void } = $props();

  const hasActiveSessions = $derived(getTabs().length > 0);

  function isActive(href: string): boolean {
    if (href === '/') {
      return page.url.pathname === '/';
    }
    return page.url.pathname.startsWith(href);
  }

  const items = $derived([
    {
      id: 'hosts',
      label: t('nav.hosts'),
      icon: 'terminal',
      href: '/',
      active: isActive('/'),
    },
    {
      id: 'session',
      label: t('shell.session') || 'Session',
      icon: 'activity',
      href: '/session',
      active: isActive('/session'),
      badge: hasActiveSessions ? true : undefined,
    },
    {
      id: 'snippets',
      label: t('nav.snippets'),
      icon: 'code',
      href: '/snippets',
      active: isActive('/snippets'),
    },
    {
      id: 'sftp',
      label: t('nav.sftp'),
      icon: 'folder',
      href: '/sftp',
      active: isActive('/sftp'),
    },
    {
      id: 'all',
      label: t('shell.allMenus') || 'Semua',
      icon: 'layout-grid',
    },
  ]);

  function onItemClick(item: { label: string; href?: string; id?: string }) {
    if (item.id === 'all' || !item.href) {
      onOpenAll();
    }
  }
</script>

<CauiBottomNav
  {items}
  variant="primary"
  size="md"
  navLabel={t('shell.bottomNav') || 'Navigasi Bawah'}
  onItemSelect={onItemClick}
/>
