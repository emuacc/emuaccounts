import { escapeText } from "./text";

/** Mirror of the Rust plugins::AvailablePluginInfo payload. */
export interface AvailablePluginInfo {
  dir_name: string;
  name: string;
  version: string;
  description: string | null;
  daily: boolean;
  installed: boolean;
}

/** The "Add Plugins" modal: the app's bundled plugin catalog with per-plugin
 * install state. `installingDirName` disables that row's button mid-install. */
export function buildAddPluginsModalHtml(
  plugins: AvailablePluginInfo[],
  installingDirName: string | undefined,
): string {
  const rows = plugins.length
    ? plugins
        .map((p) => {
          const action = p.installed
            ? `<span class="pluginItem__status">Installed</span>`
            : `<button class="btn btn--secondary" type="button" data-install-plugin="${escapeText(p.dir_name)}" ${installingDirName === p.dir_name ? "disabled" : ""}>${installingDirName === p.dir_name ? "Installing…" : "Install"}</button>`;
          return `
          <div class="pluginItem" data-catalog-plugin="${escapeText(p.dir_name)}">
            <div class="pluginItem__head">
              <span class="pluginItem__name">${escapeText(p.name)}</span>
              ${p.daily ? `<span class="pluginItem__badge">Daily</span>` : ""}
              ${action}
            </div>
            ${p.description ? `<div class="pluginItem__desc">${escapeText(p.description)}</div>` : ""}
            <div class="pluginItem__meta">v${escapeText(p.version)}</div>
          </div>`;
        })
        .join("")
    : `<div class="pluginsList__empty">No plugins available in the catalog.</div>`;
  return `
    <div class="modalOverlay" role="dialog" aria-modal="true">
      <div class="modal modal--wide">
        <h2 class="modal__title">Add Plugins</h2>
        <div class="modal__body">
          <div class="pluginsList" data-testid="available-plugins-list">${rows}</div>
        </div>
        <div class="modal__actions">
          <button id="addPluginsClose" class="btn btn--secondary" type="button">Close</button>
        </div>
      </div>
    </div>
  `;
}
