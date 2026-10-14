<script lang="ts">
  import { onMount } from 'svelte';
  import { PageHeader, Button, Input, Badge, Card, Modal, Textarea, Checkbox } from '@cecepazhar/caui';
  import AiNavTabs from '$lib/components/AiNavTabs.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import {
    type SystemPersona,
    aiGetPersonas,
    aiSavePersona,
    aiDeletePersona
  } from '$lib/api/aiRouting';

  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-sm dark:shadow-none text-neutral-900 dark:text-white';
  const SUBCARD = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const INPUT = 'w-full px-3 py-2 text-sm rounded-md border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-sky-500';
  const TEXTAREA = 'w-full px-3 py-2 text-sm font-mono rounded-md border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-sky-500';
  const LABEL = 'block text-xs font-semibold uppercase tracking-wider text-neutral-700 dark:text-neutral-300 mb-1';

  let personas = $state<SystemPersona[]>([]);
  let loading = $state(true);

  // Modal / Form state
  let showModal = $state(false);
  let editingId = $state<string | null>(null);
  let formTitle = $state('');
  let formDesc = $state('');
  let formPrompt = $state('');
  let formRules = $state<string[]>([]);
  let newRuleInput = $state('');
  let formEnvConstraints = $state('');
  let formIsDefault = $state(false);
  let saving = $state(false);

  async function loadData() {
    loading = true;
    try {
      personas = await aiGetPersonas();
    } catch (err) {
      showToast('Failed to load personas: ' + String(err), 'error');
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  function openAddPersona() {
    editingId = null;
    formTitle = '';
    formDesc = '';
    formPrompt = 'You are CATerm AI Ops Copilot, an elite Linux/DevOps Systems Architect.';
    formRules = [
      'Be concise, direct, and terminal-first.',
      'Always provide copy-pasteable bash/zsh commands.',
      'Never output dangerous destructive commands (e.g. rm -rf /) without explicit warning.'
    ];
    newRuleInput = '';
    formEnvConstraints = 'Host: Linux x86_64\nShell: bash/zsh';
    formIsDefault = personas.length === 0;
    showModal = true;
  }

  function openEditPersona(p: SystemPersona) {
    editingId = p.id;
    formTitle = p.title;
    formDesc = p.description ?? '';
    formPrompt = p.system_prompt;
    formRules = [...p.custom_rules];
    newRuleInput = '';
    formEnvConstraints = p.environment_constraints ?? '';
    formIsDefault = p.is_global_default;
    showModal = true;
  }

  function addRule() {
    if (newRuleInput.trim()) {
      formRules = [...formRules, newRuleInput.trim()];
      newRuleInput = '';
    }
  }

  function removeRule(index: number) {
    formRules = formRules.filter((_, i) => i !== index);
  }

  async function handleSavePersona() {
    if (!formTitle.trim() || !formPrompt.trim()) {
      showToast('Title and System Prompt are required.', 'error');
      return;
    }
    saving = true;
    try {
      const persona: SystemPersona = {
        id: editingId ?? `persona-${Date.now()}`,
        title: formTitle.trim(),
        description: formDesc.trim() ? formDesc.trim() : null,
        system_prompt: formPrompt.trim(),
        custom_rules: formRules,
        environment_constraints: formEnvConstraints.trim() ? formEnvConstraints.trim() : null,
        is_global_default: formIsDefault
      };
      await aiSavePersona(persona);
      showToast('Persona saved successfully.', 'success');
      showModal = false;
      await loadData();
    } catch (err) {
      showToast('Failed to save persona: ' + String(err), 'error');
    } finally {
      saving = false;
    }
  }

  async function handleDeletePersona(p: SystemPersona) {
    if (p.is_global_default && personas.length > 1) {
      showToast('Cannot delete the global default persona. Set another as default first.', 'error');
      return;
    }
    const confirmed = await confirmModal(`Delete persona "${p.title}"?`);
    if (!confirmed) return;
    try {
      await aiDeletePersona(p.id);
      showToast(`Persona "${p.title}" deleted.`, 'success');
      await loadData();
    } catch (err) {
      showToast('Failed to delete persona: ' + String(err), 'error');
    }
  }

  async function handleSetDefault(p: SystemPersona) {
    try {
      const updated = { ...p, is_global_default: true };
      await aiSavePersona(updated);
      showToast(`"${p.title}" is now the global default persona.`, 'success');
      await loadData();
    } catch (err) {
      showToast('Failed to set default persona: ' + String(err), 'error');
    }
  }
</script>

<div class="max-w-6xl mx-auto space-y-6">
  <PageHeader
    icon="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
    accent="indigo"
    title="System Personas & Prompt Engineering"
    badge="Custom Context Engine"
    subtitle="Configure role definitions, safety constraints, and automated environment rule injection."
  />

  <AiNavTabs active="personas" />

  {#if loading}
    <Card class="flex items-center justify-center py-12 text-sm text-neutral-500 dark:text-neutral-400" variant="primary" size="lg">
      <svg class="w-5 h-5 animate-spin mr-3 text-sky-500" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
      </svg>
      Loading Personas...
    </Card>
  {:else}
    <Card class="space-y-4" variant="primary" size="lg">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-semibold text-neutral-900 dark:text-white">Active System Personas</h2>
          <p class="{MUTED} text-xs mt-0.5">
            Stage 1 of Prompt Assembly. Personas define LLM tone, safety guardrails, and role expectations.
          </p>
        </div>
        <Button
          onclick={openAddPersona}
          variant="primary"
          size="sm"
          class="self-start sm:self-auto"
        >
          <svg class="w-4 h-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          New Persona
        </Button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#each personas as p (p.id)}
          <Card class="flex flex-col justify-between space-y-4" variant="secondary" size="md">
            <div class="space-y-3">
              <div class="flex items-start justify-between gap-2">
                <div>
                  <h3 class="text-sm font-semibold text-neutral-900 dark:text-white flex items-center gap-2">
                    {p.title}
                    {#if p.is_global_default}
                      <Badge variant="success" size="xs">GLOBAL DEFAULT</Badge>
                    {/if}
                  </h3>
                  {#if p.description}
                    <p class="{MUTED} text-xs mt-0.5">{p.description}</p>
                  {/if}
                </div>
              </div>

              <!-- System prompt preview -->
              <div class="space-y-1">
                <span class="text-[10px] font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
                  Base System Prompt
                </span>
                <p class="text-xs font-mono bg-white dark:bg-neutral-900 p-2.5 rounded border border-neutral-200 dark:border-neutral-800 line-clamp-3 text-neutral-800 dark:text-neutral-200">
                  {p.system_prompt}
                </p>
              </div>

              <!-- Rules preview -->
              {#if p.custom_rules && p.custom_rules.length > 0}
                <div class="space-y-1">
                  <span class="text-[10px] font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
                    Rules ({p.custom_rules.length})
                  </span>
                  <ul class="text-xs space-y-1">
                    {#each p.custom_rules as rule}
                      <li class="flex items-start gap-1.5 text-neutral-700 dark:text-neutral-300">
                        <span class="text-indigo-500 font-bold">•</span>
                        <span>{rule}</span>
                      </li>
                    {/each}
                  </ul>
                </div>
              {/if}
            </div>

            <div class="flex items-center justify-between pt-3 border-t border-neutral-200 dark:border-neutral-800 text-xs">
              <div class="flex gap-2">
                <Button
                  onclick={() => openEditPersona(p)}
                  variant="ghost"
                  size="sm"
                >
                  Edit
                </Button>
                {#if !p.is_global_default}
                  <Button
                    onclick={() => handleSetDefault(p)}
                    variant="ghost"
                    size="sm"
                  >
                    Set as Default
                  </Button>
                {/if}
              </div>
              {#if !p.is_global_default}
                <Button
                  onclick={() => handleDeletePersona(p)}
                  variant="danger"
                  size="sm"
                >
                  Delete
                </Button>
              {/if}
            </div>
          </Card>
        {/each}
      </div>
    </Card>
  {/if}
</div>

<!-- Modal: Add / Edit Persona -->
<Modal
  bind:open={showModal}
  title={editingId ? 'Edit Persona' : 'Create New System Persona'}
  size="lg"
>
  <div class="space-y-4 text-xs">
    <Input
      label="Persona Title"
      bind:value={formTitle}
      placeholder="e.g. SRE Incident Commander"
    />

    <Input
      label="Description (Optional)"
      bind:value={formDesc}
      placeholder="Short summary of this persona"
    />

    <Textarea
      label="System Instruction Prompt"
      rows={4}
      bind:value={formPrompt}
      placeholder="Instruction given to the model..."
    />

    <div>
      <span class="block text-xs font-semibold uppercase tracking-wider text-neutral-700 dark:text-neutral-300 mb-1">Custom Guardrail Rules</span>
      <div class="space-y-2 mb-2">
        {#each formRules as rule, i}
          <div class="flex items-center gap-2 bg-neutral-100 dark:bg-neutral-800 px-2.5 py-1.5 rounded-lg">
            <span class="flex-1 text-neutral-800 dark:text-neutral-200">{rule}</span>
            <Button
              size="sm"
              variant="ghost"
              onclick={() => removeRule(i)}
              class="text-rose-500 hover:text-rose-700 font-bold p-1 h-auto"
            >
              ✕
            </Button>
          </div>
        {/each}
      </div>
      <div class="flex gap-2">
        <Input
          bind:value={newRuleInput}
          placeholder="e.g. Always include curl verbose flag (-v)"
          class="flex-1"
          onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); addRule(); } }}
        />
        <Button
          type="button"
          onclick={addRule}
          variant="secondary"
          size="md"
        >
          Add
        </Button>
      </div>
    </div>

    <Textarea
      label="Default Environment Constraints"
      rows={2}
      bind:value={formEnvConstraints}
      placeholder="Host OS, Shell syntax constraints..."
    />

    <div class="pt-1">
      <Checkbox
        bind:checked={formIsDefault}
        label="Make this persona the Global Default"
      />
    </div>
  </div>

  {#snippet footer()}
    <Button
      onclick={() => (showModal = false)}
      variant="neutral"
      size="md"
    >
      Cancel
    </Button>
    <Button
      onclick={handleSavePersona}
      disabled={saving}
      loading={saving}
      variant="primary"
      size="md"
    >
      Save Persona
    </Button>
  {/snippet}
</Modal>
