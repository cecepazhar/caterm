// Drives the boot splash defined inline in app.html. The splash stays up while the app warms
// up (at least MIN_MS so it never flickers, never more than MAX_MS — app.html enforces its own
// cap too, so a stuck promise can never keep the user waiting).

const MIN_MS = 900;
const MAX_MS = 3000;

type BootApi = { start: number; lang?: string; step: (text: string, pct?: number) => void; done: () => void };

// The splash runs before i18n is loaded, so its few lines are kept here.
const STEPS = {
  en: { vault: 'Opening vault…', ui: 'Preparing workspace…', fonts: 'Loading fonts…', ready: 'Ready' },
  id: { vault: 'Membuka vault…', ui: 'Menyiapkan workspace…', fonts: 'Memuat font…', ready: 'Siap' }
};
const api = (): BootApi | undefined => (typeof window !== 'undefined' ? (window as any).__catermBoot : undefined);

// rAF never fires while the window is hidden/occluded, so cap the wait.
const frame = () =>
  Promise.race([
    new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))),
    new Promise<void>((r) => setTimeout(r, 120))
  ]);
const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
const settle = (p: Promise<unknown>) => p.then(() => undefined, () => undefined);

/** Warm-up work that makes the first screen ready; each step is best-effort. */
async function warmUp(step: BootApi['step'], lang?: string) {
  const s = lang === 'id' ? STEPS.id : STEPS.en;

  step(s.vault, 35);
  // The lock screen's first question: is there a vault yet? Asking now warms the IPC bridge
  // and the backend's data dir, so the lock screen renders its final form immediately.
  await settle(import('$lib/api/vault').then((m) => m.isVaultInitialized()));

  step(s.ui, 60);
  // Svelte has mounted by now; give the lock screen / shell one frame to lay out.
  await frame();

  step(s.fonts, 80);
  if (document.fonts?.ready) await Promise.race([settle(document.fonts.ready), sleep(400)]);

  step(s.ready, 95);
  await frame(); // let the real UI paint underneath before the splash fades
}

let finished = false;

export async function finishBoot(): Promise<void> {
  const boot = api();
  if (!boot || finished) return;
  finished = true;
  await Promise.race([warmUp(boot.step, boot.lang), sleep(MAX_MS)]);
  const elapsed = performance.now() - boot.start;
  if (elapsed < MIN_MS) await sleep(MIN_MS - elapsed);
  boot.done();
}
