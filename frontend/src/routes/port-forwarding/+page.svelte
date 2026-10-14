<script lang="ts">
  import { onMount } from 'svelte';
  import { listTunnels, saveTunnel, deleteTunnel, startTunnel, stopTunnel, type TunnelRecord, type ForwardType } from '$lib/api/tunnels';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { t } from '$lib/i18n/index.svelte';
  import { PageHeader, Button, Input, Badge, Modal } from '@cecepazhar/caui';
  import { confirmModal } from '$lib/stores/uiNotifications.svelte';

  let tunnels = $state<TunnelRecord[]>([]);
  let hosts = $state<HostRecord[]>([]);
  let isLoading = $state(true);
  let errorMsg = $state('');
  let successMsg = $state('');

  // Add modal
  let showAddModal = $state(false);
  let formHostId = $state('');
  let formType = $state<ForwardType>('local');
  let formBindAddr = $state('127.0.0.1');
  let formBindPort = $state(8080);
  let formTargetAddr = $state('127.0.0.1');
  let formTargetPort = $state(80);
  let isSaving = $state(false);

  // String-typed port inputs for Svelte 5 bind:value on Input component
  let formBindPortStr = $state('8080');
  let formTargetPortStr = $state('80');

  $effect(() => {
    formBindPort;
    formBindPortStr = String(formBindPort);
  });

  $effect(() => {
    formTargetPort;
    formTargetPortStr = String(formTargetPort);
  });

  function errorMessage(e: unknown): string {
    if (!e) return '';
    if (typeof e === 'string') return e;
    if (typeof e === 'object') {
      if ('message' in e && typeof (e as any).message === 'string') return (e as any).message;
      return JSON.stringify(e);
    }
    return String(e);
  }

  async function loadData() {
    isLoading = true;
    errorMsg = '';
    try {
      [tunnels, hosts] = await Promise.all([listTunnels(), listHosts()]);
    } catch (e: any) {
      const msg = errorMessage(e);
      if (msg.toLowerCase().includes('vault') || msg.toLowerCase().includes('database') || msg.toLowerCase().includes('kunci') || msg.includes('locked') || msg.includes('KeyError')) {
        errorMsg = t('portForwarding.vaultLocked');
      } else {
        errorMsg = msg;
      }
    } finally {
      isLoading = false;
    }
  }

  onMount(loadData);

  function openAddModal() {
    formHostId = hosts.length > 0 ? hosts[0].id : '';
    formType = 'local';
    formBindAddr = '127.0.0.1';
    formBindPort = 8080;
    formTargetAddr = '127.0.0.1';
    formTargetPort = 80;
    showAddModal = true;
    errorMsg = '';
    successMsg = '';
  }

  async function handleSave() {
    if (!formHostId || !formBindPort || !formTargetPort) return;
    isSaving = true;
    errorMsg = '';
    try {
      await saveTunnel({
        hostId: formHostId,
        forwardType: formType,
        bindAddr: formBindAddr,
        bindPort: Number(formBindPort),
        targetAddr: formTargetAddr,
        targetPort: Number(formTargetPort),
      });
      showAddModal = false;
      await loadData();
    } catch (e: any) {
      errorMsg = errorMessage(e);
    } finally {
      isSaving = false;
    }
  }

  async function handleToggle(tunnel: TunnelRecord) {
    errorMsg = '';
    try {
      if (tunnel.isActive) {
        await stopTunnel(tunnel.id);
      } else {
        await startTunnel(tunnel.id);
      }
      await loadData();
    } catch (e: any) {
      errorMsg = errorMessage(e);
    }
  }

  async function handleDelete(id: string) {
    const confirmed = await confirmModal(t('portForwarding.deleteConfirm'), t('common.delete'), true, t('common.delete'), t('common.cancel'));
    if (!confirmed) return;
    errorMsg = '';
    try {
      await deleteTunnel(id);
      await loadData();
    } catch (e: any) {
      errorMsg = errorMessage(e);
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4']}
    accent="amber"
    title={t('portForwarding.title')}
    subtitle={t('portForwarding.subtitle')}
  >
    {#snippet actions()}
      <Button
        variant="primary"
        size="sm"
        onclick={openAddModal}
      >
        + {t('portForwarding.addTunnel')}
      </Button>
    {/snippet}
  </PageHeader>

  {#if errorMsg}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-600 dark:text-red-400 text-xs">
      {errorMsg}
    </div>
  {/if}

  {#if isLoading}
    <div class="p-8 text-center text-neutral-500 text-sm">{t('portForwarding.loading')}</div>
  {:else if tunnels.length === 0}
    <div class="p-8 border border-neutral-200 dark:border-neutral-800 rounded-xl bg-white dark:bg-neutral-900/30 text-center flex flex-col items-center justify-center shadow-sm dark:shadow-none">
      <svg class="w-12 h-12 text-neutral-400 dark:text-neutral-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path></svg>
      <h3 class="text-lg font-medium text-neutral-900 dark:text-white mb-2">{t('portForwarding.emptyTitle')}</h3>
      <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-md">{t('portForwarding.emptyBody')}</p>
      <Button
        variant="primary"
        size="sm"
        onclick={openAddModal}
        class="mt-6"
      >
        {t('portForwarding.addRule')}
      </Button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each tunnels as tun (tun.id)}
        {@const host = hosts.find(h => h.id === tun.hostId)}
        <div class="flex flex-col p-4 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl hover:border-neutral-300 dark:hover:border-neutral-700 transition-colors shadow-sm dark:shadow-none">
          <div class="flex items-center justify-between mb-2">
            <span class="font-medium text-neutral-900 dark:text-white text-sm flex items-center gap-2">
              <span class="w-2 h-2 rounded-full {tun.isActive ? 'bg-emerald-400 animate-pulse' : 'bg-neutral-300 dark:bg-neutral-600'}"></span>
              {tun.bindAddr}:{tun.bindPort} &rarr; {tun.forwardType === 'dynamic' ? 'SOCKS5' : `${tun.targetAddr}:${tun.targetPort}`}
            </span>
            <Badge variant="neutral" size="xs" class="font-mono">
              {t(`portForwarding.badge.${tun.forwardType}`)}
            </Badge>
          </div>

          <p class="text-xs text-neutral-500 dark:text-neutral-400 mb-4">
            {t('portForwarding.viaHost')}: <span class="text-neutral-900 dark:text-white font-medium">{host?.label || tun.hostId}</span>
          </p>

          <div class="mt-auto pt-3 border-t border-neutral-100 dark:border-neutral-800/50 flex justify-between items-center">
            <Button
              variant={tun.isActive ? 'warning' : 'success'}
              size="sm"
              onclick={() => handleToggle(tun)}
            >
              {tun.isActive ? t('portForwarding.stopTunnel') : t('portForwarding.startTunnel')}
            </Button>
            <Button
              variant="danger"
              size="sm"
              onclick={() => handleDelete(tun.id)}
            >
              {t('common.delete')}
            </Button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<Modal
  bind:open={showAddModal}
  title={t('portForwarding.addRule')}
  size="md"
>
  <div class="space-y-4">
    <div>
      <label for="pf-host" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('portForwarding.targetHost')}</label>
      {#if hosts.length === 0}
        <div class="p-3 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded-lg text-sm text-neutral-500">
          {t('portForwarding.noHostsAddOne')}
        </div>
      {:else}
        <select id="pf-host" bind:value={formHostId} class="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 text-xs text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors">
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.username}@{h.address})</option>
          {/each}
        </select>
      {/if}
    </div>

    <div>
      <label for="pf-type" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('portForwarding.forwardingType')}</label>
      <select id="pf-type" bind:value={formType} class="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 text-xs text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors">
        <option value="local">{t('portForwarding.typeLocal')}</option>
        <option value="remote">{t('portForwarding.typeRemote')}</option>
        <option value="dynamic">{t('portForwarding.typeDynamic')}</option>
      </select>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label for="pf-bind-addr" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('portForwarding.bindAddress')}</label>
        <Input id="pf-bind-addr" type="text" bind:value={formBindAddr} size="sm" />
      </div>
      <div>
        <label for="pf-bind-port" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('portForwarding.bindPort')}</label>
        <Input
          id="pf-bind-port"
          type="text"
          bind:value={formBindPortStr}
          size="sm"
          oninput={(e) => {
            const v = (e.target as HTMLInputElement).value;
            formBindPort = parseInt(v || '0', 10);
          }}
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label for="pf-target-addr" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('portForwarding.targetAddress')}</label>
        <Input id="pf-target-addr" type="text" bind:value={formTargetAddr} size="sm" />
      </div>
      <div>
        <label for="pf-target-port" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('portForwarding.targetPort')}</label>
        <Input
          id="pf-target-port"
          type="text"
          bind:value={formTargetPortStr}
          size="sm"
          oninput={(e) => {
            const v = (e.target as HTMLInputElement).value;
            formTargetPort = parseInt(v || '0', 10);
          }}
        />
      </div>
    </div>
  </div>

  {#snippet footer()}
    <div class="flex justify-end gap-3">
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onclick={() => showAddModal = false}
      >
        {t('common.cancel')}
      </Button>
      <Button
        type="button"
        variant="primary"
        size="sm"
        onclick={handleSave}
        disabled={!formHostId || isSaving}
        loading={isSaving}
      >
        {t('portForwarding.saveRule')}
      </Button>
    </div>
  {/snippet}
</Modal>
