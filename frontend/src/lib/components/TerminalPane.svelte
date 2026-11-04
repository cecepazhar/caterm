<script lang="ts">
  import { Button } from '@cecepazhar/caui';
  import { onMount } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebglAddon } from '@xterm/addon-webgl';
  import { CanvasAddon } from '@xterm/addon-canvas';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { Unicode11Addon } from '@xterm/addon-unicode11';
  import { SearchAddon } from '@xterm/addon-search';
  import { openExternalUrl } from '$lib/utils/url';
  import '@xterm/xterm/css/xterm.css';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import {
    sshConnect,
    sshWrite,
    sshResize,
    sshDisconnect,
    onSshOutput,
    onSshClosed,
    type SshSession
  } from '$lib/api/ssh';
  import type { HostRecord } from '$lib/api/hosts';
  import { listSnippets, type SnippetRecord } from '$lib/api/snippets';
  import { setActiveSession, clearActiveSession, registerSessionPane, broadcastWriteToAll } from '$lib/stores/activeSession.svelte';
  import { getSessionView } from '$lib/stores/sessionView.svelte';
  import { getTheme, terminalTheme } from '$lib/stores/theme.svelte';
  import { LINUX_COMMANDS, commandDescription, type AutocompleteItem } from '$lib/data/terminalCommands';
  import { t } from '$lib/i18n/index.svelte';
  import { getPerformancePrefs } from '$lib/api/performance';
  import TerminalAutocomplete from '$lib/components/TerminalAutocomplete.svelte';
  import { terminalKeyAction } from '$lib/shortcuts';
  import { getTerminalPrefs, stepTerminalFontSize, setTerminalFontSize, DEFAULT_FONT_SIZE } from '$lib/stores/terminalPrefs.svelte';
  import { copyText } from '$lib/utils/clipboard';
  import { getAmbientStore } from '$lib/stores/ambient.svelte';
  import { aiDispatchTask, type DispatchResult } from '$lib/api/aiRouting';
  import {
    checkCommandBlastShield,
    checkPasteSentinel,
    type BlastShieldRisk,
    type PasteCheckResult
  } from '$lib/api/proSecurity';
  import BlastShieldModal from '$lib/components/BlastShieldModal.svelte';
  import PasteSentinelModal from '$lib/components/PasteSentinelModal.svelte';

  let {
    host,
    label,
    isActive = true,
    onSplitRight,
    onSplitDown,
    onClose
  }: {
    host: HostRecord;
    /** Disambiguated tab name, e.g. "YPC (2)" when the host is open more than once. */
    label?: string;
    /** Whether this pane is the one the user is looking at (drives focus + snippet target). */
    isActive?: boolean;
    onSplitRight?: () => void;
    onSplitDown?: () => void;
    onClose?: () => void;
  } = $props();

  function errorMessage(err: unknown): string {
    if (err && typeof err === 'object' && 'message' in err) {
      return String((err as { message?: unknown }).message);
    }
    return String(err);
  }

  const paneLabel = $derived(label ?? host.label);
  const theme = getTheme();
  const sessionView = getSessionView();
  const terminalPrefs = getTerminalPrefs();
  const ambient = getAmbientStore();
  let status = $state<'connecting' | 'connected' | 'offline'>('connecting');
  let terminalContainer: HTMLDivElement;
  let rootContainer: HTMLDivElement | undefined = $state();

  let session = $state<SshSession | null>(null);
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;

  // Autocomplete state
  let autocompleteEnabled = $state(true);
  let cursorX = $state(20);
  let cursorY = $state(60);
  let cursorTop = $state(42);
  let currentInputLine = $state('');
  let commandHistory = $state<string[]>([]);
  let userSnippets = $state<SnippetRecord[]>([]);
  let suggestions = $state<AutocompleteItem[]>([]);
  let selectedSuggestionIndex = $state(-1);

  // AI Diagnostic State
  let showDiagnosticModal = $state(false);
  let isDiagnosing = $state(false);
  let diagnosticResult = $state<DispatchResult | null>(null);
  let diagnosticError = $state<string | null>(null);

  // Security Sentinel State (Blast Shield & Paste Sentinel)
  let showBlastModal = $state(false);
  let pendingBlastCommand = $state('');
  // Terminal input is processed strictly in order: an Enter waiting on the Blast Shield check
  // (or on the user's answer in its modal) holds back everything typed after it.
  let inputChain: Promise<void> = Promise.resolve();
  let blastDecision: (() => void) | null = null;
  // Set while waiting for the user to confirm a connection to a Production host.
  let prodConfirm = $state<((ok: boolean) => void) | null>(null);
  let reconnectNow: () => void = () => {};
  // Find-in-output bar (Ctrl+Shift+F).
  let searchAddon: SearchAddon | null = null;
  let showSearch = $state(false);
  let searchQuery = $state('');
  let searchMiss = $state(false);
  let searchInput: HTMLInputElement | undefined = $state();

  function openSearch() {
    showSearch = true;
    queueMicrotask(() => searchInput?.select());
  }

  function closeSearch() {
    showSearch = false;
    searchMiss = false;
    searchAddon?.clearDecorations();
    term?.clearSelection();
    term?.focus();
  }

  function runSearch(backwards: boolean) {
    if (!searchAddon || !searchQuery) return;
    const found = backwards ? searchAddon.findPrevious(searchQuery) : searchAddon.findNext(searchQuery);
    searchMiss = !found;
  }
  /** Queues input through the same ordered path as typing (Blast Shield, audit, broadcast). */
  let submitInput: (data: string) => void = () => {};
  let pendingBlastRisk = $state<BlastShieldRisk | null>(null);

  let showPasteModal = $state(false);
  let pendingPasteResult = $state<PasteCheckResult | null>(null);

  function getTerminalTailLines(count: number = 40): string[] {
    if (!term) return [];
    const buffer = term.buffer.active;
    const lines: string[] = [];
    const total = buffer.length;
    const start = Math.max(0, total - count);
    for (let i = start; i < total; i++) {
      const line = buffer.getLine(i);
      if (line) {
        lines.push(line.translateToString(true));
      }
    }
    return lines;
  }

  async function triggerErrorDiagnostic() {
    showDiagnosticModal = true;
    isDiagnosing = true;
    diagnosticResult = null;
    diagnosticError = null;
    try {
      const tailLines = getTerminalTailLines(40);
      const res = await aiDispatchTask(
        'error_diagnostic',
        'Analyze recent terminal output, identify root error or crash, and provide explanation and fix command.',
        {
          tail_lines: tailLines,
          host_label: paneLabel,
          active_host_id: host.id
        }
      );
      diagnosticResult = res;
    } catch (err) {
      diagnosticError = errorMessage(err);
    } finally {
      isDiagnosing = false;
    }
  }

  // Load user snippets on mount
  onMount(() => {
    listSnippets()
      .then((snips) => {
        userSnippets = snips;
      })
      .catch(() => {});
  });

  // Places the popup anchor just below the terminal cursor. The WebGL/Canvas renderers draw
  // the cursor on a <canvas> (there is no `.xterm-cursor` element), so derive the cell size
  // from the rendered `.xterm-screen` box instead of xterm's private render dimensions.
  function updateCursorPosition() {
    if (!rootContainer || !term) return;
    const screenEl = terminalContainer?.querySelector('.xterm-screen') as HTMLElement | null;
    if (!screenEl || term.cols === 0 || term.rows === 0) return;

    const rootRect = rootContainer.getBoundingClientRect();
    const screenRect = screenEl.getBoundingClientRect();
    const cellWidth = screenRect.width / term.cols;
    const cellHeight = screenRect.height / term.rows;
    const buffer = term.buffer.active;

    cursorX = Math.round(screenRect.left - rootRect.left + buffer.cursorX * cellWidth);
    cursorTop = Math.round(screenRect.top - rootRect.top + buffer.cursorY * cellHeight);
    cursorY = Math.round(screenRect.top - rootRect.top + (buffer.cursorY + 1) * cellHeight);
  }

  // Autocomplete only makes sense against the normal shell line buffer. An alternate-screen
  // program (vim, less, top, htop, ...) owns the screen and its own line editing, so Tab/↑/↓/Esc
  // must reach it untouched and we must not keep suggesting against stale command-line text.
  function isAlternateScreen(): boolean {
    return term?.buffer.active.type === 'alternate';
  }

  function updateSuggestions(input: string) {
    if (!autocompleteEnabled || !input.trim() || isAlternateScreen()) {
      suggestions = [];
      selectedSuggestionIndex = -1;
      return;
    }

    const trimmed = input.trim();
    const lastWord = input.split(/\s+/).filter(Boolean).pop() || '';
    const results: AutocompleteItem[] = [];
    const seen = new Set<string>();

    // 1. Check history matches first (most relevant to user's current session)
    for (let i = commandHistory.length - 1; i >= 0; i--) {
      const hist = commandHistory[i];
      if (hist.toLowerCase().startsWith(trimmed.toLowerCase()) && !seen.has(hist)) {
        seen.add(hist);
        results.push({
          text: hist,
          type: 'history',
          desc: t('terminal.recentCommand')
        });
        if (results.length >= 3) break;
      }
    }

    // 2. User snippets
    for (const snip of userSnippets) {
      if (
        (snip.command.toLowerCase().startsWith(trimmed.toLowerCase()) ||
          snip.label.toLowerCase().includes(trimmed.toLowerCase())) &&
        !seen.has(snip.command)
      ) {
        seen.add(snip.command);
        results.push({
          text: snip.command,
          type: 'snippet',
          desc: snip.description || snip.label,
          sourceLabel: snip.label
        });
        if (results.length >= 6) break;
      }
    }

    // 3. Common Linux commands matching whole input or last word
    for (const cmd of LINUX_COMMANDS) {
      if (
        (cmd.text.toLowerCase().startsWith(trimmed.toLowerCase()) ||
          (lastWord && cmd.text.toLowerCase().startsWith(lastWord.toLowerCase()))) &&
        !seen.has(cmd.text)
      ) {
        seen.add(cmd.text);
        results.push({
          text: cmd.text,
          type: 'command',
          desc: commandDescription(cmd)
        });
        if (results.length >= 8) break;
      }
    }

    suggestions = results;
    selectedSuggestionIndex = -1;
    if (results.length > 0) {
      updateCursorPosition();
      requestAnimationFrame(updateCursorPosition);
    }
  }

  function applySuggestion(item: AutocompleteItem) {
    if (!session || !term) return;

    // Determine how many characters to clear from the active input line.
    // Snippets and history replace the entire current input; command suggestions
    // only replace the last word.
    let charsToErase: number;
    let replacementText: string;

    if (item.type === 'snippet' || item.type === 'history') {
      charsToErase = currentInputLine.length;
      replacementText = item.text;
    } else {
      const lastSpace = currentInputLine.lastIndexOf(' ');
      charsToErase = currentInputLine.length - (lastSpace + 1);
      replacementText = currentInputLine.slice(0, lastSpace + 1) + item.text;
    }

    // Send the erase sequence to clear previously typed characters.
    let erasePty = '';
    for (let i = 0; i < charsToErase; i++) {
      erasePty += '\x7f'; // Backspace to clear previous input.
    }

    // Write the final completed line to the PTY and update internal state.
    void sshWrite(session.sessionId, erasePty + replacementText).catch(() => {});
    currentInputLine = replacementText;
    suggestions = [];
    selectedSuggestionIndex = -1;
    term.focus();
  }

  function dismissSuggestions() {
    suggestions = [];
    selectedSuggestionIndex = -1;
  }

  // Snippets and AI-proposed commands go through the typing path line by line, so each line
  // gets the Blast Shield check and its own audit-log entry instead of bypassing both.
  function injectCommand(cmd: string) {
    if (!session) return;
    for (const line of cmd.replace(/\r\n/g, '\n').replace(/\n+$/, '').split('\n')) {
      if (line) submitInput(line);
      submitInput('\r');
    }
  }

  function writeRaw(data: string) {
    if (!session) return;
    void sshWrite(session.sessionId, data).catch(() => {});
  }

  function markActive() {
    if (session) {
      setActiveSession({ sessionId: session.sessionId, label: paneLabel, inject: injectCommand, write: writeRaw });
    }
  }

  $effect(() => {
    const palette = terminalTheme(theme.name);
    if (term) term.options.theme = palette;
  });

  // Font size is shared by all panes; a change re-fits the grid and tells the PTY.
  $effect(() => {
    const size = terminalPrefs.fontSize;
    if (!term || term.options.fontSize === size) return;
    term.options.fontSize = size;
    fitAddon?.fit();
    if (session) void sshResize(session.sessionId, term.cols, term.rows).catch(() => {});
  });

  $effect(() => {
    if (!isActive || !session) return;
    markActive();
    requestAnimationFrame(() => {
      fitAddon?.fit();
      if (term && session) {
        term.refresh(0, (term.rows ?? 1) - 1);
        void sshResize(session.sessionId, term.cols, term.rows).catch(() => {});
        term.focus();
      }
    });
  });

  onMount(() => {
    const terminal = new Terminal({
      theme: terminalTheme(theme.name),
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      fontSize: terminalPrefs.fontSize,
      cursorBlink: true,
      // Needed for the Unicode 11 width tables (emoji / CJK take two cells).
      allowProposedApi: true,
      scrollback: 10000,
      linkHandler: {
        activate: (_event, text) => {
          void openExternalUrl(text);
        }
      }
    });
    term = terminal;
    // Settings → Performance → scrollback (stored by the backend). It used to be saved but
    // never applied: every pane was hardcoded to 1000 lines.
    void getPerformancePrefs()
      .then((prefs) => {
        const lines = Number(prefs.scrollbackLines);
        if (!disposed && Number.isFinite(lines) && lines >= 100) terminal.options.scrollback = Math.min(lines, 200_000);
      })
      .catch(() => {});

    // Web links detector for clickable URLs in terminal output
    const webLinks = new WebLinksAddon((_event, uri) => {
      void openExternalUrl(uri);
    });
    terminal.loadAddon(webLinks);

    // Unicode 11 character widths, so emoji and CJK line up instead of overlapping.
    terminal.loadAddon(new Unicode11Addon());
    terminal.unicode.activeVersion = '11';
    const search = new SearchAddon();
    terminal.loadAddon(search);
    searchAddon = search;

    // Paste Sentinel: intercept clipboard paste into the terminal
    const handlePaste = (e: ClipboardEvent) => {
      const text = e.clipboardData?.getData('text');
      if (!text) return;
      // Stop the paste right now, synchronously: once this handler returns, xterm's own paste
      // listener would already have sent the text to the shell. The text is re-pasted below
      // only after the sentinel has looked at it.
      e.preventDefault();
      e.stopPropagation();
      void (async () => {
        try {
          const check = await checkPasteSentinel(text);
          if (check.containsSecret) {
            pendingPasteResult = check;
            showPasteModal = true;
            return;
          }
        } catch {
          // Sentinel unavailable (community build): fall through and paste normally.
        }
        terminal.paste(text);
      })();
    };

    terminalContainer.addEventListener('paste', handlePaste, true);

    const fit = new FitAddon();
    fitAddon = fit;
    terminal.loadAddon(fit);
    terminal.open(terminalContainer);

    terminal.onKey(() => {
      ambient.triggerTypingBloom();
    });

    // Hardware-accelerated terminal renderer with graceful fallbacks: WebGL -> Canvas -> DOM
    try {
    	const webgl = new WebglAddon();
    	webgl.onContextLoss(() => {
    		webgl.dispose();
    	});
    	terminal.loadAddon(webgl);
    } catch {
    	try {
    		const canvasAddon = new CanvasAddon();
    		terminal.loadAddon(canvasAddon);
    	} catch {
    		// Fall back to default DOM renderer
    	}
    }

    fit.fit();

    let disposed = false;
    const unlisteners: UnlistenFn[] = [];
    const early = new Map<string, string[]>();

    function keepUnlisten(unlisten: UnlistenFn) {
      if (disposed) unlisten();
      else unlisteners.push(unlisten);
    }

    let reportedFailure = false;
    function reportFailure(what: string, err: unknown) {
      if (disposed || reportedFailure) return;
      reportedFailure = true;
      terminal.write(`\r\n\x1b[31m${what}: ${errorMessage(err)}\x1b[0m\r\n`);
    }

    terminal.writeln(
      `\x1b[1;32m${t('terminal.welcome')}\x1b[0m — ${t('terminal.connectingTo')} \x1b[1;36m${host.label}\x1b[0m (${host.address})...`
    );

    void (async () => {
      try {
        keepUnlisten(
          await onSshOutput(({ sessionId, data }) => {
            if (disposed) return;
            if (session) {
              if (sessionId === session.sessionId) terminal.write(data);
              return;
            }
            const buffered = early.get(sessionId) ?? [];
            buffered.push(data);
            early.set(sessionId, buffered);
          })
        );

        keepUnlisten(
          await onSshClosed(({ sessionId, unexpected }) => {
            if (disposed || !session || sessionId !== session.sessionId) return;
            status = 'offline';
            clearActiveSession(session.sessionId);
            session = null;
            if (unexpected) {
              terminal.write(`\r\n\x1b[33m${t('terminal.connectionLost')}\x1b[0m\r\n`);
              scheduleReconnect();
            } else {
              terminal.write(`\r\n\x1b[33m${t('terminal.closedByRemote')}\x1b[0m\r\n`);
              terminal.write(`\x1b[2m${t('terminal.pressEnterToReconnect')}\x1b[0m\r\n`);
            }
          })
        );
      } catch (err) {
        if (!disposed) {
          status = 'offline';
          terminal.write(
            `\r\n\x1b[31m${t('terminal.subscribeFailed')}: ${errorMessage(err)}\x1b[0m\r\n`
          );
        }
        return;
      }

      if (host.options?.environment === 'production') {
        terminal.write(`\r\n\x1b[1;31m${t('terminal.prodWarning')}\x1b[0m\r\n`);
        const ok = await new Promise<boolean>((resolve) => (prodConfirm = resolve));
        prodConfirm = null;
        if (disposed) return;
        if (!ok) {
          status = 'offline';
          terminal.write(`\r\n\x1b[33m${t('terminal.prodCancelled')}\x1b[0m\r\n`);
          return;
        }
      }
      await openSession();
    })();

    async function openSession(): Promise<boolean> {
      status = 'connecting';
      try {
        const opened = await sshConnect(host.id);
        if (disposed) {
          void sshDisconnect(opened.sessionId).catch(() => {});
          return false;
        }

        session = opened;
        status = 'connected';
        reconnectAttempt = 0;
        registerSessionPane({
          sessionId: opened.sessionId,
          label: paneLabel,
          inject: injectCommand,
          write: writeRaw
        });
        fit.fit();
        void sshResize(opened.sessionId, terminal.cols, terminal.rows).catch((err) =>
          reportFailure(t('terminal.resizeFailed'), err)
        );
        terminal.write(`\r\n\x1b[32m${t('terminal.connected')}\x1b[0m (${t('terminal.session')} ${opened.sessionId})\r\n`);

        for (const chunk of early.get(opened.sessionId) ?? []) terminal.write(chunk);
        early.clear();
        return true;
      } catch (err) {
        if (disposed) return false;
        status = 'offline';
        terminal.write(`\r\n\x1b[31m${t('terminal.sshFailed')}: ${errorMessage(err)}\x1b[0m\r\n`);
        return false;
      }
    }

    // Auto-reconnect after an unexpected drop: exponential backoff (1, 2, 4, 8, 16 s), then
    // give up and leave "press Enter to reconnect" to the user.
    const MAX_RECONNECT_ATTEMPTS = 5;
    let reconnectAttempt = 0;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    function scheduleReconnect() {
      if (disposed || reconnectTimer) return;
      if (reconnectAttempt >= MAX_RECONNECT_ATTEMPTS) {
        terminal.write(`\r\n\x1b[31m${t('terminal.reconnectGaveUp')}\x1b[0m\r\n`);
        terminal.write(`\x1b[2m${t('terminal.pressEnterToReconnect')}\x1b[0m\r\n`);
        return;
      }
      reconnectAttempt += 1;
      const delayMs = 1000 * 2 ** (reconnectAttempt - 1);
      status = 'connecting';
      terminal.write(
        `\x1b[33m${t('terminal.reconnecting', { attempt: reconnectAttempt, max: MAX_RECONNECT_ATTEMPTS, sec: delayMs / 1000 })}\x1b[0m\r\n`
      );
      reconnectTimer = setTimeout(async () => {
        reconnectTimer = null;
        if (disposed) return;
        if (!(await openSession())) scheduleReconnect();
      }, delayMs);
    }
    reconnectNow = () => {
      if (session || reconnectTimer || status === 'connecting') return;
      reconnectAttempt = 0;
      void openSession();
    };

    // Intercept keystrokes for copy/paste, font size and autocomplete. App-wide shortcuts
    // (tabs, palette, …) never get here: the layout catches them in the capture phase.
    terminal.attachCustomKeyEventHandler((event: KeyboardEvent) => {
      const action = terminalKeyAction(event, terminal.hasSelection());
      if (action) {
        if (action === 'paste') {
          // Not preventDefault: the browser's own paste event reaches xterm, which pastes
          // (bracketed where the shell wants it) without needing clipboard-read permission.
          return false;
        }
        event.preventDefault();
        if (action === 'find') {
          openSearch();
        } else if (action === 'copy') {
          const text = terminal.getSelection();
          terminal.clearSelection();
          void copyText(text).then(() => terminal.focus());
        } else if (action === 'fontUp') {
          stepTerminalFontSize(1);
        } else if (action === 'fontDown') {
          stepTerminalFontSize(-1);
        } else {
          setTerminalFontSize(DEFAULT_FONT_SIZE);
        }
        return false;
      }
      if (suggestions.length > 0 && !isAlternateScreen()) {
        if (event.key === 'Enter') {
          // Non-intrusive: unless the user explicitly navigated the list with
          // ArrowDown/ArrowUp (selectedSuggestionIndex >= 0), Enter must reach the
          // PTY so the typed command runs as-is.
          if (selectedSuggestionIndex < 0) {
            dismissSuggestions();
            return true;
          }
          if (event.type === 'keydown') {
            event.preventDefault();
            event.stopPropagation();
            const picked = suggestions[selectedSuggestionIndex];
            if (picked) applySuggestion(picked);
          }
          return false;
        }
        if (event.key === 'Tab') {
          if (event.type === 'keydown') {
            event.preventDefault();
            event.stopPropagation();
            // Tab accepts the first suggestion when the user hasn't navigated.
            const picked =
              selectedSuggestionIndex < 0 ? suggestions[0] : suggestions[selectedSuggestionIndex];
            if (picked) applySuggestion(picked);
          }
          return false;
        }
        if (event.key === 'ArrowDown') {
          if (event.type === 'keydown') {
            event.preventDefault();
            event.stopPropagation();
            selectedSuggestionIndex =
              selectedSuggestionIndex < 0 ? 0 : (selectedSuggestionIndex + 1) % suggestions.length;
          }
          return false;
        }
        if (event.key === 'ArrowUp') {
          if (event.type === 'keydown') {
            event.preventDefault();
            event.stopPropagation();
            selectedSuggestionIndex =
              selectedSuggestionIndex < 0
                ? suggestions.length - 1
                : (selectedSuggestionIndex - 1 + suggestions.length) % suggestions.length;
          }
          return false;
        }
        if (event.key === 'Escape') {
          if (event.type === 'keydown') {
            event.preventDefault();
            event.stopPropagation();
            dismissSuggestions();
          }
          return false;
        }
      }
      return true;
    });

    async function handleTerminalInput(data: string) {
      // Offline pane: Enter reconnects instead of being sent nowhere.
      if (!session) {
        if (data === '\r') reconnectNow();
        return;
      }
      // Enter on a non-empty line: run the Blast Shield safety check before it reaches the shell.
      if (data === '\r' || data === '\n') {
        const fullCmd = currentInputLine.trim();
        if (fullCmd) {
          try {
            const res = await checkCommandBlastShield(fullCmd);
            if (res.isDestructive && res.risk) {
              pendingBlastCommand = fullCmd;
              pendingBlastRisk = res.risk;
              showBlastModal = true;
              // Hold all further input until the user answers; confirm/cancel send the
              // Enter or Ctrl+C themselves.
              await new Promise<void>((resolve) => (blastDecision = resolve));
              return;
            }
          } catch {
            // Ignore blast shield check error in community/fallback mode
          }
        }
      }
      dispatchPtyData(data);
    }

    submitInput = (data: string) => {
      inputChain = inputChain.then(() => handleTerminalInput(data)).catch(() => {});
    };
    terminal.onData((data) => submitInput(data));

    function dispatchPtyData(data: string) {
      // Send keystroke to the real Rust SSH PTY backend
      if (session) {
        void sshWrite(session.sessionId, data).catch((err) => reportFailure(t('terminal.sendFailed'), err));
        if (sessionView.broadcastInput) {
          broadcastWriteToAll(data, session.sessionId);
        }
      }

      if (isAlternateScreen()) {
        // Don't track command-line state or keep suggestions alive while a full-screen program
        // (vim, less, top, ...) owns the terminal — see attachCustomKeyEventHandler above.
        if (suggestions.length > 0) {
          suggestions = [];
          selectedSuggestionIndex = -1;
        }
        return;
      }

      // Track active line buffer for smart autocomplete
      if (data === '\r' || data === '\n') {
        const finishedCmd = currentInputLine.trim();
        if (finishedCmd) {
          commandHistory = [...commandHistory.filter((c) => c !== finishedCmd), finishedCmd];
        }
        currentInputLine = '';
        suggestions = [];
        selectedSuggestionIndex = -1;
      } else if (data === '\x7f' || data === '\x08') {
        // Backspace
        currentInputLine = currentInputLine.slice(0, -1);
        updateSuggestions(currentInputLine);
        updateCursorPosition();
      } else if (data === '\x03') {
        // Ctrl+C
        currentInputLine = '';
        suggestions = [];
        selectedSuggestionIndex = -1;
      } else if (!data.includes('\x1b') && data >= ' ') {
        currentInputLine += data;
        updateSuggestions(currentInputLine);
        updateCursorPosition();
      }
    }

    // Keystrokes are sent before the shell echoes them back, so follow the real cursor once
    // the echo has been rendered instead of using the pre-echo position.
    terminal.onCursorMove(() => {
      if (suggestions.length > 0) updateCursorPosition();
    });

    let fitQueued = false;
    const applyFit = () => {
      fitQueued = false;
      if (terminalContainer.clientWidth === 0 || terminalContainer.clientHeight === 0) return;
      fit.fit();
      if (session) {
        void sshResize(session.sessionId, terminal.cols, terminal.rows).catch(() => {});
      }
      if (suggestions.length > 0) {
        updateCursorPosition();
      }
    };
    const observer = new ResizeObserver(() => {
      if (fitQueued) return;
      fitQueued = true;
      // Debounce slightly or requestAnimationFrame to prevent layout thrashing on sidebar transitions
      setTimeout(() => {
        requestAnimationFrame(applyFit);
      }, 50);
    });
    observer.observe(terminalContainer);

    const handleClick = () => {
      markActive();
      terminal.focus();
    };
    terminalContainer.addEventListener('click', handleClick);

    return () => {
      disposed = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      prodConfirm?.(false);
      terminalContainer?.removeEventListener('paste', handlePaste, true);
      for (const unlisten of unlisteners) unlisten();
      unlisteners.length = 0;
      observer.disconnect();
      terminalContainer.removeEventListener('click', handleClick);
      if (session) {
        clearActiveSession(session.sessionId);
        void sshDisconnect(session.sessionId).catch(() => {});
      }
      term = null;
      fitAddon = null;
      terminal.dispose();
    };
  });

  function confirmBlastCommand() {
    showBlastModal = false;
    if (session && pendingBlastCommand) {
      void sshWrite(session.sessionId, '\r').catch(() => {});
    }
    pendingBlastCommand = '';
    pendingBlastRisk = null;
    blastDecision?.();
    blastDecision = null;
  }

  function cancelBlastCommand() {
    showBlastModal = false;
    // Send Ctrl+C to clear the line in terminal
    if (session) {
      void sshWrite(session.sessionId, '\x03').catch(() => {});
    }
    currentInputLine = '';
    pendingBlastCommand = '';
    pendingBlastRisk = null;
    blastDecision?.();
    blastDecision = null;
  }

  function pasteIntoTerminal(text: string) {
    if (term) term.paste(text);
    else if (session) void sshWrite(session.sessionId, text).catch(() => {});
  }

  function handlePasteSanitized(text: string) {
    showPasteModal = false;
    if (text) pasteIntoTerminal(text);
    pendingPasteResult = null;
  }

  function handlePasteOriginal(text: string) {
    showPasteModal = false;
    if (text) pasteIntoTerminal(text);
    pendingPasteResult = null;
  }

  function cancelPaste() {
    showPasteModal = false;
    pendingPasteResult = null;
  }
