<script lang="ts">
  import { onMount } from 'svelte';
  import { injectIntoActiveSession, hasActiveSession } from '$lib/stores/activeSession.svelte';
  import { listSnippets, saveSnippet as saveSnippetApi, deleteSnippet as deleteSnippetApi, type SnippetRecord } from '$lib/api/snippets';
  import { listGroups, type GroupRecord } from '$lib/api/groups';
  import { t } from '$lib/i18n/index.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { PageHeader, Button, Input, Badge, Modal, Pagination } from '@cecepazhar/caui';
  import { save, open } from '@tauri-apps/plugin-dialog';
  import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';

  let isAddModalOpen = $state(false);
  let creationStep = $state(1); // 1: command, 2: details
  let injectNotice = $state('');
  const isTauri = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
  let backendAvailable = $state(isTauri);
  let editingId = $state<string | null>(null);

  let snippets = $state<SnippetRecord[]>([]);
  let groups = $state<GroupRecord[]>([]);
  let filterGroup = $state('all');
  let searchQuery = $state('');
  let currentPage = $state(1);
  let pageSize = $state(10);

  let newSnippet = $state({
    cmd: '',
    label: '',
    desc: '',
    tags: '',
    selectedGroup: ''
  });

  async function loadData() {
    backendAvailable = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
    const results = await Promise.allSettled([
      listSnippets(),
      listGroups()
    ]);

    if (results[0].status === 'fulfilled') {
      snippets = results[0].value || [];
    } else {
      console.error('Failed to load snippets:', results[0].reason);
    }

    if (results[1].status === 'fulfilled') {
      groups = results[1].value || [];
    } else {
      console.error('Failed to load groups:', results[1].reason);
    }
  }

  onMount(() => {
    void loadData();
    const handleHostsChange = () => {
      loadData();
    };
    window.addEventListener('caterm:hosts-updated', handleHostsChange);
    return () => {
      window.removeEventListener('caterm:hosts-updated', handleHostsChange);
    };
  });

  async function exportSnippets() {
    try {
      const allSnippets = await listSnippets();
      const jsonContent = JSON.stringify(allSnippets, null, 2);
      let saved = false;

      try {
        const filePath = await save({
          defaultPath: 'caterm-snippets-export.json',
          filters: [{ name: 'JSON', extensions: ['json'] }]
        });

        if (filePath) {
          await writeTextFile(filePath, jsonContent);
          saved = true;
          showToast(t('snippets.exportSuccess'), 'success');
        } else {
          return;
        }
      } catch {}

      if (!saved) {
        const blob = new Blob([jsonContent], { type: 'application/json;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'caterm-snippets-export.json';
        a.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
        showToast(t('snippets.exportSuccess'), 'success');
      }
    } catch (err) {
      showToast(t('snippets.exportFailed'), 'error');
    }
  }

  async function processSnippetsJson(jsonText: string) {
    try {
      const parsed = JSON.parse(jsonText);
      if (!Array.isArray(parsed)) {
        showToast(t('snippets.importInvalidFormat'), 'error');
        return;
      }
      for (const item of parsed) {
        if (item.label && item.command) {
          await saveSnippetApi({
            label: item.label,
            command: item.command,
            description: item.description || '',
            tags: item.tags || []
          });
        }
      }
      showToast(t('snippets.importSuccess', { count: parsed.length }), 'success');
      await loadData();
    } catch {
      showToast(t('snippets.importFailed'), 'error');
    }
  }

  async function importSnippets() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });
      if (selected && typeof selected === 'string') {
        const content = await readTextFile(selected);
        await processSnippetsJson(content);
        return;
      }
    } catch {}

    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.accept = '.json,application/json';
    fileInput.onchange = async (e: Event) => {
      const target = e.target as HTMLInputElement;
      const file = target?.files?.[0];
      if (file) {
        const text = await file.text();
        await processSnippetsJson(text);
      }
    };
    fileInput.click();
  }

  function resetModal() {
    editingId = null;
    creationStep = 1;
    newSnippet = { cmd: '', label: '', desc: '', tags: '', selectedGroup: '' };
    isAddModalOpen = false;
  }

  function openEditModal(snip: SnippetRecord) {
    editingId = snip.id;
    creationStep = 2;
    newSnippet = {
      cmd: snip.command,
      label: snip.label,
      desc: snip.description,
      tags: snip.tags.join(', '),
      selectedGroup: groups.find(g => snip.tags.some(t => t.toLowerCase() === g.name.toLowerCase()))?.name || ''
    };
    isAddModalOpen = true;
  }

  async function handleSaveSnippet(e: Event) {
    e.preventDefault();
    if (!newSnippet.label.trim() || !newSnippet.cmd.trim()) return;

    let parsedTags = newSnippet.tags
      .split(',')
      .map(t => t.trim())
      .filter(t => t.length > 0);

    if (newSnippet.selectedGroup && !parsedTags.includes(newSnippet.selectedGroup)) {
      parsedTags.push(newSnippet.selectedGroup);
    }

    try {
      await saveSnippetApi({
        id: editingId ?? undefined,
        label: newSnippet.label.trim(),
        description: newSnippet.desc.trim(),
        command: newSnippet.cmd.trim(),
        tags: parsedTags
      });
      showToast(editingId ? 'Snippet diperbarui!' : 'Snippet berhasil disimpan!', 'success');
      window.dispatchEvent(new CustomEvent('caterm:snippets-updated'));
      await loadData();
    } catch (err) {
      console.error('Failed to save snippet:', err);
      showToast('Gagal menyimpan snippet', 'error');
    }

    resetModal();
  }

  async function removeSnippet(id: string, label: string) {
    const confirmed = await confirmModal(
      `Hapus snippet "${label}"?`,
      'Hapus Snippet',
      true,
      'Hapus',
      'Batal'
    );
    if (confirmed) {
      try {
        await deleteSnippetApi(id);
        snippets = snippets.filter(s => s.id !== id);
        showToast(`Snippet "${label}" dihapus!`, 'success');
        window.dispatchEvent(new CustomEvent('caterm:snippets-updated'));
      } catch (err) {
        console.error('Failed to delete snippet:', err);
        showToast('Gagal menghapus snippet', 'error');
      }
    }
  }

  function injectSnippet(cmd: string) {
    const injected = injectIntoActiveSession(cmd);
    injectNotice = injected ? t('snippets.injectedNotice') : t('snippets.noActiveSessionNotice');
    setTimeout(() => (injectNotice = ''), 3000);
  }

  const snippetGroups = $derived(
    groups.filter(g => (g.categories || ['hosts']).includes('snippets'))
  );

  const filteredSnippets = $derived.by(() => {
    let list = snippets;
    if (filterGroup !== 'all') {
      const q = filterGroup.toLowerCase();
      list = list.filter(s => s.tags.some(t => t.toLowerCase() === q) || s.label.toLowerCase().includes(q));
    }
    const q = searchQuery.toLowerCase().trim();
    if (!q) return list;
    return list.filter(s => s.label.toLowerCase().includes(q) || s.command.toLowerCase().includes(q) || s.description.toLowerCase().includes(q));
  });

  const paginatedSnippets = $derived.by(() => {
    const start = (currentPage - 1) * pageSize;
    return filteredSnippets.slice(start, start + pageSize);
  });
