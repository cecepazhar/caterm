<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    runNetworkSecurityAudit,
    runThreatWatchdog,
    runTlsAudit,
    type AuditReport,
    type ThreatWatchdogResult,
    type TlsAuditResult
  } from '$lib/api/audit';
  import {
    scanLaptopPosture,
    createIntegrityBaseline,
    verifyIntegrityTripwire,
    listEphemeralKeys,
    createEphemeralKey,
    revokeEphemeralKey,
    probeJumpChain,
    runDockerProbe,
    type LaptopPostureReport,
    type BaselineFileRecord,
    type IntegrityTripwireReport,
    type EphemeralKeyStatus,
    type JumpChainConfig,
    type JumpChainProbeResult,
    type DockerProbeReport
  } from '$lib/api/proSecurity';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { saveInvestigation } from '$lib/api/investigations';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { errorText } from '$lib/errors';
  import { PageHeader, Button, Input, Badge, Card } from '@cecepazhar/caui';
import ProGate from '$lib/components/ProGate.svelte';

  let hosts = $state<HostRecord[]>([]);
  let selectedHostId = $state<string>('');
  let activeTab = $state<'exposure' | 'hardening' | 'watchdog' | 'tls' | 'laptop' | 'tripwire' | 'ephemeral' | 'jumpchain' | 'docker'>('exposure');

  // Loading States
  let isRunningSecurity = $state(false);
  let isRunningWatchdog = $state(false);
  let isRunningTls = $state(false);
  let isSavingIncident = $state(false);
  let isRunningLaptop = $state(false);
  let isRunningBaseline = $state(false);
  let isRunningTripwire = $state(false);
  let isGeneratingEphemeral = $state(false);
  let isProbingJumpChain = $state(false);
  let isRunningDockerProbe = $state(false);

  // Results State
  let securityReport = $state<AuditReport | null>(null);
  let watchdogResult = $state<ThreatWatchdogResult | null>(null);
  let tlsResult = $state<TlsAuditResult | null>(null);
  let laptopReport = $state<LaptopPostureReport | null>(null);
  let baselineRecords = $state<BaselineFileRecord[]>([]);
  let tripwireReport = $state<IntegrityTripwireReport | null>(null);
  let ephemeralKeys = $state<EphemeralKeyStatus[]>([]);
  let jumpProbeResult = $state<JumpChainProbeResult | null>(null);
  let dockerReport = $state<DockerProbeReport | null>(null);

  // Batch 2 Inputs
  let ephKeyName = $state('Deploy Ephemeral Key');
  let ephKeyTtl = $state<number>(7200);
  let ephTargetHost = $state('prod-bastion.internal');
  let newGeneratedPubkey = $state<string | null>(null);

  let jumpTargetHost = $state('10.0.50.15');
  let jumpTargetPort = $state<number>(22);
  let jumpTargetUser = $state('root');
  let jumpBastions = $state<string>('bastion-us-east.corp.internal:22, dmz-proxy.corp.internal:2222');

  // TLS Inputs
  let tlsHost = $state('cloudflare.com');
  let tlsPort = $state<number>(443);

  onMount(async () => {
    try {
      hosts = await listHosts();
      const qHost = page.url.searchParams.get('host');
      if (qHost) {
        selectedHostId = qHost;
      } else if (hosts.length > 0) {
        selectedHostId = hosts[0].id;
      } else {
        selectedHostId = 'local';
      }
    } catch {
      selectedHostId = 'local';
    }
  });

  async function startSecurityAudit() {
    if (!selectedHostId) {
      showToast(t('security.selectHostFirst'), 'error');
      return;
    }
    isRunningSecurity = true;
    securityReport = null;
    try {
      const res = await runNetworkSecurityAudit(selectedHostId);
      securityReport = res;
      showToast(t('security.auditCompleted', { score: res.summaryScore }), res.summaryScore > 70 ? 'success' : 'error');
    } catch (e) {
      showToast(t('security.auditFailed', { error: errorText(e) }), 'error');
    } finally {
      isRunningSecurity = false;
    }
  }

  async function startThreatWatchdog() {
    if (!selectedHostId) return;
    isRunningWatchdog = true;
    try {
      watchdogResult = await runThreatWatchdog(selectedHostId);
      showToast(t('security.watchdogCompleted', { status: watchdogResult.threatLevel }), watchdogResult.threatLevel === 'CLEAN' ? 'success' : 'error');
    } catch (e) {
      showToast(t('security.watchdogFailed', { error: errorText(e) }), 'error');
    } finally {
      isRunningWatchdog = false;
    }
  }

  async function startTlsAudit() {
    if (!tlsHost.trim()) return;
    isRunningTls = true;
    try {
      tlsResult = await runTlsAudit(tlsHost, tlsPort);
      showToast(t('security.tlsAuditCompleted', { status: tlsResult.status }), tlsResult.status === 'HEALTHY' ? 'success' : 'error');
    } catch (e) {
      showToast(t('security.tlsAuditFailed', { error: errorText(e) }), 'error');
    } finally {
      isRunningTls = false;
    }
  }

  async function startLaptopScan() {
    isRunningLaptop = true;
    try {
      laptopReport = await scanLaptopPosture();
      showToast(t('security.laptopScanCompleted', { score: laptopReport.overallScore }), laptopReport.overallScore > 75 ? 'success' : 'error');
    } catch (e) {
      showToast(t('security.laptopScanFailed', { error: errorText(e) }), 'error');
    } finally {
      isRunningLaptop = false;
    }
  }

  async function createBaseline() {
    isRunningBaseline = true;
    try {
      const records = await createIntegrityBaseline('local');
      baselineRecords = records;
      showToast(t('security.baselineCreated', { count: records.length }), 'success');
    } catch (e) {
      showToast(t('security.baselineFailed', { error: errorText(e) }), 'error');
    } finally {
      isRunningBaseline = false;
    }
  }

  async function runIntegrityCheck() {
    if (baselineRecords.length === 0) {
      showToast(t('security.baselineRequired'), 'error');
      return;
    }
    isRunningTripwire = true;
    try {
      tripwireReport = await verifyIntegrityTripwire('local', baselineRecords);
      if (tripwireReport.tamperedCount === 0) {
        showToast(t('security.integritySafe'), 'success');
      } else {
        showToast(t('security.integrityAlert', { count: tripwireReport.tamperedCount }), 'error');
      }
    } catch (e) {
      showToast(t('security.integrityFailed', { error: errorText(e) }), 'error');
    } finally {
      isRunningTripwire = false;
    }
  }

  async function loadEphemeralKeys() {
    try {
      ephemeralKeys = await listEphemeralKeys();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleGenerateEphemeralKey() {
    isGeneratingEphemeral = true;
    try {
      const rec = await createEphemeralKey({
        name: ephKeyName,
        ttlSeconds: Number(ephKeyTtl),
        targetHost: ephTargetHost,
        autoRevoke: true
      });
      newGeneratedPubkey = rec.publicKey;
      await loadEphemeralKeys();
      showToast(t('security.ephemeralKeyCreated', { ttl: ephKeyTtl / 3600 }), 'success');
    } catch (e) {
      showToast(t('security.ephemeralKeyFailed', { error: errorText(e) }), 'error');
    } finally {
      isGeneratingEphemeral = false;
    }
  }

  async function handleRevokeEphemeralKey(id: string) {
    try {
      await revokeEphemeralKey(id);
      await loadEphemeralKeys();
      showToast(t('security.ephemeralKeyRevoked'), 'success');
    } catch (e) {
      showToast(t('security.revokeFailed', { error: errorText(e) }), 'error');
    }
  }

  async function handleProbeJumpChain() {
    isProbingJumpChain = true;
    try {
      const hops = jumpBastions
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
        .map((entry, idx) => {
          const parts = entry.split(':');
          const host = parts[0];
          const port = parts[1] ? parseInt(parts[1], 10) : 22;
          return {
            id: `hop-${idx + 1}`,
            name: `Bastion Step ${idx + 1}`,
            host,
            port,
            username: 'jumpuser',
            authType: 'key'
          };
        });

      const config: JumpChainConfig = {
        id: 'chain-1',
        name: 'Production Jump Chain',
        targetHost: jumpTargetHost,
        targetPort: Number(jumpTargetPort),
        targetUsername: jumpTargetUser,
        hops,
        createdAt: new Date().toISOString()
      };

      jumpProbeResult = await probeJumpChain(config);
      showToast(
        jumpProbeResult.isFullyTraversable
          ? t('security.jumpChainVerified')
          : t('security.jumpChainUnreachable'),
        jumpProbeResult.isFullyTraversable ? 'success' : 'error'
      );
    } catch (e) {
      showToast(t('security.jumpChainFailed', { error: errorText(e) }), 'error');
    } finally {
      isProbingJumpChain = false;
    }
  }

  async function handleRunDockerProbe() {
    isRunningDockerProbe = true;
    try {
      dockerReport = await runDockerProbe();
      showToast(t('security.dockerProbeCompleted', { score: dockerReport.securityScore }), dockerReport.securityScore > 75 ? 'success' : 'error');
    } catch (e) {
      showToast(t('security.dockerProbeFailed', { error: errorText(e) }), 'error');
    } finally {
      isRunningDockerProbe = false;
    }
  }

  async function exportToInvestigation() {
    if (!securityReport) return;
    isSavingIncident = true;
    try {
      const hostLabel = getHostLabel(securityReport.hostId);
      const title = `Security Incident & Audit Report - Host ${hostLabel} (Score: ${securityReport.summaryScore}%)`;
      const desc = `Automatic CATerm Security Suite Audit Report.
Security Score: ${securityReport.summaryScore}%
Inbound Wildcard Findings: ${securityReport.inboundPorts.filter((p) => p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH').length} ports
Egress Leaks: ${securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length} ports
SSH Hardening Non-Pass: ${securityReport.hardeningChecklist.filter((h) => h.status !== 'PASS').length} checks`;

      await saveInvestigation({
        title,
        evidence: desc,
        host_id: securityReport.hostId === 'local' ? '' : securityReport.hostId,
        status: securityReport.summaryScore < 70 ? 'OPEN' : 'CLOSED',
        notes: JSON.stringify(
          {
            security: securityReport,
            watchdog: watchdogResult,
            tls: tlsResult,
            exportedAt: new Date().toISOString()
          },
          null,
          2
        )
      });
      showToast(t('security.reportExported'), 'success');
    } catch (e) {
      showToast(t('security.incidentSaveFailed', { error: errorText(e) }), 'error');
    } finally {
      isSavingIncident = false;
    }
  }

  function getHostLabel(id: string): string {
    if (id === 'local') return 'Local Machine';
    const found = hosts.find((h) => h.id === id);
    return found ? `${found.label} (${found.address})` : id;
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <!-- Standard PageHeader -->
  <PageHeader
    icon={['M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z']}
    accent="neutral"
    title={t('security.title')}
    subtitle={t('security.subtitle')}
  >
    {#snippet actions()}
      <!-- Host Selector -->
      <div class="flex items-center gap-2 bg-neutral-100 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl px-3 h-9 shadow-xs">
        <label for="host-select" class="text-xs font-semibold text-neutral-500 uppercase">{t('security.targetHost')}:</label>
        <select
          id="host-select"
          bind:value={selectedHostId}
          class="bg-transparent text-xs font-bold text-neutral-900 dark:text-white focus:outline-none cursor-pointer"
        >
          <option value="local">{t('security.localhost')}</option>
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.address})</option>
          {/each}
        </select>
      </div>

      <!-- Quick Run Button -->
      <Button
        variant="primary"
        class="h-9 px-3.5 rounded-xl shrink-0 shadow-xs"
        onclick={startSecurityAudit}
        disabled={isRunningSecurity || !selectedHostId}
        loading={isRunningSecurity}
      >
        <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
        <span>{t('security.runSecurityAudit')}</span>
      </Button>

      {#if securityReport}
        <Button
          variant="secondary"
          class="h-9 px-3 rounded-xl shrink-0 flex items-center gap-1.5 text-xs shadow-xs"
          onclick={exportToInvestigation}
          disabled={isSavingIncident}
          loading={isSavingIncident}
        >
          <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
          <span>{t('security.exportIncident')}</span>
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  <!-- NAVIGATION TABS -->
  <div class="flex items-center gap-1.5 pb-2 overflow-x-auto scrollbar-none text-xs border-b border-neutral-200 dark:border-neutral-800">
    <Button
      variant={activeTab === 'exposure' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'exposure')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'exposure' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      <span>{t('security.tabExposure')}</span>
    </Button>
    <Button
      variant={activeTab === 'hardening' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'hardening')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'hardening' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg>
      <span>{t('security.tabHardening')}</span>
    </Button>
    <Button
      variant={activeTab === 'watchdog' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'watchdog')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'watchdog' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
      <span>{t('security.tabWatchdog')}</span>
      <Badge variant="neutral" size="xs">PRO</Badge>
    </Button>
    <Button
      variant={activeTab === 'tls' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'tls')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'tls' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
      <span>{t('security.tabTls')}</span>
    </Button>
    <Button
      variant={activeTab === 'laptop' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'laptop')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'laptop' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
      <span>{t('security.tabLaptop')}</span>
    </Button>
    <Button
      variant={activeTab === 'tripwire' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'tripwire')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'tripwire' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
      <span>{t('security.tabTripwire')}</span>
    </Button>
    <Button
      variant={activeTab === 'ephemeral' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => { activeTab = 'ephemeral'; loadEphemeralKeys(); }}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'ephemeral' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
      <span>{t('security.tabEphemeral')}</span>
      <Badge variant="neutral" size="xs">PRO</Badge>
    </Button>
    <Button
      variant={activeTab === 'jumpchain' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'jumpchain')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'jumpchain' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>
      <span>{t('security.tabJumpchain')}</span>
      <Badge variant="neutral" size="xs">PRO</Badge>
    </Button>
    <Button
      variant={activeTab === 'docker' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'docker')}
      class="h-8 px-3 rounded-lg text-xs shrink-0 flex items-center gap-1.5 {activeTab === 'docker' ? 'font-bold' : 'text-neutral-500 hover:text-neutral-900 dark:hover:text-white'}"
    >
      <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
      <span>{t('security.tabDocker')}</span>
      <Badge variant="neutral" size="xs">PRO</Badge>
    </Button>
  </div>

  <!-- TAB 1: EXPOSURE & HOLE HUNTER -->
  {#if activeTab === 'exposure'}
    <div class="space-y-6">
      {#if securityReport}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <Card variant="secondary" size="sm" title={t('security.securityScore')}>
            <span class="text-3xl font-black text-neutral-900 dark:text-white">{securityReport.summaryScore}%</span>
          </Card>
          <Card variant="secondary" size="sm" title={t('security.egressLeaks')}>
            <span class="text-2xl font-bold {securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length > 0 ? 'text-rose-500' : 'text-emerald-500'}">{securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length} {t('security.ports')}</span>
          </Card>
          <Card variant="secondary" size="sm" title={t('security.inboundWildcard')}>
            <span class="text-2xl font-bold {securityReport.inboundPorts.filter((p) => p.isWildcard && (p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH')).length > 0 ? 'text-amber-500' : 'text-emerald-500'}">{securityReport.inboundPorts.filter((p) => p.isWildcard && (p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH')).length} {t('security.ports')}</span>
          </Card>
          <Card variant="secondary" size="sm" title={t('security.latencyPing')}>
            <span class="text-2xl font-bold text-neutral-900 dark:text-white">{securityReport.latencyProbe.avgLatencyMs.toFixed(1)} ms</span>
          </Card>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card title={`Inbound Exposure (${securityReport.inboundPorts.length})`} variant="primary" size="md">
            <div class="max-h-72 overflow-y-auto space-y-1.5 text-xs font-mono">
              {#each securityReport.inboundPorts as p}
                <div class="p-2.5 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 flex items-center justify-between border border-neutral-100 dark:border-neutral-800/80">
                  <div>
                    <span class="font-bold text-neutral-900 dark:text-white">{p.localAddress}</span>
                    <span class="text-neutral-400 text-[11px] block">{p.process}</span>
                  </div>
                  <Badge variant={p.riskLevel === 'CRITICAL' ? 'danger' : p.riskLevel === 'HIGH' ? 'warning' : 'neutral'} size="xs">
                    {p.riskLevel}
                  </Badge>
                </div>
              {/each}
            </div>
          </Card>

          <Card title={`Egress Hole Hunter (${securityReport.egressResults.length})`} variant="primary" size="md">
            <div class="max-h-72 overflow-y-auto space-y-1.5 text-xs font-mono">
              {#each securityReport.egressResults as e}
                <div class="p-2.5 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 flex items-center justify-between border border-neutral-100 dark:border-neutral-800/80">
                  <div>
                    <span class="font-bold text-neutral-900 dark:text-white">{e.port} ({e.serviceLabel})</span>
                    <span class="text-neutral-400 text-[11px] block">{e.target}</span>
                  </div>
                  <Badge variant={e.status === 'OPEN_LEAK' ? 'danger' : 'success'} size="xs">
                    {e.status}
                  </Badge>
                </div>
              {/each}
            </div>
          </Card>
        </div>
      {:else}
        <div class="p-16 text-center bg-neutral-50 dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          <div class="w-12 h-12 rounded-full bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center mx-auto mb-4 text-neutral-400">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
          </div>
          <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-md mx-auto">{t('security.pressRunAudit')}</p>
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 2: SSH HARDENING -->
  {#if activeTab === 'hardening'}
    <div class="space-y-4">
      {#if securityReport}
        <Card title={t('security.sshHardening')} variant="primary" size="md">
          <div class="space-y-2 text-xs">
            {#each securityReport.hardeningChecklist as h}
              <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 flex items-center justify-between border border-neutral-100 dark:border-neutral-800/80">
                <div class="space-y-0.5">
                  <span class="font-bold text-neutral-900 dark:text-white">{h.name}</span>
                  <p class="text-neutral-500 text-[11px]">{h.details}</p>
                </div>
                <Badge variant={h.status === 'PASS' ? 'success' : h.status === 'FAIL' ? 'danger' : 'warning'} size="xs">
                  {h.status}
                </Badge>
              </div>
            {/each}
          </div>
        </Card>
      {:else}
        <div class="p-16 text-center bg-neutral-50 dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          <div class="w-12 h-12 rounded-full bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center mx-auto mb-4 text-neutral-400">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M7 11V7a5 5 0 0110 0v4"/><rect x="3" y="11" width="18" height="11" rx="2"/></svg>
          </div>
          <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-md mx-auto">{t('security.noAuditData')}</p>
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 3: THREAT WATCHDOG (PRO) -->
  {#if activeTab === 'watchdog'}
    <div class="space-y-4">
      <Card variant="secondary" size="md">
        <div class="flex items-center justify-between">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Reverse Shell & Outbound Watchdog</h4>
              <Badge variant="neutral" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-0.5">{t('security.watchdogDesc')}</p>
          </div>
          <Button
            variant="primary"
            size="sm"
            onclick={startThreatWatchdog}
            disabled={isRunningWatchdog || !selectedHostId}
            loading={isRunningWatchdog}
            class="h-8 px-3 rounded-lg text-xs font-semibold flex items-center gap-1.5"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
            <span>{t('security.runWatchdog')}</span>
          </Button>
        </div>

        <ProGate title="Reverse Shell Threat Watchdog Pro" description={t('security.watchdogProDesc')}>
          {#if watchdogResult}
            <div class="p-4 rounded-xl border {watchdogResult.threatLevel === 'CLEAN' ? 'border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40 text-neutral-900 dark:text-white' : 'border-rose-500/30 bg-rose-500/10 text-rose-800 dark:text-rose-300'}">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span class="font-bold text-xs uppercase tracking-wider">Status Threat Watchdog:</span>
                  <Badge variant={watchdogResult.threatLevel === 'CLEAN' ? 'success' : 'danger'} size="sm">{watchdogResult.threatLevel}</Badge>
                </div>
                <span class="text-[11px] opacity-80">{new Date(watchdogResult.timestamp).toLocaleTimeString()}</span>
              </div>
              <p class="text-xs mt-1">{watchdogResult.summary}</p>
              {#if watchdogResult.suspiciousSockets.length > 0}
                <div class="mt-3 space-y-1">
                  {#each watchdogResult.suspiciousSockets as s}
                    <div class="bg-black/20 p-2 rounded text-xs font-mono">
                      PID {s.pid} ({s.processName}): {s.exePath} &rarr; {s.remoteAddress || 'Local Socket'} [{s.reason}]
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else}
            <div class="py-8 text-center text-neutral-400 text-xs">{t('security.watchdogPressRun')}</div>
          {/if}
        </ProGate>
      </Card>
    </div>
  {/if}

  <!-- TAB 4: TLS AUDITOR -->
  {#if activeTab === 'tls'}
    <div class="space-y-4">
      <Card title={t('security.tlsCertificate')} description={t('security.tlsDesc')} variant="primary" size="md" class="max-w-xl">
        <div class="flex gap-2">
          <Input
            bind:value={tlsHost}
            placeholder="domain.com"
            size="md"
            class="flex-1 font-mono"
          />
          <Button
            variant="primary"
            class="h-9 px-3.5 rounded-xl text-xs font-semibold"
            onclick={startTlsAudit}
            disabled={isRunningTls}
            loading={isRunningTls}
          >
            <span>{isRunningTls ? t('security.checking') : t('security.auditTls')}</span>
          </Button>
        </div>

        {#if tlsResult}
          <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800/40 space-y-2 text-xs font-mono">
            <div class="flex justify-between items-center">
              <span class="text-neutral-500">{t('security.tlsStatus')}</span>
              <Badge variant={tlsResult.status === 'HEALTHY' ? 'success' : 'danger'} size="sm">{tlsResult.status}</Badge>
            </div>
            <div class="flex justify-between">
              <span class="text-neutral-500">{t('security.tlsDaysRemaining')}</span>
              <strong>{tlsResult.daysRemaining} {t('security.days')}</strong>
            </div>
            <div class="pt-2 border-t border-neutral-200 dark:border-neutral-700 text-[11px] text-neutral-400">
              Cipher Suite: {tlsResult.cipherSuite}
            </div>
          </div>
        {/if}
      </Card>
    </div>
  {/if}

  <!-- TAB 5: LAPTOP POSTURE SCANNER -->
  {#if activeTab === 'laptop'}
    <div class="space-y-6">
      <Card variant="primary" size="md">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">{t('security.laptopPostureTitle')}</h4>
              <Badge variant="neutral" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">{@html t('security.laptopPostureDesc')}</p>
          </div>
          <Button
            variant="primary"
            onclick={startLaptopScan}
            disabled={isRunningLaptop}
            loading={isRunningLaptop}
            class="h-9 px-3.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 shadow-xs"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
            <span>{t('security.scanLocalMachine')}</span>
          </Button>
        </div>

        {#if laptopReport}
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 pt-4">
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">{t('security.postureScore')}</span>
              <div class="text-3xl font-black mt-1 text-neutral-900 dark:text-white">
                {laptopReport.overallScore}%
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">{t('security.sshKeysScanned')}</span>
              <div class="text-2xl font-bold mt-1 text-neutral-900 dark:text-white">
                {laptopReport.scannedSshKeys}
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">{t('security.envFilesChecked')}</span>
              <div class="text-2xl font-bold mt-1 text-neutral-900 dark:text-white">
                {laptopReport.scannedEnvFiles}
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">{t('security.openPortsChecked')}</span>
              <div class="text-2xl font-bold mt-1 text-neutral-900 dark:text-white">
                {laptopReport.scannedListeningPorts}
              </div>
            </div>
          </div>

          <div class="space-y-3 pt-4">
            <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">{t('security.auditFindings')} ({laptopReport.findings.length})</h5>
            {#if laptopReport.findings.length === 0}
              <div class="p-4 rounded-xl bg-neutral-100 dark:bg-neutral-800/60 border border-neutral-200 dark:border-neutral-700 text-neutral-800 dark:text-neutral-200 text-xs flex items-center gap-2">
                <svg class="w-4 h-4 shrink-0 text-emerald-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                <span>{t('security.laptopPostureGood')}</span>
              </div>
            {:else}
              <div class="space-y-2">
                {#each laptopReport.findings as f}
                  <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/50 space-y-2">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <Badge variant={f.severity === 'critical' ? 'danger' : f.severity === 'high' ? 'warning' : 'neutral'} size="xs">
                          {f.severity}
                        </Badge>
                        <span class="font-bold text-xs text-neutral-900 dark:text-white">{f.title}</span>
                      </div>
                      {#if f.details}
                        <span class="text-[11px] font-mono text-neutral-500">{f.details}</span>
                      {/if}
                    </div>
                    <p class="text-xs text-neutral-400 leading-relaxed">{f.description}</p>
                    <div class="p-2.5 rounded-lg bg-neutral-900 border border-neutral-800 text-[11px] text-neutral-300 font-mono">
                      <span class="text-neutral-400 font-semibold font-sans">{t('security.fixRecommendation')}</span> {f.recommendation}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <div class="py-12 text-center text-neutral-400 text-xs border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl">
            {t('security.laptopPressScan')}
          </div>
        {/if}
      </Card>
    </div>
  {/if}

  <!-- TAB 6: INTEGRITY TRIPWIRE -->
  {#if activeTab === 'tripwire'}
    <div class="space-y-6">
      <Card variant="primary" size="md">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">{t('security.integrityBaseline')}</h4>
              <Badge variant="neutral" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">{@html t('security.integrityDesc')}</p>
          </div>
          <div class="flex items-center gap-2">
            <Button
              variant="secondary"
              size="sm"
              onclick={createBaseline}
              disabled={isRunningBaseline}
              loading={isRunningBaseline}
              class="h-8 px-3 rounded-lg text-xs font-semibold flex items-center gap-1.5"
            >
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2z"/><circle cx="12" cy="13" r="4"/></svg>
              <span>{t('security.createBaseline')}</span>
            </Button>
            <Button
              variant="primary"
              size="sm"
              onclick={runIntegrityCheck}
              disabled={isRunningTripwire || baselineRecords.length === 0}
              loading={isRunningTripwire}
              class="h-8 px-3 rounded-lg text-xs font-semibold flex items-center gap-1.5"
            >
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
              <span>{t('security.verifyIntegrity')}</span>
            </Button>
          </div>
        </div>

        {#if baselineRecords.length > 0}
          <div class="p-3 mt-4 rounded-xl bg-neutral-800/40 border border-neutral-700 text-xs flex items-center justify-between">
            <span class="text-neutral-300">{@html t('security.baselineActive', { count: baselineRecords.length })}</span>
            <span class="text-[11px] font-mono text-neutral-400">{t('security.baselineTarget')}</span>
          </div>
        {/if}

        {#if tripwireReport}
          <div class="space-y-3 pt-4">
            <div class="flex items-center justify-between">
              <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">{t('security.integrityResults')}</h5>
              <Badge variant={tripwireReport.tamperedCount === 0 ? 'success' : 'danger'} size="sm">
                {#if tripwireReport.tamperedCount === 0}
                  <span class="inline-flex items-center gap-1">
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                    <span>{t('security.allIdentical')}</span>
                  </span>
                {:else}
                  <span class="inline-flex items-center gap-1">
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                    <span>{t('security.filesChanged', { count: tripwireReport.tamperedCount })}</span>
                  </span>
                {/if}
              </Badge>
            </div>

            <div class="space-y-2">
              {#each tripwireReport.items as item}
                <div class="p-3.5 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40 space-y-1.5 font-mono text-xs">
                  <div class="flex items-center justify-between">
                    <span class="font-bold text-neutral-900 dark:text-white">{item.path}</span>
                    <Badge variant={item.status === 'unchanged' ? 'success' : 'danger'} size="xs">
                      {item.status}
                    </Badge>
                  </div>
                  {#if item.alertMessage}
                    <div class="p-2 rounded bg-rose-950/40 border border-rose-800/40 text-rose-300 text-[11px]">
                      {item.alertMessage}
                    </div>
                  {/if}
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-[11px] text-neutral-400 pt-1">
                    <div>
                      <span class="text-neutral-500">Baseline Hash:</span>
                      <div class="truncate text-neutral-300">{item.baselineHash}</div>
                    </div>
                    <div>
                      <span class="text-neutral-500">Current Hash:</span>
                      <div class="truncate {item.currentHash === item.baselineHash ? 'text-emerald-400' : 'text-rose-400'}">
                        {item.currentHash || 'N/A (Missing)'}
                      </div>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="py-12 mt-4 text-center text-neutral-400 text-xs border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl">
            {t('security.integrityPressVerify')}
          </div>
        {/if}
      </Card>
    </div>
  {/if}

  <!-- TAB 7: EPHEMERAL KEYS -->
  {#if activeTab === 'ephemeral'}
    <div class="space-y-6">
      <Card variant="primary" size="md">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">{t('security.ephemeralTitle')}</h4>
              <Badge variant="neutral" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">{t('security.ephemeralDesc')}</p>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-4">
          <div>
            <label for="eph-key-label" class="text-[11px] font-semibold text-neutral-400 block mb-1">{t('security.ephemeralKeyLabel')}</label>
            <Input
              id="eph-key-label"
              bind:value={ephKeyName}
              placeholder="e.g. CI/CD Deploy, Hotfix Session"
              size="md"
            />
          </div>
          <div>
            <label for="eph-key-ttl" class="text-[11px] font-semibold text-neutral-400 block mb-1">{t('security.ephemeralTtl')}</label>
            <select
              id="eph-key-ttl"
              bind:value={ephKeyTtl}
              class="w-full h-9 px-3 text-xs rounded-xl bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-900 dark:text-white focus:outline-hidden"
            >
              <option value={1800}>30 Minutes</option>
              <option value={3600}>1 Hour</option>
              <option value={7200}>2 Hours (Standard)</option>
              <option value={28800}>8 Hours (Full Shift)</option>
              <option value={86400}>24 Hours</option>
            </select>
          </div>
          <div>
            <label for="eph-key-target" class="text-[11px] font-semibold text-neutral-400 block mb-1">{t('security.ephemeralTarget')}</label>
            <Input
              id="eph-key-target"
              bind:value={ephTargetHost}
              placeholder="prod-db.internal"
              size="md"
            />
          </div>
        </div>

        <div class="pt-2">
          <Button
            variant="primary"
            onclick={handleGenerateEphemeralKey}
            disabled={isGeneratingEphemeral}
            loading={isGeneratingEphemeral}
            class="h-9 px-3.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 shadow-xs"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M21 2l-2 2m-1.5 1.5L14 9l-1.5-1.5L11 9l-1.5-1.5L8 9c-3.3 0-6 2.7-6 6s2.7 6 6 6 6-2.7 6-6c0-.8-.2-1.5-.5-2.2L22 4.2V2h-2.2z"/><circle cx="8" cy="15" r="1.5"/></svg>
            <span>{t('security.generateKey')}</span>
          </Button>
        </div>

        {#if newGeneratedPubkey}
          <div class="p-3.5 rounded-xl bg-neutral-100 dark:bg-neutral-800/80 border border-neutral-200 dark:border-neutral-700 text-xs space-y-2 mt-4">
            <div class="flex items-center justify-between">
              <span class="font-bold text-neutral-900 dark:text-white">{t('security.pubkeyHeader')}</span>
              <Button
                variant="secondary"
                size="sm"
                class="h-7 text-xs"
                onclick={() => {
                  navigator.clipboard.writeText(newGeneratedPubkey || '');
                  showToast(t('security.pubkeyCopied'), 'success');
                }}
              >
                Copy Key
              </Button>
            </div>
            <pre class="font-mono text-[11px] p-2 rounded bg-neutral-900/80 text-neutral-300 overflow-x-auto whitespace-pre-wrap">{newGeneratedPubkey}</pre>
          </div>
        {/if}

        <div class="space-y-2 pt-4">
          <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">{t('security.activeKeys', { count: ephemeralKeys.length })}</h5>
          {#if ephemeralKeys.length === 0}
            <div class="py-8 text-center text-neutral-500 text-xs border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl">
              {t('security.noEphemeralKeys')}
            </div>
          {:else}
            <div class="space-y-2">
              {#each ephemeralKeys as key}
                <div class="p-3 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40 flex items-center justify-between text-xs">
                  <div class="space-y-1">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-neutral-900 dark:text-white">{key.name}</span>
                      <Badge variant={key.isExpired || key.isRevoked ? 'danger' : 'neutral'} size="xs">
                        {key.isRevoked ? 'REVOKED' : key.isExpired ? 'EXPIRED' : `ACTIVE (${Math.round(key.remainingSeconds / 60)}m left)`}
                      </Badge>
                    </div>
                    <div class="font-mono text-[11px] text-neutral-400 truncate max-w-lg">{key.commentSignature}</div>
                  </div>
                  {#if !key.isRevoked && !key.isExpired}
                    <Button
                      variant="danger"
                      size="sm"
                      class="h-7 text-xs"
                      onclick={() => handleRevokeEphemeralKey(key.id)}
                    >
                      Revoke Now
                    </Button>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </Card>
    </div>
  {/if}

  <!-- TAB 8: JUMP CHAIN VISUALIZER -->
  {#if activeTab === 'jumpchain'}
    <div class="space-y-6">
      <Card variant="primary" size="md">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">{t('security.jumpchainTitle')}</h4>
              <Badge variant="neutral" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">{t('security.jumpchainDesc')}</p>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-4">
          <div class="sm:col-span-2">
            <label for="jump-bastions" class="text-[11px] font-semibold text-neutral-400 block mb-1">{t('security.jumpchainHops')}</label>
            <Input
              id="jump-bastions"
              bind:value={jumpBastions}
              placeholder="bastion1.corp:22, dmz-proxy.corp:2222"
              class="font-mono"
              size="md"
            />
          </div>
          <div>
            <label for="jump-target-host" class="text-[11px] font-semibold text-neutral-400 block mb-1">{t('security.jumpchainTarget')}</label>
            <div class="flex gap-2">
              <div class="flex-1">
                <Input
                  id="jump-target-host"
                  bind:value={jumpTargetHost}
                  placeholder="10.0.50.15"
                  class="font-mono"
                  size="md"
                />
              </div>
              <input
                type="number"
                aria-label="Target Port"
                bind:value={jumpTargetPort}
                class="w-16 h-9 px-2 text-xs rounded-xl bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-900 dark:text-white font-mono text-center focus:outline-hidden"
              />
            </div>
          </div>
        </div>

        <div class="pt-2">
          <Button
            variant="primary"
            onclick={handleProbeJumpChain}
            disabled={isProbingJumpChain}
            loading={isProbingJumpChain}
            class="h-9 px-3.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 shadow-xs"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>
            <span>{t('security.testChain')}</span>
          </Button>
        </div>

        {#if jumpProbeResult}
          <div class="p-4 mt-4 rounded-xl bg-neutral-50 dark:bg-neutral-850 border border-neutral-200 dark:border-neutral-800 space-y-3 font-mono text-xs">
            <div class="flex items-center justify-between">
              <span class="font-bold text-neutral-400">OpenSSH ProxyJump Directive:</span>
              <Badge variant={jumpProbeResult.isFullyTraversable ? 'success' : 'danger'} size="xs">
                {jumpProbeResult.isFullyTraversable ? 'ALL HOPS REACHABLE' : 'CHAIN DISRUPTED'}
              </Badge>
            </div>
            <pre class="p-2.5 rounded bg-neutral-900 text-neutral-200 font-mono text-xs overflow-x-auto border border-neutral-800">{jumpProbeResult.sshProxyCommand}</pre>

            <div class="space-y-2 pt-2">
              <div class="text-[11px] font-bold text-neutral-400 uppercase">Hop Health Breakdown:</div>
              {#each jumpProbeResult.hopStatuses as hop}
                <div class="p-2.5 rounded-lg bg-neutral-900/60 border border-neutral-800 flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="text-neutral-500">#{hop.hopIndex + 1}</span>
                    <span class="font-bold text-white">{hop.host}:{hop.port}</span>
                  </div>
                  <div class="flex items-center gap-3">
                    {#if hop.latencyMs}
                      <span class="text-neutral-400 text-[11px]">{hop.latencyMs}ms</span>
                    {/if}
                    <Badge variant={hop.reachable ? 'success' : 'danger'} size="xs">
                      {hop.reachable ? 'OK' : 'UNREACHABLE'}
                    </Badge>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </Card>
    </div>
  {/if}

  <!-- TAB 9: DOCKER DAEMON PROBE -->
  {#if activeTab === 'docker'}
    <div class="space-y-6">
      <Card variant="primary" size="md">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">{t('security.dockerTitle')}</h4>
              <Badge variant="neutral" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">{t('security.dockerDesc')}</p>
          </div>
          <Button
            variant="primary"
            onclick={handleRunDockerProbe}
            disabled={isRunningDockerProbe}
            loading={isRunningDockerProbe}
            class="h-9 px-3.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 shadow-xs"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
            <span>{t('security.runDockerProbe')}</span>
          </Button>
        </div>

        {#if dockerReport}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-4">
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-400 uppercase">{t('security.dockerScore')}</span>
              <div class="text-3xl font-black mt-2 text-neutral-900 dark:text-white">
                {dockerReport.securityScore}%
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-400 uppercase">{t('security.socketStatus')}</span>
              <div class="text-sm font-bold mt-2 font-mono text-neutral-900 dark:text-neutral-200">
                {dockerReport.dockerSocketPath} ({dockerReport.socketPermissions || 'N/A'})
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-400 uppercase">{t('security.findingsCount')}</span>
              <div class="text-2xl font-bold mt-2 text-neutral-900 dark:text-white">
                {dockerReport.findings.length} {t('security.issues')}
              </div>
            </div>
          </div>

          <div class="space-y-3 pt-4">
            <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">{t('security.dockerFindings')}</h5>
            {#if dockerReport.findings.length === 0}
              <div class="p-4 rounded-xl bg-neutral-100 dark:bg-neutral-800/60 border border-neutral-200 dark:border-neutral-700 text-neutral-800 dark:text-neutral-200 text-xs flex items-center gap-2">
                <svg class="w-4 h-4 shrink-0 text-emerald-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                <span>{t('security.dockerSafe')}</span>
              </div>
            {:else}
              <div class="space-y-2">
                {#each dockerReport.findings as f}
                  <div class="p-3.5 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40 space-y-1.5 text-xs">
                    <div class="flex items-center justify-between">
                      <span class="font-bold text-neutral-900 dark:text-white">{f.title}</span>
                      <Badge variant={f.severity === 'critical' ? 'danger' : 'warning'} size="xs">
                        {f.severity}
                      </Badge>
                    </div>
                    <p class="text-neutral-400 text-xs">{f.description}</p>
                    <div class="p-2.5 rounded-lg bg-neutral-900 border border-neutral-800 text-[11px] text-neutral-300 flex items-start gap-1.5 font-mono">
                      <span class="text-neutral-400 font-semibold font-sans">{t('security.solution')}</span> {f.recommendation}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </Card>
    </div>
  {/if}
</div>

