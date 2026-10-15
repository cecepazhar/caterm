<script lang="ts">
  import { AllMenusSheet as CauiAllMenusSheet, type MenuGroup } from '@cecepazhar/caui';
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';

  let {
    open = $bindable(false),
    onClose
  }: {
    open: boolean;
    onClose: () => void;
  } = $props();

  function navigateTo(href: string) {
    open = false;
    onClose?.();
    void goto(href);
  }

  const menus: MenuGroup[] = $derived([
    {
      id: 'workspace',
      title: t('nav.sectionWorkspace'),
      items: [
        {
          id: 'hosts',
          label: t('nav.hosts'),
          icon: hostIcon,
          action: () => navigateTo('/'),
        },
        {
          id: 'session',
          label: t('shell.session') || 'Session',
          icon: sessionIcon,
          action: () => navigateTo('/session'),
        },
        {
          id: 'sftp',
          label: t('nav.sftp'),
          icon: sftpIcon,
          action: () => navigateTo('/sftp'),
        },
        {
          id: 'snippets',
          label: t('nav.snippets'),
          icon: snippetsIcon,
          action: () => navigateTo('/snippets'),
        },
        {
          id: 'prompt-studio',
          label: t('nav.promptStudio') || 'Prompt Studio',
          icon: promptStudioIcon,
          action: () => navigateTo('/prompt-studio'),
        },
      ],
    },
    {
      id: 'operations',
      title: t('nav.sectionOps'),
      items: [
        {
          id: 'monitoring',
          label: t('nav.monitoring') || 'Monitoring',
          icon: monitoringIcon,
          action: () => navigateTo('/monitoring'),
        },
        {
          id: 'tasks',
          label: t('nav.tasks') || 'Tasks',
          icon: tasksIcon,
          action: () => navigateTo('/tasks'),
        },
        {
          id: 'port-forwarding',
          label: t('nav.portForwarding') || 'Port Fwd',
          icon: portFwdIcon,
          action: () => navigateTo('/port-forwarding'),
        },
        {
          id: 'command-logs',
          label: t('nav.commandLogs') || 'Logs',
          icon: logsIcon,
          action: () => navigateTo('/command-logs'),
        },
        {
          id: 'investigations',
          label: t('nav.investigations') || 'Investigations',
          icon: investigationsIcon,
          action: () => navigateTo('/investigations'),
        },
      ],
    },
    {
      id: 'security-access',
      title: t('nav.sectionAccess'),
      items: [
        {
          id: 'ssh-keys',
          label: t('nav.sshKeys') || 'SSH Keys',
          icon: sshKeysIcon,
          action: () => navigateTo('/ssh-keys'),
        },
        {
          id: 'totp',
          label: t('nav.totp') || 'TOTP',
          icon: totpIcon,
          action: () => navigateTo('/totp'),
        },
        {
          id: 'security',
          label: t('nav.security') || 'Security',
          icon: securityIcon,
          action: () => navigateTo('/security'),
        },
        {
          id: 'groups',
          label: t('nav.groups') || 'Groups',
          icon: groupsIcon,
          action: () => navigateTo('/groups'),
        },
        {
          id: 'teams',
          label: t('nav.teams') || 'Teams',
          icon: teamsIcon,
          action: () => navigateTo('/teams'),
        },
      ],
    },
    {
      id: 'labs-settings',
      title: t('nav.sectionLab'),
      items: [
        {
          id: 'devops',
          label: t('nav.devops') || 'DevOps Lab',
          icon: devopsIcon,
          action: () => navigateTo('/devops'),
        },
        {
          id: 'design-system',
          label: t('nav.designSystem'),
          icon: designSystemIcon,
          action: () => navigateTo('/design-system'),
        },
        {
          id: 'ambient',
          label: t('nav.ambient'),
          icon: ambientIcon,
          action: () => navigateTo('/settings?tab=ambient'),
        },
        {
          id: 'settings',
          label: t('nav.settings') || 'Settings',
          icon: settingsIcon,
          action: () => navigateTo('/settings'),
        },
      ],
    },
  ]);
</script>

{#snippet iconWrapper(d: string, color: string)}
  <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0 {color}">
    <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" {d} />
    </svg>
  </div>
{/snippet}

{#snippet hostIcon()}{@render iconWrapper('M4 17l6-5-6-5M12 19h8', 'text-rose-500 bg-rose-500/10')}{/snippet}
{#snippet sessionIcon()}{@render iconWrapper('M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z', 'text-emerald-500 bg-emerald-500/10')}{/snippet}
{#snippet sftpIcon()}{@render iconWrapper('M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z', 'text-sky-500 bg-sky-500/10')}{/snippet}
{#snippet snippetsIcon()}{@render iconWrapper('M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4', 'text-amber-500 bg-amber-500/10')}{/snippet}
{#snippet promptStudioIcon()}{@render iconWrapper('M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z', 'text-purple-500 bg-purple-500/10')}{/snippet}
{#snippet monitoringIcon()}{@render iconWrapper('M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z', 'text-indigo-500 bg-indigo-500/10')}{/snippet}
{#snippet tasksIcon()}{@render iconWrapper('M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z', 'text-teal-500 bg-teal-500/10')}{/snippet}
{#snippet portFwdIcon()}{@render iconWrapper('M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4', 'text-orange-500 bg-orange-500/10')}{/snippet}
{#snippet logsIcon()}{@render iconWrapper('M4 6h16M4 12h16M4 18h16', 'text-neutral-500 bg-neutral-500/10')}{/snippet}
{#snippet investigationsIcon()}{@render iconWrapper('M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z', 'text-cyan-500 bg-cyan-500/10')}{/snippet}
{#snippet sshKeysIcon()}{@render iconWrapper('M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z', 'text-yellow-500 bg-yellow-500/10')}{/snippet}
{#snippet totpIcon()}{@render iconWrapper('M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z', 'text-blue-500 bg-blue-500/10')}{/snippet}
{#snippet groupsIcon()}{@render iconWrapper('M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10', 'text-violet-500 bg-violet-500/10')}{/snippet}
{#snippet securityIcon()}{@render iconWrapper('M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z', 'text-emerald-500 bg-emerald-500/10')}{/snippet}
{#snippet devopsIcon()}{@render iconWrapper('M13 10V3L4 14h7v7l9-11h-7z', 'text-amber-500 bg-amber-500/10')}{/snippet}
{#snippet teamsIcon()}{@render iconWrapper('M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z', 'text-pink-500 bg-pink-500/10')}{/snippet}
{#snippet ambientIcon()}{@render iconWrapper('M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z', 'text-fuchsia-500 bg-fuchsia-500/10')}{/snippet}
{#snippet designSystemIcon()}{@render iconWrapper('M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01', 'text-cyan-500 bg-cyan-500/10')}{/snippet}
{#snippet settingsIcon()}{@render iconWrapper('M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065zM15 12a3 3 0 11-6 0 3 3 0 016 0z', 'text-zinc-500 bg-zinc-500/10')}{/snippet}

<CauiAllMenusSheet
  {menus}
  bind:open
  title={t('nav.allMenusSheet')}
  searchable={true}
  searchPlaceholder={t('nav.searchMenus')}
  size="lg"
  onOpenChange={(next) => {
    open = next;
    if (!next) onClose?.();
  }}
/>
