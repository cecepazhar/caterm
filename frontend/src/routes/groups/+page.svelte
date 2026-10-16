<script lang="ts">
  import { onMount } from 'svelte';
  import { openSession } from '$lib/nav';
  import { listGroups, saveGroup, deleteGroup, type GroupRecord } from '$lib/api/groups';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { listSnippets, type SnippetRecord } from '$lib/api/snippets';
  import { listScheduledTasks, type ScheduledTaskRecord } from '$lib/api/tasks';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n/index.svelte';
  import { PageHeader, Button, Input, Badge, Modal, Pagination } from '@cecepazhar/caui';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { copyText } from '$lib/utils/clipboard';

  const ALL_CATEGORIES = [
    { id: 'hosts', label: 'Hosts & Servers', path: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z' },
    { id: 'snippets', label: 'Command Snippets', path: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8zM14 2v6h6M16 13H8M16 17H8M10 9H8' },
    { id: 'tasks', label: 'Scheduled Tasks', path: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
    { id: 'tunnels', label: 'Port Forwarding', path: 'M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4' },
    { id: 'keys', label: 'SSH Keys', path: 'M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z' },
    { id: 'totp', label: '2FA Authenticator', path: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z' },
    { id: 'investigations', label: 'Investigations', path: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z' },
    { id: 'teams', label: 'Teams', path: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z' }
  ];

  let isAddModalOpen = $state(false);
  const isTauri = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
  let backendAvailable = $state(isTauri);
  let groups = $state<GroupRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);
  let availableSnippets = $state<SnippetRecord[]>([]);
  let availableTasks = $state<ScheduledTaskRecord[]>([]);
  let availableTotp = $state<any[]>([]);
  let availableKeys = $state<any[]>([]);
  let availableTunnels = $state<any[]>([]);
  let availableInvestigations = $state<any[]>([]);

  let activeCategoryFilter = $state('all');
  let searchQuery = $state('');
  let currentPage = $state(1);
  let pageSize = $state(10);

  let editingId = $state<string | null>(null);
  let formName = $state('');
  let formColor = $state('#3b82f6');
  let formSelectedHosts = $state<string[]>([]);
  let formSelectedCategories = $state<string[]>(['hosts', 'snippets']);

  const colors = ['#ef4444', '#f97316', '#f59e0b', '#10b981', '#06b6d4', '#3b82f6', '#8b5cf6', '#ec4899'];

  async function loadAllData() {
    backendAvailable = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
    const results = await Promise.allSettled([
      listGroups(),
      listHosts(),
      listSnippets(),
      listScheduledTasks().catch(() => [])
    ]);

    if (results[0].status === 'fulfilled') {
      groups = results[0].value || [];
    } else {
      console.error('Failed to load groups:', results[0].reason);
    }

    if (results[1].status === 'fulfilled') {
      availableHosts = results[1].value || [];
    } else {
      console.error('Failed to load hosts:', results[1].reason);
    }

    if (results[2].status === 'fulfilled') {
      availableSnippets = results[2].value || [];
    } else {
      console.error('Failed to load snippets:', results[2].reason);
    }

    if (results[3].status === 'fulfilled') {
      availableTasks = results[3].value || [];
    } else {
      console.error('Failed to load scheduled tasks:', results[3].reason);
    }

    invoke('list_totp_entries').then((res: any) => { availableTotp = res || []; }).catch(() => {});
    invoke('list_keys').then((res: any) => { availableKeys = res || []; }).catch(() => {});
    invoke('list_tunnels').then((res: any) => { availableTunnels = res || []; }).catch(() => {});
    invoke('list_investigations').then((res: any) => { availableInvestigations = res || []; }).catch(() => {});
  }

  onMount(() => {
    void loadAllData();
    const handleUpdate = () => { void loadAllData(); };
    window.addEventListener('caterm:groups-updated', handleUpdate);
    window.addEventListener('caterm:hosts-updated', handleUpdate);
    window.addEventListener('caterm:snippets-updated', handleUpdate);
    window.addEventListener('caterm:tasks-updated', handleUpdate);
    return () => {
      window.removeEventListener('caterm:groups-updated', handleUpdate);
      window.removeEventListener('caterm:hosts-updated', handleUpdate);
      window.removeEventListener('caterm:snippets-updated', handleUpdate);
      window.removeEventListener('caterm:tasks-updated', handleUpdate);
    };
  });

  function hostLabel(id: string) {
    return availableHosts.find((h) => h.id === id)?.label ?? id;
  }

  function getSnippetsForGroup(groupName: string) {
    const q = groupName.toLowerCase();
    return availableSnippets.filter(s => s.tags.some(t => t.toLowerCase() === q) || s.label.toLowerCase().includes(q));
  }

  function getTotpForGroup(groupName: string) {
    const q = groupName.toLowerCase();
    return availableTotp.filter(t => (t.issuer && t.issuer.toLowerCase().includes(q)) || t.label.toLowerCase().includes(q));
  }

  function openAddModal() {
    editingId = null;
    formName = '';
    formColor = '#3b82f6';
    formSelectedHosts = [];
    formSelectedCategories = ['hosts', 'snippets', 'tasks'];
    isAddModalOpen = true;
  }

  function openEditModal(group: GroupRecord) {
    editingId = group.id;
    formName = group.name;
    formColor = group.color;
    formSelectedHosts = [...group.hostIds];
    formSelectedCategories = group.categories && group.categories.length > 0 ? [...group.categories] : ['hosts', 'snippets'];
    isAddModalOpen = true;
  }

  function toggleCategory(catId: string) {
    if (formSelectedCategories.includes(catId)) {
      formSelectedCategories = formSelectedCategories.filter(c => c !== catId);
    } else {
      formSelectedCategories = [...formSelectedCategories, catId];
    }
  }

  function toggleHost(hostId: string) {
    if (formSelectedHosts.includes(hostId)) {
      formSelectedHosts = formSelectedHosts.filter(id => id !== hostId);
    } else {
      formSelectedHosts = [...formSelectedHosts, hostId];
    }
  }

  async function handleSaveGroup(e: Event) {
    e.preventDefault();
    if (!formName.trim()) return;

    try {
      const saved = await saveGroup({
        id: editingId ?? undefined,
        name: formName.trim(),
        color: formColor,
        hostIds: formSelectedHosts,
        categories: formSelectedCategories.length > 0 ? formSelectedCategories : ['hosts']
      });

      groups = editingId
        ? groups.map((g) => (g.id === saved.id ? saved : g))
        : [...groups, saved];

      showToast(editingId ? t('groups.updatedToast') : t('groups.createdToast'), 'success');
      window.dispatchEvent(new CustomEvent('caterm:groups-updated'));
    } catch (err) {
      showToast(String(err), 'error');
    }

    editingId = null;
    isAddModalOpen = false;
  }

  async function removeGroup(id: string, name: string) {
    const confirmed = await confirmModal(
      t('groups.deleteConfirm', { name }),
      t('groups.deleteTitle'),
      true,
      t('common.delete'),
      t('common.cancel')
    );
    if (confirmed) {
      try {
        await deleteGroup(id);
        groups = groups.filter((g) => g.id !== id);
        showToast(t('groups.deletedToast', { name }), 'success');
        window.dispatchEvent(new CustomEvent('caterm:groups-updated'));
      } catch (e: any) {
        showToast(String(e), 'error');
      }
    }
  }

  function exportGroupsJson() {
    const jsonStr = JSON.stringify(groups, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `caterm-work-groups-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    showToast(t('groups.exportSuccess'), 'success');
  }

  function importGroupsJson() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e: any) => {
      const file = e.target?.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const imported = JSON.parse(text);
        if (Array.isArray(imported)) {
          for (const item of imported) {
            if (item.name) {
              await saveGroup({
                name: item.name,
                color: item.color || '#3b82f6',
                hostIds: item.hostIds || [],
                categories: item.categories || ['hosts']
              });
            }
          }
          showToast(t('groups.importSuccess', { count: imported.length }), 'success');
          await loadAllData();
        }
      } catch (err) {
        showToast(t('groups.importInvalidFormat'), 'error');
      }
    };
    input.click();
  }

  const filteredGroups = $derived.by(() => {
    let list = groups;
    if (activeCategoryFilter !== 'all') {
      list = list.filter(g => (g.categories || ['hosts']).includes(activeCategoryFilter));
    }
    const q = searchQuery.toLowerCase().trim();
    if (!q) return list;
    return list.filter(g => g.name.toLowerCase().includes(q));
  });

  const paginatedGroups = $derived.by(() => {
    const start = (currentPage - 1) * pageSize;
    return filteredGroups.slice(start, start + pageSize);
  });
</script>

<svelte:head>
  <title>Work Groups · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10']}
    accent="indigo"
    title="Work Groups"
    subtitle="Pengelompokan klaster kerja multi-kategori untuk Hosts, Snippets, Tasks, Port Forwarding, Keys, 2FA, dan Tim."
  >
    {#snippet actions()}
      <Button
        variant="secondary"
        size="sm"
        class="h-9 px-3 rounded-xl shrink-0 flex items-center gap-1.5"
        onclick={importGroupsJson}
        title="Impor Grup Kerja dari JSON"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/></svg>
        <span>Impor</span>
      </Button>
      <Button
        variant="secondary"
        size="sm"
        class="h-9 px-3 rounded-xl shrink-0 flex items-center gap-1.5"
        onclick={exportGroupsJson}
        title="Ekspor Grup Kerja ke JSON"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
        <span>Ekspor</span>
      </Button>
      <Button
        variant="primary"
        class="h-9 rounded-xl shrink-0 shadow-xs"
        onclick={openAddModal}
      >
        <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v14m7-7H5" />
        </svg>
        {t('groups.createGroup')}
      </Button>
    {/snippet}
  </PageHeader>

  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  <!-- Category Filter & Search Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-neutral-200 dark:border-neutral-800 pb-3">
    <!-- Category Tabs -->
    <div class="flex items-center gap-1.5 overflow-x-auto pb-1 text-xs">
      <Button
        variant={activeCategoryFilter === 'all' ? 'secondary' : 'ghost'}
        size="sm"
        onclick={() => { activeCategoryFilter = 'all'; currentPage = 1; }}
      >
        Semua Kategori
      </Button>
      {#each ALL_CATEGORIES as cat}
        <Button
          variant={activeCategoryFilter === cat.id ? 'info' : 'ghost'}
          size="sm"
          onclick={() => { activeCategoryFilter = cat.id; currentPage = 1; }}
          class="flex items-center gap-1.5"
        >
          <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d={cat.path}/></svg>
          <span>{cat.label}</span>
        </Button>
      {/each}
    </div>

    <!-- Search Box -->
    <div class="relative w-full sm:w-60 shrink-0">
      <Input
        bind:value={searchQuery}
        oninput={() => (currentPage = 1)}
        placeholder="Cari Work Group..."
        size="sm"
      >
        {#snippet leadingIcon()}
          <svg class="w-3.5 h-3.5 text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        {/snippet}
      </Input>
    </div>
  </div>

  <!-- Groups Grid -->
  {#if filteredGroups.length === 0}
    <div class="text-center py-16 px-4 bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800 space-y-3">
      <div class="w-12 h-12 rounded-full bg-indigo-500/10 text-indigo-500 flex items-center justify-center mx-auto border border-indigo-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
      </div>
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">Belum Ada Work Group</h3>
      <p class="text-xs text-neutral-500 max-w-sm mx-auto">
        Buat grup kerja untuk mengkategorikan host, snippet, tugas cron, kunci SSH, 2FA, dan port forwarding dalam satu kesatuan.
      </p>
      <Button
        variant="info"
        size="sm"
        onclick={openAddModal}
        class="mt-2"
      >
        + Buat Work Group Pertama
      </Button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each paginatedGroups as group (group.id)}
        {@const groupSnippets = getSnippetsForGroup(group.name)}
        {@const groupTotp = getTotpForGroup(group.name)}
        {@const cats = group.categories || ['hosts']}

        <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-all shadow-xs group">
          <div class="space-y-3">
            <!-- Header -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2.5">
                <span class="w-3.5 h-3.5 rounded-full inline-block shrink-0 shadow-xs" style="background-color: {group.color}"></span>
                <h3 class="font-bold text-neutral-900 dark:text-white text-base truncate">{group.name}</h3>
              </div>
              {#if group.hostIds.length > 0}
                <Button
                  variant="info"
                  size="sm"
                  onclick={() => openSession(group.hostIds)}
                  title="Buka semua host di grup ini"
                  class="flex items-center gap-1.5"
                >
                  <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
                  <span>Launch</span>
                </Button>
              {/if}
            </div>

            <!-- Active Category Badges -->
            <div class="flex gap-1 flex-wrap">
              {#each cats as cId}
                {@const catMeta = ALL_CATEGORIES.find(c => c.id === cId)}
                {#if catMeta}
                  <Badge variant="neutral" size="xs">
                    <span class="inline-flex items-center gap-1">
                      <svg class="w-3 h-3 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d={catMeta.path}/></svg>
                      <span>{catMeta.label}</span>
                    </span>
                  </Badge>
                {/if}
              {/each}
            </div>

            <!-- Resource Counts Grid -->
            <div class="grid grid-cols-3 gap-1.5 text-center text-[11px] py-1.5 bg-neutral-50 dark:bg-neutral-950/60 rounded-xl border border-neutral-100 dark:border-neutral-800/60">
              <div>
                <span class="block font-bold text-neutral-900 dark:text-white">{group.hostIds.length}</span>
                <span class="text-[10px] text-neutral-500">Hosts</span>
              </div>
              <div>
                <span class="block font-bold text-neutral-900 dark:text-white">{groupSnippets.length}</span>
                <span class="text-[10px] text-neutral-500">Snippets</span>
              </div>
              <div>
                <span class="block font-bold text-neutral-900 dark:text-white">{groupTotp.length}</span>
                <span class="text-[10px] text-neutral-500">2FA Tokens</span>
              </div>
            </div>

            <!-- Assigned Hosts -->
            {#if group.hostIds.length > 0}
              <div>
                <span class="text-[10px] font-semibold uppercase tracking-wider text-neutral-400 block mb-1">Assigned Hosts</span>
                <div class="flex gap-1 flex-wrap">
                  {#each group.hostIds as hostId}
                    <Badge variant="neutral" size="xs" class="font-mono">
                      {hostLabel(hostId)}
                    </Badge>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Footer Actions -->
          <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center justify-between text-xs">
            <span class="text-[11px] text-neutral-400">
              {cats.length} Kategori Aktif
            </span>
            <div class="flex gap-1.5">
              <Button
                variant="secondary"
                size="sm"
                onclick={() => openEditModal(group)}
              >
                {t('common.edit')}
              </Button>
              <Button
                variant="danger"
                size="sm"
                onclick={() => removeGroup(group.id, group.name)}
              >
                {t('common.delete')}
              </Button>
            </div>
          </div>
        </div>
      {/each}
    </div>

    <!-- CAUI Pagination Component -->
    <Pagination
      total={filteredGroups.length}
      page={currentPage}
      pageSize={pageSize}
      pageSizeOptions={[9, 18, 36]}
      onPageChange={(p) => (currentPage = p)}
      onPageSizeChange={(ps) => { pageSize = ps; currentPage = 1; }}
      size="sm"
    />
  {/if}
</div>

<!-- Create / Edit Work Group Modal -->
<Modal bind:open={isAddModalOpen} title={editingId ? 'Edit Work Group' : 'Buat Work Group Baru'} size="lg">
  <form id="caterm-group-form" onsubmit={handleSaveGroup} class="space-y-4 text-xs">
    <!-- Name -->
    <div>
      <label for="group-name" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Nama Work Group *</label>
      <Input
        id="group-name"
        bind:value={formName}
        required
        placeholder="misal: Production VPS / Client Riset / Gerlink Store"
        size="sm"
      />
    </div>

    <!-- Color -->
    <div>
      <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-2">{t('groups.color')}</span>
      <div class="flex gap-2">
        {#each colors as c}
          <Button
            type="button"
            size="icon"
            variant="ghost"
            onclick={() => (formColor = c)}
            class="w-7 h-7 min-w-0 p-0 rounded-full transition-transform border-2 {formColor === c ? 'border-white scale-110 shadow-md' : 'border-transparent'} cursor-pointer"
            aria-label={`Warna ${c}`}
          >
            <span class="w-full h-full rounded-full" style="background-color: {c};"></span>
          </Button>
        {/each}
      </div>
    </div>

    <!-- Category Checklist (REQ: Multi-Category Scope) -->
    <div>
      <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
        Kategori yang Didukung Work Group Ini:
      </span>
      <div class="grid grid-cols-2 gap-1.5 p-2.5 bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl">
        {#each ALL_CATEGORIES as cat}
          <label class="flex items-center gap-2 p-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800 cursor-pointer text-neutral-800 dark:text-neutral-200 select-none">
            <input
              type="checkbox"
              checked={formSelectedCategories.includes(cat.id)}
              onchange={() => toggleCategory(cat.id)}
              class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
            />
            <span class="inline-flex items-center gap-1.5 text-xs">
              <svg class="w-3.5 h-3.5 shrink-0 text-neutral-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d={cat.path}/></svg>
              <span>{cat.label}</span>
            </span>
          </label>
        {/each}
      </div>
      <p class="text-[11px] text-neutral-500 mt-1">
        {t('groups.categoryHint')}
      </p>
    </div>

    <!-- Linked Hosts (if hosts category enabled) -->
    {#if formSelectedCategories.includes('hosts')}
      <div>
        <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('groups.selectHosts')}</span>
        <div class="space-y-1 max-h-36 overflow-y-auto p-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg">
          {#each availableHosts as h}
            <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded-md cursor-pointer text-neutral-700 dark:text-neutral-300 select-none">
              <input
                type="checkbox"
                checked={formSelectedHosts.includes(h.id)}
                onchange={() => toggleHost(h.id)}
                class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
              />
              <span class="font-medium">{h.label}</span>
              <span class="text-[10px] text-neutral-400 font-mono ml-auto">{h.address}</span>
            </label>
          {/each}
        </div>
      </div>
    {/if}
  </form>

  {#snippet footer()}
    <Button type="button" variant="secondary" size="sm" onclick={() => (isAddModalOpen = false)}>
      {t('common.cancel')}
    </Button>
    <Button
      type="submit"
      variant="info"
      size="sm"
      onclick={(e) => {
        e.preventDefault();
        handleSaveGroup(e as unknown as Event);
      }}
    >
      {editingId ? 'Simpan Perubahan' : 'Buat Work Group'}
    </Button>
  {/snippet}
</Modal>
