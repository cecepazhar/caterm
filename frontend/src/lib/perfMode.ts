// Rendering-cost switches, applied as classes on <html> so plain CSS (app.css) can react
// without every component having to know about them.
//
//   ca-paused  — the window is in the background (unfocused, minimized or hidden). Decorative
//                animations (ambient glow, border beam) stop: nobody is looking, and a looping
//                full-panel blur keeps the renderer busy and makes the foreground app lag.
//   ca-no-gpu  — GPU acceleration is off (Settings → Performance, the default). Every frame is
//                composited in software, where animated `filter: blur()` and `backdrop-filter`
//                are the most expensive things on screen. Glow stays visible but static, and
//                frosted-glass surfaces fall back to their plain translucent background.
//
// The GPU flag is read once: it is a Chromium switch fixed at WebView creation, so the value at
// startup is exactly what the renderer is running with until the next launch.

import { getPerformancePrefs } from '$lib/api/performance';

export function startPerfMode(): () => void {
  const root = document.documentElement;

  const syncPaused = () => {
    root.classList.toggle('ca-paused', document.hidden || !document.hasFocus());
  };

  void getPerformancePrefs()
    .then((prefs) => root.classList.toggle('ca-no-gpu', !prefs.gpuAcceleration))
    .catch(() => {
      // No backend (vite dev in a plain browser): leave rendering as is.
    });

  syncPaused();
  window.addEventListener('focus', syncPaused);
  window.addEventListener('blur', syncPaused);
  document.addEventListener('visibilitychange', syncPaused);

  return () => {
    window.removeEventListener('focus', syncPaused);
    window.removeEventListener('blur', syncPaused);
    document.removeEventListener('visibilitychange', syncPaused);
    root.classList.remove('ca-paused');
  };
}
