<script lang="ts">
  import { Button, Modal } from '@cecepazhar/caui';
  import { t } from '$lib/i18n/index.svelte';
  import { submitCrashReport, dismissCrashReport, type ScrubbedCrashReport } from '$lib/api/crash';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';

  let {
    report,
    onClose
  }: {
    report: ScrubbedCrashReport;
    onClose: () => void;
  } = $props();

  let open = $state(true);
  let showDetails = $state(false);
  let neverAgain = $state(false);
  let isSubmitting = $state(false);
  let isDeleting = $state(false);
  let sent = $state(false);

  function handleClose() {
    open = false;
    onClose();
  }

  async function handleSend() {
    isSubmitting = true;
    try {
      await submitCrashReport(report.id);
      sent = true;
      showToast(t('crash.sentThanks'), 'success');
      // If the person also asked to never be asked again, that preference still applies even
      // though this dump is already gone via the server-accepted delete inside submit.
      if (neverAgain) {
        await dismissCrashReport(report.id, true);
      }
      handleClose();
    } catch (err) {
      showToast(`${t('crash.sendFailed')} ${errorText(err)}`, 'error');
    } finally {
      isSubmitting = false;
    }
  }

  async function handleDelete() {
    isDeleting = true;
    try {
      await dismissCrashReport(report.id, neverAgain);
      handleClose();
    } catch (err) {
      showToast(errorText(err), 'error');
    } finally {
      isDeleting = false;
    }
  }
</script>

<Modal
  bind:open={open}
  onclose={handleClose}
  title={t('crash.title')}
  description={t('crash.subtitle')}
  size="lg"
>
  <div class="space-y-4 text-left">
    <div class="p-3 bg-neutral-50 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800 rounded-lg space-y-1">
      <p class="text-[11px] font-semibold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
        {t('crash.whatHappened')}
      </p>
      <p class="text-sm text-neutral-800 dark:text-neutral-200 break-words">{report.message}</p>
      {#if report.location}
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          {t('crash.location', { location: report.location })}
        </p>
      {/if}
    </div>

    <div>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => (showDetails = !showDetails)}
        class="text-sky-600 dark:text-sky-400 font-medium p-0 h-auto"
      >
        {showDetails ? t('crash.hideDetails') : t('crash.viewDetails')}
      </Button>
      {#if showDetails}
        <pre class="mt-2 p-3 bg-neutral-950 text-neutral-300 text-[11px] leading-relaxed rounded-lg overflow-auto max-h-56">{report.previewJson}</pre>
      {/if}
    </div>

    <label class="flex items-center gap-2 text-xs text-neutral-600 dark:text-neutral-400 cursor-pointer">
      <input type="checkbox" bind:checked={neverAgain} class="rounded border-neutral-300 dark:border-neutral-700" />
      {t('crash.neverAskAgain')}
    </label>
  </div>

  {#snippet footer()}
    <div class="flex items-center justify-end gap-3 w-full">
      <Button
        variant="secondary"
        size="sm"
        onclick={handleDelete}
        disabled={isSubmitting || isDeleting}
      >
        {t('crash.deleteAndClose')}
      </Button>
      <Button
        variant="primary"
        size="sm"
        onclick={handleSend}
        disabled={isSubmitting || isDeleting || sent}
        loading={isSubmitting}
      >
        <span>{isSubmitting ? t('crash.sending') : t('crash.send')}</span>
      </Button>
    </div>
  {/snippet}
</Modal>
