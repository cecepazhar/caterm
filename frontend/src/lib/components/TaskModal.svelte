<script lang="ts">
  import { Button, Modal } from '@cecepazhar/caui';
  import { t } from '$lib/i18n/index.svelte';
  import type { HostRecord } from '$lib/api/hosts';
  import type { TaskInput, TaskType, ScheduledTaskRecord } from '$lib/api/tasks';

  let {
    task = null,
    hosts = [],
    onClose,
    onSave
  }: {
    task?: ScheduledTaskRecord | null;
    hosts: HostRecord[];
    onClose: () => void;
    onSave: (task: TaskInput) => Promise<void>;
  } = $props();

  let open = $state(true);
  let name = $state(task?.name ?? '');
  let description = $state(task?.description ?? '');
  let taskType = $state<TaskType>(task?.taskType ?? 'ssh_command');
  let hostId = $state(task?.hostId ?? (hosts.length > 0 ? hosts[0].id : ''));
  let scheduleExpr = $state(task?.scheduleExpr ?? '0 2 * * *');
  let isEnabled = $state(task?.isEnabled ?? true);

  // SSH fields
  let commandScript = $state(task?.commandScript ?? '#!/bin/bash\n# Scheduled script\necho "Running periodic maintenance..."\ndate\n');
  let timeoutSeconds = $state(task?.timeoutSeconds ?? 300);

  // SFTP fields
  let remoteSrcPath = $state(task?.remoteSrcPath ?? '/tmp/db_backup.sql.gz');
  let localDestDir = $state(task?.localDestDir ?? '~/Backups/caterm');
  let remotePreCmd = $state(task?.remotePreCmd ?? 'mysqldump -u root --all-databases | gzip > /tmp/db_backup.sql.gz');
  let remotePostCmd = $state(task?.remotePostCmd ?? 'rm -f /tmp/db_backup.sql.gz');
  let retentionCount = $state(task?.retentionCount ?? 7);

  // Notifications
  let notifyOnSuccess = $state(task?.notifyOnSuccess ?? false);
  let notifyOnFailure = $state(task?.notifyOnFailure ?? true);

  let isSaving = $state(false);
  let formError = $state('');

  // Preset Cron Generator options
  const cronPresets = [
    { label: 'Every hour (Setiap jam)', expr: '0 * * * *' },
    { label: 'Every 6 hours (Setiap 6 jam)', expr: '0 */6 * * *' },
    { label: 'Every 12 hours (Setiap 12 jam)', expr: '0 */12 * * *' },
    { label: 'Every day at 02:00 AM (Setiap hari jam 02:00)', expr: '0 2 * * *' },
    { label: 'Every day at midnight (Setiap tengah malam)', expr: '0 0 * * *' },
    { label: 'Every Sunday at 03:00 AM (Setiap Minggu jam 03:00)', expr: '0 3 * * 0' },
    { label: 'Every 1st of month (Setiap tanggal 1)', expr: '0 0 1 * *' },
    { label: 'Interval: Every 30 minutes', expr: 'every 30m' },
    { label: 'Interval: Every 2 hours', expr: 'every 2h' },
    { label: 'Interval: Every 24 hours', expr: 'every 24h' }
  ];

  function humanizeCron(expr: string): string {
    const trimmed = expr.trim();
    if (!trimmed) return 'Invalid expression';

    if (trimmed.startsWith('every ')) {
      const interval = trimmed.replace('every ', '').trim();
      return `Runs automatically every ${interval}`;
    }

    if (trimmed === '0 2 * * *') return 'Runs everyday at 02:00 AM';
    if (trimmed === '0 0 * * *') return 'Runs everyday at midnight (00:00)';
    if (trimmed === '0 * * * *') return 'Runs at minute 0 of every hour';
    if (trimmed === '0 */6 * * *') return 'Runs every 6 hours (00:00, 06:00, 12:00, 18:00)';
    if (trimmed === '0 */12 * * *') return 'Runs every 12 hours (00:00, 12:00)';
    if (trimmed === '0 3 * * 0') return 'Runs every Sunday at 03:00 AM';
    if (trimmed === '0 0 1 * *') return 'Runs on the 1st of every month at midnight';

    const parts = trimmed.split(/\s+/);
    if (parts.length === 5) {
      const [m, h, dom, mon, dow] = parts;
      if (dom === '*' && mon === '*' && dow === '*' && !m.includes('*') && !h.includes('*')) {
        return `Runs everyday at ${h.padStart(2, '0')}:${m.padStart(2, '0')}`;
      }
      return `Standard Cron: [min: ${m}, hour: ${h}, day-of-month: ${dom}, month: ${mon}, day-of-week: ${dow}]`;
    }

    return `Schedule expression: ${trimmed}`;
  }

  function handleClose() {
    open = false;
    onClose();
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!name.trim()) {
      formError = t('tasks.nameRequired');
      return;
    }
    if (!scheduleExpr.trim()) {
      formError = t('tasks.scheduleRequired');
      return;
    }

    formError = '';
    isSaving = true;

    try {
      const payload: TaskInput = {
        id: task?.id,
        name: name.trim(),
        description: description.trim() || undefined,
        taskType: taskType,
        hostId: hostId || undefined,
        scheduleExpr: scheduleExpr.trim(),
        isEnabled: isEnabled,
        commandScript: taskType === 'ssh_command' ? commandScript : undefined,
        timeoutSeconds: Number(timeoutSeconds),
        remoteSrcPath: taskType === 'sftp_backup' ? remoteSrcPath.trim() : undefined,
        localDestDir: taskType === 'sftp_backup' ? localDestDir.trim() : undefined,
        remotePreCmd: taskType === 'sftp_backup' ? remotePreCmd.trim() || undefined : undefined,
        remotePostCmd: taskType === 'sftp_backup' ? remotePostCmd.trim() || undefined : undefined,
        retentionCount: Number(retentionCount),
        notifyOnSuccess: notifyOnSuccess,
        notifyOnFailure: notifyOnFailure
      };

      await onSave(payload);
      handleClose();
    } catch (err: any) {
      formError = err?.message || String(err);
    } finally {
      isSaving = false;
    }
  }
