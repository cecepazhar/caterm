<script lang="ts">
  import { onMount } from 'svelte';
  import { Button, Modal } from '@cecepazhar/caui';
  import { t } from '$lib/i18n/index.svelte';
  import { getTaskExecutionLogs, type TaskExecutionLog, type ScheduledTaskRecord } from '$lib/api/tasks';

  let {
    task,
    onClose
  }: {
    task: ScheduledTaskRecord;
    onClose: () => void;
  } = $props();

  let open = $state(true);
  let logs = $state<TaskExecutionLog[]>([]);
  let selectedLog = $state<TaskExecutionLog | null>(null);
  let isLoading = $state(true);
  let errorMsg = $state('');

  function handleClose() {
    open = false;
    onClose();
  }

  async function loadLogs() {
    isLoading = true;
    errorMsg = '';
    try {
      logs = await getTaskExecutionLogs(task.id);
      if (logs.length > 0) {
        selectedLog = logs[0];
      }
    } catch (e: any) {
      errorMsg = e?.message || String(e);
    } finally {
      isLoading = false;
    }
  }

  onMount(loadLogs);

  function formatBytes(bytes?: number): string {
    if (bytes === undefined || bytes === null || bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let val = bytes;
    let unitIdx = 0;
    while (val >= 1024 && unitIdx < units.length - 1) {
      val /= 1024;
      unitIdx++;
    }
    return `${val.toFixed(1)} ${units[unitIdx]}`;
  }

  function formatDuration(ms?: number): string {
    if (ms === undefined || ms === null) return '-';
    if (ms < 1000) return `${ms}ms`;
    const sec = (ms / 1000).toFixed(1);
    return `${sec}s`;
  }

  function getStatusBadge(status: string) {
    switch (status) {
      case 'success':
        return { bg: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30', label: 'Success' };
      case 'failed':
        return { bg: 'bg-rose-500/10 text-rose-600 dark:text-rose-400 border-rose-500/30', label: 'Failed' };
      case 'timeout':
        return { bg: 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30', label: 'Timeout' };
      case 'running':
        return { bg: 'bg-sky-500/10 text-sky-600 dark:text-sky-400 border-sky-500/30 animate-pulse', label: 'Running' };
      default:
        return { bg: 'bg-neutral-500/10 text-neutral-500 border-neutral-500/30', label: status };
    }
  }
</script>

<Modal
  bind:open={open}
  onclose={handleClose}
  title={`${t('tasks.executionLogs')}: ${task.name}`}
  description={`${task.taskType === 'ssh_command' ? 'SSH Script Playbook' : 'SFTP Pull Auto-Backup'} · ${task.scheduleExpr}`}
  size="xl"
  class="!max-w-4xl"
>
  <!-- Main Container: 2-column log inspector -->
  <div class="-mx-6 -my-5 h-[65vh] flex min-h-0 divide-x divide-neutral-200 dark:divide-neutral-800">
    <!-- Left sidebar: Run history list -->
    <div class="w-1/3 min-w-[240px] max-w-[320px] overflow-y-auto bg-neutral-50/50 dark:bg-neutral-900/20 divide-y divide-neutral-200/60 dark:divide-neutral-800/60">
      {#if isLoading && logs.length === 0}
        <div class="p-6 text-center text-xs text-neutral-400">
          <svg class="w-5 h-5 animate-spin mx-auto mb-2 text-purple-500" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
          {t('common.loading')}
        </div>
      {:else if errorMsg}
        <div class="p-4 text-xs text-rose-500">
          {errorMsg}
        </div>
      {:else if logs.length === 0}
        <div class="p-6 text-center text-xs text-neutral-400">
          {t('tasks.noLogsYet')}
        </div>
      {:else}
        {#each logs as log}
          {@const badge = getStatusBadge(log.status)}
          <Button
            type="button"
            variant="ghost"
            onclick={() => (selectedLog = log)}
            class={`w-full p-3 text-left justify-start items-stretch flex-col gap-1.5 rounded-none h-auto font-normal ${
              selectedLog?.id === log.id
                ? 'bg-purple-500/10 dark:bg-purple-500/20 border-l-2 border-purple-500 text-neutral-900 dark:text-white'
                : 'hover:bg-neutral-100 dark:hover:bg-neutral-900/60 text-neutral-600 dark:text-neutral-400'
            }`}
          >
            <div class="flex items-center justify-between gap-1 w-full">
              <span class={`text-[10px] px-1.5 py-0.5 rounded-full font-semibold border ${badge.bg}`}>
                {badge.label}
              </span>
              <span class="text-[10px] font-mono text-neutral-400">
                {formatDuration(log.durationMs)}
              </span>
            </div>
            <div class="text-xs font-mono font-medium truncate text-neutral-900 dark:text-neutral-200">
              {log.startedAt}
            </div>
            {#if log.bytesTransferred}
              <div class="text-[10px] text-purple-600 dark:text-purple-400">
                Transferred: {formatBytes(log.bytesTransferred)}
              </div>
            {/if}
          </Button>
        {/each}
      {/if}
    </div>

    <!-- Right area: Detailed Log & Output View (Dark Terminal Style) -->
    <div class="flex-1 flex flex-col min-w-0 bg-[#0c0d10] text-neutral-200 font-mono text-xs">
      {#if selectedLog}
        <!-- Execution summary header -->
        <div class="px-5 py-3 bg-[#13151b] border-b border-neutral-800/80 flex flex-wrap items-center justify-between gap-3 text-[11px] shrink-0">
          <div class="flex items-center gap-3">
            <div>
              <span class="text-neutral-400">Started:</span> <span class="text-neutral-200">{selectedLog.startedAt}</span>
            </div>
            {#if selectedLog.finishedAt}
              <div>
                <span class="text-neutral-400">Duration:</span> <span class="text-emerald-400">{formatDuration(selectedLog.durationMs)}</span>
              </div>
            {/if}
          </div>

          <div class="flex items-center gap-3">
            {#if selectedLog.exitCode !== undefined && selectedLog.exitCode !== null}
              <div>
                <span class="text-neutral-400">Exit Code:</span>
                <span class={selectedLog.exitCode === 0 ? 'text-emerald-400 font-bold' : 'text-rose-400 font-bold'}>
                  {selectedLog.exitCode}
                </span>
              </div>
            {/if}
            {#if selectedLog.bytesTransferred}
              <div>
                <span class="text-neutral-400">Size:</span> <span class="text-purple-400">{formatBytes(selectedLog.bytesTransferred)}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Console Terminal Output Window -->
        <div class="flex-1 p-5 overflow-y-auto space-y-4 font-mono select-text selection:bg-purple-800">
          {#if selectedLog.errorMessage}
            <div class="p-3 rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-300 space-y-1">
              <div class="text-[11px] font-bold uppercase tracking-wider text-rose-400">Error Details:</div>
              <div class="whitespace-pre-wrap text-xs">{selectedLog.errorMessage}</div>
            </div>
          {/if}

          {#if selectedLog.stdout}
            <div class="space-y-1">
              <div class="text-[10px] text-neutral-500 uppercase tracking-wider">── Standard Output (stdout) ──</div>
              <pre class="text-emerald-400 whitespace-pre-wrap leading-relaxed text-xs">{selectedLog.stdout}</pre>
            </div>
          {/if}

          {#if selectedLog.stderr}
            <div class="space-y-1">
              <div class="text-[10px] text-amber-500 uppercase tracking-wider">── Standard Error (stderr) ──</div>
              <pre class="text-rose-400 whitespace-pre-wrap leading-relaxed text-xs">{selectedLog.stderr}</pre>
            </div>
          {/if}

          {#if !selectedLog.stdout && !selectedLog.stderr && !selectedLog.errorMessage}
            <div class="text-neutral-500 italic py-8 text-center">
              (No console output recorded for this run)
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex-1 flex items-center justify-center text-neutral-500 text-xs">
          {t('tasks.selectLogRun')}
        </div>
      {/if}
    </div>
  </div>

  {#snippet footer()}
    <div class="flex items-center justify-between w-full">
      <Button
        variant="ghost"
        size="sm"
        onclick={loadLogs}
        loading={isLoading}
      >
        {#if !isLoading}
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        {/if}
        <span>{t('common.refresh')}</span>
      </Button>
      <Button
        variant="secondary"
        size="sm"
        onclick={handleClose}
      >
        {t('common.close')}
      </Button>
    </div>
  {/snippet}
</Modal>