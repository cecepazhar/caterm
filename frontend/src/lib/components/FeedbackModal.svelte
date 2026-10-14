<script lang="ts">
  import { Button, Modal } from '@cecepazhar/caui';
  import { t } from '$lib/i18n/index.svelte';
  import { submitFeedback } from '$lib/api/feedback';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';

  const STORAGE_KEY_PREFIX = 'caterm_feedback_state';
  const APP_VERSION = 'v2';

  let {
    onClose,
    onSubmitted
  }: {
    onClose: () => void;
    onSubmitted?: () => void;
  } = $props();

  let rating = $state(5);
  let hoverRating = $state(0);
  let name = $state('');
  let profession = $state('');
  let content = $state('');
  let isSubmitting = $state(false);
  let submitted = $state(false);
  let errorMessage = $state('');
  let open = $state(true);

  // Label for the currently hovered/selected star count (`feedback.rating.1` … `.5`).
  function ratingLabel(stars: number): string {
    return stars >= 1 && stars <= 5 ? t(`feedback.rating.${stars}`) : '';
  }

  function storageKey(action: 'submitted' | 'dismissed'): string {
    return `${STORAGE_KEY_PREFIX}_${APP_VERSION}_${action}`;
  }

  function handleDismiss() {
    try {
      localStorage.setItem(storageKey('dismissed'), 'true');
    } catch {
      // LocalStorage access might fail in private mode
    }
    open = false;
    onClose();
  }

  async function handleSubmit(event: Event) {
    event.preventDefault();
    if (rating < 1 || rating > 5) {
      errorMessage = t('feedback.errRating');
      return;
    }
    const trimmed = content.trim();
    if (trimmed.length > 0 && trimmed.length < 5) {
      errorMessage = t('feedback.errContentLength') || 'Minimal 5 karakter';
      return;
    }

    isSubmitting = true;
    errorMessage = '';

    try {
      const effectiveContent = trimmed.length > 0
        ? trimmed
        : `${rating} stars (no notes)`;

      await submitFeedback(rating, effectiveContent, name, profession);

      try {
        localStorage.setItem(storageKey('submitted'), 'true');
      } catch {
        // Fallback
      }

      submitted = true;
      showToast(t('feedback.thanks'), 'success');
      onSubmitted?.();
    } catch (err) {
      const msg = errorText(err);
      errorMessage = msg;
      showToast(msg, 'error');
    } finally {
      isSubmitting = false;
    }
  }

  function handleReset() {
    submitted = false;
    name = '';
    profession = '';
    content = '';
    rating = 5;
    errorMessage = '';
  }
</script>

<Modal
  bind:open={open}
  onclose={handleDismiss}
  title={t('feedback.title')}
  description={t('feedback.subtitle')}
  size="lg"
