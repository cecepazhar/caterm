<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listScheduledTasks,
    saveScheduledTask,
    deleteScheduledTask,
    triggerTaskRunNow,
    getTaskExecutionLogs,
    type ScheduledTaskRecord,
    type TaskExecutionLog,
    type TaskType
  } from '$lib/api/tasks';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { t } from '$lib/i18n/index.svelte';
  import { PageHeader, Button, Input, Badge, Modal, SegmentedControl } from '@cecepazhar/caui';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';

  interface TaskFilterOption {
    value: string;
    label: string;
  }

  let tasks = $state<ScheduledTaskRecord[]>([]);
  let hosts = $state<HostRecord[]>([]);
  let isLoading = $state(false);
  let searchQuery = $state('');
  let filterType = $state<string>('all');

  const filterOptions = $derived<TaskFilterOption[]>([
    { value: 'all', label: t('tasks.allTasks', { count: tasks.length }) },
    { value: 'sftp_backup', label: t('tasks.sftpBackup') },
    { value: 'ssh_command', label: t('tasks.sshPlaybook') }
  ]);

  // Modal State: Create / Edit Task
  let isAddModalOpen = $state(false);
  let editingId = $state<string | null>(null);
  let formName = $state('');
  let formDescription = $state('');
  let formTaskType = $state<TaskType>('ssh_command');
  let formHostId = $state('');
  let formScheduleExpr = $state('0 2 * * *');
  let formCommandScript = $state('');
  let formTimeoutSeconds = $state(300);
  let formRemoteSrcPath = $state('');
  let formLocalDestDir = $state('');
  let formRemotePreCmd = $state('');
  let formRemotePostCmd = $state('');
  let formRetentionCount = $state(7);
  let formNotifySuccess = $state(false);
  let formNotifyFailure = $state(true);

  // Modal State: Log Inspector
  let isLogModalOpen = $state(false);
  let activeTaskForLogs = $state<ScheduledTaskRecord | null>(null);
  let activeTaskLogs = $state<TaskExecutionLog[]>([]);
  let isExecutingTaskId = $state<string | null>(null);

  async function loadData() {
    try {
      const [tList, hList] = await Promise.all([
        listScheduledTasks(),
        listHosts()
      ]);
      tasks = tList || [];
      hosts = hList || [];
    } catch (e) {
      console.warn('Failed to load tasks/hosts:', e);
    }
  }

  onMount(() => {
    loadData();
    const handleUpdate = () => { void loadData(); };
    window.addEventListener('caterm:tasks-updated', handleUpdate);
    return () => {
      window.removeEventListener('caterm:tasks-updated', handleUpdate);
    };
  });

  function hostName(id?: string) {
    if (!id) return 'Local PC';
    return hosts.find(h => h.id === id)?.label || id;
  }

  function openAddModal() {
    editingId = null;
    formName = '';
    formDescription = '';
    formTaskType = 'ssh_command';
    formHostId = hosts[0]?.id || '';
    formScheduleExpr = '0 2 * * *';
    formCommandScript = '#!/usr/bin/env bash\necho "Running maintenance..."\n';
    formTimeoutSeconds = 300;
    formRemoteSrcPath = '/var/backups/db.sql.gz';
    formLocalDestDir = '/home/cecepazhar/Backups';
    formRemotePreCmd = 'mysqldump -u root ... | gzip > /var/backups/db.sql.gz';
    formRemotePostCmd = 'rm -f /var/backups/db.sql.gz';
    formRetentionCount = 7;
    formNotifySuccess = false;
    formNotifyFailure = true;
    isAddModalOpen = true;
  }

  function openEditModal(task: ScheduledTaskRecord) {
    editingId = task.id;
    formName = task.name;
    formDescription = task.description || '';
    formTaskType = task.taskType;
    formHostId = task.hostId || '';
    formScheduleExpr = task.scheduleExpr;
    formCommandScript = task.commandScript || '';
    formTimeoutSeconds = task.timeoutSeconds || 300;
    formRemoteSrcPath = task.remoteSrcPath || '';
    formLocalDestDir = task.localDestDir || '';
    formRemotePreCmd = task.remotePreCmd || '';
    formRemotePostCmd = task.remotePostCmd || '';
    formRetentionCount = task.retentionCount || 7;
    formNotifySuccess = task.notifyOnSuccess;
    formNotifyFailure = task.notifyOnFailure;
    isAddModalOpen = true;
  }

  async function handleSaveTask() {
    if (!formName.trim()) {
      showToast(t('tasks.nameRequired'), 'error');
      return;
    }

    isLoading = true;
    try {
      await saveScheduledTask({
        id: editingId ?? undefined,
        name: formName.trim(),
        description: formDescription.trim() || undefined,
        taskType: formTaskType,
        hostId: formHostId || undefined,
        scheduleExpr: formScheduleExpr.trim(),
        isEnabled: true,
        commandScript: formTaskType === 'ssh_command' || formTaskType === 'local_script' ? formCommandScript : undefined,
        timeoutSeconds: formTimeoutSeconds,
        remoteSrcPath: formTaskType === 'sftp_backup' ? formRemoteSrcPath : undefined,
        localDestDir: formTaskType === 'sftp_backup' ? formLocalDestDir : undefined,
        remotePreCmd: formTaskType === 'sftp_backup' ? formRemotePreCmd : undefined,
        remotePostCmd: formTaskType === 'sftp_backup' ? formRemotePostCmd : undefined,
        retentionCount: formRetentionCount,
        notifyOnSuccess: formNotifySuccess,
        notifyOnFailure: formNotifyFailure
      });

      isAddModalOpen = false;
      showToast(editingId ? t('tasks.taskUpdated') : t('tasks.taskScheduled'), 'success');
      await loadData();
    } catch (e: any) {
      showToast(String(e), 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleDeleteTask(task: ScheduledTaskRecord) {
    const confirmed = await confirmModal(
      t('tasks.deleteConfirm', { name: task.name }),
      t('tasks.deleteTitle'),
      true,
      t('common.delete'),
      t('common.cancel')
    );
    if (confirmed) {
      try {
        await deleteScheduledTask(task.id);
        showToast(t('tasks.taskDeleted', { name: task.name }), 'success');
        await loadData();
      } catch (e: any) {
        showToast(String(e), 'error');
      }
    }
  }

  async function handleRunNow(task: ScheduledTaskRecord) {
    isExecutingTaskId = task.id;
    try {
      const log = await triggerTaskRunNow(task.id);
      showToast(t('tasks.taskExecuted', { name: task.name, status: log.status }), log.status === 'success' ? 'success' : 'error');
      await loadData();
    } catch (e: any) {
      showToast(t('tasks.taskRunFailed', { error: String(e) }), 'error');
    } finally {
      isExecutingTaskId = null;
    }
  }

  async function openLogModal(task: ScheduledTaskRecord) {
    activeTaskForLogs = task;
    try {
      activeTaskLogs = await getTaskExecutionLogs(task.id);
    } catch {
      activeTaskLogs = [];
    }
    isLogModalOpen = true;
  }

  const filteredTasks = $derived.by(() => {
    let list = tasks;
    if (filterType !== 'all') {
      list = list.filter(t => t.taskType === filterType);
    }
    const q = searchQuery.toLowerCase().trim();
    if (!q) return list;
    return list.filter(t => t.name.toLowerCase().includes(q) || (t.description && t.description.toLowerCase().includes(q)));
  });
</script>

<svelte:head>
  <title>Scheduled Tasks & SFTP Auto-Backup · CATerm</title>
</svelte:head>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z']}
    accent="indigo"
    title={t('tasks.title')}
    subtitle={t('tasks.subtitle')}
  >
    {#snippet actions()}
      <Button
        type="button"
        variant="primary"
        class="h-9 rounded-xl shrink-0 shadow-xs"
        onclick={openAddModal}
      >
        <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v14m7-7H5" />
        </svg>
        {t('tasks.newTask')}
      </Button>
    {/snippet}
  </PageHeader>

  <!-- Filter & Status Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-neutral-200 dark:border-neutral-800 pb-3">
    <div class="overflow-x-auto">
      <SegmentedControl options={filterOptions} bind:value={filterType} size="sm" />
    </div>

    <!-- Search Input -->
    <div class="w-full sm:w-64">
      <Input
        type="text"
        bind:value={searchQuery}
        placeholder={t('tasks.searchPlaceholder')}
        size="sm"
      />
    </div>
  </div>

  <!-- Tasks Grid -->
  {#if filteredTasks.length === 0}
    <div class="text-center py-16 px-4 bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800 space-y-3">
      <div class="w-12 h-12 rounded-full bg-indigo-500/10 text-indigo-500 flex items-center justify-center mx-auto border border-indigo-500/20">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </div>
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">{t('tasks.emptyTitle')}</h3>
      <p class="text-xs text-neutral-500 max-w-sm mx-auto">
        {t('tasks.emptyBody')}
      </p>
      <Button
        type="button"
        variant="primary"
        size="sm"
        onclick={openAddModal}
        class="mt-2"
      >
        {t('tasks.scheduleFirst')}
      </Button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredTasks as task (task.id)}
        <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-all shadow-xs group">
          <div class="space-y-3">
            <!-- Header -->
            <div class="flex items-start justify-between gap-2">
              <div>
                <div class="flex items-center gap-2">
                  <Badge variant={task.taskType === 'sftp_backup' ? 'brand' : 'info'} size="xs">
                    <span class="inline-flex items-center gap-1">
                      {#if task.taskType === 'sftp_backup'}
                        <svg class="w-3 h-3 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
                        <span>SFTP Backup</span>
                      {:else}
                        <svg class="w-3 h-3 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
                        <span>SSH Playbook</span>
                      {/if}
                    </span>
                  </Badge>
                  <span class="text-xs text-neutral-500 font-mono inline-flex items-center gap-1">
                    <svg class="w-3 h-3 shrink-0 text-neutral-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                    <span>{task.scheduleExpr}</span>
                  </span>
                </div>
                <h3 class="font-bold text-neutral-900 dark:text-white text-base mt-1.5">{task.name}</h3>
                {#if task.description}
                  <p class="text-xs text-neutral-500 line-clamp-1 mt-0.5">{task.description}</p>
                {/if}
              </div>

              <!-- Status Badge -->
              <Badge variant={task.isEnabled ? 'success' : 'neutral'} size="xs">
                {task.isEnabled ? 'Active' : 'Paused'}
              </Badge>
            </div>

            <!-- Details Box -->
            <div class="bg-neutral-50 dark:bg-neutral-950/60 rounded-xl p-3 border border-neutral-100 dark:border-neutral-800/60 text-xs space-y-1.5 font-mono">
              <div class="flex items-center justify-between text-neutral-600 dark:text-neutral-400">
                <span class="text-[10px] uppercase font-sans font-semibold text-neutral-400">Target Host:</span>
                <span class="font-medium text-neutral-900 dark:text-white">{hostName(task.hostId)}</span>
              </div>
              {#if task.taskType === 'sftp_backup'}
                <div class="flex items-center justify-between text-neutral-600 dark:text-neutral-400 truncate">
                  <span class="text-[10px] uppercase font-sans font-semibold text-neutral-400 shrink-0">Remote Path:</span>
                  <span class="truncate ml-2 text-indigo-600 dark:text-indigo-400">{task.remoteSrcPath}</span>
                </div>
                <div class="flex items-center justify-between text-neutral-600 dark:text-neutral-400">
                  <span class="text-[10px] uppercase font-sans font-semibold text-neutral-400">Retensi Lokal:</span>
                  <span>{task.retentionCount} file terakhir</span>
                </div>
              {/if}
            </div>
          </div>

          <!-- Footer Actions -->
          <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center justify-between text-xs">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onclick={() => openLogModal(task)}
              class="flex items-center gap-1.5"
            >
              <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
              <span>Riwayat Log</span>
            </Button>

            <div class="flex items-center gap-1.5">
              <Button
                type="button"
                variant="secondary"
                size="sm"
                onclick={() => handleRunNow(task)}
                disabled={isExecutingTaskId === task.id}
                loading={isExecutingTaskId === task.id}
                class="flex items-center gap-1.5"
              >
                <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
                <span>Run Now</span>
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onclick={() => openEditModal(task)}
              >
                Edit
              </Button>
              <Button
                type="button"
                variant="danger"
                size="sm"
                onclick={() => handleDeleteTask(task)}
              >
                Hapus
              </Button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Modal Create / Edit Task -->
<Modal
  bind:open={isAddModalOpen}
  title={editingId ? 'Edit Tugas Terjadwal' : 'Tambah Tugas Terjadwal Baru'}
  size="lg"
>
  <div class="space-y-3 text-xs">
    <div>
      <label for="task-name-input" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Nama Tugas *</label>
      <Input
        id="task-name-input"
        type="text"
        bind:value={formName}
        placeholder="misal: Daily Backup Database VPS Hostinger"
        size="sm"
      />
    </div>

    <div>
      <span class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Tipe Tugas</span>
      <div class="grid grid-cols-2 gap-2">
        <Button
          type="button"
          variant={formTaskType === 'sftp_backup' ? 'brand' : 'outline'}
          size="sm"
          onclick={() => (formTaskType = 'sftp_backup')}
          class="justify-start flex items-center gap-1.5"
        >
          <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          <span>SFTP Auto-Backup</span>
        </Button>
        <Button
          type="button"
          variant={formTaskType === 'ssh_command' ? 'primary' : 'outline'}
          size="sm"
          onclick={() => (formTaskType = 'ssh_command')}
          class="justify-start flex items-center gap-1.5"
        >
          <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
          <span>SSH Script Playbook</span>
        </Button>
      </div>
    </div>

    <div>
      <label for="task-host-select" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Target Server (Host)</label>
      <select
        id="task-host-select"
        bind:value={formHostId}
        class="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 text-xs"
      >
        {#each hosts as h}
          <option value={h.id}>{h.label} ({h.address})</option>
        {/each}
      </select>
    </div>

    <div>
      <label for="task-schedule-input" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Jadwal Eksekusi (Cron / Interval) *</label>
      <Input
        id="task-schedule-input"
        type="text"
        bind:value={formScheduleExpr}
        placeholder="misal: 0 2 * * * (Jam 02:00 WIB) atau every 6h"
        size="sm"
        class="font-mono"
      />
    </div>

    {#if formTaskType === 'sftp_backup'}
      <div>
        <label for="task-remote-src-input" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Remote File Path *</label>
        <Input
          id="task-remote-src-input"
          type="text"
          bind:value={formRemoteSrcPath}
          placeholder="/var/backups/db.sql.gz"
          size="sm"
          class="font-mono"
        />
      </div>

      <div>
        <label for="task-local-dest-input" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Folder Penyimpanan Lokal *</label>
        <Input
          id="task-local-dest-input"
          type="text"
          bind:value={formLocalDestDir}
          placeholder="/home/cecepazhar/Backups"
          size="sm"
          class="font-mono"
        />
      </div>

      <div>
        <label for="task-remote-precmd-input" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Pre-command Dump di Server (Opsional)</label>
        <Input
          id="task-remote-precmd-input"
          type="text"
          bind:value={formRemotePreCmd}
          placeholder="mysqldump -u root mydb | gzip > /var/backups/db.sql.gz"
          size="sm"
          class="font-mono"
        />
      </div>
    {:else}
      <div>
        <label for="task-command-script" class="block font-medium text-neutral-700 dark:text-neutral-300 mb-1">Bash Script Perintah *</label>
        <textarea
          id="task-command-script"
          bind:value={formCommandScript}
          rows="5"
          placeholder="#!/usr/bin/env bash&#10;docker system prune -f&#10;"
          class="w-full px-3 py-2 font-mono rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 text-xs"
        ></textarea>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <div class="flex justify-end gap-2.5">
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onclick={() => (isAddModalOpen = false)}
      >
        Batal
      </Button>
      <Button
        type="button"
        variant="primary"
        size="sm"
        onclick={handleSaveTask}
        disabled={isLoading}
        loading={isLoading}
      >
        Simpan Jadwal
      </Button>
    </div>
  {/snippet}
</Modal>

<!-- Modal Log Inspector -->
<Modal
  bind:open={isLogModalOpen}
  title={`Riwayat Log: ${activeTaskForLogs?.name || ''}`}
  size="xl"
>
  {#if activeTaskLogs.length === 0}
    <div class="text-center py-8 text-neutral-500 text-xs">
      Belum ada riwayat eksekusi untuk tugas ini.
    </div>
  {:else}
    <div class="space-y-3 text-xs max-h-[60vh] overflow-y-auto">
      {#each activeTaskLogs as log}
        <div class="p-3 rounded-xl bg-neutral-950 border border-neutral-800 font-mono text-[11px] text-neutral-300 space-y-1">
          <div class="flex items-center justify-between text-neutral-400">
            <span>{log.startedAt}</span>
            <Badge variant={log.status === 'success' ? 'success' : 'danger'} size="xs">
              {log.status.toUpperCase()} ({log.durationMs || 0}ms)
            </Badge>
          </div>
          {#if log.stdout}
            <div class="mt-2 text-neutral-200 whitespace-pre-wrap">{log.stdout}</div>
          {/if}
          {#if log.errorMessage}
            <div class="mt-1 text-rose-400">{log.errorMessage}</div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#snippet footer()}
    <div class="flex justify-end">
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onclick={() => (isLogModalOpen = false)}
      >
        Tutup
      </Button>
    </div>
  {/snippet}
</Modal>
