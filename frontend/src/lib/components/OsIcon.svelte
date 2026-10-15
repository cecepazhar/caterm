<script lang="ts">
  let { os = '', name = '', tags = [], address = '', size = 16, class: className = '' }: {
    os?: string;
    name?: string;
    tags?: string[];
    address?: string;
    size?: number;
    class?: string;
  } = $props();

  export function detectOsKey(explicitOs?: string, hostName?: string, hostTags?: string[], hostAddress?: string): string {
    const raw = (explicitOs || '').trim().toLowerCase();
    if (raw) {
      if (raw.includes('kubuntu')) return 'kubuntu';
      if (raw.includes('xubuntu')) return 'xubuntu';
      if (raw.includes('lubuntu')) return 'lubuntu';
      if (raw.includes('ubuntu')) return 'ubuntu';
      if (raw.includes('debian')) return 'debian';
      if (raw.includes('fedora')) return 'fedora';
      if (raw.includes('red hat') || raw.includes('redhat') || raw.includes('rhel')) return 'redhat';
      if (raw.includes('centos')) return 'centos';
      if (raw.includes('rocky')) return 'rocky';
      if (raw.includes('almalinux') || raw.includes('alma')) return 'almalinux';
      if (raw.includes('arch')) return 'arch';
      if (raw.includes('manjaro')) return 'manjaro';
      if (raw.includes('alpine')) return 'alpine';
      if (raw.includes('opensuse') || raw.includes('suse')) return 'opensuse';
      if (raw.includes('mint')) return 'mint';
      if (raw.includes('kali')) return 'kali';
      if (raw.includes('pop') || raw.includes('pop!_os') || raw.includes('popos')) return 'popos';
      if (raw.includes('gentoo')) return 'gentoo';
      if (raw.includes('void')) return 'void';
      if (raw.includes('nixos') || raw.includes('nix')) return 'nixos';
      if (raw.includes('endeavour')) return 'endeavour';
      if (raw.includes('elementary')) return 'elementary';
      if (raw.includes('zorin')) return 'zorin';
      if (raw.includes('raspbian') || raw.includes('raspberry') || raw.includes('rpi')) return 'raspberry';
      if (raw.includes('amazon') || raw.includes('amzn')) return 'amazon';
      if (raw.includes('oracle')) return 'oracle';
      if (raw.includes('slackware')) return 'slackware';
      if (raw.includes('mageia')) return 'mageia';
      if (raw.includes('solus')) return 'solus';
      if (raw.includes('tails')) return 'tails';
      if (raw.includes('deepin')) return 'deepin';
      if (raw.includes('clear')) return 'clear';
      if (raw.includes('garuda')) return 'garuda';
      if (raw.includes('steamos') || raw.includes('steam')) return 'steam';
      if (raw.includes('coreos')) return 'coreos';
      if (raw.includes('flatcar')) return 'flatcar';
      if (raw.includes('devuan')) return 'devuan';
      if (raw.includes('parrot')) return 'parrot';
      if (raw.includes('mx')) return 'mx';
      if (raw.includes('freebsd')) return 'freebsd';
      if (raw.includes('openbsd')) return 'openbsd';
      if (raw.includes('netbsd')) return 'netbsd';
      if (raw.includes('mikrotik') || raw.includes('routeros')) return 'mikrotik';
      if (raw.includes('cisco')) return 'cisco';
      if (raw.includes('windows') || raw.includes('win')) return 'windows';
      if (raw.includes('macos') || raw.includes('apple') || raw.includes('mac') || raw.includes('darwin')) return 'macos';
      if (raw.includes('android')) return 'android';
      if (raw.includes('ios')) return 'ios';
      if (raw.includes('server')) return 'server';
      if (raw.includes('linux')) return 'linux';
      return raw;
    }

    const corpus = `${hostName || ''} ${(hostTags || []).join(' ')} ${hostAddress || ''}`.toLowerCase();
    if (corpus.includes('kubuntu')) return 'kubuntu';
    if (corpus.includes('xubuntu')) return 'xubuntu';
    if (corpus.includes('lubuntu')) return 'lubuntu';
    if (corpus.includes('ubuntu')) return 'ubuntu';
    if (corpus.includes('debian')) return 'debian';
    if (corpus.includes('fedora') || corpus.includes('fc4')) return 'fedora';
    if (corpus.includes('rhel') || corpus.includes('redhat') || corpus.includes('red hat')) return 'redhat';
    if (corpus.includes('centos')) return 'centos';
    if (corpus.includes('rocky')) return 'rocky';
    if (corpus.includes('alma') || corpus.includes('almalinux')) return 'almalinux';
    if (corpus.includes('arch')) return 'arch';
    if (corpus.includes('manjaro')) return 'manjaro';
    if (corpus.includes('alpine')) return 'alpine';
    if (corpus.includes('opensuse') || corpus.includes('suse')) return 'opensuse';
    if (corpus.includes('mint')) return 'mint';
    if (corpus.includes('kali')) return 'kali';
    if (corpus.includes('pop-os') || corpus.includes('popos') || corpus.includes('pop!_os')) return 'popos';
    if (corpus.includes('gentoo')) return 'gentoo';
    if (corpus.includes('void')) return 'void';
    if (corpus.includes('nixos') || corpus.includes('nix')) return 'nixos';
    if (corpus.includes('endeavour')) return 'endeavour';
    if (corpus.includes('elementary')) return 'elementary';
    if (corpus.includes('zorin')) return 'zorin';
    if (corpus.includes('raspbian') || corpus.includes('raspberry') || corpus.includes('rpi')) return 'raspberry';
    if (corpus.includes('amazon') || corpus.includes('amzn') || corpus.includes('ec2') || corpus.includes('aws')) return 'amazon';
    if (corpus.includes('oracle') || corpus.includes('oci')) return 'oracle';
    if (corpus.includes('slackware')) return 'slackware';
    if (corpus.includes('mageia')) return 'mageia';
    if (corpus.includes('solus')) return 'solus';
    if (corpus.includes('tails')) return 'tails';
    if (corpus.includes('deepin')) return 'deepin';
    if (corpus.includes('clear linux') || corpus.includes('clearlinux')) return 'clear';
    if (corpus.includes('garuda')) return 'garuda';
    if (corpus.includes('steam')) return 'steam';
    if (corpus.includes('flatcar')) return 'flatcar';
    if (corpus.includes('coreos')) return 'coreos';
    if (corpus.includes('devuan')) return 'devuan';
    if (corpus.includes('parrot')) return 'parrot';
    if (corpus.includes('mx linux') || corpus.includes('mxlinux')) return 'mx';
    if (corpus.includes('freebsd') || corpus.includes('bsd')) return 'freebsd';
    if (corpus.includes('openbsd')) return 'openbsd';
    if (corpus.includes('netbsd')) return 'netbsd';
    if (corpus.includes('mikrotik') || corpus.includes('routeros')) return 'mikrotik';
    if (corpus.includes('cisco') || corpus.includes('ios-xe')) return 'cisco';
    if (corpus.includes('windows') || corpus.includes('win10') || corpus.includes('win11') || corpus.includes('winserver') || corpus.includes('rdp')) return 'windows';
    if (corpus.includes('mac') || corpus.includes('osx') || corpus.includes('apple') || corpus.includes('darwin')) return 'macos';
    if (corpus.includes('android')) return 'android';
    if (corpus.includes('ios') || corpus.includes('iphone') || corpus.includes('ipad')) return 'ios';
    if (corpus.includes('vps') || corpus.includes('srv') || corpus.includes('node') || corpus.includes('host') || corpus.includes('server')) return 'server';

    return 'linux';
  }

  let osKey = $derived(detectOsKey(os, name, tags, address));
