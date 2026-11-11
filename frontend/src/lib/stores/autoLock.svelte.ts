// Auto-lock: locks the vault after a period without keyboard/mouse activity, and right away
// after the machine wakes from sleep. Remote terminal output is NOT activity — a busy `tail -f`
// on an unattended laptop must not keep the vault open.
//
// The timeout is a per-device preference, so localStorage is enough; storage failures just mean
// the default is used.

const STORAGE_KEY = 'caterm.autoLockMinutes';
/** Allowed choices in Settings. 0 = never. */
export const AUTO_LOCK_CHOICES = [0, 1, 5, 15, 30, 60] as const;
export const DEFAULT_AUTO_LOCK_MINUTES = 0;

function load(): number {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === null) return DEFAULT_AUTO_LOCK_MINUTES;
    const n = Number(raw);
    return (AUTO_LOCK_CHOICES as readonly number[]).includes(n) ? n : DEFAULT_AUTO_LOCK_MINUTES;
  } catch {
    return DEFAULT_AUTO_LOCK_MINUTES;
  }
}

let minutes = $state(typeof localStorage === 'undefined' ? DEFAULT_AUTO_LOCK_MINUTES : load());

export function getAutoLockMinutes(): number {
  return minutes;
}

export function setAutoLockMinutes(value: number) {
  minutes = (AUTO_LOCK_CHOICES as readonly number[]).includes(value) ? value : DEFAULT_AUTO_LOCK_MINUTES;
  try {
    localStorage.setItem(STORAGE_KEY, String(minutes));
  } catch {
    // not remembered; still applies for this run
  }
}

const TICK_MS = 15_000;
/** A gap this much larger than one tick means the timers were frozen: the machine slept. */
const SLEEP_GAP_MS = 2 * 60_000;
const ACTIVITY_EVENTS = ['keydown', 'pointerdown', 'pointermove', 'wheel', 'touchstart'] as const;

/**
 * Starts watching for inactivity. `onLock` is called (once per idle period) when the timeout
 * passes or a sleep/wake is detected (both off when the timeout is "never"), but only while `isUnlocked()` is true. Returns a stop
 * function for cleanup.
 */
export function startAutoLock(isUnlocked: () => boolean, onLock: () => void): () => void {
  let lastActivity = Date.now();
  let lastTick = Date.now();
  let lastMove = 0;

  const markActive = (e: Event) => {
    // pointermove fires constantly; sampling it once a second is plenty.
    if (e.type === 'pointermove') {
      const now = Date.now();
      if (now - lastMove < 1000) return;
      lastMove = now;
    }
    lastActivity = Date.now();
  };
  for (const ev of ACTIVITY_EVENTS) window.addEventListener(ev, markActive, { passive: true, capture: true });

  const timer = setInterval(() => {
    const now = Date.now();
    const sleptAndWoke = now - lastTick > SLEEP_GAP_MS;
    lastTick = now;
    if (!isUnlocked()) {
      lastActivity = now;
      return;
    }
    const idleLimit = minutes * 60_000;
    if (idleLimit > 0 && (sleptAndWoke || now - lastActivity >= idleLimit)) {
      lastActivity = now;
      onLock();
    }
  }, TICK_MS);

  return () => {
    clearInterval(timer);
    for (const ev of ACTIVITY_EVENTS) window.removeEventListener(ev, markActive, { capture: true });
  };
}
