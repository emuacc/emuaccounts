// Central app branding — the single frontend source for the product name.
//
// Override at build time (no code change) via a Vite env var:
//   VITE_APP_NAME    product name (window / browser tab, sidebar, vault picker)  default "EmuAcc"
//
// The native shell carries the same name in static files that a runtime
// variable can't reach — keep these in sync when renaming:
//   - src-tauri/tauri.conf.json    (productName, window title)
//   - index.html                   (<title>, apple-mobile-web-app-title)
//   - public/manifest.webmanifest  (name, short_name)

const env = import.meta.env as Record<string, string | undefined>;

/** Product name — window/tab title, sidebar and vault picker. */
export const APP_NAME = env.VITE_APP_NAME ?? "EmuAcc";