</script>

<svelte:head>
  <title>{t('snippets.title')} · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4']}
    accent="emerald"
    title={t('snippets.title')}
    subtitle={t('snippets.subtitle')}
  >
    {#snippet actions()}
      <div class="flex items-center gap-2">
        <Button
          variant="secondary"
          size="sm"
          onclick={importSnippets}
          title={t('snippets.import')}
        >
          📥 {t('snippets.import')}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onclick={exportSnippets}
          disabled={snippets.length === 0}
          title={t('snippets.export')}
        >
          📤 {t('snippets.export')}
        </Button>
        <Button
          variant="info"
          size="sm"
          onclick={() => { resetModal(); isAddModalOpen = true; }}
        >
          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
          {t('snippets.addSnippet')}
        </Button>
      </div>
    {/snippet}
  </PageHeader>

  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  {#if injectNotice}
    <div class="px-4 py-2 bg-emerald-500/10 border border-emerald-500/30 rounded-xl text-xs text-emerald-600 dark:text-emerald-400 font-medium">
      {injectNotice}
    </div>
  {/if}

  <!-- Toolbar: Search + Work Group Filter -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-neutral-200 dark:border-neutral-800 pb-3">
    <!-- Work Group Filter Dropdown -->
    <div class="flex items-center gap-2 text-xs">
      <span class="text-neutral-500 font-medium shrink-0">Work Group:</span>
      <select
        bind:value={filterGroup}
        onchange={() => (currentPage = 1)}
        class="px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-white font-medium focus:outline-none focus:border-sky-500 shadow-2xs"
      >
        <option value="all">Semua Grup ({snippets.length})</option>
        {#each groups as g}
          <option value={g.name}>{g.name}</option>
        {/each}
      </select>
    </div>

    <!-- Search Box -->
    <div class="relative w-full sm:w-64">
      <Input
        bind:value={searchQuery}
        oninput={() => (currentPage = 1)}
        placeholder="Cari snippet atau perintah..."
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

  <!-- Snippets Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    {#each paginatedSnippets as snip (snip.id)}
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl overflow-hidden group shadow-xs flex flex-col justify-between">
        <div>
          <div class="p-4 border-b border-neutral-100 dark:border-neutral-800/80">
            <div class="flex justify-between items-start gap-2">
              <h3 class="font-bold text-neutral-900 dark:text-white text-sm">{snip.label}</h3>
              <div class="flex items-center gap-1 shrink-0">
                <Button
                  variant="info"
                  size="sm"
                  onclick={() => injectSnippet(snip.command)}
                  disabled={!hasActiveSession()}
                >
                  {t('snippets.inject')}
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onclick={() => openEditModal(snip)}
                >
                  {t('common.edit')}
                </Button>
                <Button
                  variant="danger"
                  size="sm"
                  onclick={() => removeSnippet(snip.id, snip.label)}
                >
                  {t('common.delete')}
                </Button>
              </div>
            </div>
            {#if snip.description}
              <p class="text-neutral-500 text-xs mt-1">{snip.description}</p>
            {/if}
          </div>

          <div class="bg-neutral-50 dark:bg-neutral-950 p-3 font-mono text-xs text-neutral-800 dark:text-neutral-200 relative overflow-x-auto">
            <span class="opacity-50 select-none mr-2 text-sky-500">$</span>{snip.command}
          </div>
        </div>

        <!-- Tags & Group Footer -->
        <div class="px-4 py-2 bg-neutral-50/50 dark:bg-neutral-900/50 flex gap-1.5 flex-wrap border-t border-neutral-100 dark:border-neutral-800/80">
          {#each snip.tags as tag}
            <Badge variant="neutral" size="xs">
              #{tag}
            </Badge>
          {/each}
        </div>
      </div>
    {/each}

    {#if filteredSnippets.length === 0}
      <div class="md:col-span-2 py-16 px-4 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-2xl text-center space-y-2">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">{t('snippets.emptyTitle')}</h3>
        <p class="text-xs text-neutral-500">{t('snippets.emptyBody')}</p>
      </div>
    {/if}
  </div>

  <!-- CAUI Pagination Component -->
  <Pagination
    total={filteredSnippets.length}
    page={currentPage}
    pageSize={pageSize}
    pageSizeOptions={[10, 20, 50]}
    onPageChange={(p) => (currentPage = p)}
    onPageSizeChange={(ps) => { pageSize = ps; currentPage = 1; }}
    size="sm"
  />
</div>

<!-- Modal Create / Edit Snippet -->
<Modal bind:open={isAddModalOpen} title={editingId ? t('snippets.editSnippet') : t('snippets.addSnippet')} size="lg">
  {#snippet header()}
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <h3 class="text-base font-bold text-neutral-900 dark:text-white">{editingId ? t('snippets.editSnippet') : t('snippets.addSnippet')}</h3>
        <Badge variant="neutral" size="xs">{t('common.step', { current: creationStep, total: 2 })}</Badge>
      </div>
    </div>
  {/snippet}

  {#if creationStep === 1}
    <div class="space-y-4 text-xs">
      <div>
        <label for="snippet-cmd" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1.5">{t('snippets.bashCommand')} *</label>
        <textarea
          id="snippet-cmd"
          bind:value={newSnippet.cmd}
          rows="5"
          placeholder={t('snippets.commandPlaceholder')}
          class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 font-mono border border-neutral-300 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"
        ></textarea>
      </div>
      <div class="flex justify-end gap-2">
        <Button
          variant="secondary"
          size="sm"
          onclick={resetModal}
        >
          {t('common.cancel')}
        </Button>
        <Button
          variant="info"
          size="sm"
          onclick={() => (creationStep = 2)}
          disabled={!newSnippet.cmd.trim()}
        >
          {t('common.next')} →
        </Button>
      </div>
    </div>
  {:else}
    <form id="caterm-snippet-form" onsubmit={handleSaveSnippet} class="space-y-3 text-xs">
      <div>
        <label for="snippet-label" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('snippets.label')} *</label>
        <Input id="snippet-label" bind:value={newSnippet.label} required placeholder={t('snippets.labelPlaceholder')} size="sm" />
      </div>

      <div>
        <label for="snippet-group" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Work Group (Opsional)</label>
        <select
          id="snippet-group"
          bind:value={newSnippet.selectedGroup}
          class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 text-xs"
        >
          <option value="">Tanpa Work Group</option>
          {#each snippetGroups as g}
            <option value={g.name}>{g.name}</option>
          {/each}
        </select>
      </div>

      <div>
        <label for="snippet-desc" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('snippets.descriptionOptional')}</label>
        <Input id="snippet-desc" bind:value={newSnippet.desc} placeholder={t('snippets.descriptionPlaceholder')} size="sm" />
      </div>

      <div>
        <label for="snippet-tags" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('snippets.tagsCommaSeparated')}</label>
        <Input id="snippet-tags" bind:value={newSnippet.tags} placeholder="misal: docker, logs, maintenance" size="sm" />
      </div>
    </form>

    {#snippet footer()}
      <div class="flex justify-between items-center w-full">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onclick={() => (creationStep = 1)}
          class="text-neutral-500"
        >
          ← {t('common.back')}
        </Button>
        <div class="flex gap-2">
          <Button
            type="button"
            variant="secondary"
            size="sm"
            onclick={resetModal}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            variant="success"
            size="sm"
            onclick={(e) => {
              e.preventDefault();
              handleSaveSnippet(e as unknown as Event);
            }}
          >
            {t('snippets.saveSnippet')}
          </Button>
        </div>
      </div>
    {/snippet}
  {/if}
</Modal>
