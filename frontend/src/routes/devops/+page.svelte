<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    runServerBenchmark,
    runNetworkSpeedTest,
    runQosAudit,
    runMeshLatencyMatrix,
    runPmtudProbe,
    runSubnetSweep,
    type ServerBenchmarkResult,
    type SpeedTestResult,
    type QosAuditResult,
    type MeshLatencyMatrix,
    type PmtudResult,
    type SubnetSweepResult
  } from '$lib/api/audit';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import { PageHeader, Button, Input, Badge, Tabs } from '@cecepazhar/caui';
  import ProGate from '$lib/components/ProGate.svelte';

  let hosts = $state<HostRecord[]>([]);
  let selectedHostId = $state<string>('');
  let activeTab = $state<'speed_qos' | 'benchmark' | 'mesh' | 'network_tools'>('speed_qos');

  interface DevOpsTabItem {
    id: string;
    label: string;
    badge?: string | number;
  }

  let devopsTabs: DevOpsTabItem[] = [
    { id: 'speed_qos', label: '⚡ Throughput & QoS' },
    { id: 'benchmark', label: '🚀 Hardware Benchmark' },
    { id: 'mesh', label: '🕸️ Mesh Matrix', badge: 'PRO' },
    { id: 'network_tools', label: '🔍 PMTUD & Subnet' }
  ];

  // Loading States
  let isRunningSpeed = $state(false);
  let isRunningQos = $state(false);
  let isRunningBench = $state(false);
  let isRunningMesh = $state(false);
  let isRunningSubnet = $state(false);
  let isRunningPmtud = $state(false);

  // Results State
  let speedResult = $state<SpeedTestResult | null>(null);
  let qosResult = $state<QosAuditResult | null>(null);
  let benchResult = $state<ServerBenchmarkResult | null>(null);
  let meshResult = $state<MeshLatencyMatrix | null>(null);
  let subnetResult = $state<SubnetSweepResult | null>(null);
  let pmtudResult = $state<PmtudResult | null>(null);

  // Tools Inputs
  let subnetCidr = $state('192.168.1.0/24');
  let pmtudTarget = $state('1.1.1.1');

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

  async function startSpeedTest() {
    if (!selectedHostId) return;
    isRunningSpeed = true;
    try {
      speedResult = await runNetworkSpeedTest(selectedHostId);
      showToast(`Speed Test selesai: Down ${speedResult.downloadMbps} Mbps, Up ${speedResult.uploadMbps} Mbps`, 'success');
    } catch (e) {
      showToast(`Speed Test gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningSpeed = false;
    }
  }

  async function startQosAudit() {
    if (!selectedHostId) return;
    isRunningQos = true;
    try {
      qosResult = await runQosAudit(selectedHostId);
      showToast(`QoS Audit selesai: Grade ${qosResult.bufferbloatGrade}`, 'success');
    } catch (e) {
      showToast(`QoS Audit gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningQos = false;
    }
  }

  async function startServerBenchmark() {
    if (!selectedHostId) return;
    isRunningBench = true;
    try {
      benchResult = await runServerBenchmark(selectedHostId);
      showToast(`Benchmark selesai! RAM: ${benchResult.ramBandwidthMbps} MB/s, Disk: ${benchResult.diskWriteMbps1m} MB/s`, 'success');
    } catch (e) {
      showToast(`Benchmark gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningBench = false;
    }
  }

  async function startMeshMatrix() {
    if (hosts.length === 0) {
      showToast('Perlu minimal 1 host untuk menguji mesh matrix', 'error');
      return;
    }
    isRunningMesh = true;
    try {
      const targetHosts: [string, string][] = hosts.map((h) => [h.id, h.label]);
      meshResult = await runMeshLatencyMatrix(selectedHostId || 'local', targetHosts);
      showToast('Mesh Latency Matrix selesai dihitung!', 'success');
    } catch (e) {
      showToast(`Mesh Matrix gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningMesh = false;
    }
  }

  async function startSubnetSweep() {
    if (!selectedHostId) return;
    isRunningSubnet = true;
    try {
      subnetResult = await runSubnetSweep(selectedHostId, subnetCidr);
      showToast(`Subnet Sweep selesai: ${subnetResult.activeHosts} host aktif`, 'success');
    } catch (e) {
      showToast(`Subnet Sweep gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningSubnet = false;
    }
  }

  async function startPmtudProbe() {
    if (!selectedHostId) return;
    isRunningPmtud = true;
    try {
      pmtudResult = await runPmtudProbe(selectedHostId, pmtudTarget);
      showToast(`PMTUD Selesai! Optimal MTU: ${pmtudResult.optimalMtu}`, 'success');
    } catch (e) {
      showToast(`PMTUD gagal: ${errorText(e)}`, 'error');
    } finally {
      isRunningPmtud = false;
    }
  }
</script>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- HEADER -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-neutral-900 border border-neutral-800 flex items-center justify-center text-[var(--ca-brand,#ef4444)] shrink-0">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
      </div>
      <div>
        <h1 class="text-xl font-bold text-white tracking-tight">DevOps</h1>
        <p class="text-xs text-neutral-400 mt-0.5">Benchmark komputasi server, throughput kecepatan jaringan & bufferbloat QoS, serta mesh latency matrix multi-node.</p>
      </div>
    </div>
    <div class="flex items-center gap-3">
      <!-- Host Selector -->
      <div class="flex items-center gap-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl px-3 py-1.5 shadow-xs">
        <label for="host-select-devops" class="text-xs font-semibold text-neutral-500 uppercase">Target Host:</label>
        <select
          id="host-select-devops"
          bind:value={selectedHostId}
          class="bg-transparent text-xs font-bold text-neutral-900 dark:text-white focus:outline-none cursor-pointer"
        >
          <option value="local">Localhost (Mesin Ini)</option>
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.address})</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <!-- NAVIGATION TABS -->
  <div class="border-b border-neutral-200 dark:border-neutral-800 pb-2">
    <Tabs items={devopsTabs} bind:value={activeTab} variant="pills" size="sm" />
  </div>

  <!-- TAB 1: SPEED TEST & QOS -->
  {#if activeTab === 'speed_qos'}
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
        <div>
          <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Throughput & QoS Bufferbloat Auditor</h3>
          <p class="text-xs text-neutral-500">Mengukur bandwidth download/upload real-time dan degradasi latensi di bawah beban (Bufferbloat Grade A+ hingga F).</p>
        </div>
        <div class="flex items-center gap-2">
          <Button
            variant="primary"
            size="sm"
            onclick={startSpeedTest}
            disabled={isRunningSpeed || !selectedHostId}
            loading={isRunningSpeed}
          >
            ⚡ Jalankan Speed Test
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onclick={startQosAudit}
            disabled={isRunningQos || !selectedHostId}
            loading={isRunningQos}
          >
            📊 Audit Bufferbloat QoS
          </Button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Speed Results -->
        <div class="p-6 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Bandwidth Throughput</h4>
          {#if speedResult}
            <div class="grid grid-cols-2 gap-4">
              <div class="p-4 rounded-xl bg-sky-500/10 border border-sky-500/20">
                <span class="text-xs text-neutral-400">Download</span>
                <div class="text-3xl font-black text-sky-600 dark:text-sky-400 mt-1">{speedResult.downloadMbps} <span class="text-xs font-normal">Mbps</span></div>
              </div>
              <div class="p-4 rounded-xl bg-indigo-500/10 border border-indigo-500/20">
                <span class="text-xs text-neutral-400">Upload</span>
                <div class="text-3xl font-black text-indigo-600 dark:text-indigo-400 mt-1">{speedResult.uploadMbps} <span class="text-xs font-normal">Mbps</span></div>
              </div>
            </div>
            <div class="text-xs text-neutral-500 font-mono space-y-1">
              <div>Latency: <strong>{speedResult.pingMs} ms</strong> (Jitter: {speedResult.jitterMs} ms)</div>
              <div>Target: {speedResult.serverTarget} ({speedResult.status})</div>
            </div>
          {:else}
            <div class="py-12 text-center text-neutral-400 text-xs">Tekan "Jalankan Speed Test" untuk menguji bandwidth target.</div>
          {/if}
        </div>

        <!-- QoS Bufferbloat -->
        <div class="p-6 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Bufferbloat QoS Rating</h4>
          {#if qosResult}
            <div class="flex items-center gap-4 p-4 rounded-xl bg-neutral-50 dark:bg-neutral-800/50 border border-neutral-200 dark:border-neutral-700">
              <div class="w-16 h-16 rounded-2xl flex items-center justify-center font-black text-2xl {qosResult.bufferbloatGrade.startsWith('A') ? 'bg-emerald-500/20 text-emerald-500' : qosResult.bufferbloatGrade.startsWith('B') ? 'bg-sky-500/20 text-sky-500' : 'bg-rose-500/20 text-rose-500'}">
                {qosResult.bufferbloatGrade}
              </div>
              <div>
                <span class="text-xs text-neutral-400">Degradasi Latensi Saat Beban Penuh:</span>
                <div class="text-sm font-bold text-neutral-900 dark:text-white mt-0.5">+{qosResult.deltaLoadMs} ms delay</div>
                <p class="text-xs text-neutral-500 mt-1">{qosResult.recommendation}</p>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-2 text-xs font-mono">
              <div class="p-2.5 rounded-lg bg-neutral-100 dark:bg-neutral-800/40">Idle Ping: <strong>{qosResult.idleLatencyMs} ms</strong></div>
              <div class="p-2.5 rounded-lg bg-neutral-100 dark:bg-neutral-800/40">Loaded Ping: <strong>{qosResult.downloadLoadLatencyMs} ms</strong></div>
            </div>
          {:else}
            <div class="py-12 text-center text-neutral-400 text-xs">Tekan "Audit Bufferbloat QoS" untuk mendiagnosis bufferbloat antrian router.</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- TAB 2: SERVER HARDWARE BENCHMARK -->
  {#if activeTab === 'benchmark'}
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800">
        <div>
          <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Server Hardware Benchmark Suite</h3>
          <p class="text-xs text-neutral-500">Benchmark komputasi multi-core CPU, throughput baca/tulis RAM, dan sequential & random I/O write speed.</p>
        </div>
        <Button
          variant="primary"
          size="sm"
          onclick={startServerBenchmark}
          disabled={isRunningBench || !selectedHostId}
          loading={isRunningBench}
        >
          🚀 Jalankan Server Benchmark
        </Button>
      </div>

      {#if benchResult}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">CPU Single-Core</span>
            <div class="text-2xl font-black text-neutral-900 dark:text-white mt-1">{benchResult.cpuSingleScore} <span class="text-xs font-normal text-neutral-400">pts</span></div>
          </div>
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">CPU Multi-Core</span>
            <div class="text-2xl font-black text-amber-500 mt-1">{benchResult.cpuMultiScore} <span class="text-xs font-normal text-neutral-400">pts</span></div>
          </div>
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">RAM Bandwidth</span>
            <div class="text-2xl font-black text-sky-500 mt-1">{benchResult.ramBandwidthMbps} <span class="text-xs font-normal text-neutral-400">MB/s</span></div>
          </div>
          <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 shadow-xs">
            <span class="text-xs font-semibold text-neutral-500 uppercase">Disk I/O Write</span>
            <div class="text-2xl font-black text-emerald-500 mt-1">{benchResult.diskWriteMbps1m} <span class="text-xs font-normal text-neutral-400">MB/s</span></div>
          </div>
        </div>

        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
          <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Hardware Profile Target</h4>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs font-mono">
            <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50">Details: <strong>{benchResult.details || 'Standard'}</strong></div>
            <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50">Core Count: <strong>{benchResult.cpuCores} Cores</strong></div>
            <div class="p-3 rounded-xl bg-neutral-50 dark:bg-neutral-800/50">Disk IOPS 4K: <strong>{benchResult.diskWriteIops4k} IOPS</strong></div>
          </div>
        </div>
      {:else}
        <div class="py-16 text-center text-neutral-400 text-xs bg-white dark:bg-neutral-900/40 rounded-2xl border border-neutral-200 dark:border-neutral-800">
          Belum ada benchmark dijalankan. Tekan "Jalankan Server Benchmark".
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 3: MESH LATENCY MATRIX (PRO) -->
  {#if activeTab === 'mesh'}
    <div class="space-y-6">
      <ProGate title="Fleet Mesh Latency Matrix Pro" description="Uji interkoneksi full-mesh antar server armada DevOps membutuhkan lisensi CATerm Pro.">
        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <h4 class="text-xs font-bold uppercase tracking-wider text-neutral-900 dark:text-white">Multi-Host Mesh Latency Grid</h4>
              <p class="text-xs text-neutral-500">Matriks latensi antar node armada (ThinkPad X1, DELL YPC, Hostinger VPS, Lenovo XPC).</p>
            </div>
            <Button
              variant="secondary"
              size="sm"
              onclick={startMeshMatrix}
              disabled={isRunningMesh}
              loading={isRunningMesh}
            >
              Compute Mesh Matrix
            </Button>
          </div>

          {#if meshResult}
            <div class="overflow-x-auto">
              <table class="w-full text-xs text-left border-collapse font-mono">
                <thead>
                  <tr class="border-b border-neutral-200 dark:border-neutral-800 text-neutral-500">
                    <th class="p-2">Target Host</th>
                    <th class="p-2">Status</th>
                    <th class="p-2">RTT (ms)</th>
                    <th class="p-2">Packet Loss</th>
                  </tr>
                </thead>
                <tbody>
                  {#each meshResult.nodes as node}
                    <tr class="border-b border-neutral-100 dark:border-neutral-800/60">
                      <td class="p-2 font-bold text-neutral-900 dark:text-white">{node.targetLabel || node.targetHost}</td>
                      <td class="p-2">
                        <Badge variant={node.reachable ? 'success' : 'danger'} size="xs">
                          {node.reachable ? 'Reachable' : 'Unreachable'}
                        </Badge>
                      </td>
                      <td class="p-2 font-bold {node.rttMs < 30 ? 'text-emerald-500' : node.rttMs < 100 ? 'text-amber-500' : 'text-rose-500'}">
                        {node.reachable ? `${node.rttMs.toFixed(1)} ms` : 'Timeout'}
                      </td>
                      <td class="p-2">{node.packetLossPct}%</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="py-8 text-center text-neutral-400 text-xs">Tekan "Compute Mesh Matrix" untuk menghitung matriks latensi antar host.</div>
          {/if}
        </div>
      </ProGate>
    </div>
  {/if}

  <!-- TAB 4: PMTUD & SUBNET TOOLS -->
  {#if activeTab === 'network_tools'}
    <div class="space-y-6">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Subnet CIDR Sweeper -->
        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
          <div>
            <span class="text-xs font-bold text-neutral-900 dark:text-white uppercase tracking-wider">Subnet CIDR Sweeper</span>
            <p class="text-xs text-neutral-500 mt-0.5">Pemindaian host aktif dan port umum dalam blok CIDR jaringan lokal.</p>
          </div>
          <div class="flex gap-2">
            <div class="flex-1">
              <Input
                bind:value={subnetCidr}
                placeholder="192.168.1.0/24"
                size="sm"
              />
            </div>
            <Button
              variant="primary"
              size="sm"
              onclick={startSubnetSweep}
              disabled={isRunningSubnet || !selectedHostId}
              loading={isRunningSubnet}
            >
              Sweep CIDR
            </Button>
          </div>
          {#if subnetResult}
            <div class="text-xs text-neutral-500">Ditemukan <strong>{subnetResult.activeHosts}</strong> host aktif:</div>
            <div class="max-h-48 overflow-y-auto space-y-1 font-mono text-xs">
              {#each subnetResult.hosts as h}
                <div class="p-2 rounded-lg bg-neutral-50 dark:bg-neutral-800 flex justify-between border border-neutral-100 dark:border-neutral-700/60">
                  <span class="font-bold">{h.ip}</span>
                  <span class="text-[11px] text-sky-500">Open: {h.openPorts.join(', ') || 'ICMP Only'}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- PMTUD Probe -->
        <div class="p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 space-y-3">
          <div>
            <span class="text-xs font-bold text-neutral-900 dark:text-white uppercase tracking-wider">Path MTU Discovery (PMTUD)</span>
            <p class="text-xs text-neutral-500 mt-0.5">Uji batas ukuran paket maksimal tanpa fragmentasi untuk mencegah paket drop.</p>
          </div>
          <div class="flex gap-2">
            <div class="flex-1">
              <Input
                bind:value={pmtudTarget}
                placeholder="1.1.1.1"
                size="sm"
              />
            </div>
            <Button
              variant="primary"
              size="sm"
              onclick={startPmtudProbe}
              disabled={isRunningPmtud || !selectedHostId}
              loading={isRunningPmtud}
            >
              Probe MTU
            </Button>
          </div>
          {#if pmtudResult}
            <div class="p-4 rounded-xl bg-sky-500/10 border border-sky-500/20 text-center space-y-1">
              <span class="text-xs text-neutral-400 block uppercase">Optimal Safe MTU</span>
              <div class="text-3xl font-black text-sky-600 dark:text-sky-300 font-mono">{pmtudResult.optimalMtu}</div>
              <p class="text-xs text-neutral-500">{pmtudResult.notes}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
