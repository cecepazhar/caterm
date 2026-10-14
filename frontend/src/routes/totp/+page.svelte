<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import TotpBadge from '$lib/components/TotpBadge.svelte';
  import { PageHeader, Button, Input, Modal } from '@cecepazhar/caui';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { t } from '$lib/i18n/index.svelte';

  interface TotpEntry {
    id: string;
    label: string;
    issuer?: string;
    hasSecret: boolean;
    token: string;
    remainingSeconds: number;
    createdAt: number;
    updatedAt: number;
  }

  let entries = $state<TotpEntry[]>([]);
  let searchQuery = $state('');
  let isAddModalOpen = $state(false);
  let isLoading = $state(false);
  let timerInterval: any = null;

  // Form State
  let editingId = $state<string | null>(null);
  let formLabel = $state('');
  let formIssuer = $state('');
  let formSecret = $state('');
  let formError = $state('');

  const filteredEntries = $derived.by(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return entries;
    return entries.filter(
      (e) =>
        e.label.toLowerCase().includes(q) ||
        (e.issuer && e.issuer.toLowerCase().includes(q))
    );
  });

  onMount(() => {
    refreshEntries();
    // Live countdown update every second
    timerInterval = setInterval(() => {
      let needsFetch = false;
      entries = entries.map((entry) => {
        if (entry.remainingSeconds <= 1) {
          needsFetch = true;
          return { ...entry, remainingSeconds: 30 };
        }
        return { ...entry, remainingSeconds: entry.remainingSeconds - 1 };
      });
      if (needsFetch) {
        refreshEntries();
      }
    }, 1000);
  });

  onDestroy(() => {
    if (timerInterval) clearInterval(timerInterval);
  });

  async function refreshEntries() {
    try {
      entries = await invoke('list_totp_entries');
    } catch (e: any) {
      console.warn('Failed to load 2FA entries:', e);
    }
  }

  function openAddModal() {
    editingId = null;
    formLabel = '';
    formIssuer = '';
    formSecret = '';
    formError = '';
    isAddModalOpen = true;
  }

  function openEditModal(entry: TotpEntry) {
    editingId = entry.id;
    formLabel = entry.label;
    formIssuer = entry.issuer || '';
    formSecret = '';
    formError = '';
    isAddModalOpen = true;
  }

  async function handleSave() {
    if (!formLabel.trim()) {
      formError = 'Label token 2FA wajib diisi';
      return;
    }
    if (!editingId && !formSecret.trim()) {
      formError = 'Secret key Base32 atau URL otpauth:// wajib diisi';
      return;
    }

    isLoading = true;
    formError = '';
    try {
      await invoke('save_totp_entry', {
        input: {
          id: editingId ?? undefined,
          label: formLabel.trim(),
          issuer: formIssuer.trim() || undefined,
          secret: formSecret.trim() || undefined
        }
      });
      isAddModalOpen = false;
      showToast(editingId ? 'Token 2FA diperbarui!' : 'Token 2FA berhasil ditambahkan!', 'success');
      await refreshEntries();
    } catch (err: any) {
      formError = typeof err === 'string' ? err : err?.message || 'Gagal menyimpan 2FA token';
      showToast(formError, 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleDelete(id: string, label: string) {
    const confirmed = await confirmModal(
      `Apakah Anda yakin ingin menghapus token 2FA "${label}"? Tindakan ini tidak dapat dibatalkan.`,
      'Hapus Token 2FA',
      true,
      'Hapus',
      'Batal'
    );
    if (confirmed) {
      try {
        await invoke('delete_totp_entry', { id });
        showToast(`Token "${label}" dihapus!`, 'success');
        await refreshEntries();
      } catch (e: any) {
        showToast(String(e), 'error');
      }
    }
  }
</script>

<svelte:head>
  <title>2FA Authenticator Vault · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <!-- Standard Page Header -->
  <PageHeader
    icon={['M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z']}
    accent="sky"
    title={t('totp.title')}
    subtitle={t('totp.subtitle')}
  >
    {#snippet actions()}
      <Button
        variant="info"
        size="sm"
        onclick={openAddModal}
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        {t('totp.newKey')}
      </Button>
    {/snippet}
  </PageHeader>

  <!-- Search & Stats Bar -->
  <div class="flex items-center justify-between gap-3">
    <div class="relative flex-1 max-w-md">
      <Input
        bind:value={searchQuery}
        placeholder={t('totp.searchPlaceholder')}
        size="sm"
      >
        {#snippet leadingIcon()}
          <svg class="w-4 h-4 text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        {/snippet}
      </Input>
    </div>
    <span class="text-xs font-medium text-neutral-500 shrink-0">
      {filteredEntries.length} {t('totp.title')}
    </span>
  </div>

  <!-- Cards Grid -->
  {#if filteredEntries.length === 0}
    <div class="text-center py-16 px-4 bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800 space-y-3">
      <div class="w-12 h-12 rounded-full bg-sky-500/10 text-sky-500 flex items-center justify-center mx-auto border border-sky-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">{t('totp.emptyTitle')}</h3>
      <p class="text-xs text-neutral-500 max-w-sm mx-auto">
        {t('totp.emptyBody')}
      </p>
      <Button
        variant="secondary"
        size="sm"
        onclick={openAddModal}
        class="mt-2"
      >
        {t('totp.addFirst')}
      </Button>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
      {#each filteredEntries as entry (entry.id)}
        <TotpBadge
          id={entry.id}
          token={entry.token}
          remainingSeconds={entry.remainingSeconds}
          label={entry.label}
          issuer={entry.issuer}
          onEdit={() => openEditModal(entry)}
          onDelete={() => handleDelete(entry.id, entry.label)}
        />
      {/each}
    </div>
  {/if}
</div>

<!-- Add / Edit 2FA Modal -->
<Modal bind:open={isAddModalOpen} title={editingId ? 'Edit Token 2FA' : 'Tambah Token 2FA Baru'} size="md">
  <div class="space-y-4 text-xs">
    {#if formError}
      <div class="p-2.5 text-xs rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400">
        {formError}
      </div>
    {/if}

    <div>
      <label for="totp-label" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
        Nama Akun / Server *
      </label>
      <Input
        id="totp-label"
        bind:value={formLabel}
        placeholder="misal: Production Bastion / AWS Root"
        size="sm"
      />
    </div>

    <div>
      <label for="totp-issuer" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
        Penerbit / Layanan (Opsional)
      </label>
      <Input
        id="totp-issuer"
        bind:value={formIssuer}
        placeholder="misal: AWS, Cloudflare, GitHub, Gerlink"
        size="sm"
      />
    </div>

    <div>
      <label for="totp-secret" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">
        Secret Key (Base32 atau URL otpauth://) {editingId ? '(Kosongkan jika tidak ingin mengubah)' : '*'}
      </label>
      <Input
        id="totp-secret"
        type="password"
        bind:value={formSecret}
        placeholder="misal: JBSWY3DPEHPK3PXP atau otpauth://totp/..."
        size="sm"
        class="font-mono"
      />
      <p class="text-[11px] text-neutral-500 mt-1">
        Kunci rahasia akan langsung dienkripsi menggunakan Zero-Knowledge AES-256-GCM vault.
      </p>
    </div>
  </div>
  {#snippet footer()}
    <Button variant="secondary" size="sm" onclick={() => (isAddModalOpen = false)}>
      Batal
    </Button>
    <Button variant="info" size="sm" onclick={handleSave} disabled={isLoading} loading={isLoading}>
      {isLoading ? 'Menyimpan...' : 'Simpan Token'}
    </Button>
  {/snippet}
</Modal>
