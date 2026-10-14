<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { changeMasterPassword, MIN_VAULT_PASSWORD_LEN } from '$lib/api/vault';
  import { exportEncryptedBackup, importEncryptedBackup } from '$lib/api/backup';
  import AiSettingsForm from '$lib/components/AiSettingsForm.svelte';
  import ProfileAvatar from '$lib/components/ProfileAvatar.svelte';
  import AvatarPicker from '$lib/components/AvatarPicker.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { getProfile, saveProfile, setProfilePlan } from '$lib/stores/profile.svelte';
  import { getAppearance, setReduceMotion } from '$lib/stores/appearance.svelte';
  import { getUpdater, checkForUpdates, installUpdate } from '$lib/stores/updater.svelte';
  import { APP_VERSION, releaseNotesUrl, pricingUrl, KOFI_URL } from '$lib/appInfo';
  import { PRO_PRICING, formatUsd, type BillingInterval } from '$lib/pro/pricing';
  import { openExternalUrl } from '$lib/utils/url';
  import { errorText } from '$lib/errors';
	import { getPerformancePrefs, setPerformancePrefs, type PerformancePrefs } from '$lib/api/performance';
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import ProLoginForm from '$lib/components/ProLoginForm.svelte';
  import { getPro, refreshProStatus, checkProServer, syncPro, setProStatus } from '$lib/stores/pro.svelte';
  import { proStartTrial, proAccount, proRevokeDevice, proLogout, proActivateLicense, proSyncVault, type ProDevice, type DeviceLimit, type SyncOutcome } from '$lib/api/pro';
  import { proErrorMessage } from '$lib/pro/errors';
  import ProTeamPanel from '$lib/components/ProTeamPanel.svelte';
  import { SHORTCUT_GROUPS } from '$lib/shortcuts';
  import {
    PageHeader,
    Button,
    Input,
    Badge,
    Card,
    Modal,
    Switch,
    Select,
    Tabs,
    Kbd,
    Checkbox,
    Alert
  } from '@cecepazhar/caui';
  import AmbientSettingsCard from '$lib/components/AmbientSettingsCard.svelte';

  // Shared surface classes so every tab reads the same in light and dark mode.
  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-sm dark:shadow-none text-neutral-900 dark:text-white';
  const SUBCARD = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';

  // ---- CATerm Pro account (Subscription tab) ----------------------------------------------
  const pro = getPro();
  let showProLogin = $state(false);
  let proDevices = $state<ProDevice[] | null>(null);
  let proDeviceLimit = $state<number | null>(null);
  let deviceLimit = $state<DeviceLimit | null>(null);
  let proBusy = $state(false);

  $effect(() => {
    if (activeTab === 'subscription') {
      void refreshProStatus();
      void checkProServer();
    }
  });

  // Literal class strings per tone so Tailwind's scanner sees them.
  const TONE: Record<string, string> = {
    sky: 'bg-sky-500/10 text-sky-700 dark:text-sky-300 border-sky-500/30',
    emerald: 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/30',
    amber: 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/30',
    rose: 'bg-rose-500/10 text-rose-700 dark:text-rose-300 border-rose-500/30',
    neutral: 'bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 border-neutral-300 dark:border-neutral-700'
  };
  const SMALL_BUTTON = 'px-3 py-1.5 text-xs font-medium rounded-md border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 disabled:opacity-50 transition-colors';

  function formatProDate(ts: number | null | undefined): string {
    return ts ? new Date(ts * 1000).toLocaleDateString(intlLocale(), { day: 'numeric', month: 'long', year: 'numeric' }) : '';
  }

  function formatProTime(ts: number | null | undefined): string {
    return ts ? new Date(ts * 1000).toLocaleString(intlLocale(), { dateStyle: 'medium', timeStyle: 'short' }) : '';
  }

  const proState = $derived.by(() => {
    const status = pro.status;
    if (!status?.signedIn) return null;
    const ent = status.entitlement;
    const lic = status.license;
    if (!status.keyConfigured) return { tone: 'amber', text: t('pro.account.stateKeyMissing') };
    if (ent.state === 'valid') {
      if (ent.tier === 'team') return { tone: 'sky', text: t('pro.account.stateTeamMember') };
      if (lic?.status === 'trialing') {
        const end = lic.trialEndsAt ?? ent.expiresAt;
        const count = Math.max(0, Math.ceil((end - Date.now() / 1000) / 86400));
        return { tone: 'sky', text: t('pro.account.stateTrial', { count }) };
      }
      if (lic?.status === 'cancelled') return { tone: 'amber', text: t('pro.account.stateCancelled', { date: formatProDate(lic.currentPeriodEnd) }) };
      if (lic?.status === 'past_due') return { tone: 'amber', text: t('pro.account.statePastDue') };
      return lic?.currentPeriodEnd
        ? { tone: 'emerald', text: t('pro.account.stateActive', { date: formatProDate(lic.currentPeriodEnd) }) }
        : { tone: 'emerald', text: t('pro.account.stateActiveNoDate') };
    }
    if (ent.state === 'expired' || (lic && !lic.entitled)) return { tone: 'rose', text: t('pro.account.stateExpired') };
    if (ent.state === 'revalidationRequired' || ent.state === 'invalid' || lic) return { tone: 'amber', text: t('pro.account.stateNeedsSync') };
    return { tone: 'neutral', text: t('pro.account.stateNoPlan') };
  });

  function handleOutcome(outcome: SyncOutcome | null, successText: string) {
    if (!outcome) return;
    if (outcome.deviceLimit) deviceLimit = outcome.deviceLimit;
    else showToast(successText, 'success');
  }

  async function runProSync() {
    proBusy = true;
    try {
      const outcome = await syncPro();
      let vaultSynced = false;
      try {
        vaultSynced = await proSyncVault();
      } catch (e) {
        console.warn('Vault sync skipped or failed:', e);
      }
      if (outcome?.deviceLimit) {
        deviceLimit = outcome.deviceLimit;
      } else if (vaultSynced) {
        showToast('Sinkronisasi Cloud Berhasil! Host & pengaturan tersinkronisasi aman.', 'success');
      } else {
        handleOutcome(outcome, t('pro.account.synced'));
      }
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    } finally {
      proBusy = false;
    }
  }

  async function handleProSignedIn() {
    showProLogin = false;
    await refreshProStatus();
    await runProSync();
  }

  async function startProTrial() {
    if (!pro.status?.signedIn) {
      showProLogin = true;
      return;
    }
    proBusy = true;
    try {
      const outcome = await proStartTrial();
      setProStatus(outcome.status);
      handleOutcome(outcome, t('pro.account.trialStarted', { days: PRO_PRICING.trialDays }));
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    } finally {
      proBusy = false;
    }
  }

  async function loadProDevices() {
    try {
      const details = await proAccount();
      proDevices = details.devices;
      proDeviceLimit = details.access?.deviceLimit ?? null;
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    }
  }

  async function removeProDevice(id: string) {
    try {
      await proRevokeDevice(id);
      proDevices = proDevices?.filter((d) => d.id !== id) ?? null;
      if (deviceLimit) {
        deviceLimit = null;
        await runProSync(); // a slot is free now: activate this device
      }
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    }
  }

  async function signOutPro() {
    const ok = await confirmModal(
      t('pro.account.signOutConfirm'),
      t('pro.account.signOutTitle'),
      true,
      t('pro.account.signOut'),
      t('common.cancel')
    );
    if (!ok) return;
    try {
      await proLogout();
      setProfilePlan('free');
      proDevices = null;
      await refreshProStatus();
      showToast('Signed out of Pro account. Switched to Free Mode.', 'info');
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    }
  }

  let licenseInput = $state('');
  let licenseBusy = $state(false);

  async function handleActivateLicense(keyToUse?: string) {
    const key = (keyToUse || licenseInput).trim();
    if (!key) {
      showToast('Please enter a license key.', 'error');
      return;
    }
    licenseBusy = true;
    try {
      await proActivateLicense(key);
      setProfilePlan('pro');
      await refreshProStatus();
      licenseInput = '';
      showToast('License activated! CATerm Pro features unlocked.', 'success');
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    } finally {
      licenseBusy = false;
    }
  }
  const LABEL_BASE = 'block text-xs font-medium text-neutral-600 dark:text-neutral-400 uppercase';
  const LABEL = `${LABEL_BASE} mb-1`;
  const INPUT = 'w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500';
  const EYE_BUTTON = 'absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors p-1';

  // Subscription tab
  let billing = $state<BillingInterval>('monthly');
  const proFeatures = $derived([
    t('settings.subscription.featureSync', { devices: PRO_PRICING.syncDevices }),
    t('settings.subscription.featureAi', { requests: PRO_PRICING.aiRequestsPerMonth }),
    t('settings.subscription.featureTeam', { extra: PRO_PRICING.maxMembers - 1, members: PRO_PRICING.maxMembers }),
    t('settings.subscription.featureLogs'),
    t('settings.subscription.featureCommunity')
  ]);

  const TABS = ['profile', 'updates', 'ai', 'subscription', 'security', 'backup', 'performance', 'shortcuts'] as const;
  type SettingsTab = (typeof TABS)[number];
  // `?tab=` lets the profile menu deep-link straight to a tab.
  const requestedTab = page.url.searchParams.get('tab') ?? '';
  let activeTab = $state<string>((TABS as readonly string[]).includes(requestedTab) ? (requestedTab as SettingsTab) : 'profile');

  // Subscription only makes sense once the user has a CATerm account (Cloud): a purely local
  // vault has nothing to subscribe to. Hidden until sign-in, shown for both free-cloud and Pro.
    const visibleTabs = $derived(TABS.filter((tab) => tab !== 'subscription' || pro.status?.signedIn));
  const tabItems = $derived(
    visibleTabs.map((tab) => ({
      id: tab,
      label: t(`settings.tabs.${tab}`)
    }))
  );
  let scrollbackStr = $state('5000');
  $effect(() => {
    scrollbackStr = String(scrollbackLines);
  });
  $effect(() => {
    const num = parseInt(scrollbackStr, 10);
    if (!isNaN(num) && num !== scrollbackLines) {
      void saveAllPerformancePrefs({ scrollbackLines: num });
    }
  });
  $effect(() => {
    if (activeTab === 'subscription' && pro.status && !pro.status.signedIn) activeTab = 'profile';
  });

  const updater = getUpdater();
  const downloadPercent = $derived(
    updater.progress.total ? Math.round((updater.progress.downloaded / updater.progress.total) * 100) : null
  );

  // Profile (Free plan: display name + preset avatar)
  const profile = getProfile();
  const appearance = getAppearance();
  let profileName = $state(profile.name);
  let profileAvatar = $state(profile.avatar);
  const profileDirty = $derived(profileName.trim() !== profile.name || profileAvatar !== profile.avatar);

  function handleSaveProfile(e: Event) {
    e.preventDefault();
    saveProfile({ name: profileName, avatar: profileAvatar });
    profileName = profile.name;
    showToast(t('settings.profile.saved'), 'success');
  }

  // Master password change (re-keys the encrypted database)
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let showPasswords = $state(false);
  let isChangingPassword = $state(false);

  // Audit Logs Setting
  let maxAuditRecords = $state(1000);

  onMount(() => {
    try {
      const stored = localStorage.getItem('caterm_max_audit_records');
      if (stored) maxAuditRecords = parseInt(stored, 10) || 1000;
    } catch {}
		getPerformancePrefs()
			.then((prefs) => {
				gpuAcceleration = prefs.gpuAcceleration ?? false;
				backgroundMemorySaving = prefs.backgroundMemorySaving ?? true;
				scrollbackLines = prefs.scrollbackLines ?? 5000;
				inactiveSessionSleep = prefs.inactiveSessionSleep ?? true;
				lowPowerMode = prefs.lowPowerMode ?? false;
			})
			.catch(() => {}); // browser preview: no backend, keep default
	});

	// Performance preferences state
	let gpuAcceleration = $state(false);
	let backgroundMemorySaving = $state(true);
	let scrollbackLines = $state(5000);
	let inactiveSessionSleep = $state(true);
	let lowPowerMode = $state(false);
	let isSavingPerformance = $state(false);

	async function saveAllPerformancePrefs(partial: Partial<PerformancePrefs>, isGpuChange = false) {
		isSavingPerformance = true;
		const updated: PerformancePrefs = {
			gpuAcceleration: partial.gpuAcceleration ?? gpuAcceleration,
			backgroundMemorySaving: partial.backgroundMemorySaving ?? backgroundMemorySaving,
			scrollbackLines: partial.scrollbackLines ?? scrollbackLines,
			inactiveSessionSleep: partial.inactiveSessionSleep ?? inactiveSessionSleep,
			lowPowerMode: partial.lowPowerMode ?? lowPowerMode
		};
		try {
			await setPerformancePrefs(updated);
			gpuAcceleration = updated.gpuAcceleration;
			backgroundMemorySaving = updated.backgroundMemorySaving;
			scrollbackLines = updated.scrollbackLines;
			inactiveSessionSleep = updated.inactiveSessionSleep;
			lowPowerMode = updated.lowPowerMode;
			if (!isGpuChange) {
				showToast(t('settings.performance.saved'), 'success');
			}
		} catch (err) {
			showToast(t('settings.performance.saveFailed', { error: errorText(err) }), 'error');
			return;
		} finally {
			isSavingPerformance = false;
		}

		if (isGpuChange) {
			const restart = await confirmModal(
				t('settings.performance.restartBody'),
				t('settings.performance.restartTitle'),
				false,
				t('settings.performance.restartNow'),
				t('settings.performance.later')
			);
			if (restart) await relaunch();
		}
	}

  function handleSaveAuditSettings(e: Event) {
    e.preventDefault();
    try {
      localStorage.setItem('caterm_max_audit_records', maxAuditRecords.toString());
      showToast(t('settings.audit.saved', { count: maxAuditRecords }), 'success');
    } catch {
      showToast(t('settings.audit.saveFailed'), 'error');
    }
  }

  let backupPassphrase = $state('');
  let showBackupPassphrase = $state(false);
  let backupMsg = $state('');
  let backupMsgKind = $state<'success' | 'error'>('success');
  let restorePassphrase = $state('');
  let showRestorePassphrase = $state(false);
  let restoreMsg = $state('');
  let restoreMsgKind = $state<'success' | 'error'>('success');

  async function handleExport(e: Event) {
    e.preventDefault();
    if (backupPassphrase.length < 8) {
      showToast(t('settings.backup.errPassphrase'), 'error');
      return;
    }

    try {
      showToast(t('settings.backup.creating'), 'info');
      const b64 = await exportEncryptedBackup(backupPassphrase);
      
      const blob = new Blob([b64], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `caterm-backup-${new Date().toISOString().split('T')[0]}.catb`;
      a.click();
      URL.revokeObjectURL(url);

      showToast(t('settings.backup.savedToast'), 'success');
      backupPassphrase = '';
    } catch (err: any) {
      showToast(err?.message || t('settings.backup.exportFailed'), 'error');
    }
  }

  async function handleImport(e: Event) {
    e.preventDefault();
    if (restorePassphrase.length < 8) {
      showToast(t('settings.backup.errPassphrase'), 'error');
      return;
    }

    try {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.catb,.enc,.txt';
      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) return;
        
        showToast(t('settings.backup.importing'), 'info');
        
        const reader = new FileReader();
        reader.onload = async (event) => {
          try {
            const b64 = event.target?.result as string;
            const importedCount = await importEncryptedBackup(b64, restorePassphrase);
            showToast(t('settings.backup.restored', { count: importedCount }), 'success');
            restorePassphrase = '';
          } catch (err: any) {
            showToast(err?.message || t('settings.backup.restoreFailed'), 'error');
          }
        };
        reader.readAsText(file);
      };
      input.click();
    } catch (err: any) {
      showToast(err?.message || t('settings.backup.restoreFailed'), 'error');
    }
  }

  async function handleChangeMasterPassword(e: Event) {
    e.preventDefault();
    if (newPassword.length < MIN_VAULT_PASSWORD_LEN) {
      showToast(t('settings.security.errMin', { min: MIN_VAULT_PASSWORD_LEN }), 'error');
      return;
    }
    if (newPassword !== confirmPassword) {
      showToast(t('settings.security.errMismatch'), 'error');
      return;
    }
    isChangingPassword = true;
    try {
      await changeMasterPassword(currentPassword, newPassword);
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
      const restartNow = await confirmModal(
        t('settings.security.changedConfirm'),
        t('settings.security.changedTitle'),
        false,
        t('settings.security.restartNow'),
        t('settings.security.later')
      );
      if (restartNow) {
        await relaunch();
        return;
      }
      showToast(t('settings.security.changedToast'), 'success');
    } catch (err) {
      showToast(errorText(err), 'error');
    } finally {
      isChangingPassword = false;
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z', 'M15 12a3 3 0 11-6 0 3 3 0 016 0z']}
    accent="cyan"
    title={t('settings.title')}
    subtitle={t('settings.subtitle')}
  />

  <!-- Settings Tabs -->
  <Tabs items={tabItems} bind:value={activeTab} variant="underline" size="md" class="overflow-x-auto" />

  {#if activeTab === 'profile'}
    <Card class="space-y-6" variant="primary" size="lg">
      <div class="flex items-center gap-4">
        <ProfileAvatar avatar={profileAvatar} name={profileName} size={56} pro={pro.isPro || profile.plan === 'pro'} />
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold text-neutral-900 dark:text-white truncate">{profileName.trim() || profile.name}</h2>
            {#if pro.isPro || profile.plan === 'pro'}
              <Badge variant="warning" size="xs">PRO</Badge>
            {:else}
              <Badge variant="neutral" size="xs">{t('profileMenu.planFree')}</Badge>
            {/if}
          </div>
          <p class="{MUTED} text-sm">{t('settings.profile.localNote')}</p>
        </div>
      </div>

      <form onsubmit={handleSaveProfile} class="space-y-5 max-w-md">
        <Input
          id="profile-name"
          label={t('settings.profile.displayName')}
          bind:value={profileName}
          placeholder="CATerm User"
          size="md"
        />
        <div>
          <span class="{LABEL_BASE} mb-2">{t('settings.profile.picture')}</span>
          <AvatarPicker bind:value={profileAvatar} size={40} />
        </div>
        <Button type="submit" disabled={!profileDirty} variant="primary" size="md">
          {t('settings.profile.save')}
        </Button>
      </form>

      <div class="pt-5 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-between gap-4">
        <div>
          <p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.profile.masterPassword')}</p>
          <p class="text-xs {MUTED}">{t('settings.profile.masterPasswordBody')}</p>
        </div>
        <Button
          onclick={() => (activeTab = 'security')}
          variant="neutral"
          size="sm"
        >
          {t('settings.profile.changeMasterPassword')}
        </Button>
      </div>

    </Card>

    <!-- PRO Ambient Underglow Lighting Card -->
    <AmbientSettingsCard defaultAccent="#ef4444" appName="CATerm" />
  {:else if activeTab === 'updates'}
    <Card class="space-y-4" variant="primary" size="lg">
      <div class="flex flex-wrap justify-between items-center gap-4">
        <div>
          <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.updates.title')}</h2>
          <p class="{MUTED} text-sm">{t('settings.updates.current')} <span class="font-mono text-sky-600 dark:text-sky-400">v{APP_VERSION}</span></p>
        </div>
        {#if updater.status === 'available'}
          <Button onclick={installUpdate} variant="success" size="md">
            {t('settings.updates.install')}
          </Button>
        {:else}
          <Button
            onclick={() => checkForUpdates()}
            disabled={updater.status === 'checking' || updater.status === 'downloading'}
            loading={updater.status === 'checking'}
            variant="primary"
            size="md"
          >
            {updater.status === 'checking' ? t('settings.updates.checking') : t('settings.updates.check')}
          </Button>
        {/if}
      </div>

      {#if updater.status === 'up-to-date'}
        <Alert variant="success" title="" description={t('settings.updates.latest', { version: APP_VERSION })} />
      {:else if updater.status === 'available'}
        <Alert variant="info" title="" description={t('settings.updates.available', { version: updater.version })}>
          <a href={releaseNotesUrl(updater.version)} class="underline underline-offset-2 hover:text-sky-900 dark:hover:text-white">{t('settings.updates.readChangelog')}</a>
        </Alert>
      {:else if updater.status === 'downloading'}
        <Alert variant="info" title="" description={t('settings.updates.downloading', { version: updater.version, progress: downloadPercent === null ? '...' : ` (${downloadPercent}%)` })} />
      {:else if updater.status === 'error'}
        <Alert variant="danger" title="" description={updater.error} />
      {/if}
    </Card>
  {:else if activeTab === 'ai'}
    <div class="space-y-6">
      <!-- Master AI Architecture Navigation Cards -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <a
          href="/settings/ai-routing"
          class="{SUBCARD} hover:border-sky-500/50 hover:bg-sky-500/5 transition-all block group"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="p-2 rounded bg-sky-500/10 text-sky-600 dark:text-sky-400">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </span>
            <h3 class="text-sm font-semibold text-neutral-900 dark:text-white group-hover:text-sky-600 dark:group-hover:text-sky-400">
              Routing Matrix
            </h3>
          </div>
          <p class="{MUTED} text-xs">
            Multi-provider failover, BYO keys (Anthropic, OpenAI, Ollama), and task-specific model routing.
          </p>
        </a>

        <a
          href="/settings/ai-personas"
          class="{SUBCARD} hover:border-indigo-500/50 hover:bg-indigo-500/5 transition-all block group"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="p-2 rounded bg-indigo-500/10 text-indigo-600 dark:text-indigo-400">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </span>
            <h3 class="text-sm font-semibold text-neutral-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400">
              Personas & Rules
            </h3>
          </div>
          <p class="{MUTED} text-xs">
            System prompts, safety guardrails, environment constraints, and terminal context formatting.
          </p>
        </a>

        <a
          href="/settings/ai-habits"
          class="{SUBCARD} hover:border-teal-500/50 hover:bg-teal-500/5 transition-all block group"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="p-2 rounded bg-teal-500/10 text-teal-600 dark:text-teal-400">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
              </svg>
            </span>
            <h3 class="text-sm font-semibold text-neutral-900 dark:text-white group-hover:text-teal-600 dark:group-hover:text-teal-400">
              Habit Memory (FTS5)
            </h3>
          </div>
          <p class="{MUTED} text-xs">
            Encrypted zero-knowledge memory of CLI habits, container tools, and command preferences.
          </p>
        </a>
      </div>

      <Card class="space-y-4" variant="primary" size="lg">
        <div>
          <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.ai.title')}</h2>
          <p class="{MUTED} text-sm">{t('settings.ai.subtitle')}</p>
        </div>
        <AiSettingsForm />
      </Card>
    </div>
  {:else if activeTab === 'subscription'}
    <Card class="space-y-6" variant="primary" size="lg">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.subscription.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.subscription.subtitle')}</p>
      </div>

      <!-- CATerm Pro account on this device -->
      <div class="{SUBCARD} space-y-3">
        {#if !pro.status?.signedIn}
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div>
              <p class="text-sm font-semibold text-neutral-900 dark:text-white">{t('pro.account.title')}</p>
              <p class="text-xs {MUTED} mt-0.5">{t('pro.account.signedOutBody')}</p>
            </div>
            <Button type="button" onclick={() => (showProLogin = true)} variant="primary" size="md" class="shrink-0">
              {t('pro.account.signInOrRegister')}
            </Button>
          </div>
        {:else}
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div class="min-w-0">
              <p class="text-sm font-semibold text-neutral-900 dark:text-white truncate">{pro.status.account?.name || pro.status.account?.email}</p>
              <p class="text-xs {MUTED} truncate">{pro.status.account?.email}</p>
              {#if proState}
                <Badge variant={proState.tone === 'sky' ? 'info' : proState.tone === 'emerald' ? 'success' : proState.tone === 'amber' ? 'warning' : proState.tone === 'rose' ? 'danger' : 'neutral'} size="sm" class="mt-1.5">
                  {proState.text}
                </Badge>
              {/if}
            </div>
            <div class="flex flex-wrap gap-2 shrink-0">
              <Button type="button" onclick={runProSync} disabled={proBusy || pro.syncing} loading={pro.syncing} variant="neutral" size="sm">
                {pro.syncing ? t('pro.account.syncing') : t('pro.account.sync')}
              </Button>
              <Button type="button" onclick={loadProDevices} variant="neutral" size="sm">
                {t('pro.account.devices')}
              </Button>
              <Button type="button" onclick={signOutPro} variant="danger" size="sm">
                {t('pro.account.signOut')}
              </Button>
            </div>
          </div>
          {#if pro.status.lastSyncAt}
            <p class="text-[11px] text-neutral-500">{t('pro.account.lastSync', { time: formatProTime(pro.status.lastSyncAt) })}</p>
          {/if}
          {#if proDevices}
            {#if proDeviceLimit}
              <p class="text-xs {MUTED}">{t('pro.account.deviceUsage', { used: proDevices.length, limit: proDeviceLimit })}</p>
            {/if}
            {#if proDevices.length === 0}
              <p class="text-xs {MUTED}">{t('pro.devices.empty')}</p>
            {:else}
              <ul class="divide-y divide-neutral-200 dark:divide-neutral-800 border border-neutral-200 dark:border-neutral-800 rounded-md bg-white dark:bg-neutral-900">
                {#each proDevices as device (device.id)}
                  <li class="flex items-center justify-between gap-3 px-3 py-2 text-xs">
                    <div class="min-w-0">
                      <p class="font-medium text-neutral-900 dark:text-white truncate">
                        {device.name}
                        {#if device.current}
                          <Badge variant="info" size="xs" class="ml-1.5">{t('pro.devices.thisDevice')}</Badge>
                        {/if}
                      </p>
                      <p class="text-neutral-500 truncate">{device.os} · v{device.clientVersion} · {t('pro.devices.lastSeen', { time: formatProTime(device.lastSeenAt) })}</p>
                    </div>
                    {#if !device.current}
                      <Button type="button" onclick={() => removeProDevice(device.id)} variant="ghost" size="sm" class="text-rose-600 dark:text-rose-400 hover:underline shrink-0 p-0">
                        {t('pro.devices.remove')}
                      </Button>
                    {/if}
                  </li>
                {/each}
              </ul>
            {/if}
          {/if}
        {/if}
      </div>

      <!-- License Key Activation Card -->
      {#if !pro.isPro}
        <div class="{SUBCARD} space-y-4">
          <div>
            <p class="text-sm font-semibold text-neutral-900 dark:text-white">Activate License Key</p>
            <p class="text-xs {MUTED} mt-0.5">Activate CATerm Pro with your license key from Mayar, Ko-fi, or Founder Lifetime.</p>
          </div>

          <div class="flex flex-col sm:flex-row gap-2">
            <Input
              type="text"
              bind:value={licenseInput}
              placeholder="e.g. CATERM-LIFE-FOUNDER-CECEP"
              class="font-mono uppercase flex-1"
            />
            <Button
              type="button"
              onclick={() => handleActivateLicense()}
              disabled={licenseBusy || !licenseInput.trim()}
              loading={licenseBusy}
              variant="primary"
              size="md"
              class="shrink-0"
            >
              Activate License
            </Button>
          </div>

          <div class="pt-3 border-t border-neutral-200 dark:border-neutral-800 flex flex-wrap items-center justify-between gap-2">
            <span class="text-xs text-neutral-500">Creator / Owner Quick Unlock:</span>
            <Button
              type="button"
              onclick={() => handleActivateLicense('CATERM-LIFE-FOUNDER-CECEP')}
              disabled={licenseBusy}
              variant="warning"
              size="sm"
            >
              👑 Activate Founder Lifetime (Cecep Azhar)
            </Button>
          </div>
        </div>
      {/if}

      {#if pro.status?.signedIn && !pro.status.pending}
        <ProTeamPanel onOutcome={handleOutcome} />
      {/if}

      {#if pro.isPro}
        <!-- Active Pro / Lifetime Billing Overview (No need to show upgrade pricing cards) -->
        <div class="rounded-xl border border-sky-500/40 bg-linear-to-br from-sky-500/10 via-neutral-50 dark:via-neutral-900/60 to-transparent p-6 space-y-5">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-4 border-b border-neutral-200 dark:border-neutral-800">
            <div>
              <div class="flex items-center gap-2">
                <span class="text-xs font-bold uppercase tracking-wider text-sky-600 dark:text-sky-400">Current Plan</span>
                <Badge variant="success" size="xs">
                  ACTIVE
                </Badge>
              </div>
              <h3 class="text-xl font-bold text-neutral-900 dark:text-white mt-1">
                {pro.status?.license?.tier?.toUpperCase() === 'LIFETIME' ? 'CATerm Pro — Founder Lifetime' : 'CATerm Pro Edition'}
              </h3>
              <p class="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
                Zero-knowledge encrypted cloud synchronization & AI Copilot active.
              </p>
            </div>
            <div class="text-right sm:text-right shrink-0">
              <span class="text-2xl font-black text-neutral-900 dark:text-white">
                {pro.status?.license?.tier?.toUpperCase() === 'LIFETIME' ? 'Lifetime Access' : '$6.00 / mo'}
              </span>
              <p class="text-[11px] text-emerald-600 dark:text-emerald-400 font-medium">
                {pro.status?.license?.tier?.toUpperCase() === 'LIFETIME' ? 'Never Expires · VIP Founder Tier' : 'Next billing cycle synced'}
              </p>
            </div>
          </div>

          <!-- Billing Info Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
            <div class="bg-white/60 dark:bg-neutral-950/60 rounded-lg p-3 border border-neutral-200 dark:border-neutral-800/80">
              <span class="text-neutral-500 dark:text-neutral-400 block text-[11px]">Customer Account</span>
              <span class="font-semibold text-neutral-900 dark:text-white block mt-0.5 truncate">{pro.status?.account?.email || 'Registered User'}</span>
            </div>
            <div class="bg-white/60 dark:bg-neutral-950/60 rounded-lg p-3 border border-neutral-200 dark:border-neutral-800/80">
              <span class="text-neutral-500 dark:text-neutral-400 block text-[11px]">Device Seats</span>
              <span class="font-semibold text-neutral-900 dark:text-white block mt-0.5">
                {proDevices ? proDevices.length : 1} / {proDeviceLimit || 5} Devices Used
              </span>
            </div>
            <div class="bg-white/60 dark:bg-neutral-950/60 rounded-lg p-3 border border-neutral-200 dark:border-neutral-800/80">
              <span class="text-neutral-500 dark:text-neutral-400 block text-[11px]">License Signature</span>
              <span class="font-mono text-neutral-900 dark:text-white block mt-0.5 truncate">
                Ed25519 v1 (Verified Offline)
              </span>
            </div>
          </div>

          <!-- Active Features Checklist -->
          <div class="pt-2">
            <p class="text-xs font-semibold text-neutral-900 dark:text-white mb-2">Entitlements & Included Features:</p>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs text-neutral-700 dark:text-neutral-300">
              <div class="flex items-center gap-2">
                <svg class="w-4 h-4 text-emerald-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                <span>End-to-end encrypted cloud sync</span>
              </div>
              <div class="flex items-center gap-2">
                <svg class="w-4 h-4 text-emerald-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                <span>AI Ops Copilot & Prompt Studio</span>
              </div>
              <div class="flex items-center gap-2">
                <svg class="w-4 h-4 text-emerald-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                <span>Team Session & Tag Sharing</span>
              </div>
              <div class="flex items-center gap-2">
                <svg class="w-4 h-4 text-emerald-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                <span>Multi-Platform License (Desktop & Android)</span>
              </div>
            </div>
          </div>
        </div>
      {:else}
        <!-- Free / Community Tier: Offer Pro Upgrade options -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 items-start">
          <!-- Community: what everyone has today -->
          <div class="{SUBCARD} space-y-2">
            <div class="flex items-center gap-2">
              <span class="font-semibold text-neutral-900 dark:text-white">{t('settings.subscription.freePlan')}</span>
              <Badge variant="neutral" size="xs">{t('settings.subscription.current')}</Badge>
            </div>
            <div class="text-2xl font-bold text-neutral-900 dark:text-white">$0</div>
            <p class="{MUTED} text-xs leading-relaxed">{t('settings.subscription.freeBody')}</p>
          </div>

          <!-- Pro Upgrade -->
          <div class="rounded-lg border-2 border-sky-500/50 bg-sky-50/60 dark:bg-sky-950/20 p-5 space-y-4">
            <div class="flex items-center justify-between gap-2 flex-wrap">
              <span class="font-semibold text-neutral-900 dark:text-white">{t('settings.subscription.proPlan')}</span>
              <Badge variant="danger" size="xs">
                {t('settings.subscription.introBadge', { percent: PRO_PRICING.introDiscountPercent, months: PRO_PRICING.introMonths })}
              </Badge>
            </div>

            <div class="inline-flex items-center gap-2 rounded-lg border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900 p-1 text-xs">
              <Button
                type="button"
                onclick={() => (billing = 'monthly')}
                variant={billing === 'monthly' ? 'primary' : 'neutral'}
                size="sm"
              >
                {t('settings.subscription.monthly')}
              </Button>
              <Button
                type="button"
                onclick={() => (billing = 'yearly')}
                variant={billing === 'yearly' ? 'primary' : 'neutral'}
                size="sm"
              >
                {t('settings.subscription.yearly')}
                <span class="ml-1 text-[10px] font-semibold {billing === 'yearly' ? 'text-sky-100' : 'text-emerald-600 dark:text-emerald-400'}">{t('settings.subscription.saveYearly', { percent: PRO_PRICING.yearlyDiscountPercent })}</span>
              </Button>
            </div>

            <div>
              <div class="flex items-baseline gap-2">
                <span class="text-sm text-neutral-400 line-through">{formatUsd(PRO_PRICING[billing].list)}</span>
                <span class="text-3xl font-bold text-neutral-900 dark:text-white">{formatUsd(PRO_PRICING[billing].intro)}</span>
                <span class="text-sm text-neutral-500">{billing === 'monthly' ? t('settings.subscription.perMonth') : t('settings.subscription.perYear')}</span>
              </div>
              <p class="text-xs {MUTED} mt-1">
                {#if billing === 'monthly'}
                  {t('settings.subscription.monthlyThen', { months: PRO_PRICING.introMonths, price: formatUsd(PRO_PRICING.monthly.list) })}
                {:else}
                  {t('settings.subscription.yearlyThen', { perMonth: formatUsd(PRO_PRICING.yearly.intro / 12), price: formatUsd(PRO_PRICING.yearly.list) })}
                {/if}
              </p>
            </div>

            <ul class="space-y-1.5 text-xs text-neutral-700 dark:text-neutral-300">
              {#each proFeatures as feature (feature)}
                <li class="flex items-start gap-2">
                  <svg class="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                  <span>{feature}</span>
                </li>
              {/each}
            </ul>

            <div class="flex flex-col sm:flex-row gap-2">
              {#if pro.serverAvailable === false}
                <Button disabled class="flex-1 w-full" variant="outline" size="lg" title={t('settings.subscription.comingSoon')}>
                  {t('settings.subscription.startTrial', { days: PRO_PRICING.trialDays })}
                  <span class="block text-[10px] font-normal">{t('settings.subscription.comingSoon')}</span>
                </Button>
              {:else if !pro.status?.license}
                <!-- Signed out: opens Pro Login first. A license (trial or paid) hides the trial. -->
                <Button type="button" onclick={startProTrial} disabled={proBusy} loading={proBusy} class="flex-1 w-full" variant="info" size="lg">
                  {t('settings.subscription.startTrial', { days: PRO_PRICING.trialDays })}
                </Button>
              {/if}
              <Button
                type="button"
                onclick={() => openExternalUrl(pricingUrl(pro.status?.account?.id))}
                class="flex-1 w-full"
                variant="primary"
                size="lg"
              >
                {t('settings.subscription.subscribe')}
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
              </Button>
            </div>
            <p class="text-[11px] text-neutral-500">{t('settings.subscription.trialNote')}</p>
            {#if pro.status?.signedIn}
              <p class="text-[11px] text-neutral-500">{t('pro.account.checkoutHint')}</p>
            {/if}
          </div>
        </div>
      {/if}

      <div class="pt-4 border-t border-neutral-200 dark:border-neutral-800/80 flex items-center justify-between gap-4">
        <div>
          <p class="text-xs text-neutral-800 dark:text-neutral-300 font-medium">{t('settings.subscription.supportTitle')}</p>
          <p class="text-[11px] text-neutral-500">{t('settings.subscription.supportBody')}</p>
        </div>
        <a
          href={KOFI_URL}
          target="_blank"
          rel="noopener"
          class="inline-flex items-center gap-2 px-4 py-2 bg-[#ff5e5b] hover:bg-[#e54d4a] text-white text-xs font-semibold rounded-lg transition-colors shrink-0"
        >
          <span>{t('contribution.donateVia')}</span>
        </a>
      </div>
    </Card>
  {:else if activeTab === 'security'}
    <Card class="space-y-4" variant="primary" size="lg">
      <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.security.title')}</h2>
      <p class="{MUTED} text-sm">{t('settings.security.subtitle')}</p>

      <form onsubmit={handleChangeMasterPassword} class="max-w-md space-y-4 pt-2">
        <p class="text-xs {MUTED}">{t('settings.security.rekeyNote')}</p>
        <Input
          id="current-pass"
          type={showPasswords ? 'text' : 'password'}
          label={t('settings.security.current')}
          required
          bind:value={currentPassword}
        />
        <Input
          id="new-pass"
          type={showPasswords ? 'text' : 'password'}
          label={t('settings.security.new', { min: MIN_VAULT_PASSWORD_LEN })}
          required
          bind:value={newPassword}
        />
        <Input
          id="confirm-pass"
          type={showPasswords ? 'text' : 'password'}
          label={t('settings.security.confirm')}
          required
          bind:value={confirmPassword}
        />
        <Checkbox bind:checked={showPasswords} label={t('settings.security.showPasswords')} />
        <div>
          <Button
            type="submit"
            disabled={isChangingPassword}
            loading={isChangingPassword}
            variant="success"
            size="md"
          >
            {isChangingPassword ? t('settings.security.reencrypting') : t('settings.security.change')}
          </Button>
        </div>
      </form>

      <!-- Audit Trail Settings -->
      <div class="border-t border-neutral-200 dark:border-neutral-800 pt-6 space-y-3">
        <div>
          <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.audit.title')}</h3>
          <p class="{MUTED} text-xs mt-1">{t('settings.audit.body')}</p>
        </div>
        <form onsubmit={handleSaveAuditSettings} class="flex items-end gap-4 max-w-sm">
          <div class="flex-1">
            <Input
              id="max-audit"
              type="number"
              label={t('settings.audit.maxRecords')}
              value={String(maxAuditRecords)}
              oninput={(e) => {
                const target = e.target as HTMLInputElement;
                maxAuditRecords = parseInt(target.value, 10) || 1000;
              }}
            />
          </div>
          <Button
            type="submit"
            variant="primary"
            size="md"
            class="whitespace-nowrap"
          >
            {t('common.save')}
          </Button>
        </form>
      </div>
    </Card>
  {:else if activeTab === 'backup'}
    {#snippet backupPassEye()}
      <Button
        type="button"
        variant="ghost"
        size="icon"
        onclick={() => (showBackupPassphrase = !showBackupPassphrase)}
        class="h-6 w-6 text-neutral-400 hover:text-neutral-900 dark:hover:text-white"
        aria-label={showBackupPassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
      >
        {#if showBackupPassphrase}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
          </svg>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
        {/if}
      </Button>
    {/snippet}

    {#snippet restorePassEye()}
      <Button
        type="button"
        variant="ghost"
        size="icon"
        onclick={() => (showRestorePassphrase = !showRestorePassphrase)}
        class="h-6 w-6 text-neutral-400 hover:text-neutral-900 dark:hover:text-white"
        aria-label={showRestorePassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
      >
        {#if showRestorePassphrase}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
          </svg>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
        {/if}
      </Button>
    {/snippet}

    <Card class="space-y-8" variant="primary" size="lg">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.backup.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.backup.subtitle')}</p>
      </div>

      <div class="space-y-4">
        <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.backup.exportTitle')}</h3>
        <p class="text-xs {MUTED}">{t('settings.backup.exportBody')}</p>
        <form onsubmit={handleExport} class="max-w-md space-y-4 pt-2">
          <Input
            id="backup-pass"
            type={showBackupPassphrase ? 'text' : 'password'}
            label={t('settings.backup.exportPassphrase')}
            required
            bind:value={backupPassphrase}
            trailingAction={backupPassEye}
          />
          {#if backupMsg}
            <p class="text-xs {backupMsgKind === 'success' ? 'text-emerald-600 dark:text-emerald-500' : 'text-red-600 dark:text-red-500'}">{backupMsg}</p>
          {/if}
          <Button type="submit" variant="primary" size="md">
            {t('settings.backup.exportButton')}
          </Button>
        </form>
      </div>

      <hr class="border-neutral-200 dark:border-neutral-800" />

      <div class="space-y-4">
        <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.backup.restoreTitle')}</h3>
        <p class="text-xs {MUTED}">{t('settings.backup.restoreBody')}</p>
        <form onsubmit={handleImport} class="max-w-md space-y-4 pt-2">
          <Input
            id="restore-pass"
            type={showRestorePassphrase ? 'text' : 'password'}
            label={t('settings.backup.restorePassphrase')}
            required
            bind:value={restorePassphrase}
            trailingAction={restorePassEye}
          />
          {#if restoreMsg}
            <p class="text-xs {restoreMsgKind === 'success' ? 'text-emerald-600 dark:text-emerald-500' : 'text-red-600 dark:text-red-500'}">{restoreMsg}</p>
          {/if}
          <Button type="submit" variant="warning" size="md">
            {t('settings.backup.restoreButton')}
          </Button>
        </form>
      </div>
    </Card>
	{:else if activeTab === 'performance'}
		<Card class="space-y-6" variant="primary" size="lg">
			<div>
				<h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.performance.title')}</h2>
				<p class="{MUTED} text-sm mt-1">{t('settings.performance.subtitle')}</p>
			</div>

			<!-- 1. Background Memory Saving (Toggleable, On by default) -->
			<div class="{SUBCARD} flex items-start justify-between gap-4">
				<div class="min-w-0">
					<p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.backgroundTitle')}</p>
					<p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.backgroundBody')}</p>
					<p class="text-[11px] text-neutral-500 mt-2">{t('settings.performance.recommended')}</p>
				</div>
				<Switch
					checked={backgroundMemorySaving}
					disabled={isSavingPerformance}
					onchange={(val) => saveAllPerformancePrefs({ backgroundMemorySaving: val })}
					aria-label={t('settings.performance.backgroundTitle')}
				/>
			</div>

			<!-- 2. GPU Acceleration (Off by default) -->
			<div class="{SUBCARD} flex items-start justify-between gap-4">
				<div class="min-w-0">
					<p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.gpuTitle')}</p>
					<p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.gpuBody')}</p>
					<p class="text-[11px] text-neutral-500 mt-2">{t('settings.performance.recommendedGpuOff')} · {t('settings.performance.restartNote')}</p>
				</div>
				<Switch
					checked={gpuAcceleration}
					disabled={isSavingPerformance}
					onchange={(val) => saveAllPerformancePrefs({ gpuAcceleration: val }, true)}
					aria-label={t('settings.performance.gpuTitle')}
				/>
			</div>

			<!-- 3. Terminal Scrollback Buffer Limit -->
			<div class="{SUBCARD} flex items-start justify-between gap-4">
				<div class="min-w-0">
					<p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.scrollbackTitle')}</p>
					<p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.scrollbackBody')}</p>
				</div>
				<div class="w-36 shrink-0">
					<Select
						options={[
							{ value: '1000', label: t('settings.performance.scrollback1k') },
							{ value: '5000', label: t('settings.performance.scrollback5k') },
							{ value: '10000', label: t('settings.performance.scrollback10k') },
							{ value: '50000', label: t('settings.performance.scrollback50k') }
						]}
						bind:value={scrollbackStr}
						disabled={isSavingPerformance}
						size="sm"
					/>
				</div>
			</div>

			<!-- 4. Inactive Tab Sleep / Power Saver -->
			<div class="{SUBCARD} flex items-start justify-between gap-4">
				<div class="min-w-0">
					<p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.sleepTitle')}</p>
					<p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.sleepBody')}</p>
					<p class="text-[11px] text-neutral-500 mt-2">{t('settings.performance.recommended')}</p>
				</div>
				<Switch
					checked={inactiveSessionSleep}
					disabled={isSavingPerformance}
					onchange={(val) => saveAllPerformancePrefs({ inactiveSessionSleep: val })}
					aria-label={t('settings.performance.sleepTitle')}
				/>
			</div>

			<!-- 5. Low-Power Mode (30 FPS Limiter) -->
			<div class="{SUBCARD} flex items-start justify-between gap-4">
				<div class="min-w-0">
					<p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.lowPowerTitle')}</p>
					<p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.lowPowerBody')}</p>
				</div>
				<Switch
					checked={lowPowerMode}
					disabled={isSavingPerformance}
					onchange={(val) => saveAllPerformancePrefs({ lowPowerMode: val })}
					aria-label={t('settings.performance.lowPowerTitle')}
				/>
			</div>

			<!-- 6. Lock Screen Animation (On by default) -->
			<div class="{SUBCARD} flex items-start justify-between gap-4">
				<div class="min-w-0">
					<p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.performance.lockAnimationTitle')}</p>
					<p class="text-xs {MUTED} mt-1 leading-relaxed">{t('settings.performance.lockAnimationBody')}</p>
				</div>
				<Switch
					checked={!appearance.reduceMotion}
					onchange={(val) => setReduceMotion(!val)}
					aria-label={t('settings.performance.lockAnimationTitle')}
				/>
			</div>
		</Card>
  {:else if activeTab === 'shortcuts'}
    <Card class="space-y-6" variant="primary" size="lg">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.shortcuts.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.shortcuts.subtitle')}</p>
      </div>

      <div class="space-y-4">
        {#each SHORTCUT_GROUPS as group, gi (group.title)}
          <h3 class="text-sm font-semibold text-neutral-600 dark:text-neutral-300 uppercase tracking-wider {gi > 0 ? 'pt-2' : ''}">{t(`settings.shortcuts.${group.title}`)}</h3>
          <div class="space-y-2">
            {#each group.items as item (item.label)}
              <div class="flex justify-between items-center gap-3 p-3 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded-md">
                <span class="text-sm text-neutral-800 dark:text-neutral-200">{t(`settings.shortcuts.${item.label}`, item.vars)}</span>
                <span class="flex flex-wrap justify-end items-center gap-1.5 text-xs text-neutral-500">
                  {#each item.keys as combo, ci (ci)}
                    {#if ci > 0}<span>{t('settings.shortcuts.or')}</span>{/if}
                    <Kbd size="sm">{combo.join(' + ')}</Kbd>
                  {/each}
                </span>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    </Card>
  {/if}
  <Modal bind:open={showProLogin} size="sm" title={t('pro.account.signInOrRegister')}>
    <ProLoginForm onSignedIn={handleProSignedIn} />
  </Modal>

  <Modal open={!!deviceLimit} onclose={() => (deviceLimit = null)} size="md" title={t('pro.devices.limitTitle')}>
    {#if deviceLimit}
      <div class="space-y-4">
        <div>
          <p class="text-xs text-neutral-500 dark:text-neutral-400 mt-1">{t('pro.devices.limitBody', { count: deviceLimit.limit })}</p>
          {#if deviceLimit.limit < PRO_PRICING.syncDevices}
            <p class="text-xs text-sky-700 dark:text-sky-300 mt-2">
              {t('pro.devices.limitUpsell', { count: PRO_PRICING.syncDevices })}
              <Button type="button" variant="ghost" size="sm" onclick={() => openExternalUrl(pricingUrl(pro.status?.account?.id))} class="font-semibold underline p-0">{t('pro.devices.seePlans')}</Button>
            </p>
          {/if}
        </div>
        <ul class="divide-y divide-neutral-200 dark:divide-neutral-800 border border-neutral-200 dark:border-neutral-800 rounded-md">
          {#each deviceLimit.devices as device (device.id)}
            <li class="flex items-center justify-between gap-3 px-3 py-2 text-xs">
              <div class="min-w-0">
                <p class="font-medium text-neutral-900 dark:text-white truncate">{device.name}</p>
                <p class="text-neutral-500 truncate">{device.os} · {t('pro.devices.lastSeen', { time: formatProTime(device.lastSeenAt) })}</p>
              </div>
              <Button type="button" variant="ghost" size="sm" onclick={() => removeProDevice(device.id)} class="text-rose-600 dark:text-rose-400 hover:underline shrink-0 p-0">{t('pro.devices.remove')}</Button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
    {#snippet footer()}
      <Button type="button" variant="neutral" size="sm" onclick={() => (deviceLimit = null)}>{t('common.close')}</Button>
    {/snippet}
  </Modal>
</div>
