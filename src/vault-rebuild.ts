/** "Rebuilding cache" gate. Opening a data folder — at launch, or switching to
 * another one — brings its generated output up to date on a background thread
 * (`spawn_vault_rebuild`). A large vault's full rebuild takes minutes, so the
 * window shows this screen instead of the main page until it ends. Pure
 * helpers here; Tauri wiring lives in main.ts. */

import { escapeText } from "./render";
import type { StartupDeps } from "./startup";

export const VAULT_REBUILDING_TESTID = "vault-rebuilding";

/** Hold the window on the rebuilding screen while the backend rebuilds the
 * vault: `state.rebuildingCache` is set, and rendered, only while a rebuild is
 * in flight. Non-Tauri contexts have no backend rebuild and pass straight
 * through. */
export async function waitForVaultRebuild(
  state: { rebuildingCache?: boolean },
  deps: Pick<StartupDeps, "invoke" | "render">,
): Promise<void> {
  try {
    if (!(await deps.invoke<boolean>("vault_rebuild_running"))) return;
    state.rebuildingCache = true;
    deps.render();
    await deps.invoke("await_vault_rebuild");
  } catch {
    /* not available in non-Tauri contexts */
  } finally {
    state.rebuildingCache = false;
  }
}

/** Full-screen card in the vault-picker style. */
export function renderVaultRebuilding(appName: string): string {
  return `<div class="vaultPicker" data-testid="${VAULT_REBUILDING_TESTID}" role="status" aria-live="polite">
        <div class="vaultPicker__card">
          <div class="vaultPicker__brand">${escapeText(appName)}</div>
          <h1 class="vaultPicker__title">Rebuilding cache</h1>
          <p class="vaultPicker__desc">This can take a while for large ledgers.</p>
        </div>
      </div>`;
}
