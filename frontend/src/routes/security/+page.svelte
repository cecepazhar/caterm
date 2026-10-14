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
      showToast('Pilih host target terlebih dahulu', 'error');
      return;
    }
    isRunningSecurity = true;
    securityReport = null;
    try {
      const res = await runNetworkSecurityAudit(selectedHostId);
      securityReport = res;
      showToast(`Audit keamanan selesai! Skor: ${res.summaryScore}%`, res.summaryScore > 70 ? 'success' : 'error');
    } catch (e) {
      showToast(`Audit gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningSecurity = false;
    }
  }

  async function startThreatWatchdog() {
    if (!selectedHostId) return;
    isRunningWatchdog = true;
    try {
      watchdogResult = await runThreatWatchdog(selectedHostId);
      showToast(`Threat Watchdog selesai: Status ${watchdogResult.threatLevel}`, watchdogResult.threatLevel === 'CLEAN' ? 'success' : 'error');
    } catch (e) {
      showToast(`Watchdog gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningWatchdog = false;
    }
  }

  async function startTlsAudit() {
    if (!tlsHost.trim()) return;
    isRunningTls = true;
    try {
      tlsResult = await runTlsAudit(tlsHost, tlsPort);
      showToast(`TLS Audit selesai: ${tlsResult.status}`, tlsResult.status === 'HEALTHY' ? 'success' : 'error');
    } catch (e) {
      showToast(`TLS Audit gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningTls = false;
    }
  }

  async function startLaptopScan() {
    isRunningLaptop = true;
    try {
      laptopReport = await scanLaptopPosture();
      showToast(`Laptop Posture Scan selesai! Skor: ${laptopReport.overallScore}%`, laptopReport.overallScore > 75 ? 'success' : 'error');
    } catch (e) {
      showToast(`Scan gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningLaptop = false;
    }
  }

  async function createBaseline() {
    isRunningBaseline = true;
    try {
      const records = await createIntegrityBaseline('local');
      baselineRecords = records;
      showToast(`Baseline hash dibuat untuk ${records.length} file kritis!`, 'success');
    } catch (e) {
      showToast(`Gagal membuat baseline: ${errorText(e)}`, 'error');
    } finally {
      isRunningBaseline = false;
    }
  }

  async function runIntegrityCheck() {
    if (baselineRecords.length === 0) {
      showToast('Buat baseline snapshot terlebih dahulu!', 'error');
      return;
    }
    isRunningTripwire = true;
    try {
      tripwireReport = await verifyIntegrityTripwire('local', baselineRecords);
      if (tripwireReport.tamperedCount === 0) {
        showToast('Integrity Check: Semua file konfigurasi identik & aman!', 'success');
      } else {
        showToast(`Integrity Alert: Ditemukan ${tripwireReport.tamperedCount} perubahan konfigurasi!`, 'error');
      }
    } catch (e) {
      showToast(`Integrity check gagal: ${errorText(e)}`, 'error');
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
      showToast(`Kunci Ed25519 Ephemeral dibuat (TTL: ${ephKeyTtl / 3600}h)!`, 'success');
    } catch (e) {
      showToast(`Gagal membuat ephemeral key: ${errorText(e)}`, 'error');
    } finally {
      isGeneratingEphemeral = false;
    }
  }

  async function handleRevokeEphemeralKey(id: string) {
    try {
      await revokeEphemeralKey(id);
      await loadEphemeralKeys();
      showToast('Ephemeral key berhasil di-revoke!', 'success');
    } catch (e) {
      showToast(`Revoke gagal: ${errorText(e)}`, 'error');
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
          ? 'Multi-hop Jump Chain berhasil diverifikasi!'
          : 'Beberapa hop dalam jump chain tidak dapat dijangkau!',
        jumpProbeResult.isFullyTraversable ? 'success' : 'error'
      );
    } catch (e) {
      showToast(`Probe jump chain gagal: ${errorText(e)}`, 'error');
    } finally {
      isProbingJumpChain = false;
    }
  }

  async function handleRunDockerProbe() {
    isRunningDockerProbe = true;
    try {
      dockerReport = await runDockerProbe();
      showToast(`Docker Leak Probe selesai! Skor: ${dockerReport.securityScore}%`, dockerReport.securityScore > 75 ? 'success' : 'error');
    } catch (e) {
      showToast(`Docker probe gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningDockerProbe = false;
    }
  }

  async function exportToInvestigation() {
    if (!securityReport) return;
    isSavingIncident = true;
    try {
      const hostLabel = getHostLabel(securityReport.hostId);
      const title = `Security Incident & Audit Report - Host ${hostLabel} (Skor: ${securityReport.summaryScore}%)`;
      const desc = `Laporan audit keamanan otomatis CATerm Security Suite.
Skor Keamanan: ${securityReport.summaryScore}%
Temuan Inbound Wildcard: ${securityReport.inboundPorts.filter((p) => p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH').length} port
Temuan Egress Leak: ${securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length} port
SSH Hardening Non-Pass: ${securityReport.hardeningChecklist.filter((h) => h.status !== 'PASS').length} check`;

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
      showToast('Laporan audit berhasil diekspor ke menu Investigations!', 'success');
    } catch (e) {
      showToast(`Gagal menyimpan incident: ${errorText(e)}`, 'error');
    } finally {
      isSavingIncident = false;
    }
  }

  function getHostLabel(id: string): string {
    if (id === 'local') return 'Local Machine (Fedora)';
    const found = hosts.find((h) => h.id === id);
    return found ? `${found.label} (${found.address})` : id;
  }
</script>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- HEADER -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-neutral-900 border border-neutral-800 flex items-center justify-center text-emerald-500 shrink-0">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
      </div>
      <div>
        <h1 class="text-xl font-bold text-white tracking-tight">Security</h1>
        <p class="text-xs text-neutral-400 mt-0.5">Audit celah port listening 0.0.0.0, outbound hole hunter, integritas SSH hardening, dan live reverse shell watchdog.</p>
      </div>
    </div>
    <div class="flex flex-wrap items-center gap-3">
      <!-- Host Selector -->
      <div class="flex items-center gap-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl px-3 py-1.5 shadow-xs">
        <label for="host-select" class="text-xs font-semibold text-neutral-500 uppercase">Target Host:</label>
        <select
          id="host-select"
          bind:value={selectedHostId}
          class="bg-transparent text-xs font-bold text-neutral-900 dark:text-white focus:outline-none cursor-pointer"
        >
          <option value="local">Localhost (Mesin Ini)</option>
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.address})</option>
          {/each}
        </select>
      </div>

      <!-- Quick Run Button -->
      <Button
        variant="success"
        size="sm"
        onclick={startSecurityAudit}
        disabled={isRunningSecurity || !selectedHostId}
        loading={isRunningSecurity}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
        <span>Run Security Audit</span>
      </Button>

      {#if securityReport}
        <Button
          variant="secondary"
          size="sm"
          onclick={exportToInvestigation}
          disabled={isSavingIncident}
          loading={isSavingIncident}
        >
          <span>📁 Export Incident</span>
        </Button>
      {/if}
    </div>
  </div>

  <!-- NAVIGATION TABS -->
  <div class="flex items-center gap-2 border-b border-neutral-200 dark:border-neutral-800 pb-2 overflow-x-auto text-xs font-semibold">
    <Button
      variant={activeTab === 'exposure' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'exposure')}
      class="whitespace-nowrap {activeTab === 'exposure' ? 'text-emerald-500 font-bold' : ''}"
    >
      <span>🛡️ Exposure & Hole Hunter</span>
    </Button>
    <Button
      variant={activeTab === 'hardening' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'hardening')}
      class="whitespace-nowrap {activeTab === 'hardening' ? 'text-sky-500 font-bold' : ''}"
    >
      <span>🔒 SSH Hardening Checklist</span>
    </Button>
    <Button
      variant={activeTab === 'watchdog' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'watchdog')}
      class="whitespace-nowrap {activeTab === 'watchdog' ? 'text-rose-500 font-bold' : ''}"
    >
      <span>🚨 Threat Watchdog (Pro)</span>
      <Badge variant="danger" size="xs">PRO</Badge>
    </Button>
    <Button
      variant={activeTab === 'tls' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'tls')}
      class="whitespace-nowrap {activeTab === 'tls' ? 'text-amber-500 font-bold' : ''}"
    >
      <span>📜 TLS & SSL Auditor</span>
    </Button>
    <Button
      variant={activeTab === 'laptop' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'laptop')}
      class="whitespace-nowrap {activeTab === 'laptop' ? 'text-indigo-500 font-bold' : ''}"
    >
      <span>💻 Laptop Posture Scanner</span>
    </Button>
    <Button
      variant={activeTab === 'tripwire' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'tripwire')}
      class="whitespace-nowrap {activeTab === 'tripwire' ? 'text-rose-500 font-bold' : ''}"
    >
      <span>⚡ Integrity Tripwire</span>
    </Button>
    <Button
      variant={activeTab === 'ephemeral' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => { activeTab = 'ephemeral'; loadEphemeralKeys(); }}
      class="whitespace-nowrap {activeTab === 'ephemeral' ? 'text-purple-500 font-bold' : ''}"
    >
      <span>⏳ Ephemeral SSH Keys</span>
      <Badge variant="brand" size="xs">PRO</Badge>
    </Button>
    <Button
      variant={activeTab === 'jumpchain' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'jumpchain')}
      class="whitespace-nowrap {activeTab === 'jumpchain' ? 'text-cyan-500 font-bold' : ''}"
    >
      <span>🔗 Bastion Jump Visualizer</span>
      <Badge variant="info" size="xs">PRO</Badge>
    </Button>
    <Button
      variant={activeTab === 'docker' ? 'secondary' : 'ghost'}
      size="sm"
      onclick={() => (activeTab = 'docker')}
      class="whitespace-nowrap {activeTab === 'docker' ? 'text-emerald-500 font-bold' : ''}"
    >
      <span>🐳 Docker Daemon Probe</span>
      <Badge variant="success" size="xs">PRO</Badge>
    </Button>
  </div>

  <!-- TAB 1: EXPOSURE & HOLE HUNTER -->
  {#if activeTab === 'exposure'}
    <div class="space-y-6">
      {#if securityReport}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <Card variant="secondary" size="sm" title="Skor Keamanan">
            <span class="text-3xl font-black {securityReport.summaryScore >= 80 ? 'text-emerald-500' : securityReport.summaryScore >= 60 ? 'text-amber-500' : 'text-rose-500'}">{securityReport.summaryScore}%</span>
          </Card>
          <Card variant="secondary" size="sm" title="Egress Leaks">
            <span class="text-2xl font-bold {securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length > 0 ? 'text-rose-500' : 'text-emerald-500'}">{securityReport.egressResults.filter((e) => e.status === 'OPEN_LEAK').length} port</span>
          </Card>
          <Card variant="secondary" size="sm" title="Inbound Wildcard (0.0.0.0)">
            <span class="text-2xl font-bold {securityReport.inboundPorts.filter((p) => p.isWildcard && (p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH')).length > 0 ? 'text-amber-500' : 'text-emerald-500'}">{securityReport.inboundPorts.filter((p) => p.isWildcard && (p.riskLevel === 'CRITICAL' || p.riskLevel === 'HIGH')).length} port</span>
          </Card>
          <Card variant="secondary" size="sm" title="Latency Ping Target">
            <span class="text-2xl font-bold text-sky-500">{securityReport.latencyProbe.avgLatencyMs.toFixed(1)} ms</span>
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
        <div class="py-16 text-center text-neutral-400 text-xs bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          Tekan "Run Security Audit" untuk memindai exposure port dan lubang outbound pada target host.
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 2: SSH HARDENING -->
  {#if activeTab === 'hardening'}
    <div class="space-y-4">
      {#if securityReport}
        <Card title="SSH Configuration Hardening Audit" variant="primary" size="md">
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
        <div class="py-16 text-center text-neutral-400 text-xs bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          Belum ada data audit. Silakan jalankan Security Audit terlebih dahulu.
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
              <Badge variant="warning" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-0.5">Mendeteksi proses siluman / socket outbound mencurigakan yang berpotensi menjadi backdoor atau reverse shell.</p>
          </div>
          <Button
            variant="danger"
            size="sm"
            onclick={startThreatWatchdog}
            disabled={isRunningWatchdog || !selectedHostId}
            loading={isRunningWatchdog}
          >
            <span>🚨 Jalankan Watchdog</span>
          </Button>
        </div>

        <ProGate title="Reverse Shell Threat Watchdog Pro" description="Pemantauan soket outbound berisiko tinggi dan deteksi reverse shell secara live membutuhkan CATerm Pro.">
          {#if watchdogResult}
            <div class="p-4 rounded-xl border {watchdogResult.threatLevel === 'CLEAN' ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-800 dark:text-emerald-300' : 'border-rose-500/30 bg-rose-500/10 text-rose-800 dark:text-rose-300'}">
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
            <div class="py-8 text-center text-neutral-400 text-xs">Tekan "Jalankan Watchdog" untuk memindai soket mencurigakan pada target.</div>
          {/if}
        </ProGate>
      </Card>
    </div>
  {/if}

  <!-- TAB 4: TLS AUDITOR -->
  {#if activeTab === 'tls'}
    <div class="space-y-4">
      <Card title="TLS Certificate Auditor" description="Audit tanggal kedaluwarsa sertifikat SSL/TLS, cipher suite, dan status validitas rantai sertifikat." variant="primary" size="md" class="max-w-xl">
        <div class="flex gap-2">
          <Input
            bind:value={tlsHost}
            placeholder="domain.com"
            class="flex-1 font-mono"
          />
          <Button
            variant="info"
            size="sm"
            onclick={startTlsAudit}
            disabled={isRunningTls}
            loading={isRunningTls}
          >
            <span>{isRunningTls ? 'Checking...' : 'Audit TLS'}</span>
          </Button>
        </div>

        {#if tlsResult}
          <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800/40 space-y-2 text-xs font-mono">
            <div class="flex justify-between items-center">
              <span class="text-neutral-500">Status Validitas:</span>
              <Badge variant={tlsResult.status === 'HEALTHY' ? 'success' : 'danger'} size="sm">{tlsResult.status}</Badge>
            </div>
            <div class="flex justify-between">
              <span class="text-neutral-500">Sisa Masa Aktif:</span>
              <strong>{tlsResult.daysRemaining} hari</strong>
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
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Local Machine Posture Checker</h4>
              <Badge variant="brand" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">Audit izin file <code class="text-neutral-300">~/.ssh</code> (0600 vs 0644), secret pada <code class="text-neutral-300">.env</code> world-readable, dan exposed unauthenticated listening ports.</p>
          </div>
          <Button
            variant="brand"
            onclick={startLaptopScan}
            disabled={isRunningLaptop}
            loading={isRunningLaptop}
          >
            <span>💻 Scan Local Machine</span>
          </Button>
        </div>

        {#if laptopReport}
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 pt-4">
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">Posture Score</span>
              <div class="text-3xl font-black mt-1 {laptopReport.overallScore >= 80 ? 'text-emerald-500' : laptopReport.overallScore >= 60 ? 'text-amber-500' : 'text-rose-500'}">
                {laptopReport.overallScore}%
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">SSH Keys Scanned</span>
              <div class="text-2xl font-bold mt-1 text-sky-400">
                {laptopReport.scannedSshKeys}
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">.env Files Checked</span>
              <div class="text-2xl font-bold mt-1 text-amber-400">
                {laptopReport.scannedEnvFiles}
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-500 uppercase">Open Local Ports Checked</span>
              <div class="text-2xl font-bold mt-1 text-purple-400">
                {laptopReport.scannedListeningPorts}
              </div>
            </div>
          </div>

          <div class="space-y-3 pt-4">
            <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">Audit Findings ({laptopReport.findings.length})</h5>
            {#if laptopReport.findings.length === 0}
              <div class="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs flex items-center gap-2">
                <span>✓</span> Mesin lokal Anda dalam postur prima: izin SSH aman, tidak ada kebocoran .env, dan tidak ada port database tanpa autentikasi yang terekspos.
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
                      <span class="text-indigo-400 font-semibold font-sans">Saran Perbaikan:</span> {f.recommendation}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <div class="py-12 text-center text-neutral-400 text-xs border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl">
            Klik "Scan Local Machine" untuk memeriksa keamanan permission SSH, plaintext .env, dan port rentan pada laptop Anda.
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
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Server Configuration Baseline & Tamper Tripwire</h4>
              <Badge variant="danger" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">Snapshot kriptografis SHA-256 berkas konfigurasi kunci (<code class="text-neutral-300">/etc/passwd</code>, <code class="text-neutral-300">/etc/sudoers</code>, <code class="text-neutral-300">sshd_config</code>, <code class="text-neutral-300">crontab</code>) untuk mendeteksi tampering seketika.</p>
          </div>
          <div class="flex items-center gap-2">
            <Button
              variant="secondary"
              size="sm"
              onclick={createBaseline}
              disabled={isRunningBaseline}
              loading={isRunningBaseline}
            >
              <span>📷 Create Baseline Snapshot</span>
            </Button>
            <Button
              variant="danger"
              size="sm"
              onclick={runIntegrityCheck}
              disabled={isRunningTripwire || baselineRecords.length === 0}
              loading={isRunningTripwire}
            >
              <span>⚡ Verify Integrity</span>
            </Button>
          </div>
        </div>

        {#if baselineRecords.length > 0}
          <div class="p-3 mt-4 rounded-xl bg-neutral-800/40 border border-neutral-700 text-xs flex items-center justify-between">
            <span class="text-neutral-300">Baseline aktif memantau <strong>{baselineRecords.length}</strong> file konfigurasi.</span>
            <span class="text-[11px] font-mono text-neutral-400">Target: Local / Remote Baseline</span>
          </div>
        {/if}

        {#if tripwireReport}
          <div class="space-y-3 pt-4">
            <div class="flex items-center justify-between">
              <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">Hasil Verifikasi Integritas</h5>
              <Badge variant={tripwireReport.tamperedCount === 0 ? 'success' : 'danger'} size="sm">
                {tripwireReport.tamperedCount === 0 ? '✓ SEMUA BERKAS IDENTIK' : `🚨 ${tripwireReport.tamperedCount} BERKAS BERUBAH / TAMPERED`}
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
            Buat "Create Baseline Snapshot" terlebih dahulu, lalu klik "Verify Integrity" kapan saja untuk memeriksa apakah ada perubahan tidak sah pada konfigurasi sistem.
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
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Ephemeral Self-Destructing SSH Keys</h4>
              <Badge variant="brand" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">Hasilkan pasangan kunci Ed25519 sementara dengan masa berlaku TTL terbatas. Otomatis dibersihkan dari server target setelah kedaluwarsa.</p>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-4">
          <div>
            <label class="text-[11px] font-semibold text-neutral-400 block mb-1">Key Label / Purpose</label>
            <Input
              bind:value={ephKeyName}
              placeholder="e.g. CI/CD Deploy, Hotfix Session"
              size="sm"
            />
          </div>
          <div>
            <label class="text-[11px] font-semibold text-neutral-400 block mb-1">TTL (Time to Live)</label>
            <select
              bind:value={ephKeyTtl}
              class="w-full px-3 py-2 text-xs rounded-xl bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-900 dark:text-white focus:outline-hidden"
            >
              <option value={1800}>30 Minutes</option>
              <option value={3600}>1 Hour</option>
              <option value={7200}>2 Hours (Standard)</option>
              <option value={28800}>8 Hours (Full Shift)</option>
              <option value={86400}>24 Hours</option>
            </select>
          </div>
          <div>
            <label class="text-[11px] font-semibold text-neutral-400 block mb-1">Target Host / Bastion</label>
            <Input
              bind:value={ephTargetHost}
              placeholder="prod-db.internal"
              size="sm"
            />
          </div>
        </div>

        <div class="pt-2">
          <Button
            variant="brand"
            onclick={handleGenerateEphemeralKey}
            disabled={isGeneratingEphemeral}
            loading={isGeneratingEphemeral}
          >
            <span>🔑 Generate Self-Destructing Key</span>
          </Button>
        </div>

        {#if newGeneratedPubkey}
          <div class="p-3.5 rounded-xl bg-purple-950/20 border border-purple-800/40 text-xs space-y-2 mt-4">
            <div class="flex items-center justify-between">
              <span class="font-bold text-purple-300">Public Key Injeksi (Authorized Keys):</span>
              <Button
                variant="secondary"
                size="sm"
                onclick={() => {
                  navigator.clipboard.writeText(newGeneratedPubkey || '');
                  showToast('Public key disalin ke clipboard!', 'success');
                }}
              >
                Copy Key
              </Button>
            </div>
            <pre class="font-mono text-[11px] p-2 rounded bg-neutral-900/80 text-neutral-300 overflow-x-auto whitespace-pre-wrap">{newGeneratedPubkey}</pre>
          </div>
        {/if}

        <div class="space-y-2 pt-4">
          <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">Daftar Kunci Aktif ({ephemeralKeys.length})</h5>
          {#if ephemeralKeys.length === 0}
            <div class="py-8 text-center text-neutral-500 text-xs border border-dashed border-neutral-800 rounded-xl">
              Belum ada ephemeral keys yang dibuat.
            </div>
          {:else}
            <div class="space-y-2">
              {#each ephemeralKeys as key}
                <div class="p-3 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40 flex items-center justify-between text-xs">
                  <div class="space-y-1">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-neutral-900 dark:text-white">{key.name}</span>
                      <Badge variant={key.isExpired || key.isRevoked ? 'danger' : 'success'} size="xs">
                        {key.isRevoked ? 'REVOKED' : key.isExpired ? 'EXPIRED' : `ACTIVE (${Math.round(key.remainingSeconds / 60)}m left)`}
                      </Badge>
                    </div>
                    <div class="font-mono text-[11px] text-neutral-400 truncate max-w-lg">{key.commentSignature}</div>
                  </div>
                  {#if !key.isRevoked && !key.isExpired}
                    <Button
                      variant="danger"
                      size="sm"
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
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Multi-Hop Bastion & Jump Proxy Visualizer</h4>
              <Badge variant="info" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">Rancang rantai loncatan multi-hop SSH (Local -> Bastion 1 -> Bastion 2 -> Target Host) dan uji konektivitas per-hop.</p>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-4">
          <div class="sm:col-span-2">
            <label class="text-[11px] font-semibold text-neutral-400 block mb-1">Bastion Hops (Comma Separated host:port)</label>
            <Input
              bind:value={jumpBastions}
              placeholder="bastion1.corp:22, dmz-proxy.corp:2222"
              class="font-mono"
              size="sm"
            />
          </div>
          <div>
            <label class="text-[11px] font-semibold text-neutral-400 block mb-1">Target Host & Port</label>
            <div class="flex gap-2">
              <div class="flex-1">
                <Input
                  bind:value={jumpTargetHost}
                  placeholder="10.0.50.15"
                  class="font-mono"
                  size="sm"
                />
              </div>
              <input
                type="number"
                bind:value={jumpTargetPort}
                class="w-16 px-2 py-2 text-xs rounded-xl bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-900 dark:text-white font-mono text-center focus:outline-hidden"
              />
            </div>
          </div>
        </div>

        <div class="pt-2">
          <Button
            variant="info"
            onclick={handleProbeJumpChain}
            disabled={isProbingJumpChain}
            loading={isProbingJumpChain}
          >
            <span>🔗 Test & Validate Chain</span>
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
            <pre class="p-2.5 rounded bg-neutral-900 text-cyan-300 font-mono text-xs overflow-x-auto">{jumpProbeResult.sshProxyCommand}</pre>

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
              <h4 class="text-sm font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Docker Daemon & Container Leak Probe</h4>
              <Badge variant="success" size="xs">PRO</Badge>
            </div>
            <p class="text-xs text-neutral-500 mt-1">Pindai izin berkas socket Docker, deteksi unauthenticated TCP port 2375, dan identifikasi kontainer dengan flag --privileged atau bind mount berbahaya ke host root.</p>
          </div>
          <Button
            variant="success"
            onclick={handleRunDockerProbe}
            disabled={isRunningDockerProbe}
            loading={isRunningDockerProbe}
          >
            <span>🐳 Run Docker Probe</span>
          </Button>
        </div>

        {#if dockerReport}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-4">
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-400 uppercase">Docker Security Score</span>
              <div class="text-3xl font-black mt-2 {dockerReport.securityScore >= 80 ? 'text-emerald-400' : dockerReport.securityScore >= 60 ? 'text-amber-400' : 'text-rose-400'}">
                {dockerReport.securityScore}%
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-400 uppercase">Socket Status</span>
              <div class="text-sm font-bold mt-2 font-mono text-neutral-200">
                {dockerReport.dockerSocketPath} ({dockerReport.socketPermissions || 'N/A'})
              </div>
            </div>
            <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40">
              <span class="text-xs font-semibold text-neutral-400 uppercase">Findings Count</span>
              <div class="text-2xl font-bold mt-2 {dockerReport.findings.length === 0 ? 'text-emerald-400' : 'text-rose-400'}">
                {dockerReport.findings.length} Issue(s)
              </div>
            </div>
          </div>

          <div class="space-y-3 pt-4">
            <h5 class="text-xs font-bold uppercase tracking-wider text-neutral-400">Security Findings</h5>
            {#if dockerReport.findings.length === 0}
              <div class="p-4 rounded-xl bg-emerald-950/20 border border-emerald-800/40 text-emerald-300 text-xs">
                ✓ Konfigurasi daemon Docker dan seluruh container berada dalam parameter aman!
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
                    <div class="p-2 rounded bg-neutral-900/60 border border-neutral-800 text-[11px] text-emerald-400">
                      💡 Solusi: {f.recommendation}
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

