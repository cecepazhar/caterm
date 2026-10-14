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
  {#if osKey === 'ubuntu' || osKey === 'kubuntu' || osKey === 'xubuntu' || osKey === 'lubuntu'}
    <!-- Ubuntu Circle of Friends 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#E95420" />
      <!-- Central circle ring -->
      <circle cx="128" cy="128" r="54" fill="none" stroke="#FFFFFF" stroke-width="26" />
      <!-- Cutouts separating the three sections -->
      <circle cx="48" cy="128" r="16" fill="#E95420" />
      <circle cx="168" cy="58" r="16" fill="#E95420" />
      <circle cx="168" cy="198" r="16" fill="#E95420" />
      <!-- Three heads (dots) -->
      <circle cx="48" cy="128" r="18" fill="#FFFFFF" />
      <circle cx="168" cy="58" r="18" fill="#FFFFFF" />
      <circle cx="168" cy="198" r="18" fill="#FFFFFF" />
    </svg>
  {:else if osKey === 'debian'}
    <!-- Authentic Debian Swirl 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#D70A53" />
      <!-- Debian Spiral Vector -->
      <path
        d="M138 52 C184 54 216 88 214 134 C212 178 178 212 134 212 C92 212 58 178 58 136 C58 98 88 68 126 68 C158 68 184 94 184 126 C184 154 162 176 134 176 C112 176 94 158 94 136 C94 118 108 104 126 104 C138 104 148 114 148 126 C148 134 142 140 134 140 C128 140 124 136 124 130 C124 116 142 116 142 106 C142 98 134 94 126 94 C112 94 98 108 98 126 C98 148 118 168 140 168 C166 168 188 146 188 120 C188 90 162 66 128 66 C90 66 56 98 56 138 C56 184 94 220 140 220 C190 220 228 182 230 132 C232 80 194 42 138 42 Z"
        fill="#FFFFFF"
      />
    </svg>
  {:else if osKey === 'fedora'}
    <!-- Authentic Fedora Infinity 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#294172" />
      <!-- Infinity loop and f -->
      <path
        d="M152 72 C120 72 98 96 98 128 L98 184 C98 196 108 206 120 206 C132 206 142 196 142 184 L142 144 L164 144 C176 144 186 134 186 122 C186 110 176 100 164 100 L142 100 L142 96 C142 86 148 82 156 82 C164 82 170 86 172 90 L194 78 C186 64 172 56 152 56 Z"
        fill="#51A2DA"
      />
      <path
        d="M104 184 L104 128 C104 108 118 92 138 92 C146 92 152 94 156 98 L156 68 C148 64 138 62 128 62 C96 62 74 86 74 120 L74 184 C74 204 88 218 108 218 C118 218 128 214 136 208 L124 192 C118 196 112 198 106 198 C96 198 88 192 88 182 Z"
        fill="#FFFFFF"
      />
      <!-- White crossbar -->
      <rect x="74" y="122" width="60" height="20" rx="10" fill="#FFFFFF" />
    </svg>
  {:else if osKey === 'arch'}
    <!-- Authentic Arch Linux 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#1793D1" />
      <!-- Arch Mountain Swoop -->
      <path
        d="M128 44 C116 72 104 98 94 122 C104 132 118 138 134 138 C150 138 162 132 170 124 C158 98 144 70 128 44 Z"
        fill="#FFFFFF"
      />
      <path
        d="M128 44 L56 210 C78 198 98 192 118 192 C104 178 96 160 96 140 C108 152 122 158 138 158 C154 158 168 150 178 138 C176 160 166 180 152 194 C172 196 192 202 212 212 L128 44 Z"
        fill="#FFFFFF"
        opacity="0.95"
      />
    </svg>
  {:else if osKey === 'centos'}
    <!-- Authentic CentOS 4-Color Gem 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#262577" />
      <g transform="translate(128,128)">
        <!-- Top: Purple/Magenta -->
        <path d="M0 -18 L46 -64 L-46 -64 Z" fill="#93227F" />
        <path d="M0 -64 L46 -110 L-46 -110 Z" fill="#93227F" opacity="0.85" />
        <!-- Right: Orange -->
        <path d="M18 0 L64 46 L64 -46 Z" fill="#FFA300" />
        <path d="M64 0 L110 46 L110 -46 Z" fill="#FFA300" opacity="0.85" />
        <!-- Bottom: Green -->
        <path d="M0 18 L46 64 L-46 64 Z" fill="#8EBA42" />
        <path d="M0 64 L46 110 L-46 110 Z" fill="#8EBA42" opacity="0.85" />
        <!-- Left: Blue -->
        <path d="M-18 0 L-64 46 L-64 -46 Z" fill="#2E2882" />
        <path d="M-64 0 L-110 46 L-110 -46 Z" fill="#2E2882" opacity="0.85" />
        <!-- Center Box -->
        <rect x="-18" y="-18" width="36" height="36" fill="#FFFFFF" rx="4" />
      </g>
    </svg>
  {:else if osKey === 'redhat'}
    <!-- Authentic Red Hat 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#1E1E1E" />
      <path
        d="M64 168 C76 142 108 126 148 126 C188 126 216 144 228 168 C204 182 164 190 128 190 C88 190 68 180 64 168 Z"
        fill="#EE0000"
      />
      <!-- Fedora Crown -->
      <path
        d="M96 154 C96 116 122 84 156 84 C184 84 204 104 204 138 C204 154 198 160 188 162 C158 162 120 158 96 154 Z"
        fill="#EE0000"
      />
      <!-- Black Ribbon -->
      <path
        d="M98 152 C120 156 158 158 188 158 C192 158 196 156 198 152 C186 148 154 144 128 144 C112 144 102 148 98 152 Z"
        fill="#111111"
      />
    </svg>
  {:else if osKey === 'rocky'}
    <!-- Authentic Rocky Linux 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#0F172A" />
      <!-- Emerald Mountain Polygon -->
      <path
        d="M64 192 L128 64 L192 192 L148 192 L128 150 L108 192 Z"
        fill="#10B981"
      />
      <circle cx="128" cy="116" r="14" fill="#FFFFFF" />
    </svg>
  {:else if osKey === 'almalinux'}
    <!-- Authentic AlmaLinux 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#141923" />
      <g transform="translate(128,128)">
        <circle cx="-38" cy="-38" r="28" fill="#FF5252" />
        <circle cx="38" cy="-38" r="28" fill="#FFC107" />
        <circle cx="38" cy="38" r="28" fill="#00E676" />
        <circle cx="-38" cy="38" r="28" fill="#00B0FF" />
        <circle cx="0" cy="0" r="18" fill="#FFFFFF" />
      </g>
    </svg>
  {:else if osKey === 'alpine'}
    <!-- Authentic Alpine Linux Mountain 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#0D597F" />
      <!-- Peak 1 & Peak 2 -->
      <path d="M48 184 L112 88 L144 136 L176 88 L224 184 Z" fill="#FFFFFF" />
      <path d="M112 88 L128 112 L112 120 L96 112 Z" fill="#0D597F" />
      <path d="M176 88 L192 112 L176 120 L160 112 Z" fill="#0D597F" />
    </svg>
  {:else if osKey === 'opensuse'}
    <!-- Authentic openSUSE Geeko 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#73BA25" />
      <path
        d="M64 164 C64 120 96 88 144 88 C176 88 200 104 208 132 C212 144 204 154 192 154 C182 154 176 146 172 138 C164 122 152 114 136 114 C112 114 94 132 94 158 C94 178 110 192 134 192 C148 192 160 186 166 176 L188 188 C176 206 156 216 130 216 C88 216 64 196 64 164 Z"
        fill="#FFFFFF"
      />
      <!-- Chameleon Eye -->
      <circle cx="168" cy="112" r="18" fill="#FFFFFF" />
      <circle cx="172" cy="110" r="8" fill="#225B10" />
    </svg>
  {:else if osKey === 'mint'}
    <!-- Authentic Linux Mint 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#87CF3E" />
      <circle cx="128" cy="128" r="102" fill="none" stroke="#FFFFFF" stroke-width="16" />
      <!-- LM Outline -->
      <path
        d="M92 96 L92 160 L112 160 L112 120 C112 112 118 106 126 106 C134 106 140 112 140 120 L140 160 L160 160 L160 120 C160 112 166 106 174 106 C182 106 188 112 188 120 L188 160 L208 160 L208 116 C208 98 194 86 178 86 C168 86 158 92 152 100 C146 92 136 86 124 86 C108 86 92 98 92 116 Z"
        fill="#FFFFFF"
      />
    </svg>
  {:else if osKey === 'kali'}
    <!-- Authentic Kali Linux 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#0B132B" />
      <path
        d="M72 84 C92 72 136 76 168 104 C184 118 194 136 196 156 C190 150 182 146 172 144 C184 162 178 184 160 196 C144 206 122 206 106 196 C124 192 136 182 140 168 C128 174 112 172 100 164 C90 156 86 144 88 132 C94 136 102 138 110 136 C92 124 88 104 100 86 C88 94 80 106 78 120 C74 108 72 96 72 84 Z"
        fill="#2680EB"
      />
      <circle cx="132" cy="108" r="5" fill="#FFFFFF" />
    </svg>
  {:else if osKey === 'manjaro'}
    <!-- Authentic Manjaro 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#18221E" />
      <!-- Left Column -->
      <rect x="58" y="58" width="38" height="140" rx="6" fill="#35BF5C" />
      <!-- Middle Pillar with top block -->
      <rect x="109" y="110" width="38" height="88" rx="6" fill="#35BF5C" />
      <rect x="109" y="58" width="89" height="38" rx="6" fill="#35BF5C" />
      <!-- Right Column -->
      <rect x="160" y="110" width="38" height="88" rx="6" fill="#35BF5C" />
    </svg>
  {:else if osKey === 'nixos'}
    <!-- Authentic NixOS Snowflake 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#1E2A45" />
      <g transform="translate(128,128)">
        <!-- 6 symmetric lambda branches -->
        <g id="nix-arm">
          <path d="M0 -105 L26 -60 L10 -60 L-8 -90 Z" fill="#7EBAE4" />
          <path d="M-8 -90 L16 -48 L-16 -48 L-32 -76 Z" fill="#5277C3" />
        </g>
        <use href="#nix-arm" transform="rotate(60)" />
        <use href="#nix-arm" transform="rotate(120)" />
        <use href="#nix-arm" transform="rotate(180)" />
        <use href="#nix-arm" transform="rotate(240)" />
        <use href="#nix-arm" transform="rotate(300)" />
      </g>
    </svg>
  {:else if osKey === 'raspberry'}
    <!-- Authentic Raspberry Pi 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#1B1B1B" />
      <!-- Green Leaves -->
      <path d="M128 54 C116 70 96 74 86 72 C96 86 116 88 128 78 C140 88 160 86 170 72 C160 74 140 70 128 54 Z" fill="#75A928" />
      <!-- Berry Nodes -->
      <g fill="#C51A4A">
        <circle cx="96" cy="110" r="18" />
        <circle cx="128" cy="102" r="18" />
        <circle cx="160" cy="110" r="18" />
        <circle cx="86" cy="144" r="18" />
        <circle cx="118" cy="140" r="19" />
        <circle cx="150" cy="144" r="18" />
        <circle cx="170" cy="144" r="18" />
        <circle cx="106" cy="178" r="18" />
        <circle cx="140" cy="180" r="18" />
        <circle cx="124" cy="204" r="16" />
      </g>
    </svg>
  {:else if osKey === 'gentoo'}
    <!-- Authentic Gentoo "g" 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#54487A" />
      <path
        d="M136 76 C176 76 200 102 200 134 C200 172 168 198 126 198 C82 198 56 166 56 130 C56 90 90 58 136 58 C158 58 178 66 192 80 L174 98 C164 88 152 82 136 82 C106 82 84 104 84 130 C84 154 100 174 126 174 C150 174 172 158 172 136 C172 122 162 114 144 114 L126 114 L126 94 L154 94 Z"
        fill="#FFFFFF"
      />
    </svg>
  {:else if osKey === 'void'}
    <!-- Authentic Void Linux 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#18231C" />
      <circle cx="128" cy="128" r="82" fill="none" stroke="#478061" stroke-width="26" />
      <circle cx="128" cy="128" r="32" fill="#478061" />
    </svg>
  {:else if osKey === 'freebsd'}
    <!-- Authentic FreeBSD Beastie Sphere 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#AB2B28" />
      <!-- Horns -->
      <path d="M78 82 C68 56 52 46 44 44 C54 64 68 92 82 102 Z" fill="#FFFFFF" />
      <path d="M178 82 C188 56 204 46 212 44 C202 64 188 92 174 102 Z" fill="#FFFFFF" />
      <!-- Beastie Eyes -->
      <circle cx="106" cy="132" r="12" fill="#FFFFFF" />
      <circle cx="150" cy="132" r="12" fill="#FFFFFF" />
      <circle cx="110" cy="134" r="6" fill="#1A1A1A" />
      <circle cx="154" cy="134" r="6" fill="#1A1A1A" />
    </svg>
  {:else if osKey === 'windows'}
    <!-- Authentic Windows 4-Square 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#0078D4" />
      <rect x="56" y="56" width="66" height="66" rx="4" fill="#FFFFFF" />
      <rect x="134" y="56" width="66" height="66" rx="4" fill="#FFFFFF" />
      <rect x="56" y="134" width="66" height="66" rx="4" fill="#FFFFFF" />
      <rect x="134" y="134" width="66" height="66" rx="4" fill="#FFFFFF" />
    </svg>
  {:else if osKey === 'macos' || osKey === 'ios'}
    <!-- Authentic Apple Logo 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#1D1D1F" />
      <!-- Leaf -->
      <path d="M154 52 C164 66 158 84 144 94 C134 82 140 64 154 52 Z" fill="#FFFFFF" />
      <!-- Apple Body with Bite -->
      <path
        d="M174 122 C174 102 188 92 190 90 C180 76 164 74 158 74 C144 72 130 82 122 82 C114 82 102 74 90 74 C72 74 54 88 46 112 C34 142 50 188 74 216 C84 228 96 238 110 238 C122 238 128 230 142 230 C156 230 162 238 174 238 C188 238 198 226 208 214 C218 198 222 188 224 184 C198 174 174 150 174 122 Z"
        fill="#FFFFFF"
      />
    </svg>
  {:else if osKey === 'android'}
    <!-- Authentic Android Bugdroid 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#202124" />
      <g fill="#3DDC84">
        <!-- Antennae -->
        <line x1="88" y1="74" x2="68" y2="48" stroke="#3DDC84" stroke-width="10" stroke-linecap="round" />
        <line x1="168" y1="74" x2="188" y2="48" stroke="#3DDC84" stroke-width="10" stroke-linecap="round" />
        <!-- Head -->
        <path d="M58 132 C58 84 90 68 128 68 C166 68 198 84 198 132 Z" />
        <!-- Body -->
        <path d="M58 142 L198 142 C198 194 174 204 128 204 C82 204 58 194 58 142 Z" />
      </g>
      <!-- Eyes -->
      <circle cx="98" cy="108" r="8" fill="#202124" />
      <circle cx="158" cy="108" r="8" fill="#202124" />
    </svg>
  {:else if osKey === 'cisco'}
    <!-- Authentic Cisco Soundwaves 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#049FD9" />
      <g stroke="#FFFFFF" stroke-width="12" stroke-linecap="round">
        <line x1="52" y1="140" x2="52" y2="180" />
        <line x1="77" y1="110" x2="77" y2="180" />
        <line x1="102" y1="76" x2="102" y2="180" />
        <line x1="128" y1="110" x2="128" y2="180" />
        <line x1="154" y1="76" x2="154" y2="180" />
        <line x1="179" y1="110" x2="179" y2="180" />
        <line x1="204" y1="140" x2="204" y2="180" />
      </g>
    </svg>
  {:else if osKey === 'mikrotik'}
    <!-- Authentic MikroTik 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#2A323D" />
      <path
        d="M64 96 L96 96 L128 144 L160 96 L192 96 L144 164 L144 192 L112 192 L112 164 Z"
        fill="#FFFFFF"
      />
    </svg>
  {:else if osKey === 'amazon'}
    <!-- Amazon AWS 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#232F3E" />
      <!-- Orange Smile Arrow -->
      <path
        d="M64 154 C104 184 152 184 192 154"
        fill="none"
        stroke="#FF9900"
        stroke-width="16"
        stroke-linecap="round"
      />
      <path d="M174 140 L194 156 L178 176 Z" fill="#FF9900" />
    </svg>
  {:else if osKey === 'server'}
    <!-- Sleek Datacenter Rack Server 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#0F172A" />
      <!-- Rack Chassis -->
      <rect x="52" y="74" width="152" height="48" rx="8" fill="#1E293B" stroke="#38BDF8" stroke-width="8" />
      <rect x="52" y="134" width="152" height="48" rx="8" fill="#1E293B" stroke="#38BDF8" stroke-width="8" />
      <!-- LEDs & Vent Slots -->
      <circle cx="78" cy="98" r="7" fill="#10B981" />
      <circle cx="98" cy="98" r="7" fill="#38BDF8" />
      <line x1="124" y1="98" x2="182" y2="98" stroke="#64748B" stroke-width="6" stroke-linecap="round" />
      <circle cx="78" cy="158" r="7" fill="#10B981" />
      <circle cx="98" cy="158" r="7" fill="#38BDF8" />
      <line x1="124" y1="158" x2="182" y2="158" stroke="#64748B" stroke-width="6" stroke-linecap="round" />
    </svg>
  {:else}
    <!-- Authentic Tux Penguin 256x256 -->
    <svg viewBox="0 0 256 256" width={size} height={size}>
      <circle cx="128" cy="128" r="128" fill="#F1F5F9" />
      <!-- Body & Wings -->
      <path
        d="M128 44 C96 44 80 74 76 112 C72 136 62 168 54 186 C64 192 84 194 98 184 C106 196 116 202 128 202 C140 202 150 196 158 184 C172 194 192 192 202 186 C194 168 184 136 180 112 C176 74 160 44 128 44 Z"
        fill="#111827"
      />
      <!-- White Belly -->
      <ellipse cx="128" cy="148" rx="36" ry="46" fill="#FFFFFF" />
      <!-- Eyes -->
      <ellipse cx="114" cy="84" rx="10" ry="14" fill="#FFFFFF" />
      <ellipse cx="142" cy="84" rx="10" ry="14" fill="#FFFFFF" />
      <circle cx="116" cy="86" r="5" fill="#111827" />
      <circle cx="140" cy="86" r="5" fill="#111827" />
      <!-- Orange Beak -->
      <path d="M110 102 C118 100 138 100 146 102 C142 118 136 124 128 124 C120 124 114 118 110 102 Z" fill="#F59E0B" />
      <!-- Orange Feet -->
      <ellipse cx="104" cy="204" rx="22" ry="10" fill="#F59E0B" />
      <ellipse cx="152" cy="204" rx="22" ry="10" fill="#F59E0B" />
    </svg>
  {/if}
</span>
