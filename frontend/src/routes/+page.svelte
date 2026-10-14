<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { listHosts, saveHost, deleteHost, testHostConnection, type HostRecord, type HostInput, type ConnectionProtocol } from '$lib/api/hosts';
  import { listGroups, type GroupRecord } from '$lib/api/groups';
  import { listKeys, type KeyRecord } from '$lib/api/keys';
  import { open } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import HostDetailPanel from '$lib/components/HostDetailPanel.svelte';
  import OsIcon from '$lib/components/OsIcon.svelte';
  import { openSession } from '$lib/nav';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { isFavorite, toggleFavorite, lastUsedAt } from '$lib/stores/hostPrefs.svelte';
  import { getTabs } from '$lib/stores/sessionTabs.svelte';
  import { monitorState } from '$lib/stores/monitorStore.svelte';
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import { PageHeader, Button, Input, Badge, Modal } from '@cecepazhar/caui';

  let hosts = $state<HostRecord[]>([]);
  let groups = $state<GroupRecord[]>([]);
  let vaultKeys = $state<KeyRecord[]>([]);
  let searchQuery = $state('');
  let isAddModalOpen = $state(false);
  let isLoading = $state(false);
  let errorMsg = $state('');
  let editingId = $state<string | null>(null);
  let editingHadSecret = $state(false);
  let formProtocol = $state<ConnectionProtocol>('ssh');

  // Bulk selection + detail slide-over. Both buttons on the host card were previously inert
  // markup with no onclick; this is the state behind them.
  let selectedIds = $state<string[]>([]);
  let detailHost = $state<HostRecord | null>(null);

  const isSelected = (id: string) => selectedIds.includes(id);

  function toggleSelected(id: string) {
    selectedIds = isSelected(id) ? selectedIds.filter((x) => x !== id) : [...selectedIds, id];
  }

  function clearSelection() {
    selectedIds = [];
  }

  /** Opens every selected host as tabs in one session, which is what a split view needs. */
  function connectSelected() {
    if (selectedIds.length === 0) return;
    const ids = [...selectedIds];
    clearSelection();
    void openSession(ids);
  }

  async function deleteSelected() {
    const count = selectedIds.length;
    if (count === 0) return;
    const ok = await confirmModal(
      t('hosts.deleteSelected', { count }),
      t('hosts.deleteSelectedTitle'),
      true,
      t('common.delete'),
      t('common.cancel')
    );
    if (!ok) return;

    const failed: string[] = [];
    for (const id of selectedIds) {
      try {
        await deleteHost(id);
      } catch {
        failed.push(hosts.find((h) => h.id === id)?.label ?? id);
      }
    }
    clearSelection();
    await refreshHosts();

    if (failed.length > 0) {
      showToast(t('hosts.deleteFailedList', { names: failed.join(', ') }), 'error');
    } else {
      showToast(t('hosts.hostDeleted'), 'success');
    }
  }

  // Form State
  let formLabel = $state('');
  let formAddress = $state('');
  let formPort = $state(22);
  let formUsername = $state('root');
  let formAuthType = $state<'password' | 'key' | 'keyId'>('password');
  let formKeyPath = $state('');
  let formKeyId = $state('');
  let formSecret = $state('');
  let showFormSecret = $state(false);
  let formTags = $state('');
  let formOs = $state('');
  let formTotpSecret = $state('');
  let testTotpResult = $state<{ token: string; remaining: number } | null>(null);
  let unlistenOsDetected: UnlistenFn | null = null;
  let handleHostsUpdate: (() => void) | null = null;

  onMount(async () => {
    await refreshHosts();
    handleHostsUpdate = () => {
      void refreshHosts();
    };
    window.addEventListener('caterm:hosts-updated', handleHostsUpdate);

    // Click-outside modal backdrop guard
    const handleBackdropClick = (e: PointerEvent) => {
      if (!isAddModalOpen) return;
      const target = e.target as Element;
      // If click target is directly the backdrop overlay outside the modal card
      const dialog = target.closest('[role="dialog"]');
      if (dialog && !target.closest('.rounded-2xl')) {
        e.preventDefault();
        e.stopPropagation();
        void handleRequestClose();
      }
    };
    window.addEventListener('pointerdown', handleBackdropClick, true);

    try {
      unlistenOsDetected = await listen<{ hostId: string; os: string }>('host:os_detected', (event) => {
        const { hostId, os } = event.payload;
        const targetIndex = hosts.findIndex((h) => h.id === hostId);
        if (targetIndex !== -1) {
          hosts[targetIndex] = { ...hosts[targetIndex], os };
        }
        if (detailHost && detailHost.id === hostId) {
          detailHost = { ...detailHost, os };
        }
      });
    } catch (e) {
      console.error('Failed to listen for host:os_detected', e);
    }

    return () => {
      window.removeEventListener('pointerdown', handleBackdropClick, true);
    };
  });

  onDestroy(() => {
    if (handleHostsUpdate) {
      window.removeEventListener('caterm:hosts-updated', handleHostsUpdate);
    }
    if (unlistenOsDetected) {
      unlistenOsDetected();
    }
  });

  async function refreshHosts() {
    try {
      hosts = await listHosts();
      vaultKeys = await listKeys();
      groups = await listGroups();
      groupCount = groups.length;
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  function getGroupsForHost(hostId: string): GroupRecord[] {
    return groups.filter((g) => Array.isArray(g.hostIds) && g.hostIds.includes(hostId));
  }

  // Test connection state
  let isTestingConnection = $state(false);
  let testResult = $state<{
    success: boolean;
    latencyMs: number;
    banner: string | null;
    message: string;
  } | null>(null);

  // Form dirty tracking for discard confirmation
  let initialSnapshot = $state({
    label: '',
    address: '',
    port: 22,
    username: 'root',
    protocol: 'ssh' as ConnectionProtocol,
    authType: 'password' as 'password' | 'key' | 'keyId',
    keyPath: '',
    keyId: '',
    secret: '',
    tags: '',
    os: '',
    totp: ''
  });

  function isFormDirty(): boolean {
    return (
      formLabel.trim() !== initialSnapshot.label ||
      formAddress.trim() !== initialSnapshot.address ||
      Number(formPort) !== initialSnapshot.port ||
      formUsername.trim() !== initialSnapshot.username ||
      formProtocol !== initialSnapshot.protocol ||
      formAuthType !== initialSnapshot.authType ||
      formKeyPath.trim() !== initialSnapshot.keyPath ||
      formKeyId !== initialSnapshot.keyId ||
      formSecret !== initialSnapshot.secret ||
      formTags.trim() !== initialSnapshot.tags ||
      formOs !== initialSnapshot.os ||
      formTotpSecret.trim() !== initialSnapshot.totp
    );
  }

  async function handleRequestClose() {
    if (isFormDirty()) {
      const leave = await confirmModal(
        'Perubahan yang Anda masukkan belum disimpan. Apakah Anda yakin ingin membatalkan dan keluar?',
        'Konfirmasi Batal (Discard Changes)',
        true,
        'Ya, Keluar',
        'Tetap Mengedit'
      );
      if (!leave) return;
    }
    isAddModalOpen = false;
    testResult = null;
  }

  async function runConnectionTest() {
    if (!formAddress.trim()) {
      showToast('Alamat Host / IP Address wajib diisi untuk menguji koneksi.', 'error');
      return;
    }
    isTestingConnection = true;
    testResult = null;
    try {
      const input: HostInput = {
        id: editingId ?? undefined,
        label: formLabel.trim() || formAddress.trim(),
        address: formAddress.trim(),
        port: Number(formPort) || 22,
        username: formUsername.trim() || 'root',
        protocol: formProtocol,
        authMethod: formAuthType === 'password'
          ? { type: 'password' }
          : (formAuthType === 'keyId' ? { type: 'keyId', id: formKeyId } : { type: 'key', path: formKeyPath.trim() }),
        tags: formTags.split(',').map(t => t.trim()).filter(Boolean),
        os: formOs ? formOs : undefined,
        secret: formSecret ? formSecret : undefined
      };
      testResult = await testHostConnection(input);
      if (testResult.success) {
        showToast(`Koneksi berhasil! (${testResult.latencyMs} ms)`, 'success');
      } else {
        showToast(testResult.message, 'error');
      }
    } catch (err: any) {
      testResult = {
        success: false,
        latencyMs: 0,
        banner: null,
        message: typeof err === 'string' ? err : (err?.message || 'Gagal terhubung ke host')
      };
      showToast(testResult.message, 'error');
    } finally {
      isTestingConnection = false;
    }
  }

  async function browseKeyFile() {
    try {
      const selected = await open({
        multiple: false,
        title: 'Pilih Private Key File (id_ed25519, id_rsa, .pem, .key)',
        filters: [
          { name: 'All Files', extensions: ['*'] },
          { name: 'SSH Private Keys', extensions: ['pem', 'key', 'id_rsa', 'id_ed25519', 'pkcs8'] }
        ]
      });
      if (selected && typeof selected === 'string') {
        formKeyPath = selected;
      }
    } catch (err) {
      console.warn('File dialog error/cancelled:', err);
    }
  }

  function openAddModal() {
    openMenuId = null;
    detailHost = null;
    editingId = null;
    editingHadSecret = false;
    formLabel = '';
    formAddress = '';
    formPort = 22;
    formUsername = 'root';
    formProtocol = 'ssh';
    formAuthType = 'password';
    formKeyPath = '';
    formKeyId = '';
    formSecret = '';
    formTags = '';
    formOs = '';
    formTotpSecret = '';
    testTotpResult = null;
    testResult = null;
    errorMsg = '';
    initialSnapshot = {
      label: '',
      address: '',
      port: 22,
      username: 'root',
      protocol: 'ssh',
      authType: 'password',
      keyPath: '',
      keyId: '',
      secret: '',
      tags: '',
      os: '',
      totp: ''
    };
    isAddModalOpen = true;
  }

  function openEditModal(host: HostRecord, e?: MouseEvent) {
    e?.stopPropagation();
    openMenuId = null;
    detailHost = null;
    editingId = host.id;
    editingHadSecret = host.hasSecret;
    formLabel = host.label;
    formAddress = host.address;
    formPort = host.port;
    formUsername = host.username;
    formProtocol = host.protocol || 'ssh';
    formAuthType = host.authMethod.type;
    formKeyPath = host.authMethod.type === 'key' ? host.authMethod.path : '';
    formKeyId = host.authMethod.type === 'keyId' ? host.authMethod.id : '';
    formSecret = '';
    formTags = host.tags.join(', ');
    formOs = host.os || '';
    formTotpSecret = '';
    testTotpResult = null;
    testResult = null;
    errorMsg = '';
    initialSnapshot = {
      label: host.label,
      address: host.address,
      port: host.port,
      username: host.username,
      protocol: host.protocol || 'ssh',
      authType: host.authMethod.type,
      keyPath: host.authMethod.type === 'key' ? host.authMethod.path : '',
      keyId: host.authMethod.type === 'keyId' ? host.authMethod.id : '',
      secret: '',
      tags: host.tags.join(', '),
      os: host.os || '',
      totp: ''
    };
    isAddModalOpen = true;
  }

  async function handleSaveHost() {
    if (!formLabel || !formAddress || !formUsername) {
      errorMsg = t('hosts.form.requiredFields');
      showToast(errorMsg, 'error');
      return;
    }
    isLoading = true;
    errorMsg = '';
    try {
      const input: HostInput = {
        id: editingId ?? undefined,
        label: formLabel,
        address: formAddress,
        port: Number(formPort) || 22,
        username: formUsername,
        protocol: formProtocol,
        authMethod: formAuthType === 'password' ? { type: 'password' } : (formAuthType === 'keyId' ? { type: 'keyId', id: formKeyId } : { type: 'key', path: formKeyPath }),
        tags: formTags.split(',').map(t => t.trim()).filter(Boolean),
        os: formOs ? formOs : undefined,
        // Blank = leave whatever's stored untouched (edit) or no secret at all (create).
        secret: formSecret ? formSecret : undefined
      };
      await saveHost(input);
      isAddModalOpen = false;
      showToast(editingId ? t('hosts.hostUpdated') : t('hosts.hostAdded'), 'success');
      await refreshHosts();
    } catch (err: any) {
      errorMsg = typeof err === 'string' ? err : (err?.message || t('hosts.saveFailed'));
      showToast(errorMsg, 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleDelete(id: string) {
    const confirmed = await confirmModal(t('hosts.deleteConfirm'), t('hosts.deleteTitle'), true, t('common.delete'), t('common.cancel'));
    if (confirmed) {
      try {
        await deleteHost(id);
        showToast(t('hosts.hostDeleted'), 'success');
        await refreshHosts();
      } catch (err: any) {
        showToast(err?.message || 'Failed to delete host', 'error');
      }
    }
  }

  async function handleClone(host: HostRecord) {
    try {
      const input: HostInput = {
        label: `${host.label} (Copy)`,
        address: host.address,
        port: host.port,
        username: host.username,
        protocol: host.protocol,
        authMethod: host.authMethod,
        tags: [...host.tags],
        os: host.os
      };
      await saveHost(input);
      showToast(t('hosts.hostCloned', { name: host.label }), 'success');
      await refreshHosts();
    } catch (err: any) {
      showToast(err?.message || 'Failed to clone host', 'error');
    }
  }

  // --- List: search, filters, sort ------------------------------------------------------------
  type SortKey = 'name' | 'lastUsed' | 'ip';
  let groupCount = $state(0);
  let sortBy = $state<SortKey>('name');
  let starredOnly = $state(false);
  let connectedOnly = $state(false);
  let filterOpen = $state(false);
  let openMenuId = $state<string | null>(null);
  let focusedIndex = $state(0);
  let searchInput: HTMLInputElement | undefined = $state();

  const sessionTabs = $derived(getTabs());

  function sessionCount(hostId: string): number {
    return sessionTabs.filter((t) => t.host.id === hostId).length;
  }

  function matchesSearch(h: HostRecord, q: string): boolean {
    if (!q) return true;
    return (
      h.label.toLowerCase().includes(q) ||
      h.address.toLowerCase().includes(q) ||
      h.username.toLowerCase().includes(q) ||
      h.tags.some((t) => t.toLowerCase().includes(q))
    );
  }

  const byName = (a: HostRecord, b: HostRecord) => a.label.localeCompare(b.label, undefined, { sensitivity: 'base' });
  const comparators: Record<SortKey, (a: HostRecord, b: HostRecord) => number> = {
    name: byName,
    lastUsed: (a, b) => (lastUsedAt(b.id) ?? 0) - (lastUsedAt(a.id) ?? 0) || byName(a, b),
    // `numeric` orders 10.0.0.9 before 10.0.0.10.
    ip: (a, b) => a.address.localeCompare(b.address, undefined, { numeric: true }) || byName(a, b)
  };

  // Starred hosts always lead, then the chosen order.
  let filteredHosts = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    return hosts
      .filter((h) => matchesSearch(h, q))
      .filter((h) => !starredOnly || isFavorite(h.id))
      .filter((h) => !connectedOnly || sessionCount(h.id) > 0)
      .sort((a, b) => Number(isFavorite(b.id)) - Number(isFavorite(a.id)) || comparators[sortBy](a, b));
  });

  const activeIndex = $derived(filteredHosts.length === 0 ? -1 : Math.min(focusedIndex, filteredHosts.length - 1));
  const filtersActive = $derived(starredOnly || connectedOnly);

  // --- Card status ------------------------------------------------------------------------------
  const OS_LABELS: Record<string, string> = {
    ubuntu: 'Ubuntu', debian: 'Debian', fedora: 'Fedora', redhat: 'Red Hat', centos: 'CentOS',
    rocky: 'Rocky Linux', almalinux: 'AlmaLinux', arch: 'Arch Linux', manjaro: 'Manjaro',
    alpine: 'Alpine Linux', opensuse: 'openSUSE', mint: 'Linux Mint', kali: 'Kali Linux',
    popos: 'Pop!_OS', raspberry: 'Raspberry Pi OS', amazon: 'Amazon Linux', oracle: 'Oracle Linux',
    freebsd: 'FreeBSD', openbsd: 'OpenBSD', netbsd: 'NetBSD', mikrotik: 'RouterOS', cisco: 'Cisco IOS',
    windows: 'Windows', macos: 'macOS', android: 'Android', linux: 'Linux'
  };

  /** Live metrics give the full distro string ("Debian GNU/Linux 12 (bookworm)") while connected. */
  function osName(host: HostRecord): string | null {
    const live = monitorState.metrics.find((m) => m.host_id === host.id)?.os_name;
    if (live) return live;
    if (!host.os) return null;
    return OS_LABELS[host.os] ?? host.os.charAt(0).toUpperCase() + host.os.slice(1);
  }

  function relativeTime(ts: number): string {
    const minutes = Math.floor((Date.now() - ts) / 60000);
    if (minutes < 1) return t('time.justNow');
    if (minutes < 60) return t('time.minutesAgo', { n: minutes });
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return t('time.hoursAgo', { n: hours });
    const days = Math.floor(hours / 24);
    return days < 30 ? t('time.daysAgo', { n: days }) : new Date(ts).toLocaleDateString(intlLocale());
  }

  function hostStatus(host: HostRecord): { live: boolean; text: string } {
    const sessions = sessionCount(host.id);
    if (sessions > 0) {
      const uptime = monitorState.metrics.find((m) => m.host_id === host.id)?.uptime;
      const label = t('hosts.sessionsCount', { count: sessions });
      return { live: true, text: uptime ? `${label} · up ${uptime}` : label };
    }
    const used = lastUsedAt(host.id);
    return { live: false, text: used ? t('hosts.usedAgo', { time: relativeTime(used) }) : t('hosts.neverConnected') };
  }

  let revealAddressMap = $state<Record<string, boolean>>({});

  function isRevealed(hostId: string): boolean {
    return !!revealAddressMap[hostId];
  }

  function toggleReveal(hostId: string, e: MouseEvent) {
    e.stopPropagation();
    revealAddressMap[hostId] = !revealAddressMap[hostId];
  }

  function formatMaskedAddress(host: HostRecord): string {
    if (isRevealed(host.id)) {
      return `${host.username}@${host.address}${host.port !== 22 ? `:${host.port}` : ''}`;
    }
    const u = 'xxxx';
    const parts = host.address.split('.');
    let maskedHost = 'xxx';
    if (parts.length === 4) {
      maskedHost = `${parts[0]}.xxx.xxx.${parts[3]}`;
    } else if (host.address.includes('.')) {
      maskedHost = `xxx.${host.address.split('.').slice(1).join('.')}`;
    }
    return `${u}@${maskedHost}`;
  }

  function hostAddress(host: HostRecord): string {
    return `${host.username}@${host.address}${host.port !== 22 ? `:${host.port}` : ''}`;
  }

  // --- Keyboard: ↑↓ select, ↵ connect, ⇧↵ details ------------------------------------------------
  function handleListKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isAddModalOpen) {
      e.preventDefault();
      void handleRequestClose();
      return;
    }
    if (isAddModalOpen || detailHost) return;
    const target = e.target instanceof Element ? e.target : null;
    const typing = target?.closest('input, textarea, select, [contenteditable="true"]');
    if (typing && target !== searchInput) return;
    if (filteredHosts.length === 0) return;

    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const step = e.key === 'ArrowDown' ? 1 : -1;
      focusedIndex = Math.max(0, Math.min(filteredHosts.length - 1, activeIndex + step));
      document.getElementById(`host-card-${filteredHosts[focusedIndex]?.id}`)?.scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'Enter' && activeIndex >= 0) {
      const host = filteredHosts[activeIndex];
      if (!host) return;
      e.preventDefault();
      if (e.shiftKey) detailHost = host;
      else void openSession(host.id);
    }
  }

  const kbdClass =
    'px-1.5 py-0.5 rounded border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-900 font-mono text-[10px] text-neutral-600 dark:text-neutral-300';
  const menuItemBase = 'w-full flex items-center px-3 py-1.5 text-left transition-colors';
  const menuItem = `${menuItemBase} text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800/60`;

  function handleOutsidePointer(e: PointerEvent) {
    const target = e.target instanceof Element ? e.target : null;
    if (openMenuId && !target?.closest('[data-host-menu]')) openMenuId = null;
    if (filterOpen && !target?.closest('[data-filter-menu]')) filterOpen = false;
  }
