<script lang="ts">
  import type { BlastShieldRisk } from '$lib/api/proSecurity';
  import { Button, Modal } from '@cecepazhar/caui';

  let {
    isOpen = $bindable(false),
    command = '',
    risk = null,
    onConfirm,
    onCancel
  }: {
    isOpen: boolean;
    command: string;
    risk: BlastShieldRisk | null;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  let isBlocked = $derived(risk?.level === 'blocked');
</script>

<Modal
  bind:open={isOpen}
  onclose={onCancel}
  title="CATerm Blast Shield Intercept"
  description="Potentially destructive command detected before PTY dispatch"
  size="lg"
>
  {#if risk}
    <div class="space-y-4 text-sm text-neutral-300">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <div>
          <span class="inline-block text-xs px-2 py-0.5 rounded bg-red-500/20 text-red-300 font-mono uppercase tracking-wide">
            {risk.level}
          </span>
        </div>
      </div>

      <div class="p-3 rounded-lg bg-black/40 border border-neutral-800 font-mono text-xs text-neutral-200 break-all overflow-x-auto">
        <code>{command}</code>
      </div>

      <div class="space-y-2 text-xs">
        <div class="p-3 rounded-lg bg-red-950/30 border border-red-800/30 text-red-200/90 leading-relaxed">
          <span class="font-semibold text-red-300">Risk Analysis:</span> {risk.reason}
        </div>

        {#if risk.suggestedAlternative}
          <div class="p-3 rounded-lg bg-neutral-900 border border-neutral-800 text-neutral-300 leading-relaxed">
            <span class="font-semibold text-neutral-200">Recommendation:</span> {risk.suggestedAlternative}
          </div>
        {/if}
      </div>

      {#if isBlocked}
        <div class="text-[11px] text-red-400/90 font-mono italic">
          Execution strictly blocked for system safety.
        </div>
      {/if}
    </div>
  {/if}

  {#snippet footer()}
    <div class="flex items-center justify-end gap-3">
      <Button
        variant="secondary"
        size="sm"
        onclick={onCancel}
      >
        Abort Execution
      </Button>

      {#if !isBlocked}
        <Button
          variant="danger"
          size="sm"
          onclick={onConfirm}
        >
          I Understand, Execute Anyway
        </Button>
      {/if}
    </div>
  {/snippet}
</Modal>