</script>

<span
  class="inline-flex items-center justify-center shrink-0 {className}"
  style="width: {size}px; height: {size}px;"
  title={osKey}
>
  <svg
    viewBox="0 0 24 24"
    width={size}
    height={size}
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    {#if osKey === 'ubuntu' || osKey === 'kubuntu' || osKey === 'xubuntu' || osKey === 'lubuntu' || osKey === 'popos' || osKey === 'elementary' || osKey === 'zorin'}
      <!-- ubuntu / *buntu: circle outline with 3 friend nodes -->
      <circle cx="12" cy="12" r="5" />
      <circle cx="12" cy="4" r="1.5" />
      <circle cx="5" cy="16" r="1.5" />
      <circle cx="19" cy="16" r="1.5" />
      <path d="M9.5 7.5a5.5 5.5 0 0 0-2.5 4.5" />
      <path d="M9 16.5a5.5 5.5 0 0 0 6 0" />
      <path d="M17 12a5.5 5.5 0 0 0-2.5-4.5" />
    {:else if osKey === 'debian' || osKey === 'devuan' || osKey === 'parrot' || osKey === 'mx' || osKey === 'tails' || osKey === 'deepin'}
      <!-- debian: swirl line curve -->
      <path d="M12 3.5c4.5 0 8 3.5 8 8 0 5-4 9-9 9-4.5 0-7.5-3.5-7.5-7.5 0-3.5 2.5-6 6-6 3 0 5 2 5 4.5 0 2-1.5 3.5-3.5 3.5s-2.5-1-2.5-2" />
    {:else if osKey === 'fedora'}
      <!-- fedora: infinity loop & stem -->
      <path d="M12 12c-2.5 0-4.5-2-4.5-4.5S9.5 3 12 3s4.5 2 4.5 4.5v11c0 1.4-1.1 2.5-2.5 2.5s-2.5-1.1-2.5-2.5V11" />
      <path d="M7.5 11.5h7" />
      <path d="M16.5 12c0 2.5-2 4.5-4.5 4.5" />
    {:else if osKey === 'arch' || osKey === 'endeavour' || osKey === 'garuda' || osKey === 'steam'}
      <!-- arch: mountain peak silhouette -->
      <path d="M12 3.5L4 20.5c2.5-1.2 5-1.7 8-1.7s5.5.5 8 1.7L12 3.5z" />
      <path d="M9.5 15c1-1 2.5-1.5 4-1 1 .3 1.5 1 2.5 1.5" />
    {:else if osKey === 'manjaro'}
      <!-- manjaro: 3 vertical column bars -->
      <rect x="4" y="4" width="4" height="16" rx="1" />
      <rect x="10" y="10" width="4" height="10" rx="1" />
      <rect x="16" y="4" width="4" height="16" rx="1" />
      <path d="M10 4h6" />
    {:else if osKey === 'alpine'}
      <!-- alpine: dual mountain peaks -->
      <path d="M3 19.5L9.5 6.5l4.5 9 2.5-5 4.5 9H3z" />
      <path d="M7.5 13l4 2.5" />
      <path d="M15 13.5l3 2" />
    {:else if osKey === 'centos'}
      <!-- centos: 4 interlocking quadrant rectangles -->
      <rect x="4" y="4" width="6.5" height="6.5" rx="1" />
      <rect x="13.5" y="4" width="6.5" height="6.5" rx="1" />
      <rect x="4" y="13.5" width="6.5" height="6.5" rx="1" />
      <rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1" />
      <circle cx="12" cy="12" r="1.5" />
    {:else if osKey === 'redhat' || osKey === 'oracle'}
      <!-- redhat: fedora hat outline silhouette -->
      <path d="M3 16.5c3-1.8 6-2 9-2s6 .2 9 2c-3 1.8-6 2-9 2s-6-.2-9-2z" />
      <path d="M7 15c.5-4.5 2.5-8.5 5-8.5 1 0 1.5.8 2.5.8s1.5-.8 2.5-.8c2.5 0 4.5 4 5 8.5" />
    {:else if osKey === 'rocky' || osKey === 'almalinux'}
      <!-- rocky / almalinux: mountain peak with circle sun -->
      <path d="M3.5 19.5L12 6.5l8.5 13H3.5z" />
      <circle cx="12" cy="11.5" r="2" />
      <path d="M10 19.5l2-3.5 2 3.5" />
    {:else if osKey === 'opensuse' || osKey === 'suse'}
      <!-- opensuse / suse: chameleon curve -->
      <path d="M6 14.5c0-4 3.5-7.5 8-7.5 3.5 0 6 2.5 6 5.5 0 2-1 3.5-2.5 4-2 .8-4-.5-5-2s-2-2.5-4-1c-1.5 1-1.5 3.5.5 5 2.5 1.8 6.5 1.5 9-.5" />
      <circle cx="15.5" cy="10" r="1.2" />
    {:else if osKey === 'mint'}
      <!-- mint: leaf outline with m -->
      <path d="M19 5c-7 0-14 3-14 10 0 4 3 6 6 6 7 0 10-7 10-14 0-.7-.3-1.4-2-2z" />
      <path d="M8.5 15v-4c0-1 1-1.8 2-1.8s2 .8 2 1.8v4" />
      <path d="M12.5 11c0-1 1-1.8 2-1.8s2 .8 2 1.8v4" />
    {:else if osKey === 'kali'}
      <!-- kali: dragon wing swoop -->
      <path d="M4 17.5c2.5-1 5-3.5 6-6.5 1-3 3-5.5 7-6.5-2.5 3-2.5 6-.5 8.5-3 .5-5 2.5-6 5.5 2.5-.5 5-2 6.5-4-1 4-3.5 7-7.5 8-3 .7-4.5-.5-5.5-5z" />
      <circle cx="16.5" cy="6.5" r="1" />
    {:else if osKey === 'gentoo'}
      <!-- gentoo: geometric smooth curve -->
      <path d="M14 6c3.5 0 6 2.5 6 6s-3 7-7.5 7C8 19 5 16 5 12s3.2-7 7.5-7c2 0 3.8.8 5 2l-3 3" />
      <path d="M12 12h7.5" />
    {:else if osKey === 'nixos'}
      <!-- nixos: 6-fold geometric lambda lines -->
      <path d="M12 3v7l4-2M12 21v-7l-4 2" />
      <path d="M4.2 7.5l6 3.5v4.5M19.8 16.5l-6-3.5v-4.5" />
      <path d="M4.2 16.5l6-3.5-4-2M19.8 7.5l-6 3.5 4 2" />
    {:else if osKey === 'raspberry'}
      <!-- raspberry: berry outline with leaf -->
      <path d="M12 6.5c-1.5-2.5-4-2.5-5-2 0 2 1.5 3.5 3 3.5" />
      <path d="M12 6.5c1.5-2.5 4-2.5 5-2 0 2-1.5 3.5-3 3.5" />
      <path d="M12 6.5V9" />
      <path d="M7 10.5c-2 1.5-2.5 4-1 6.5 1.5 2.5 4 4 6 4s4.5-1.5 6-4c1.5-2.5 1-5-1-6.5" />
      <circle cx="9.5" cy="13" r="1.2" />
      <circle cx="14.5" cy="13" r="1.2" />
      <circle cx="12" cy="16.5" r="1.2" />
    {:else if osKey === 'amazon'}
      <!-- amazon: cloud outline with smile curve -->
      <path d="M6.5 14.5A4.5 4.5 0 0 1 6 6a5.5 5.5 0 0 1 10.5-1 4.5 4.5 0 0 1 4 5 4 4 0 0 1-2.5 4.5" />
      <path d="M7 16.5c3 2 7 2 10 0" />
      <path d="M15.5 16l2 1-.5 2" />
    {:else if osKey === 'windows'}
      <!-- windows: 4 outline pane rectangles -->
      <rect x="4" y="4" width="6.5" height="6.5" rx="1" />
      <rect x="13.5" y="4" width="6.5" height="6.5" rx="1" />
      <rect x="4" y="13.5" width="6.5" height="6.5" rx="1" />
      <rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1" />
    {:else if osKey === 'macos'}
      <!-- macos: apple silhouette outline -->
      <path d="M14.5 3.5c-.5 1.5-1.8 2.5-3.5 2.5 0-1.7 1.2-3 3.5-2.5z" />
      <path d="M16.5 9.5c-1-.6-2.2-.5-3.2.2-.8-.6-1.8-.7-2.8-.3-1.6.6-2.5 2.2-2.3 3.8.2 1.8 1 3.5 2 4.8.6.8 1.3 1.5 2.2 1.5.7 0 1-.4 1.7-.4s1 .4 1.7.4c.9 0 1.6-.7 2.2-1.5.5-.7.8-1.4 1-2.1-1.5-.6-2.2-2.7-1-3.9.5-.4 1-.6 1.4-.7-.4-1-.9-1.5-1.9-1.8z" />
    {:else if osKey === 'android'}
      <!-- android: robot head outline -->
      <path d="M8 5l-1.5-2M16 5l1.5-2" />
      <path d="M6 14v-2a6 6 0 0 1 12 0v2H6z" />
      <circle cx="9.5" cy="10" r="1" />
      <circle cx="14.5" cy="10" r="1" />
      <line x1="6" y1="14" x2="18" y2="14" />
    {:else if osKey === 'ios'}
      <!-- ios: phone outline -->
      <rect x="6.5" y="3" width="11" height="18" rx="2.5" />
      <line x1="10.5" y1="6" x2="13.5" y2="6" />
      <line x1="11" y1="18" x2="13" y2="18" />
    {:else if osKey === 'freebsd' || osKey === 'openbsd' || osKey === 'netbsd'}
      <!-- freebsd: horns outline -->
      <path d="M7 6c-1.5 2-2 5 .5 7 1.5 1 3 .5 3.5-1.5" />
      <path d="M17 6c1.5 2 2 5-.5 7-1.5 1-3 .5-3.5-1.5" />
      <path d="M8 14.5c2 2 6 2 8 0" />
      <circle cx="9.5" cy="12" r="1" />
      <circle cx="14.5" cy="12" r="1" />
    {:else if osKey === 'mikrotik' || osKey === 'cisco'}
      <!-- mikrotik / cisco: router with 4 arrows -->
      <circle cx="12" cy="12" r="8.5" />
      <path d="M12 7v4m0 0l-1.5-1.5m1.5 1.5l1.5-1.5" />
      <path d="M12 17v-4m0 0l-1.5 1.5m1.5-1.5l1.5 1.5" />
      <path d="M7 12h4m0 0l-1.5-1.5m1.5 1.5l-1.5 1.5" />
      <path d="M17 12h-4m0 0l1.5-1.5m-1.5 1.5l1.5 1.5" />
    {:else}
      <!-- server / linux: server blade outline with status dot and rack lines -->
      <rect x="3.5" y="4.5" width="17" height="7" rx="1.5" />
      <rect x="3.5" y="12.5" width="17" height="7" rx="1.5" />
      <circle cx="6.5" cy="8" r="1" />
      <circle cx="6.5" cy="16" r="1" />
      <line x1="11" y1="8" x2="17.5" y2="8" />
      <line x1="11" y1="16" x2="17.5" y2="16" />
    {/if}
  </svg>
</span>