</script>

<svelte:window onkeydown={handleListKeydown} onpointerdown={handleOutsidePointer} />

<div class="max-w-5xl mx-auto space-y-6">
	<PageHeader
		icon={['M4 17l6-5-6-5', 'M12 19h8']}
		accent="sky"
		title={t('hosts.title')}
		subtitle={t('hosts.subtitle', { count: hosts.length, groups: groupCount })}
	>
		{#snippet actions()}
			<Button
				variant="secondary"
				size="icon"
				class="h-9 w-9 rounded-xl shrink-0"
				aria-label={t('hosts.localTerminal')}
				title={t('hosts.localTerminal')}
				onclick={() => openSession('local')}
			>
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="12" rx="2" stroke-width="1.8" /><path stroke-linecap="round" stroke-width="1.8" d="M8 20h8M12 16v4" /></svg>
			</Button>
			<Button
				variant="primary"
				class="h-9 rounded-xl shrink-0 shadow-md shadow-sky-600/20"
				onclick={openAddModal}
			>
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v14m7-7H5" /></svg>
				{t('hosts.addHost')}
			</Button>
		{/snippet}
	</PageHeader>

	<!-- Search and filter -->
	<div class="flex items-center gap-2">
    <div class="flex-1">
      <Input
        bind:ref={searchInput}
        bind:value={searchQuery}
        oninput={() => (focusedIndex = 0)}
        placeholder={t('hosts.searchPlaceholder')}
        ariaLabel={t('hosts.searchPlaceholder')}
        size="lg"
      >
        {#snippet leadingIcon()}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>
        {/snippet}
      </Input>
    </div>

    <div class="relative" data-filter-menu>
      <Button
        variant={filterOpen || filtersActive ? 'secondary' : 'outline'}
        size="icon"
        onclick={() => (filterOpen = !filterOpen)}
        class="relative h-11 w-11 shrink-0"
        title={t('hosts.filterHosts')}
        aria-label={t('hosts.filterHosts')}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 5h18l-7 8.5V19l-4 2v-7.5z" /></svg>
        {#if filtersActive}
          <span class="absolute top-2 right-2 w-1.5 h-1.5 rounded-full bg-sky-500"></span>
        {/if}
      </Button>
      {#if filterOpen}
        <div class="absolute right-0 top-full mt-2 z-30 w-52 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl p-1.5 text-sm">
          <p class="px-2.5 pt-1.5 pb-1 text-[10px] font-semibold uppercase tracking-wider text-neutral-400">{t('hosts.show')}</p>
          <label class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-200 cursor-pointer">
            <input type="checkbox" bind:checked={starredOnly} class="rounded" />
            {t('hosts.starredOnly')}
          </label>
          <label class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-200 cursor-pointer">
            <input type="checkbox" bind:checked={connectedOnly} class="rounded" />
            {t('hosts.connectedOnly')}
          </label>
        </div>
      {/if}
    </div>
  </div>

  <!-- Keyboard hints + sort -->
  <div class="mt-3 flex items-center gap-4 text-[11px] text-neutral-500 dark:text-neutral-400">
    <!-- Keyboard hints are noise on touch-only phones. -->
    <span class="hidden md:flex items-center gap-1.5"><kbd class={kbdClass}>↑↓</kbd> {t('hosts.selectHint')}</span>
    <span class="hidden md:flex items-center gap-1.5"><kbd class={kbdClass}>↵</kbd> {t('hosts.connectHint')}</span>
    <span class="hidden md:flex items-center gap-1.5"><kbd class={kbdClass}>⇧↵</kbd> {t('hosts.detailsHint')}</span>
    <label class="ml-auto flex items-center gap-1.5">
      {t('hosts.sortBy')}
      <select
        bind:value={sortBy}
        class="bg-transparent text-neutral-700 dark:text-neutral-200 font-medium focus:outline-none cursor-pointer"
      >
        <option value="name">{t('hosts.sortName')}</option>
        <option value="lastUsed">{t('hosts.sortLastUsed')}</option>
        <option value="ip">{t('hosts.sortIp')}</option>
      </select>
    </label>
  </div>

  <!-- Bulk actions: only present once at least one host is ticked, so the page stays quiet
       when you are not doing a multi-host operation. -->
  {#if selectedIds.length > 0}
    <div class="mt-4 flex flex-wrap items-center gap-3 p-3 rounded-lg bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-900">
      <span class="text-sm font-medium text-sky-800 dark:text-sky-300">
        {t('hosts.hostsSelected', { count: selectedIds.length })}
      </span>
      <div class="flex items-center gap-2 ml-auto">
        <Button
          variant="primary"
          size="sm"
          onclick={connectSelected}
        >
          {t('hosts.connectAll')}
        </Button>
        <Button
          variant="danger"
          size="sm"
          onclick={deleteSelected}
        >
          {t('common.delete')}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={clearSelection}
        >
          {t('hosts.deselect')}
        </Button>
      </div>
    </div>
  {/if}

  <!-- Host cards -->
  {#if hosts.length === 0}
    <div class="mt-6 p-12 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl text-center flex flex-col items-center justify-center">
      <div class="w-12 h-12 rounded-full bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center text-neutral-400 dark:text-neutral-500 mb-4">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 17l6-5-6-5M12 19h8" /></svg>
      </div>
      <h3 class="text-lg font-medium text-neutral-900 dark:text-white mb-1">{t('hosts.emptyTitleNoHosts')}</h3>
      <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-sm mb-6">{t('hosts.emptyBodyNoHosts')}</p>
      <Button
        variant="primary"
        size="md"
        onclick={openAddModal}
      >
        + {t('hosts.addHost')}
      </Button>
    </div>
  {:else if filteredHosts.length === 0}
    <div class="mt-6 p-10 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl text-center">
      <p class="text-sm text-neutral-500 dark:text-neutral-400">{t('hosts.emptyTitleNoMatch')}</p>
      <Button
        variant="ghost"
        size="sm"
        class="mt-3 text-sky-600 dark:text-sky-400"
        onclick={() => { searchQuery = ''; starredOnly = false; connectedOnly = false; }}
      >
        {t('hosts.clearFilters')}
      </Button>
    </div>
  {:else}
    <div class="mt-4 grid grid-cols-1 lg:grid-cols-2 gap-3" role="listbox" aria-label={t('hosts.title')}>
      {#each filteredHosts as host, i (host.id)}
        {@const status = hostStatus(host)}
        {@const os = osName(host)}
        {@const focused = i === activeIndex}
        <div
          id="host-card-{host.id}"
          role="option"
          aria-selected={focused}
          tabindex="-1"
          onpointerdown={() => (focusedIndex = i)}
          ondblclick={(e) => { if (!(e.target as HTMLElement).closest('button, a')) void openSession(host.id); }}
          class="group rounded-xl border p-4 transition-colors bg-white dark:bg-neutral-900/40 {isSelected(host.id) ? 'border-sky-500 ring-1 ring-sky-500/40' : focused ? 'border-neutral-900 dark:border-neutral-200' : 'border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700'}"
        >
          <!-- Identity -->
          <div class="flex items-start gap-3">
            <span class="w-10 h-10 rounded-lg bg-neutral-100 dark:bg-white flex items-center justify-center shrink-0">
              <OsIcon os={host.os} name={host.label} tags={host.tags} address={host.address} size={22} />
            </span>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <p class="font-semibold text-neutral-900 dark:text-white truncate">{host.label}</p>
                {#each getGroupsForHost(host.id) as grp}
                  <span
                    class="inline-flex items-center gap-1 px-1.5 py-0.2 rounded text-[10px] font-medium shrink-0"
                    style="background-color: {grp.color ? grp.color + '20' : 'rgba(2, 132, 199, 0.15)'}; color: {grp.color || '#0284c7'}"
                  >
                    <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background-color: {grp.color || '#0284c7'}"></span>
                    <span>{grp.name}</span>
                  </span>
                {/each}
              </div>
              <div class="flex items-center gap-1.5 min-w-0 mt-0.5">
                <p class="text-xs font-mono text-neutral-500 dark:text-neutral-400 truncate" title="{host.username}@{host.address}:{host.port}">
                  {formatMaskedAddress(host)}
                </p>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onclick={(e) => toggleReveal(host.id, e)}
                  class="h-5 w-5 p-0.5 rounded text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 shrink-0"
                  title={isRevealed(host.id) ? "Hide User & IP" : "Show User & IP"}
                  aria-label={isRevealed(host.id) ? "Hide User & IP" : "Show User & IP"}
                >
                  {#if isRevealed(host.id)}
                    <!-- Eye Open Icon -->
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                  {:else}
                    <!-- Eye Closed / Slashed Icon -->
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l18 18" />
                    </svg>
                  {/if}
                </Button>
              </div>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onclick={() => toggleFavorite(host.id)}
              class="h-7 w-7 p-1.5 -mr-1 -mt-1 rounded-md transition-colors {isFavorite(host.id) ? 'text-amber-500' : 'text-neutral-300 dark:text-neutral-600 hover:text-amber-500'}"
              title={isFavorite(host.id) ? t('hosts.unstar') : t('hosts.star')}
              aria-label={isFavorite(host.id) ? `${t('hosts.unstar')} ${host.label}` : `${t('hosts.star')} ${host.label}`}
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill={isFavorite(host.id) ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"><path d="M12 3.5l2.6 5.3 5.9.9-4.25 4.1 1 5.8L12 16.9l-5.25 2.7 1-5.8L3.5 9.7l5.9-.9z" /></svg>
            </Button>
          </div>

          <!-- OS + status + tags -->
          <div class="mt-3 flex flex-wrap items-center gap-1.5 text-[11px]">
            {#if os}
              <Badge variant="outline" size="xs" class="max-w-[16rem] px-2 py-0.5 text-[11px] gap-1.5" ariaLabel={os}>
                <OsIcon os={host.os} name={host.label} tags={host.tags} address={host.address} size={12} />
                <span class="truncate">{os}</span>
              </Badge>
            {/if}
            <Badge variant={status.live ? 'success' : 'neutral'} size="xs" class="px-2 py-0.5 text-[11px] gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full {status.live ? 'bg-emerald-500 animate-pulse' : 'bg-neutral-300 dark:bg-neutral-600'}"></span>
              {status.text}
            </Badge>
            {#each host.tags.slice(0, 3) as tag}
              <Badge variant="neutral" size="xs" class="px-1.5 py-0.5 text-[11px]">#{tag}</Badge>
            {/each}
          </div>

          <!-- Actions -->
          <div class="mt-3 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center gap-1.5">
            <span class="text-[11px] text-neutral-400 dark:text-neutral-500 truncate">
              {host.authMethod.type === 'password' ? t('hosts.authPassword') : t('hosts.authKey')} · {host.hasSecret ? t('hosts.passwordSaved') : host.authMethod.type === 'password' ? t('hosts.passwordNotSaved') : t('hosts.noPassphrase')}
            </span>

            <Button
              variant="ghost"
              size="icon"
              class="ml-auto h-8 w-8 rounded-lg"
              onclick={(e) => openEditModal(host, e)}
              title={t('hosts.editHost')}
              aria-label={`${t('common.edit')} ${host.label}`}
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M16.9 4.1a2.1 2.1 0 013 3L8.5 18.5 4 20l1.5-4.5z" /></svg>
            </Button>

            <div class="relative" data-host-menu>
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg"
                onclick={() => (openMenuId = openMenuId === host.id ? null : host.id)}
                title={t('hosts.moreActions')}
                aria-label={`${t('hosts.moreActions')} ${host.label}`}
              >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.6" /><circle cx="12" cy="12" r="1.6" /><circle cx="19" cy="12" r="1.6" /></svg>
              </Button>
              {#if openMenuId === host.id}
                <div role="menu" class="absolute right-0 top-full mt-1 z-30 w-48 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl py-1 text-sm">
                  <a role="menuitem" href="/security?host={host.id}" class={menuItem}>Security Audit</a>
                  <a role="menuitem" href="/devops?host={host.id}" class={menuItem}>DevOps Lab</a>
                  <a role="menuitem" href="/sftp?host={host.id}" class={menuItem}>{t('hosts.filesSftp')}</a>
                  <Button variant="ghost" size="sm" class="w-full !justify-start text-left rounded-none px-3 py-1.5 font-normal" onclick={() => { openMenuId = null; detailHost = host; }}>{t('common.details')}</Button>
                  <Button variant="ghost" size="sm" class="w-full !justify-start text-left rounded-none px-3 py-1.5 font-normal" onclick={() => { openMenuId = null; handleClone(host); }}>{t('common.clone')}</Button>
                  <Button variant="ghost" size="sm" class="w-full !justify-start text-left rounded-none px-3 py-1.5 font-normal" onclick={() => { openMenuId = null; toggleSelected(host.id); }}>
                    {isSelected(host.id) ? t('hosts.deselect') : t('hosts.selectForBatch')}
                  </Button>
                  <div class="my-1 border-t border-neutral-100 dark:border-neutral-800"></div>
                  <Button variant="danger" size="sm" class="w-full !justify-start text-left rounded-none px-3 py-1.5 font-normal border-0" onclick={() => { openMenuId = null; handleDelete(host.id); }}>{t('common.delete')}</Button>
                </div>
              {/if}
            </div>

            <a
              href="/security?host={host.id}"
              class="h-8 px-2 flex items-center gap-1 rounded-lg border border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-300 text-xs font-medium transition-colors"
              title="Security Exposure & Threat Audit"
            >
              <svg class="w-3.5 h-3.5 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
              <span>Security</span>
            </a>

            <a
              href="/devops?host={host.id}"
              class="h-8 px-2 flex items-center gap-1 rounded-lg border border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-300 text-xs font-medium transition-colors"
              title="DevOps Lab, Bench & QoS"
            >
              <svg class="w-3.5 h-3.5 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>
              <span>Lab</span>
            </a>

            <Button
              variant="primary"
              size="sm"
              class="h-8 px-3 rounded-lg font-semibold"
              onclick={() => openSession(host.id)}
              title={t('hosts.connectTo', { name: host.label })}
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 7l5 5-5 5M13 17h6" /></svg>
              {t('common.connect')}
            </Button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if detailHost}
  <HostDetailPanel host={detailHost} onClose={() => (detailHost = null)} />
{/if}

<Modal
  bind:open={isAddModalOpen}
  size="xl"
  closeOnBackdrop={false}
  closeOnEsc={false}
  showCloseButton={false}
>
  {#snippet header()}
    <div class="px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between gap-3 bg-neutral-50/70 dark:bg-neutral-900/50">
      <div class="flex items-center gap-3 min-w-0">
        <div class="w-10 h-10 rounded-xl bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-600 dark:text-sky-400 shrink-0 shadow-xs">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
          </svg>
        </div>
        <div class="min-w-0">
          <h3 class="text-base font-bold text-neutral-900 dark:text-white truncate">
            {editingId ? 'Edit SSH Host' : 'Tambah Host SSH Baru'}
          </h3>
          <p class="text-xs text-neutral-500 dark:text-neutral-400">
            {editingId ? `Konfigurasi kredensial & koneksi untuk ${formLabel || 'server'}` : 'Hubungkan server Linux / VPS baru ke CATerm'}
          </p>
        </div>
      </div>
      <button
        type="button"
        onclick={handleRequestClose}
        class="p-2 rounded-lg text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50 transition-colors cursor-pointer"
        aria-label="Tutup"
        title="Tutup Modal"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
      </button>
    </div>
  {/snippet}

  {#if errorMsg}
    <div class="p-3 mb-4 text-xs rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400 flex items-center gap-2">
      <svg class="w-4 h-4 shrink-0 text-rose-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg>
      <span>{errorMsg}</span>
    </div>
  {/if}

  <div class="space-y-6 text-sm">
    <!-- SECTION 1: Detail Server & Jaringan -->
    <div class="p-5 rounded-2xl bg-neutral-100/60 dark:bg-neutral-900/40 border border-neutral-200/80 dark:border-neutral-800/80 space-y-4">
      <div class="flex items-center justify-between pb-2 border-b border-neutral-200/60 dark:border-neutral-800/60">
        <div class="flex items-center gap-2 text-xs font-bold text-neutral-800 dark:text-neutral-200 uppercase tracking-wider">
          <svg class="w-4 h-4 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" /></svg>
          <span>Detail Server & Jaringan</span>
        </div>
        <span class="text-[11px] text-neutral-400">Parameter Koneksi Utama</span>
      </div>

      <div>
        <label for="form-label" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
          {t('hosts.form.labelField')} <span class="text-rose-500">*</span>
        </label>
        <Input 
          id="form-label"
          bind:value={formLabel} 
          placeholder="Misal: Production Web Server / ThinkPad X1"
          class="h-10 text-sm"
        />
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-4 gap-3">
        <div>
          <label for="form-protocol" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
            {t('hosts.form.protocol')}
          </label>
          <select
            id="form-protocol"
            bind:value={formProtocol}
            onchange={(e) => {
              const proto = (e.currentTarget as HTMLSelectElement).value;
              if (proto === 'ftp' && formPort === 22) formPort = 21;
              else if (proto === 'ssh' && formPort === 21) formPort = 22;
              else if ((proto === 'webdav' || proto === 's3') && (formPort === 22 || formPort === 21)) formPort = 443;
            }}
            class="w-full h-10 px-3 bg-white dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-xs text-xs font-medium cursor-pointer"
          >
            <option value="ssh">SSH / SFTP</option>
            <option value="ftp">FTP</option>
            <option value="ftps">FTPS</option>
            <option value="webdav">WebDAV</option>
            <option value="s3">S3</option>
          </select>
        </div>

        <div class="sm:col-span-2">
          <label for="form-address" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
            {t('hosts.form.addressField')} <span class="text-rose-500">*</span>
          </label>
          <Input 
            id="form-address"
            bind:value={formAddress} 
            placeholder="192.168.1.100 atau ssh.example.com"
            class="h-10 text-sm font-mono"
          />
        </div>

        <div>
          <label for="form-port" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
            {t('hosts.form.port')} <span class="text-rose-500">*</span>
          </label>
          <input 
            id="form-port"
            type="number" 
            bind:value={formPort} 
            placeholder="22"
            class="w-full h-10 px-3 bg-white dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-xs text-sm font-mono"
          />
        </div>
      </div>

      <div>
        <label for="form-username" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
          {t('hosts.form.usernameField')} <span class="text-rose-500">*</span>
        </label>
        <Input 
          id="form-username"
          bind:value={formUsername} 
          placeholder="root / ubuntu / debian"
          class="h-10 text-sm font-mono"
        />
      </div>
    </div>

    <!-- SECTION 2: Metode Autentikasi -->
    <div class="p-5 rounded-2xl bg-neutral-100/60 dark:bg-neutral-900/40 border border-neutral-200/80 dark:border-neutral-800/80 space-y-4">
      <div class="flex items-center justify-between pb-2 border-b border-neutral-200/60 dark:border-neutral-800/60">
        <div class="flex items-center gap-2 text-xs font-bold text-neutral-800 dark:text-neutral-200 uppercase tracking-wider">
          <svg class="w-4 h-4 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg>
          <span>Metode Autentikasi & SSH Key</span>
        </div>
        <span class="inline-flex items-center gap-1 text-[11px] font-mono text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">
          AES-256-GCM Zero-Knowledge
        </span>
      </div>

      <!-- Segmented Radio Cards -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5">
        <label class="flex items-center gap-3 p-3 rounded-xl border transition-all cursor-pointer {formAuthType === 'password' ? 'bg-sky-500/10 border-sky-500/40 text-sky-800 dark:text-sky-300 shadow-xs' : 'bg-white dark:bg-neutral-950 border-neutral-200 dark:border-neutral-800 text-neutral-700 dark:text-neutral-300 hover:border-neutral-300 dark:hover:border-neutral-700'}">
          <input type="radio" value="password" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
          <div class="min-w-0">
            <div class="text-xs font-bold">Password</div>
            <div class="text-[10px] text-neutral-500 dark:text-neutral-400">Kata sandi login</div>
          </div>
        </label>

        <label class="flex items-center gap-3 p-3 rounded-xl border transition-all cursor-pointer {formAuthType === 'keyId' ? 'bg-sky-500/10 border-sky-500/40 text-sky-800 dark:text-sky-300 shadow-xs' : 'bg-white dark:bg-neutral-950 border-neutral-200 dark:border-neutral-800 text-neutral-700 dark:text-neutral-300 hover:border-neutral-300 dark:hover:border-neutral-700'}">
          <input type="radio" value="keyId" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
          <div class="min-w-0">
            <div class="text-xs font-bold flex items-center gap-1.5">
              <span>SSH Key (Vault)</span>
              <span class="px-1.5 py-0.2 rounded-full text-[9px] bg-sky-500/20 text-sky-700 dark:text-sky-300 font-mono">{vaultKeys.length}</span>
            </div>
            <div class="text-[10px] text-neutral-500 dark:text-neutral-400">Tersimpan di Vault</div>
          </div>
        </label>

        <label class="flex items-center gap-3 p-3 rounded-xl border transition-all cursor-pointer {formAuthType === 'key' ? 'bg-sky-500/10 border-sky-500/40 text-sky-800 dark:text-sky-300 shadow-xs' : 'bg-white dark:bg-neutral-950 border-neutral-200 dark:border-neutral-800 text-neutral-700 dark:text-neutral-300 hover:border-neutral-300 dark:hover:border-neutral-700'}">
          <input type="radio" value="key" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
          <div class="min-w-0">
            <div class="text-xs font-bold">Local Key File</div>
            <div class="text-[10px] text-neutral-500 dark:text-neutral-400">File ~/.ssh di laptop</div>
          </div>
        </label>
      </div>

      <!-- Detail Per-Auth Method -->
      {#if formAuthType === 'keyId'}
        <div class="space-y-3 pt-1">
          <div class="flex items-center justify-between">
            <label for="form-key-id" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide">
              Pilih SSH Key dari Vault
            </label>
            <a 
              href="/ssh-keys" 
              class="text-xs font-semibold text-sky-600 dark:text-sky-400 hover:underline flex items-center gap-1"
              title="Buka Manajemen SSH Keys"
            >
              <span>🔑 Kelola SSH Keys ({vaultKeys.length})</span>
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
            </a>
          </div>

          {#if vaultKeys.length === 0}
            <div class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 text-xs text-amber-700 dark:text-amber-300 flex items-center justify-between gap-3">
              <span>Belum ada SSH Key yang tersimpan di Vault CATerm.</span>
              <a 
                href="/ssh-keys" 
                class="px-3 py-1.5 bg-amber-600 hover:bg-amber-500 text-white rounded-lg font-semibold text-xs transition-colors shrink-0"
              >
                + Buat / Impor Key Sekarang
              </a>
            </div>
          {:else}
            <div class="relative">
              <select
                id="form-key-id"
                bind:value={formKeyId}
                class="w-full h-10 px-3 bg-white dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-xs text-xs font-mono cursor-pointer"
              >
                <option value="" disabled>Pilih SSH Key yang tersedia...</option>
                {#each vaultKeys as k}
                  <option value={k.id}>{k.name} [{k.algorithm}]</option>
                {/each}
              </select>
            </div>
          {/if}
        </div>
      {:else if formAuthType === 'key'}
        <div class="space-y-3 pt-1">
          <label for="form-key-path" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide">
            Path File Private Key di Komputer
          </label>
          <div class="flex gap-2">
            <Input
              id="form-key-path"
              bind:value={formKeyPath}
              placeholder="/home/username/.ssh/id_ed25519"
              class="h-10 text-xs font-mono flex-1"
            />
            <Button
              type="button"
              variant="secondary"
              onclick={browseKeyFile}
              class="h-10 px-3.5 text-xs font-semibold shrink-0"
            >
              📁 Browse...
            </Button>
          </div>

          <!-- Quick pick common key paths -->
          <div class="flex flex-wrap items-center gap-2 pt-1 text-[11px] text-neutral-500">
            <span>Contoh path:</span>
            <button
              type="button"
              onclick={() => (formKeyPath = '~/.ssh/id_ed25519')}
              class="px-2 py-0.5 rounded-md bg-neutral-200/70 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:text-sky-500 font-mono transition-colors"
            >
              ~/.ssh/id_ed25519
            </button>
            <button
              type="button"
              onclick={() => (formKeyPath = '~/.ssh/id_rsa')}
              class="px-2 py-0.5 rounded-md bg-neutral-200/70 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:text-sky-500 font-mono transition-colors"
            >
              ~/.ssh/id_rsa
            </button>
          </div>
        </div>
      {/if}

      {#if formAuthType !== 'keyId'}
        <div class="pt-2">
          <label for="form-secret" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
            {formAuthType === 'key' ? 'Passphrase Key (Opsional)' : 'Password Server'}
          </label>
          <Input
            id="form-secret"
            type={showFormSecret ? 'text' : 'password'}
            bind:value={formSecret}
            placeholder={editingId && editingHadSecret ? t('hosts.form.leaveBlankUnchanged') : (formAuthType === 'key' ? t('hosts.form.leaveBlankNoPassphrase') : t('hosts.form.requiredToConnect'))}
            class="h-10 text-sm font-mono"
          >
            {#snippet trailingAction()}
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onclick={() => (showFormSecret = !showFormSecret)}
                aria-label={showFormSecret ? t('hosts.form.hideSecret') : t('hosts.form.showSecret')}
                title={showFormSecret ? t('hosts.form.hideSecret') : t('hosts.form.showSecret')}
                class="h-7 w-7 p-0 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200"
              >
                {#if showFormSecret}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                {/if}
              </Button>
            {/snippet}
          </Input>
        </div>
      {/if}
    </div>

    <!-- SECTION 3: Metadata Distro & Keamanan Tambahan -->
    <div class="p-5 rounded-2xl bg-neutral-100/60 dark:bg-neutral-900/40 border border-neutral-200/80 dark:border-neutral-800/80 space-y-4">
      <div class="flex items-center justify-between pb-2 border-b border-neutral-200/60 dark:border-neutral-800/60">
        <div class="flex items-center gap-2 text-xs font-bold text-neutral-800 dark:text-neutral-200 uppercase tracking-wider">
          <svg class="w-4 h-4 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" /></svg>
          <span>Metadata Distro & 2FA</span>
        </div>
        <span class="text-[11px] text-neutral-400">Visual Icon & Keamanan</span>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- OS Selection with Big 32px Vector Icon -->
        <div>
          <label for="form-os" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
            {t('hosts.form.osField')}
          </label>
          <div class="flex items-center gap-3 p-1.5 rounded-xl border border-neutral-300 dark:border-neutral-800 bg-white dark:bg-neutral-950">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center shrink-0 bg-neutral-100 dark:bg-neutral-900 shadow-xs">
              <OsIcon os={formOs} name={formLabel} tags={formTags.split(',')} address={formAddress} size={32} />
            </div>
            <select
              id="form-os"
              bind:value={formOs}
              class="w-full bg-transparent text-neutral-900 dark:text-white focus:outline-none text-xs font-medium cursor-pointer"
            >
              <option value="">{t('hosts.form.osAutoDetect')}</option>
              <optgroup label={t('hosts.form.osPopularGroup')}>
                <option value="ubuntu">Ubuntu</option>
                <option value="debian">Debian</option>
                <option value="fedora">Fedora</option>
                <option value="redhat">Red Hat / RHEL</option>
                <option value="centos">CentOS</option>
                <option value="rocky">Rocky Linux</option>
                <option value="almalinux">AlmaLinux</option>
                <option value="arch">Arch Linux</option>
                <option value="manjaro">Manjaro</option>
                <option value="alpine">Alpine Linux</option>
                <option value="opensuse">openSUSE / SUSE</option>
                <option value="mint">Linux Mint</option>
                <option value="kali">Kali Linux</option>
                <option value="popos">Pop!_OS</option>
                <option value="gentoo">Gentoo</option>
                <option value="void">Void Linux</option>
                <option value="nixos">NixOS</option>
                <option value="endeavour">EndeavourOS</option>
                <option value="elementary">Elementary OS</option>
                <option value="zorin">Zorin OS</option>
                <option value="raspberry">Raspberry Pi OS</option>
                <option value="amazon">Amazon Linux</option>
                <option value="oracle">Oracle Linux</option>
                <option value="slackware">Slackware</option>
                <option value="mageia">Mageia</option>
                <option value="solus">Solus</option>
                <option value="tails">Tails</option>
                <option value="deepin">Deepin</option>
                <option value="clear">Clear Linux</option>
                <option value="garuda">Garuda Linux</option>
                <option value="steam">SteamOS</option>
                <option value="coreos">CoreOS / Flatcar</option>
                <option value="devuan">Devuan</option>
                <option value="parrot">Parrot OS</option>
                <option value="mx">MX Linux</option>
                <option value="lubuntu">Lubuntu / Xubuntu / Kubuntu</option>
                <option value="linux">{t('hosts.osGenericLinux')}</option>
              </optgroup>
              <optgroup label={t('hosts.form.osOtherGroup')}>
                <option value="windows">Windows</option>
                <option value="macos">macOS / Apple</option>
                <option value="android">Android</option>
                <option value="ios">iOS</option>
                <option value="freebsd">FreeBSD</option>
                <option value="openbsd">OpenBSD</option>
                <option value="netbsd">NetBSD</option>
                <option value="mikrotik">MikroTik / RouterOS</option>
                <option value="cisco">Cisco IOS</option>
                <option value="server">{t('hosts.osGenericServer')}</option>
              </optgroup>
            </select>
          </div>
        </div>

        <!-- Tags Input -->
        <div>
          <label for="form-tags" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide mb-1.5">
            {t('hosts.form.tagsField')}
          </label>
          <Input 
            id="form-tags"
            bind:value={formTags} 
            placeholder="misal: web, prod, vps, x1"
            class="h-10 text-xs"
          />
        </div>
      </div>

      <!-- 2FA / TOTP Authenticator Secret -->
      <div class="space-y-1.5 pt-1">
        <label for="form-totp" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wide">
          2FA / TOTP Secret Key (Opsional)
        </label>
        <Input 
          id="form-totp"
          type="password" 
          bind:value={formTotpSecret} 
          placeholder="Base32 Key (misal: JBSWY3DPEHPK3PXP) atau otpauth:// URL"
          class="h-10 text-xs font-mono"
        />
        <p class="text-[11px] text-neutral-500 dark:text-neutral-400">
          Digunakan untuk auto-inject kode 6-digit saat server meminta verifikasi SSH 2FA / OTP.
        </p>
      </div>
    </div>

    <!-- CONNECTION TEST RESULT BANNER -->
    {#if testResult}
      <div class="p-4 rounded-2xl animate-in fade-in zoom-in-95 duration-150 {testResult.success ? 'bg-emerald-500/10 border border-emerald-500/30 text-emerald-800 dark:text-emerald-300' : 'bg-rose-500/10 border border-rose-500/30 text-rose-800 dark:text-rose-300'}">
        <div class="flex items-start justify-between gap-3">
          <div class="flex items-start gap-2.5 min-w-0">
            {#if testResult.success}
              <svg class="w-5 h-5 text-emerald-500 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            {:else}
              <svg class="w-5 h-5 text-rose-500 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            {/if}
            <div class="space-y-1">
              <div class="text-xs font-bold flex items-center gap-2">
                <span>{testResult.success ? 'Uji Koneksi Berhasil!' : 'Uji Koneksi Gagal!'}</span>
                {#if testResult.latencyMs > 0}
                  <span class="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold {testResult.success ? 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-400' : 'bg-rose-500/20 text-rose-600 dark:text-rose-400'}">
                    ⚡ {testResult.latencyMs} ms
                  </span>
                {/if}
              </div>
              <p class="text-xs leading-relaxed opacity-90">{testResult.message}</p>
            </div>
          </div>
          <button 
            type="button" 
            onclick={() => (testResult = null)} 
            class="p-1 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 transition-colors"
            title="Tutup Hasil Uji"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
          </button>
        </div>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <div class="w-full flex items-center justify-between gap-3">
      <Button 
        type="button"
        variant="secondary"
        onclick={runConnectionTest}
        disabled={isTestingConnection || isLoading}
        loading={isTestingConnection}
        class="h-9 px-3.5 text-xs font-semibold gap-1.5 border-neutral-300 dark:border-neutral-700 hover:border-sky-500 transition-colors"
      >
        <svg class="w-4 h-4 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
        <span>{isTestingConnection ? 'Menguji...' : 'Uji Koneksi (Test)'}</span>
      </Button>

      <div class="flex items-center gap-2">
        <Button 
          type="button"
          variant="secondary"
          onclick={handleRequestClose}
          disabled={isLoading}
          class="h-9 px-4 text-xs font-semibold"
        >
          {t('common.cancel')}
        </Button>
        <Button 
          type="button"
          variant="primary"
          onclick={handleSaveHost}
          disabled={isLoading}
          loading={isLoading}
          class="h-9 px-5 text-xs font-bold shadow-sm"
        >
          {isLoading ? t('common.saving') : t('common.save')}
        </Button>
      </div>
    </div>
  {/snippet}
</Modal>