>
  {#if submitted}
    <div class="p-5 bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/60 rounded-xl space-y-3">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-emerald-500/10 rounded-full text-emerald-600 dark:text-emerald-400 shrink-0">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <div>
          <h3 class="text-sm font-semibold text-neutral-900 dark:text-white">{t('feedback.submittedTitle')}</h3>
          <p class="text-xs text-neutral-600 dark:text-neutral-300 mt-0.5 leading-relaxed">
            {t('feedback.submittedBody')}
          </p>
        </div>
      </div>
    </div>
  {:else}
    <form id="feedback-form" onsubmit={handleSubmit} class="space-y-4 pt-1">
      <!-- Star Rating -->
      <div class="space-y-1.5">
        <label for="feedback-stars-modal" class="block text-xs font-semibold text-neutral-600 dark:text-neutral-400 uppercase tracking-wider">
          {t('feedback.ratingLabel')}
        </label>
        <div id="feedback-stars-modal" class="flex items-center gap-2">
          <div class="flex items-center gap-1" role="radiogroup" aria-label={t('feedback.starRating')}>
            {#each [1, 2, 3, 4, 5] as star}
              <Button
                variant="ghost"
                size="icon"
                type="button"
                role="radio"
                aria-checked={rating === star}
                aria-label={t('feedback.starsOutOf', { count: star })}
                onclick={() => (rating = star)}
                onmouseenter={() => (hoverRating = star)}
                onmouseleave={() => (hoverRating = 0)}
                class="!p-1 !h-auto !w-auto text-neutral-300 dark:text-neutral-700 hover:scale-110 transition-transform focus:outline-none focus:ring-1 focus:ring-sky-500 rounded cursor-pointer"
              >
                <svg
                  class="w-7 h-7 transition-colors {(hoverRating || rating) >= star ? 'text-amber-400 fill-amber-400' : 'text-neutral-300 dark:text-neutral-700'}"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
                  />
                </svg>
              </Button>
            {/each}
          </div>
          <span class="text-xs font-medium text-neutral-600 dark:text-neutral-400 ml-2">
            {ratingLabel(hoverRating || rating)}
          </span>
        </div>
      </div>

      <!-- Name & Profession Inputs -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div class="space-y-1.5">
          <label for="feedback-name-modal" class="block text-xs font-semibold text-neutral-600 dark:text-neutral-400 uppercase tracking-wider">
            {t('feedback.nameLabel')} <span class="text-neutral-400 font-normal normal-case">{t('feedback.optional')}</span>
          </label>
          <input
            id="feedback-name-modal"
            type="text"
            bind:value={name}
            maxlength="100"
            placeholder={t('feedback.namePlaceholder')}
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500"
          />
        </div>

        <div class="space-y-1.5">
          <label for="feedback-profession-modal" class="block text-xs font-semibold text-neutral-600 dark:text-neutral-400 uppercase tracking-wider">
            {t('feedback.activityLabel')} <span class="text-neutral-400 font-normal normal-case">{t('feedback.optional')}</span>
          </label>
          <input
            id="feedback-profession-modal"
            type="text"
            bind:value={profession}
            maxlength="100"
            placeholder={t('feedback.activityPlaceholder')}
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500"
          />
        </div>
      </div>

      <!-- Textarea -->
      <div class="space-y-1.5">
        <label for="feedback-content-modal" class="block text-xs font-semibold text-neutral-600 dark:text-neutral-400 uppercase tracking-wider">
          {t('feedback.messageLabel')} <span class="text-neutral-400 font-normal normal-case">{t('feedback.optional')}</span>
        </label>
        <textarea
          id="feedback-content-modal"
          bind:value={content}
          rows="4"
          maxlength="2000"
          placeholder={t('feedback.messagePlaceholder')}
          class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500 resize-y"
        ></textarea>
        <div class="flex justify-between items-center text-[11px] text-neutral-500 dark:text-neutral-400">
          <span>{t('feedback.minChars') || 'Minimal 5 karakter'} · {t('feedback.maxChars')}</span>
          <span>{content.length} / 2000</span>
        </div>
      </div>

      {#if errorMessage}
        <div class="p-3 bg-red-50 dark:bg-red-950/40 border border-red-200 dark:border-red-800/60 rounded-lg text-xs text-red-600 dark:text-red-400">
          {errorMessage}
        </div>
      {/if}

      <!-- Moderation Note -->
      <div class="p-2.5 bg-neutral-50 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800 rounded-lg text-[11px] text-neutral-500 dark:text-neutral-400 flex items-start gap-2">
        <svg class="w-3.5 h-3.5 text-neutral-400 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p>
          {t('feedback.moderationNote')}
        </p>
      </div>
    </form>
  {/if}

  {#snippet footer()}
    {#if submitted}
      <div class="flex items-center justify-between w-full">
        <Button
          variant="ghost"
          size="sm"
          onclick={handleReset}
        >
          {t('feedback.sendAnother')}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onclick={handleDismiss}
        >
          {t('feedback.done')}
        </Button>
      </div>
    {:else}
      <div class="flex items-center justify-end gap-3 w-full">
        <Button
          variant="secondary"
          size="sm"
          onclick={handleDismiss}
        >
          {t('common.cancel')}
        </Button>

        <Button
          type="button"
          variant="primary"
          size="sm"
          disabled={isSubmitting}
          loading={isSubmitting}
          onclick={handleSubmit}
        >
          <span>{isSubmitting ? t('feedback.sending') : t('feedback.send')}</span>
        </Button>
      </div>
    {/if}
  {/snippet}
</Modal>
