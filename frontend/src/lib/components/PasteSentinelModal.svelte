<script lang="ts">
  import type { PasteCheckResult } from '$lib/api/proSecurity';
  import { Button, Modal } from '@cecepazhar/caui';

  let {
    isOpen = $bindable(false),
    pasteResult = null,
    onPasteSanitized,
    onPasteOriginal,
    onCancel
  }: {
    isOpen: boolean;
    pasteResult: PasteCheckResult | null;
    onPasteSanitized: (text: string) => void;
    onPasteOriginal: (text: string) => void;
    onCancel: () => void;
  } = $props();
</script>

<Modal
  bind:open={isOpen}
  onclose={onCancel}
  title="CATerm Paste Sentinel Alert"
  description="Sensitive credentials detected in clipboard payload"
  size="lg"
>
  {#if pasteResult && pasteResult.containsSecret}
    <div class="space-y-4">
      <div class="flex items-center gap-2">
        <span class="text-xs px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 font-mono uppercase tracking-wide">
          {pasteResult.secrets.length} Secret{pasteResult.secrets.length > 1 ? 's' : ''} Found
        </span>
      </div>

      <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
        {#each pasteResult.secrets as secret}
          <div class="p-3 rounded-lg bg-amber-950/20 border border-amber-800/30 text-xs space-y-1">
            <div class="flex items-center justify-between font-semibold text-amber-300">
              <span>{secret.title}</span>
              <span class="font-mono text-[11px] text-amber-400/80">{secret.matchedSnippetMasked}</span>
            </div>
            <p class="text-neutral-400 text-[11px] leading-relaxed">{secret.description}</p>
          </div>
        {/each}
      </div>

      <div class="p-3 rounded-lg bg-neutral-900 border border-neutral-800 text-xs space-y-1.5 text-neutral-300">
        <div class="font-semibold text-neutral-200">Recommended Safe Action:</div>
        <p class="text-[11px] text-neutral-400 leading-relaxed">
          Prefix command with a leading whitespace (<code class="text-amber-300">HISTCONTROL=ignorespace</code>) to avoid writing secrets to shell history logs (<code class="text-neutral-400">~/.bash_history</code> / <code class="text-neutral-400">~/.zsh_history</code>).
        </p>
      </div>
    </div>
  {/if}

  {#snippet footer()}
    <div class="flex items-center justify-end gap-2 flex-wrap">
      <Button
        variant="secondary"
        size="sm"
        onclick={onCancel}
      >
        Cancel
      </Button>

      <Button
        variant="warning"
        size="sm"
        onclick={() => onPasteOriginal(pasteResult?.sanitizedText.trim() ?? '')}
      >
        Paste Raw (History Visible)
      </Button>

      <Button
        variant="primary"
        size="sm"
        onclick={() => onPasteSanitized(pasteResult?.sanitizedText ?? '')}
      >
        Paste with Space Prefix (Safe)
      </Button>
    </div>
  {/snippet}
</Modal>
