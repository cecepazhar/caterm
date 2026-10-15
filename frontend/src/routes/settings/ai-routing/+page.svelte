<script lang="ts">
  import { onMount } from 'svelte';
  import { PageHeader, Button, Input, Badge, Card, Modal, Checkbox } from '@cecepazhar/caui';
  import AiNavTabs from '$lib/components/AiNavTabs.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import {
    type AiProvider,
    type AiProviderConfig,
    type TaskRouteRule,
    type TaskType,
    type ProviderType,
    type SystemPersona,
    aiGetProviders,
    aiSaveProvider,
    aiDeleteProvider,
    aiGetRoutingMatrix,
    aiSaveRoutingRule,
    aiGetPersonas
  } from '$lib/api/aiRouting';

  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-sm dark:shadow-none text-neutral-900 dark:text-white';
  const SUBCARD = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const INPUT = 'w-full px-3 py-2 text-sm rounded-md border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-sky-500';
  const LABEL = 'block text-xs font-semibold uppercase tracking-wider text-neutral-700 dark:text-neutral-300 mb-1';

  let providers = $state<AiProvider[]>([]);
  let routingRules = $state<TaskRouteRule[]>([]);
  let personas = $state<SystemPersona[]>([]);
  let loading = $state(true);

  // Modal / Form state for Add/Edit Provider
  let showProviderModal = $state(false);
  let editingProviderId = $state<string | null>(null);
  let formName = $state('');
  let formType = $state<ProviderType>('openai_compatible');
  let formBaseUrl = $state('');
  let formApiKey = $state('');
  let formModel = $state('');
  let formActive = $state(true);
  let savingProvider = $state(false);

  // Task descriptions
  const TASK_INFO: Record<TaskType, { title: string; desc: string }> = {
    chat: {
      title: 'Interactive AI Chat',
      desc: 'Real-time multi-turn conversational terminal assistant & execution planner.'
    },
    error_diagnostic: {
      title: 'Terminal Error Auto-Diagnostic',
      desc: 'Automatic diagnosis of non-zero exit codes & bash crash traces.'
    },
    prompt_studio: {
      title: 'Prompt Studio & Templates',
      desc: 'Automated script generation, cron tasks, and playbook templates.'
    },
    command_autocomplete: {
      title: 'Command Autocomplete & Suggestion',
      desc: 'Fast, lightweight sub-command prediction and flag completion.'
    },
    security_review: {
      title: 'Command Security Review',
      desc: 'Pre-execution risk evaluation for destructive CLI commands.'
    }
  };

  async function loadData() {
    loading = true;
    try {
      const [provs, matrix, pers] = await Promise.all([
        aiGetProviders(),
        aiGetRoutingMatrix(),
        aiGetPersonas()
      ]);
      providers = provs;
      routingRules = matrix;
      personas = pers;
    } catch (err) {
      showToast('Failed to load AI Routing data: ' + String(err), 'error');
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  function openAddProvider() {
    editingProviderId = null;
    formName = '';
    formType = 'openai_compatible';
    formBaseUrl = 'https://api.openai.com/v1';
    formApiKey = '';
    formModel = 'gpt-4o';
    formActive = true;
    showProviderModal = true;
  }

  function openEditProvider(p: AiProvider) {
    editingProviderId = p.id;
    formName = p.name;
    formType = p.provider_type;
    formBaseUrl = p.base_url;
    formApiKey = '';
    formModel = p.default_model;
    formActive = p.is_active;
    showProviderModal = true;
  }

  function handleTypeChange() {
    if (formType === 'caterm_hosted') {
      formBaseUrl = 'https://ai.caterm.com/v1';
      formModel = 'caterm-omni-fast';
    } else if (formType === 'anthropic') {
      formBaseUrl = 'https://api.anthropic.com/v1';
      formModel = 'claude-3-7-sonnet';
    } else if (formType === 'openai_compatible') {
      formBaseUrl = 'https://api.openai.com/v1';
      formModel = 'gpt-4o';
    } else if (formType === 'ollama') {
      formBaseUrl = 'http://localhost:11434';
      formModel = 'llama3.2';
    }
  }

  async function handleSaveProvider() {
    if (!formName.trim() || !formBaseUrl.trim() || !formModel.trim()) {
      showToast('Name, Base URL, and Default Model are required.', 'error');
      return;
    }
    savingProvider = true;
    try {
      const config: AiProviderConfig = {
        id: editingProviderId ?? `prov-${Date.now()}`,
        name: formName.trim(),
        provider_type: formType,
        base_url: formBaseUrl.trim(),
        api_key: formApiKey.trim() ? formApiKey.trim() : null,
        default_model: formModel.trim(),
        is_active: formActive
      };
      await aiSaveProvider(config);
      showToast('Provider saved successfully.', 'success');
      showProviderModal = false;
      await loadData();
    } catch (err) {
      showToast('Failed to save provider: ' + String(err), 'error');
    } finally {
      savingProvider = false;
    }
  }

  async function handleDeleteProvider(p: AiProvider) {
    if (p.id === 'caterm-hosted') {
      showToast('Built-in CATerm Hosted provider cannot be deleted.', 'error');
      return;
    }
    const confirmed = await confirmModal(
      `Delete AI Provider "${p.name}"? This action cannot be undone.`
    );
    if (!confirmed) return;
    try {
      await aiDeleteProvider(p.id);
      showToast(`Provider "${p.name}" deleted.`, 'success');
      await loadData();
    } catch (err) {
      showToast('Failed to delete provider: ' + String(err), 'error');
    }
  }

  async function handleSaveRoutingRule(rule: TaskRouteRule) {
    try {
      await aiSaveRoutingRule(rule);
      showToast(`Routing rule for "${rule.task_type}" updated.`, 'success');
    } catch (err) {
      showToast('Failed to save routing rule: ' + String(err), 'error');
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon="M13 10V3L4 14h7v7l9-11h-7z"
    accent="sky"
    title="AI Routing & Fallback Matrix"
    badge="Multi-Provider Engine"
    subtitle="Zero-knowledge encrypted model routing with automatic failover and local privacy boundaries."
  />

  <AiNavTabs active="routing" />

  {#if loading}
    <Card class="flex items-center justify-center py-12 text-sm text-neutral-500 dark:text-neutral-400" variant="primary" size="lg">
      <svg class="w-5 h-5 animate-spin mr-3 text-sky-500" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
      </svg>
      Loading AI Provider & Routing matrix...
    </Card>
  {:else}
    <!-- AI Providers Section -->
    <Card class="space-y-4" variant="primary" size="lg">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-semibold text-neutral-900 dark:text-white">Registered AI Providers</h2>
          <p class="{MUTED} text-xs mt-0.5">
            BYO API keys (Anthropic, OpenAI, DeepSeek, Ollama) and CATerm Pro Hosted AI. All keys are encrypted at rest with SQLCipher Argon2id.
          </p>
        </div>
        <Button
          onclick={openAddProvider}
          variant="primary"
          size="sm"
          class="self-start sm:self-auto"
        >
          <svg class="w-4 h-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Add Provider
        </Button>
      </div>

      <div class="overflow-x-auto border border-neutral-200 dark:border-neutral-800 rounded-lg">
        <table class="w-full text-left text-xs">
          <thead class="bg-neutral-100 dark:bg-neutral-800/60 text-neutral-700 dark:text-neutral-300 uppercase tracking-wider font-semibold border-b border-neutral-200 dark:border-neutral-800">
            <tr>
              <th class="px-4 py-3">Name</th>
              <th class="px-4 py-3">Type</th>
              <th class="px-4 py-3">Base URL</th>
              <th class="px-4 py-3">Default Model</th>
              <th class="px-4 py-3">API Key Status</th>
              <th class="px-4 py-3">Status</th>
              <th class="px-4 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800">
            {#each providers as p (p.id)}
              <tr class="hover:bg-neutral-50 dark:hover:bg-neutral-800/30 transition-colors">
                <td class="px-4 py-3 font-medium text-neutral-900 dark:text-white">
                  {p.name}
                  {#if p.id === 'caterm-hosted'}
                    <Badge variant="info" size="xs" class="ml-1.5">DEFAULT</Badge>
                  {/if}
                </td>
                <td class="px-4 py-3 font-mono text-neutral-600 dark:text-neutral-300">
                  <Badge variant={p.provider_type === 'caterm_hosted' ? 'brand' : p.provider_type === 'anthropic' ? 'warning' : p.provider_type === 'ollama' ? 'success' : 'info'} size="xs">
                    {p.provider_type}
                  </Badge>
                </td>
                <td class="px-4 py-3 font-mono text-neutral-500 dark:text-neutral-400 truncate max-w-xs">{p.base_url}</td>
                <td class="px-4 py-3 font-mono text-neutral-800 dark:text-neutral-200">{p.default_model}</td>
                <td class="px-4 py-3">
                  {#if p.provider_type === 'caterm_hosted'}
                    <span class="text-neutral-500 dark:text-neutral-400">Pro Token Handshake</span>
                  {:else if p.provider_type === 'ollama'}
                    <span class="text-emerald-600 dark:text-emerald-400">No Key Needed (Local)</span>
                  {:else if p.has_api_key}
                    <Badge variant="success" size="xs">Encrypted in Vault</Badge>
                  {:else}
                    <Badge variant="warning" size="xs">Missing Key</Badge>
                  {/if}
                </td>
                <td class="px-4 py-3">
                  <Badge variant={p.is_active ? 'success' : 'neutral'} size="xs">
                    {p.is_active ? 'ACTIVE' : 'INACTIVE'}
                  </Badge>
                </td>
                <td class="px-4 py-3 text-right space-x-1">
                  <Button
                    onclick={() => openEditProvider(p)}
                    variant="ghost"
                    size="sm"
                  >
                    Edit
                  </Button>
                  {#if p.id !== 'caterm-hosted'}
                    <Button
                      onclick={() => handleDeleteProvider(p)}
                      variant="danger"
                      size="sm"
                    >
                      Delete
                    </Button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </Card>

    <!-- Task Routing Matrix Section -->
    <Card class="space-y-5" variant="primary" size="lg">
      <div>
        <h2 class="text-base font-semibold text-neutral-900 dark:text-white">Deterministic Task Routing Matrix</h2>
        <p class="{MUTED} text-xs mt-0.5">
          Assign specialized models for specific terminal operations. If primary provider returns 429, 503, or times out (>15s), the fallback model seamlessly executes.
        </p>
      </div>

      <div class="space-y-4">
        {#each routingRules as rule (rule.task_type)}
          <Card class="space-y-3" variant="secondary" size="md">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-neutral-200 dark:border-neutral-800 pb-2">
              <div>
                <h3 class="text-sm font-semibold text-neutral-900 dark:text-white flex items-center gap-2">
                  <Badge variant="neutral" size="sm">{rule.task_type}</Badge>
                  {TASK_INFO[rule.task_type]?.title ?? rule.task_type}
                </h3>
                <p class="{MUTED} text-xs mt-0.5">{TASK_INFO[rule.task_type]?.desc ?? ''}</p>
              </div>
              <Button
                onclick={() => handleSaveRoutingRule(rule)}
                variant="primary"
                size="sm"
                class="self-start sm:self-auto"
              >
                Save Rule
              </Button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 text-xs">
              <!-- Primary Provider & Model -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-sky-600 dark:text-sky-400 uppercase tracking-wide text-[10px]">Primary Execution</div>
                <div>
                  <label for="primary-prov-{rule.task_type}" class={LABEL}>Provider</label>
                  <select id="primary-prov-{rule.task_type}" bind:value={rule.primary_provider_id} class={INPUT}>
                    {#each providers as prov}
                      <option value={prov.id}>{prov.name} ({prov.provider_type})</option>
                    {/each}
                  </select>
                </div>
                <Input
                  id="primary-model-{rule.task_type}"
                  label="Model"
                  bind:value={rule.primary_model}
                  placeholder="e.g. claude-3-7-sonnet"
                  size="sm"
                />
              </div>

              <!-- Fallback Provider & Model -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-amber-600 dark:text-amber-400 uppercase tracking-wide text-[10px]">Fallback Failover (15s Timeout)</div>
                <div>
                  <label for="fallback-prov-{rule.task_type}" class={LABEL}>Provider</label>
                  <select id="fallback-prov-{rule.task_type}" bind:value={rule.fallback_provider_id} class={INPUT}>
                    <option value={null}>None (Fail Fast)</option>
                    {#each providers as prov}
                      <option value={prov.id}>{prov.name} ({prov.provider_type})</option>
                    {/each}
                  </select>
                </div>
                <Input
                  id="fallback-model-{rule.task_type}"
                  label="Model"
                  value={rule.fallback_model || ''}
                  oninput={(e) => { rule.fallback_model = (e.target as HTMLInputElement).value; }}
                  placeholder="e.g. llama3.2 or gpt-4o"
                  size="sm"
                />
              </div>

              <!-- Persona & System Prompt -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-purple-600 dark:text-purple-400 uppercase tracking-wide text-[10px]">System Persona Context</div>
                <div>
                  <label for="persona-{rule.task_type}" class={LABEL}>Active Persona</label>
                  <select id="persona-{rule.task_type}" bind:value={rule.system_persona_id} class={INPUT}>
                    <option value={null}>Global Default Persona</option>
                    {#each personas as pers}
                      <option value={pers.id}>{pers.title} {pers.is_global_default ? '(Default)' : ''}</option>
                    {/each}
                  </select>
                </div>
              </div>

              <!-- Hyperparameters -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-emerald-600 dark:text-emerald-400 uppercase tracking-wide text-[10px]">Generation Parameters</div>
                <div class="flex items-center justify-between gap-2">
                  <label for="temp-{rule.task_type}" class={LABEL}>Temp ({rule.temperature.toFixed(2)})</label>
                  <input
                    id="temp-{rule.task_type}"
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    bind:value={rule.temperature}
                    class="w-24 accent-sky-500"
                  />
                </div>
                <div>
                  <label for="tokens-{rule.task_type}" class={LABEL}>Max Tokens</label>
                  <input
                    id="tokens-{rule.task_type}"
                    type="number"
                    min="100"
                    max="8192"
                    step="128"
                    bind:value={rule.max_tokens}
                    class={INPUT}
                  />
                </div>
              </div>
            </div>
          </Card>
        {/each}
      </div>
    </Card>
  {/if}
</div>

<!-- Modal: Add / Edit Provider -->
<Modal
  bind:open={showProviderModal}
  title={editingProviderId ? 'Edit AI Provider' : 'Add New AI Provider'}
  size="md"
>
  <div class="space-y-4 text-xs">
    <Input
      label="Provider Name"
      bind:value={formName}
      placeholder="e.g. Personal Anthropic"
    />

    <div>
      <label for="provider-type-select" class={LABEL}>Provider Protocol / Type</label>
      <select id="provider-type-select" bind:value={formType} onchange={handleTypeChange} class={INPUT}>
        <option value="caterm_hosted">CATerm Hosted AI (OmniRoute Pro)</option>
        <option value="anthropic">Anthropic Messages API (/v1/messages)</option>
        <option value="openai_compatible">OpenAI Compatible (OpenAI, DeepSeek, Groq)</option>
        <option value="ollama">Local Ollama (/api/chat)</option>
      </select>
    </div>

    <Input
      label="Base URL"
      bind:value={formBaseUrl}
    />

    <Input
      label="Default Model"
      bind:value={formModel}
    />

    {#if formType !== 'ollama' && formType !== 'caterm_hosted'}
      <Input
        type="password"
        label="API Key {editingProviderId ? '(Leave blank to keep current key)' : ''}"
        bind:value={formApiKey}
        placeholder="sk-..."
      />
    {/if}

    <div class="pt-2">
      <Checkbox
        bind:checked={formActive}
        label="Enable this provider in Routing Matrix"
      />
    </div>
  </div>

  {#snippet footer()}
    <Button
      onclick={() => (showProviderModal = false)}
      variant="neutral"
      size="md"
    >
      Cancel
    </Button>
    <Button
      onclick={handleSaveProvider}
      disabled={savingProvider}
      loading={savingProvider}
      variant="primary"
      size="md"
    >
      Save Provider
    </Button>
  {/snippet}
</Modal>