</script>

<div
	bind:this={rootContainer}
	class="relative flex flex-col h-full bg-white dark:bg-[#09090b] border {status === 'offline' ? 'border-rose-500/60 ring-1 ring-rose-500/30' : 'border-neutral-200 dark:border-neutral-800'} rounded-md overflow-hidden transition-colors"
>
	<!-- Top Bar -->
	<div class="h-7 sm:h-8 bg-neutral-100 dark:bg-neutral-900/90 border-b border-neutral-200 dark:border-neutral-800 px-2.5 flex items-center justify-between text-xs font-mono text-neutral-500 dark:text-neutral-400 shrink-0">
		<div class="flex items-center gap-2 min-w-0">
			<span
				class="w-2.5 h-2.5 rounded-full shrink-0 transition-all duration-300 {status === 'connected'
					? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]'
					: status === 'connecting'
						? 'bg-amber-400 animate-pulse shadow-[0_0_6px_rgba(251,191,36,0.6)]'
						: 'bg-rose-500 shadow-[0_0_10px_rgba(244,63,94,0.9)] animate-pulse ring-2 ring-rose-500/30'}"
				title={status === 'connected' ? t('terminal.connected') : status === 'connecting' ? t('terminal.connecting') : t('terminal.sshFailed')}
			></span>
			<span class="text-neutral-900 dark:text-white font-medium truncate">{paneLabel}</span>
			<span class="text-neutral-500 dark:text-neutral-500 hidden sm:inline truncate">({host.address})</span>
			{#if sessionView.broadcastInput}
			  <span class="ml-1 px-1.5 py-0.5 rounded text-[9px] font-bold tracking-wider uppercase bg-amber-500/20 text-amber-500 border border-amber-500/40 animate-pulse">
			    SYNC
			  </span>
			{/if}
			</div>
    <div class="flex items-center gap-2 text-neutral-500 dark:text-neutral-400 shrink-0">
      <!-- Autocomplete toggle button -->
      <Button
        variant="ghost"
        size="sm"
        onclick={() => {
          autocompleteEnabled = !autocompleteEnabled;
          if (!autocompleteEnabled) {
            suggestions = [];
            selectedSuggestionIndex = -1;
          }
        }}
        class="!px-1.5 !py-0.5 rounded text-[10px] font-sans font-semibold transition-colors flex items-center gap-1 {autocompleteEnabled ? 'bg-sky-500/15 dark:bg-sky-500/20 text-sky-700 dark:text-sky-400 border border-sky-500/30' : 'bg-neutral-200 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400 border border-transparent'}"
        title={t('terminal.toggleAutocompleteTitle')}
        aria-label={t('terminal.toggleAutocomplete')}
      >
        <span class="w-1.5 h-1.5 rounded-full {autocompleteEnabled ? 'bg-sky-400' : 'bg-neutral-500'}"></span>
        <span>{t('terminal.autocomplete')}</span>
      </Button>

      <!-- AI Error Diagnose button -->
      <Button
        variant="ghost"
        size="sm"
        onclick={triggerErrorDiagnostic}
        class="!px-1.5 !py-0.5 rounded text-[10px] font-sans font-semibold transition-colors flex items-center gap-1 bg-purple-500/15 dark:bg-purple-500/20 text-purple-700 dark:text-purple-400 border border-purple-500/30 hover:bg-purple-500/25"
        title="Diagnose Recent Errors with Master AI Router"
        aria-label="AI Diagnose"
      >
        <svg class="w-3 h-3 text-purple-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
        <span>Diagnose</span>
      </Button>

      {#if onSplitRight}
        <Button variant="ghost" size="icon" onclick={onSplitRight} class="hover:text-sky-600 dark:hover:text-sky-400 !p-0.5 rounded transition-colors" title={t('terminal.splitRightTitle')} aria-label={t('terminal.splitRight')}>
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </Button>
      {/if}
      {#if onSplitDown}
        <Button variant="ghost" size="icon" onclick={onSplitDown} class="hover:text-sky-600 dark:hover:text-sky-400 !p-0.5 rounded transition-colors" title={t('terminal.splitDownTitle')} aria-label={t('terminal.splitDown')}>
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </Button>
      {/if}
      <span class="ml-1 text-neutral-500 hidden sm:inline">SSH</span>
      <span class="text-neutral-500 hidden sm:inline">UTF-8</span>
      {#if onClose}
        <Button variant="ghost" size="icon" onclick={onClose} class="hover:text-rose-600 dark:hover:text-rose-400 !p-0.5 rounded transition-colors ml-1" title={t('terminal.closePane')} aria-label={t('terminal.closePane')}>
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
        </Button>
      {/if}
    </div>
  </div>

  <!-- Terminal canvas container -->
  <div bind:this={terminalContainer} class="flex-1 p-1 overflow-hidden outline-none relative"></div>

  {#if showSearch}
    <div class="absolute top-9 right-3 z-20 flex items-center gap-1.5 rounded-md border border-neutral-300 dark:border-neutral-700 bg-white/95 dark:bg-neutral-900/95 shadow px-2 py-1">
      <input
        bind:this={searchInput}
        bind:value={searchQuery}
        class="w-56 bg-transparent text-xs font-mono outline-none text-neutral-800 dark:text-neutral-200"
        placeholder={t('terminal.searchPlaceholder')}
        aria-label={t('terminal.searchPlaceholder')}
        oninput={() => runSearch(false)}
        onkeydown={(e) => {
          if (e.key === 'Enter') { e.preventDefault(); runSearch(e.shiftKey); }
          else if (e.key === 'Escape') { e.preventDefault(); closeSearch(); }
        }}
      />
      {#if searchMiss && searchQuery}
        <span class="text-[10px] text-rose-500">{t('terminal.searchNoMatch')}</span>
      {/if}
      <button type="button" class="text-xs px-1 text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200" onclick={() => runSearch(true)} aria-label="previous">↑</button>
      <button type="button" class="text-xs px-1 text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200" onclick={() => runSearch(false)} aria-label="next">↓</button>
      <button type="button" class="text-xs px-1 text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200" onclick={closeSearch} aria-label="close">✕</button>
    </div>
  {/if}

  {#if prodConfirm}
    <div class="absolute inset-x-0 bottom-0 z-20 m-2 p-3 rounded-md border border-rose-500/50 bg-rose-50 dark:bg-rose-950/80 text-xs flex flex-wrap items-center gap-2">
      <span class="font-bold text-rose-700 dark:text-rose-300">PRODUCTION</span>
      <span class="flex-1 min-w-0 text-rose-800 dark:text-rose-200">{t('terminal.prodConfirmText', { name: host.label })}</span>
      <Button variant="secondary" size="sm" onclick={() => prodConfirm?.(false)}>{t('common.cancel')}</Button>
      <Button variant="primary" size="sm" onclick={() => prodConfirm?.(true)}>{t('terminal.prodConfirmConnect')}</Button>
    </div>
  {/if}

  <!-- Smart Autocomplete Overlay -->
  {#if autocompleteEnabled && suggestions.length > 0}
    <TerminalAutocomplete
      {suggestions}
      selectedIndex={selectedSuggestionIndex}
      {cursorX}
      {cursorY}
      {cursorTop}
      containerWidth={rootContainer?.clientWidth ?? 0}
      containerHeight={rootContainer?.clientHeight ?? 0}
      onSelect={applySuggestion}
      onClose={dismissSuggestions}
    />
  {/if}

  <!-- AI Error Diagnostic Modal Overlay -->
  {#if showDiagnosticModal}
    <div class="absolute inset-0 z-40 bg-black/70 backdrop-blur-xs flex flex-col p-4 overflow-hidden">
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-4 flex flex-col max-h-full overflow-hidden shadow-2xl text-neutral-900 dark:text-white space-y-3">
        <div class="flex items-center justify-between pb-2 border-b border-neutral-200 dark:border-neutral-800">
          <div class="flex items-center gap-2">
            <span class="p-1 rounded bg-purple-500/10 text-purple-600 dark:text-purple-400">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </span>
            <h3 class="text-sm font-semibold">AI Error Diagnostic & Fix</h3>
            {#if diagnosticResult}
              <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300">
                {diagnosticResult.model}
                {#if diagnosticResult.fell_back}
                  (Failover Active)
                {/if}
              </span>
            {/if}
          </div>
          <Button
            variant="ghost"
            size="icon"
            onclick={() => (showDiagnosticModal = false)}
            class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 font-bold text-lg !p-1"
            title={t('common.close')}
            aria-label={t('common.close')}
          >
            &times;
          </Button>
        </div>

        <div class="flex-1 overflow-y-auto space-y-3 text-xs">
          {#if isDiagnosing}
            <div class="flex flex-col items-center justify-center py-12 text-neutral-500 space-y-3">
              <svg class="w-6 h-6 animate-spin text-purple-500" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
              </svg>
              <p>Analyzing terminal output & learned habits via Master AI Router...</p>
            </div>
          {:else if diagnosticError}
            <div class="p-3 rounded bg-rose-500/10 border border-rose-500/30 text-rose-700 dark:text-rose-300">
              Diagnostic failed: {diagnosticError}
            </div>
          {:else if diagnosticResult}
            <div class="p-3 rounded bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 space-y-2 whitespace-pre-wrap font-mono text-xs">
              {diagnosticResult.content}
            </div>
          {/if}
        </div>

        <div class="flex justify-between items-center pt-2 border-t border-neutral-200 dark:border-neutral-800">
          <span class="text-[10px] text-neutral-400">Zero-knowledge local memory & PII scrubber enabled.</span>
          <div class="flex gap-2">
            <Button
              variant="secondary"
              size="sm"
              onclick={() => (showDiagnosticModal = false)}
              class="px-3 py-1.5 rounded border border-neutral-300 dark:border-neutral-700 text-xs font-medium hover:bg-neutral-100 dark:hover:bg-neutral-800"
            >
              Close
            </Button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Security Sentinel Modals -->
  <BlastShieldModal
    isOpen={showBlastModal}
    command={pendingBlastCommand}
    risk={pendingBlastRisk}
    onConfirm={confirmBlastCommand}
    onCancel={cancelBlastCommand}
  />

  <PasteSentinelModal
    isOpen={showPasteModal}
    pasteResult={pendingPasteResult}
    onPasteSanitized={handlePasteSanitized}
    onPasteOriginal={handlePasteOriginal}
    onCancel={cancelPaste}
  />
</div>
