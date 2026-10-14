<script lang="ts">
  import { onMount } from 'svelte';
  import { listTeams, saveTeam, deleteTeam, type TeamRecord } from '$lib/api/teams';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { listGroups, type GroupRecord } from '$lib/api/groups';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { PageHeader, Button, Input, Badge, Card, Modal, ColorPicker } from '@cecepazhar/caui';

  let isAddModalOpen = $state(false);
  const isTauri = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
  let backendAvailable = $state(isTauri);
  let teams = $state<TeamRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);
  let availableGroups = $state<GroupRecord[]>([]);
  let editingId = $state<string | null>(null);

  let newTeam = $state({
    name: "",
    color: "#3b82f6",
    avatar: "",
    membersStr: "",
    selectedHosts: [] as string[],
    selectedGroups: [] as string[]
  });

  const colors = ["#ef4444", "#f97316", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ec4899"];

  onMount(async () => {
    backendAvailable = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
    const results = await Promise.allSettled([
      listTeams(),
      listHosts(),
      listGroups()
    ]);

    if (results[0].status === 'fulfilled') {
      teams = results[0].value || [];
    } else {
      console.error('Failed to load teams:', results[0].reason);
      showToast(t('teams.loadFailed') || 'Gagal memuat daftar tim', 'error');
    }

    if (results[1].status === 'fulfilled') {
      availableHosts = results[1].value || [];
    } else {
      console.error('Failed to load hosts:', results[1].reason);
    }

    if (results[2].status === 'fulfilled') {
      availableGroups = results[2].value || [];
    } else {
      console.error('Failed to load groups:', results[2].reason);
    }
  });

  function hostLabel(id: string) {
    return availableHosts.find((h) => h.id === id)?.label ?? id;
  }

  function groupLabel(id: string) {
    return availableGroups.find((g) => g.id === id)?.name ?? id;
  }

  function openAddModal() {
    editingId = null;
    newTeam = { name: "", color: "#3b82f6", avatar: "", membersStr: "", selectedHosts: [], selectedGroups: [] };
    isAddModalOpen = true;
  }

  function openEditModal(team: TeamRecord) {
    editingId = team.id;
    newTeam = {
      name: team.name,
      color: team.color,
      avatar: team.avatar ?? "",
      membersStr: team.members.join(", "),
      selectedHosts: [...team.hostIds],
      selectedGroups: [...team.groupIds]
    };
    isAddModalOpen = true;
  }

  async function createTeam(e: Event) {
    e.preventDefault();
    if (!newTeam.name) return;

    const membersArray = newTeam.membersStr.split(',').map(m => m.trim()).filter(m => m.length > 0);

    try {
      const saved = await saveTeam({
        id: editingId ?? undefined,
        name: newTeam.name,
        color: newTeam.color,
        avatar: newTeam.avatar || null,
        members: membersArray,
        hostIds: newTeam.selectedHosts,
        groupIds: newTeam.selectedGroups
      });
      teams = editingId
        ? teams.map((t) => (t.id === saved.id ? saved : t))
        : [...teams, saved];
      showToast(editingId ? 'Tim berhasil diperbarui' : 'Tim berhasil dibuat', 'success');
      editingId = null;
      isAddModalOpen = false;
    } catch (err) {
      console.error('Failed to save team:', err);
      showToast(t('teams.saveFailed') || 'Gagal menyimpan tim', 'error');
    }
  }

  async function removeTeam(id: string) {
    try {
      await deleteTeam(id);
      teams = teams.filter((t) => t.id !== id);
      showToast('Tim berhasil dihapus', 'info');
    } catch (err) {
      console.error('Failed to delete team:', err);
      showToast(t('teams.deleteFailed') || 'Gagal menghapus tim', 'error');
    }
  }

  function toggleHost(hostId: string) {
    if (newTeam.selectedHosts.includes(hostId)) {
      newTeam.selectedHosts = newTeam.selectedHosts.filter(id => id !== hostId);
    } else {
      newTeam.selectedHosts.push(hostId);
    }
  }

  function toggleGroup(groupId: string) {
    if (newTeam.selectedGroups.includes(groupId)) {
      newTeam.selectedGroups = newTeam.selectedGroups.filter(id => id !== groupId);
    } else {
      newTeam.selectedGroups.push(groupId);
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z']}
    accent="teal"
    title={t('teams.title')}
    subtitle={t('teams.subtitle')}
  >
    {#snippet actions()}
      <Button
        onclick={openAddModal}
        variant="primary"
        size="md"
      >
        <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        {t('teams.createTeam')}
      </Button>
    {/snippet}
  </PageHeader>
  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  <!-- Teams Grid / Empty State -->
  {#if teams.length === 0}
    <Card class="text-center py-16 px-4 space-y-4" variant="primary" size="lg">
      <div class="w-12 h-12 rounded-full bg-teal-500/10 text-teal-600 dark:text-teal-400 flex items-center justify-center mx-auto border border-teal-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
        </svg>
      </div>
      <div>
        <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('teams.emptyTitle')}</h3>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 max-w-sm mx-auto mt-1">
          {t('teams.emptyBody')}
        </p>
      </div>
      <div>
        <Button
          onclick={openAddModal}
          variant="primary"
          size="sm"
        >
          <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {t('teams.createTeam')}
        </Button>
      </div>
    </Card>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each teams as team}
        <Card class="flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-colors" variant="secondary" size="md">
          <div>
            <div class="flex items-center gap-3">
              {#if team.avatar}
                <div class="w-8 h-8 rounded-full overflow-hidden border border-neutral-300 dark:border-neutral-700 flex items-center justify-center bg-neutral-100 dark:bg-neutral-800">
                  <span class="text-xs">{team.avatar}</span>
                </div>
              {:else}
                <span class="w-4 h-4 rounded-full inline-block" style="background-color: {team.color}"></span>
              {/if}
              <h3 class="font-semibold text-neutral-900 dark:text-white text-lg">{team.name}</h3>
            </div>
            <p class="text-neutral-500 dark:text-neutral-400 text-sm mt-2">{t('teams.membersCount', { count: team.members.length })}</p>
            {#if team.members.length > 0}
              <div class="flex gap-1 flex-wrap mt-1">
                {#each team.members as member}
                  <span class="text-xs text-neutral-500 dark:text-neutral-500">{member}</span>
                {/each}
              </div>
            {/if}

            <div class="mt-3">
              {#if team.hostIds.length > 0}
                <p class="text-neutral-500 dark:text-neutral-400 text-xs mt-2 uppercase font-medium">{t('teams.hostsLabel')}</p>
                <div class="flex gap-1 flex-wrap mt-1">
                  {#each team.hostIds as hostId}
                    <Badge variant="neutral" size="xs">{hostLabel(hostId)}</Badge>
                  {/each}
                </div>
              {/if}
              {#if team.groupIds.length > 0}
                <p class="text-neutral-500 dark:text-neutral-400 text-xs mt-2 uppercase font-medium">{t('teams.groupsLabel')}</p>
                <div class="flex gap-1 flex-wrap mt-1">
                  {#each team.groupIds as groupId}
                    <Badge variant="neutral" size="xs">{groupLabel(groupId)}</Badge>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
          <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex justify-end gap-2">
            <Button onclick={() => openEditModal(team)} variant="neutral" size="sm">{t('common.edit')}</Button>
            <Button onclick={() => removeTeam(team.id)} variant="danger" size="sm">{t('common.delete')}</Button>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<!-- Create Team Modal -->
<Modal
  bind:open={isAddModalOpen}
  title={editingId ? t('teams.editTeam') : t('teams.createTeam')}
  size="lg"
>
  <form id="team-form" onsubmit={createTeam} class="space-y-4">
    <Input
      id="team-name"
      label={t('teams.teamName')}
      bind:value={newTeam.name}
      required
      placeholder={t('teams.teamNamePlaceholder')}
    />

    <Input
      id="team-avatar"
      label={t('teams.avatar')}
      bind:value={newTeam.avatar}
      placeholder={t('teams.avatarPlaceholder')}
    />

    <Input
      id="team-members"
      label={t('teams.members')}
      bind:value={newTeam.membersStr}
      placeholder={t('teams.membersPlaceholder')}
    />

    <Input
      id="team-cloud-account"
      label={t('teams.accountUsername')}
      disabled
      placeholder={t('teams.accountUsernamePlaceholder')}
    />

    <ColorPicker
      label={t('teams.colorSwatch')}
      bind:value={newTeam.color}
    />

    <div>
      <span class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-2">{t('teams.assignHosts')}</span>
      <div class="space-y-1 max-h-32 overflow-y-auto p-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded-lg">
        {#each availableHosts as h}
          <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-900 rounded cursor-pointer text-sm text-neutral-700 dark:text-neutral-300">
            <input
              type="checkbox"
              checked={newTeam.selectedHosts.includes(h.id)}
              onchange={() => toggleHost(h.id)}
              class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
            />
            <span>{h.label}</span>
          </label>
        {/each}
        {#if availableHosts.length === 0}
          <span class="text-xs text-neutral-500">{t('teams.noHostsAvailable')}</span>
        {/if}
      </div>
    </div>

    <div>
      <span class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-2">{t('teams.assignGroups')}</span>
      <div class="space-y-1 max-h-32 overflow-y-auto p-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded-lg">
        {#each availableGroups as g}
          <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-900 rounded cursor-pointer text-sm text-neutral-700 dark:text-neutral-300">
            <input
              type="checkbox"
              checked={newTeam.selectedGroups.includes(g.id)}
              onchange={() => toggleGroup(g.id)}
              class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
            />
            <span>{g.name}</span>
          </label>
        {/each}
        {#if availableGroups.length === 0}
          <span class="text-xs text-neutral-500">{t('teams.noGroupsAvailable')}</span>
        {/if}
      </div>
    </div>
  </form>

  {#snippet footer()}
    <Button
      type="button"
      onclick={() => (isAddModalOpen = false)}
      variant="neutral"
      size="md"
    >
      {t('common.cancel')}
    </Button>
    <Button
      type="button"
      onclick={createTeam}
      variant="primary"
      size="md"
    >
      {t('teams.saveTeam')}
    </Button>
  {/snippet}
</Modal>