</script>

<Modal
  bind:open={open}
  onclose={handleClose}
  title={task ? t('tasks.editTask') : t('tasks.newTask')}
  description={t('tasks.modalSubtitle')}
  size="xl"
>
  <form id="task-modal-form" onsubmit={handleSubmit} class="space-y-5">
    {#if formError}
      <div class="p-3 text-xs rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400">
        {formError}
      </div>
    {/if}

      <!-- Task Type Tabs -->
      <div>
        <span class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-2">
          {t('tasks.taskType')}
        </span>
        <div class="grid grid-cols-2 gap-3">
          <Button
            type="button"
            variant={taskType === 'ssh_command' ? 'primary' : 'outline'}
            onclick={() => (taskType = 'ssh_command')}
            class={`p-3.5 rounded-xl border text-left transition flex items-start justify-start gap-3 w-full h-auto font-normal ${
              taskType === 'ssh_command'
                ? 'bg-purple-500/10 dark:bg-purple-500/20 border-purple-500/50 text-purple-900 dark:text-purple-200 shadow-sm'
                : 'bg-neutral-50 dark:bg-neutral-900/50 border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-400 hover:border-neutral-300 dark:hover:border-neutral-700'
            }`}
          >
            <span class="text-xl">⚡</span>
            <div>
              <div class="text-xs font-bold text-neutral-900 dark:text-white flex items-center gap-1.5">
                {t('tasks.typeSshScript')}
              </div>
              <div class="text-[11px] text-neutral-500 dark:text-neutral-400 mt-0.5">
                {t('tasks.typeSshDesc')}
              </div>
            </div>
          </Button>

          <Button
            type="button"
            variant={taskType === 'sftp_backup' ? 'primary' : 'outline'}
            onclick={() => (taskType = 'sftp_backup')}
            class={`p-3.5 rounded-xl border text-left transition flex items-start justify-start gap-3 w-full h-auto font-normal ${
              taskType === 'sftp_backup'
                ? 'bg-purple-500/10 dark:bg-purple-500/20 border-purple-500/50 text-purple-900 dark:text-purple-200 shadow-sm'
                : 'bg-neutral-50 dark:bg-neutral-900/50 border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-400 hover:border-neutral-300 dark:hover:border-neutral-700'
            }`}
          >
            <span class="text-xl">📦</span>
            <div>
              <div class="text-xs font-bold text-neutral-900 dark:text-white flex items-center gap-1.5">
                {t('tasks.typeSftpBackup')}
              </div>
              <div class="text-[11px] text-neutral-500 dark:text-neutral-400 mt-0.5">
                {t('tasks.typeSftpDesc')}
              </div>
            </div>
          </Button>
        </div>
      </div>

      <!-- Basic Info: Name & Target Host -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label for="task-name" class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-1.5">
            {t('tasks.taskName')} *
          </label>
          <input
            id="task-name"
            type="text"
            bind:value={name}
            placeholder="e.g. Daily DB Backup & Vacuum"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-xl text-xs text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-purple-500"
            required
          />
        </div>

        <div>
          <label for="task-host" class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-1.5">
            {t('tasks.targetHost')}
          </label>
          <select
            id="task-host"
            bind:value={hostId}
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-xl text-xs text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
          >
            <option value="">{t('tasks.noHostLocal')}</option>
            {#each hosts as h}
              <option value={h.id}>{h.label} ({h.username}@{h.address}:{h.port})</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Description -->
      <div>
        <label for="task-desc" class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-1.5">
          {t('tasks.taskDescOptional')}
        </label>
        <input
          id="task-desc"
          type="text"
          bind:value={description}
          placeholder="Brief summary of what this automated task does"
          class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-xl text-xs text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:border-purple-500"
        />
      </div>

      <!-- Schedule / Cron Expression Section -->
      <div class="p-4 rounded-xl bg-purple-500/5 dark:bg-purple-500/10 border border-purple-500/20 space-y-3">
        <div class="flex items-center justify-between">
          <label for="task-schedule" class="text-xs font-bold text-neutral-900 dark:text-white flex items-center gap-1.5">
            <svg class="w-4 h-4 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            {t('tasks.scheduleExpression')} *
          </label>
          <span class="text-[11px] text-purple-600 dark:text-purple-400 font-mono">
            {humanizeCron(scheduleExpr)}
          </span>
        </div>

        <div class="flex gap-2">
          <input
            id="task-schedule"
            type="text"
            bind:value={scheduleExpr}
            placeholder="0 2 * * * or every 6h"
            class="flex-1 px-3 py-2 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs font-mono text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
          />
          <select
            onchange={(e) => {
              const val = (e.target as HTMLSelectElement).value;
              if (val) scheduleExpr = val;
            }}
            class="px-2.5 py-2 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs text-neutral-700 dark:text-neutral-300 focus:outline-none focus:border-purple-500"
          >
            <option value="">{t('tasks.quickPresets')}</option>
            {#each cronPresets as p}
              <option value={p.expr}>{p.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- TYPE-SPECIFIC CONFIGURATION -->
      {#if taskType === 'ssh_command'}
        <!-- SSH Script Playbook Configuration -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <label for="task-script" class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
              {t('tasks.bashScript')} *
            </label>
            <div class="flex items-center gap-2">
              <label for="task-timeout" class="text-[11px] text-neutral-500 dark:text-neutral-400">
                {t('tasks.timeoutSec')}:
              </label>
              <input
                id="task-timeout"
                type="number"
                min="10"
                max="86400"
                bind:value={timeoutSeconds}
                class="w-20 px-2 py-1 bg-neutral-50 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-md text-xs font-mono text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
              />
            </div>
          </div>

          <textarea
            id="task-script"
            rows="6"
            bind:value={commandScript}
            placeholder="#!/bin/bash&#10;docker system prune -f&#10;systemctl restart my-service"
            class="w-full px-3 py-2.5 bg-neutral-900 text-emerald-400 font-mono text-xs rounded-xl border border-neutral-800 focus:outline-none focus:border-purple-500 selection:bg-purple-900"
          ></textarea>
        </div>
      {:else}
        <!-- SFTP Pull Backup Configuration -->
        <div class="space-y-4 p-4 rounded-xl bg-neutral-50 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
          <div class="text-xs font-bold text-neutral-900 dark:text-white flex items-center gap-1.5">
            <span>📦</span> {t('tasks.sftpPipelineTitle')}
          </div>

          <!-- Step 1: Remote Pre-Command -->
          <div>
            <label for="task-pre-cmd" class="block text-[11px] font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
              1. {t('tasks.sftpPreCmd')}
            </label>
            <input
              id="task-pre-cmd"
              type="text"
              bind:value={remotePreCmd}
              placeholder="e.g. mysqldump -u root mydb | gzip > /tmp/db.sql.gz"
              class="w-full px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs font-mono text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
            />
          </div>

          <!-- Step 2: Remote Source & Local Destination -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <label for="task-src-path" class="block text-[11px] font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
                2. {t('tasks.sftpSrcPath')} *
              </label>
              <input
                id="task-src-path"
                type="text"
                bind:value={remoteSrcPath}
                placeholder="/tmp/db_backup.sql.gz"
                class="w-full px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs font-mono text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
                required
              />
            </div>

            <div>
              <label for="task-dest-dir" class="block text-[11px] font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
                3. {t('tasks.sftpDestDir')} *
              </label>
              <input
                id="task-dest-dir"
                type="text"
                bind:value={localDestDir}
                placeholder="/home/user/Backups"
                class="w-full px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs font-mono text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
                required
              />
            </div>
          </div>

          <!-- Step 3: Remote Post-Command & Retention -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <label for="task-post-cmd" class="block text-[11px] font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
                4. {t('tasks.sftpPostCmd')}
              </label>
              <input
                id="task-post-cmd"
                type="text"
                bind:value={remotePostCmd}
                placeholder="rm -f /tmp/db_backup.sql.gz"
                class="w-full px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs font-mono text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
              />
            </div>

            <div>
              <label for="task-retention" class="block text-[11px] font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
                5. {t('tasks.retentionPolicy')}
              </label>
              <div class="flex items-center gap-2">
                <input
                  id="task-retention"
                  type="number"
                  min="1"
                  max="100"
                  bind:value={retentionCount}
                  class="w-20 px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs font-mono text-neutral-900 dark:text-white focus:outline-none focus:border-purple-500"
                />
                <span class="text-xs text-neutral-500 dark:text-neutral-400">{t('tasks.retentionCopies')}</span>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Toggles: Enable & Notifications -->
      <div class="pt-2 border-t border-neutral-200 dark:border-neutral-800 grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
        <label class="flex items-center gap-2 cursor-pointer text-neutral-700 dark:text-neutral-300">
          <input type="checkbox" bind:checked={isEnabled} class="rounded text-purple-600 focus:ring-purple-500" />
          <span class="font-medium">{t('tasks.enableTask')}</span>
        </label>

        <label class="flex items-center gap-2 cursor-pointer text-neutral-700 dark:text-neutral-300">
          <input type="checkbox" bind:checked={notifyOnFailure} class="rounded text-purple-600 focus:ring-purple-500" />
          <span>{t('tasks.notifyFailure')}</span>
        </label>

        <label class="flex items-center gap-2 cursor-pointer text-neutral-700 dark:text-neutral-300">
          <input type="checkbox" bind:checked={notifyOnSuccess} class="rounded text-purple-600 focus:ring-purple-500" />
          <span>{t('tasks.notifySuccess')}</span>
        </label>
      </div>
    </form>

    {#snippet footer()}
      <div class="flex items-center justify-end gap-3 w-full">
        <Button
          variant="secondary"
          size="sm"
          onclick={handleClose}
        >
          {t('common.cancel')}
        </Button>

        <Button
          type="button"
          variant="primary"
          size="sm"
          disabled={isSaving}
          loading={isSaving}
          onclick={handleSubmit}
        >
          {task ? t('common.update') : t('common.create')}
        </Button>
      </div>
    {/snippet}
  </Modal>
