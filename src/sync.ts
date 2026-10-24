/**
 * Multi-device sync UI (Automerge relay sync): the sidebar "Sync" section
 * (Pair Device / Sync Now / View Sync Log / Devices), the Sync Log modal and
 * the Pair Device modal.
 *
 * Compiled only into the sync build (VITE_SYNC=1, see docs/SYNC.md): main.ts
 * loads this module through a dynamic import behind that flag, so the default
 * bundle has none of it. It keeps its own state; the app lends it only its
 * busy flag, its status line and a re-render (SyncHost).
 */
import "./sync.css";
import { invoke } from "./ipc";
import { escapeText } from "./text";

type SyncEvent = {
  timestamp: number;
  device_id: string;
  event_type: string;
  target_id: string;
  details: string;
};

type DeviceInfo = {
  device_id: string;
  device_name: string;
  last_seen: number;
};

type PairInitiateResult = {
  group_id: string;
  pairing_code: string;
  expires_in: number;
};

type RelayConfig = {
  relay_url: string;
  group_id: string;
};

type RelaySyncResponse = {
  metadata_merged: boolean;
  blobs_uploaded: number;
  blobs_downloaded: number;
};

/** What the app lends the sync UI: its busy flag and status line, and a re-render. */
export interface SyncHost {
  app: { busy: boolean; status?: string };
  render(): void;
}

export interface SyncUi {
  /** Startup, after the core startup finished: invoke("init_sync"), then get_relay_config, list_devices, get_sync_log. Failures leave the section empty, as today. */
  load(): Promise<void>;
  /** Vault switch: drop all sync state (what resetVaultScopedState used to clear). */
  reset(): void;
  /** The Sync `<details>` accordion at the bottom of the sidebar. */
  renderSidebar(): string;
  /** The Sync Log modal and the Pair Device modal (each empty while closed). */
  renderModals(): string;
  /** Bind every sync button; called on each render, inside the app's bindOnceDuring window. */
  attachHandlers(): void;
  /** Close the sync modals (the app's click-outside-to-dismiss). */
  closeModals(): void;
}

type SyncState = {
  logOpen?: boolean;
  log?: SyncEvent[];
  devices?: DeviceInfo[];
  relayConfig?: RelayConfig;
  pairingOpen?: boolean;
  pairingMode?: "create" | "join";
  pairingCode?: string;
  pairingError?: string;
  /** The relay + group a "Create Code" pairing saves once the user clicks Done. */
  pendingRelayConfig?: RelayConfig;
  /** The relay URL entered before "Enter Code", which the Join step pairs against. */
  joinRelayUrl?: string;
};

/** Bind a click handler to the element with this id, when it is rendered. */
function onClick(id: string, handler: () => void | Promise<void>): void {
  document.getElementById(id)?.addEventListener("click", () => void handler());
}

function inputValue(id: string): string {
  return document.querySelector<HTMLInputElement>(`#${id}`)?.value?.trim() ?? "";
}

export function createSyncUi(host: SyncHost): SyncUi {
  const { app } = host;
  let s: SyncState = {};

  async function load(): Promise<void> {
    // A vault switch (reset) replaces `s`, so a load still in flight for the
    // old vault fills the dropped state, never the new vault's.
    const target = s;
    try {
      await invoke("init_sync");
    } catch { /* sync metadata not available */ }
    const [relayConfig, devices, log] = await Promise.all([
      invoke<RelayConfig | null>("get_relay_config").catch(() => undefined),
      invoke<DeviceInfo[]>("list_devices").catch(() => []),
      invoke<SyncEvent[]>("get_sync_log").catch(() => []),
    ]);
    if (relayConfig) target.relayConfig = relayConfig;
    target.devices = devices;
    target.log = log;
    host.render();
  }

  function renderSidebar(): string {
    const relayHtml = s.relayConfig
      ? `<div class="relayStatus"><div class="relayStatus__label">Relay</div><div class="relayStatus__url">${escapeText(s.relayConfig.relay_url)}</div><button id="relaySyncBtn" class="btn btn--full" type="button" ${app.busy ? "disabled" : ""}>Sync Now</button></div>`
      : `<button id="relayPairBtn" class="btn btn--secondary btn--full" type="button" style="margin-top:4px" ${app.busy ? "disabled" : ""}>Pair Device</button>`;
    const devicesHtml = s.devices && s.devices.length > 0
      ? `<div class="syncDevices"><div class="syncDevices__title">Devices</div>${s.devices.map((d) => `<div class="syncDevices__item"><span class="syncDevices__name">${escapeText(d.device_name)}</span><span class="syncDevices__id">${escapeText(d.device_id)}</span></div>`).join("")}</div>`
      : "";
    return `<details class="syncAccordion"><summary class="syncAccordion__toggle">Sync</summary><div class="syncAccordion__body">${relayHtml}<button id="syncViewLogBtn" class="btn btn--secondary btn--full" type="button" style="margin-top:4px">View Sync Log</button>${devicesHtml}</div></details>`;
  }

  function renderLogModal(): string {
    if (!s.logOpen) return "";
    return `<div class="modalOverlay" role="dialog" aria-modal="true" data-testid="sync-log-modal">
    <div class="modal modal--wide">
      <div class="modal__title">Sync Log</div>
      ${s.devices && s.devices.length > 0 ? `<div class="syncLogDevices"><strong>Known devices:</strong> ${s.devices.map((d) => `<span class="syncLogDevice">${escapeText(d.device_name)} (${escapeText(d.device_id)}) — last seen ${new Date(d.last_seen * 1000).toLocaleString()}</span>`).join(", ")}</div>` : ""}
      ${s.log && s.log.length > 0 ? `<table class="syncLogTable"><thead><tr><th>Time</th><th>Device</th><th>Event</th><th>Details</th></tr></thead><tbody>${s.log.slice().reverse().map((e) => `<tr><td>${escapeText(new Date(e.timestamp * 1000).toLocaleString())}</td><td>${escapeText(e.device_id)}</td><td><span class="syncLogEvent__type">${escapeText(e.event_type)}</span></td><td>${escapeText(e.details)}</td></tr>`).join("")}</tbody></table>` : `<div class="syncLogEmpty">No sync events yet.</div>`}
      <div class="modal__actions"><button id="syncLogExport" class="btn btn--secondary">Export JSON</button><button id="syncLogClose" class="btn">Close</button></div>
    </div>
  </div>`;
  }

  function renderPairingBody(): string {
    const d = app.busy ? "disabled" : "";
    const errorHtml = s.pairingError ? `<div class="modal__error">${escapeText(s.pairingError)}</div>` : "";
    if (!s.pairingMode) {
      return `<label class="field"><div class="field__label">Relay Server URL</div><input id="relayUrlInput" class="field__input" value="" placeholder="http://your-server:8384" ${d} /></label>
      <div class="modal__actions" style="margin-top:12px"><button id="relayPairCancel" class="btn btn--secondary" ${d}>Cancel</button><button id="relayCreateCode" class="btn btn--secondary" ${d}>Create Code</button><button id="relayEnterCode" class="btn" ${d}>Enter Code</button></div>`;
    }
    if (s.pairingMode === "create") {
      return `<div class="pairingCode"><div class="pairingCode__label">Share this code with the other device:</div><div class="pairingCode__value">${escapeText(s.pairingCode ?? "")}</div></div>
      ${errorHtml}
      <div class="modal__actions"><button id="relayPairDone" class="btn">Done</button></div>`;
    }
    return `<label class="field"><div class="field__label">Enter the 6-digit pairing code</div><input id="relayJoinCodeInput" class="field__input" maxlength="6" placeholder="000000" ${d} /></label>
      ${errorHtml}
      <div class="modal__actions"><button id="relayPairCancel2" class="btn btn--secondary" ${d}>Cancel</button><button id="relayJoinSubmit" class="btn" ${d}>Join</button></div>`;
  }

  function renderPairingModal(): string {
    if (!s.pairingOpen) return "";
    return `<div class="modalOverlay" role="dialog" aria-modal="true" data-testid="relay-pairing-modal"><div class="modal"><div class="modal__title">Pair Device</div>${renderPairingBody()}</div></div>`;
  }

  // ── Sync Log ──

  async function openLog(): Promise<void> {
    try {
      const log: SyncEvent[] = await invoke("get_sync_log");
      const devices: DeviceInfo[] = await invoke("list_devices");
      s.log = log;
      s.devices = devices;
    } catch {
      // May fail if metadata not initialized, show empty
    }
    s.logOpen = true;
    host.render();
  }

  function closeLog(): void {
    s.logOpen = false;
    host.render();
  }

  function exportLog(): void {
    const data = JSON.stringify({ syncLog: s.log ?? [], devices: s.devices ?? [] }, null, 2);
    const blob = new Blob([data], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `sync-log-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  // ── Relay: sync and pairing ──

  async function syncNow(): Promise<void> {
    app.busy = true;
    app.status = "Syncing with relay...";
    host.render();
    try {
      const result = await invoke<RelaySyncResponse>("sync_with_relay_cmd");
      const devices: DeviceInfo[] = await invoke("list_devices");
      const log: SyncEvent[] = await invoke("get_sync_log");
      s.devices = devices;
      s.log = log;
      app.status = `Synced: ${result.blobs_uploaded} up, ${result.blobs_downloaded} down`;
    } catch (err) {
      app.status = `Sync failed: ${String(err)}`;
    }
    app.busy = false;
    host.render();
  }

  function openPairing(): void {
    s.pairingOpen = true;
    s.pairingMode = undefined;
    s.pairingCode = undefined;
    s.pairingError = undefined;
    host.render();
  }

  function cancelPairing(): void {
    s.pairingOpen = false;
    s.pendingRelayConfig = undefined;
    host.render();
  }

  function cancelJoin(): void {
    s.pairingOpen = false;
    s.pairingMode = undefined;
    s.pendingRelayConfig = undefined;
    host.render();
  }

  /** The relay URL from the first pairing step, or undefined after flagging it missing. */
  function requireRelayUrl(): string | undefined {
    const relayUrl = inputValue("relayUrlInput");
    if (relayUrl) return relayUrl;
    s.pairingError = "Enter a relay URL";
    host.render();
    return undefined;
  }

  async function createCode(): Promise<void> {
    const relayUrl = requireRelayUrl();
    if (!relayUrl) return;
    app.busy = true;
    app.status = "Creating pairing code...";
    host.render();
    try {
      const result = await invoke<PairInitiateResult>("pair_initiate", { relayUrl });
      s.pairingMode = "create";
      s.pairingCode = result.pairing_code;
      s.pendingRelayConfig = { relay_url: relayUrl, group_id: result.group_id };
      app.status = undefined;
    } catch (err) {
      s.pairingError = String(err);
      app.status = `Pairing failed: ${String(err)}`;
    }
    app.busy = false;
    host.render();
  }

  function enterCode(): void {
    const relayUrl = requireRelayUrl();
    if (!relayUrl) return;
    s.pairingMode = "join";
    s.joinRelayUrl = relayUrl;
    host.render();
  }

  async function joinWithCode(): Promise<void> {
    const code = inputValue("relayJoinCodeInput");
    if (!code || code.length !== 6) {
      s.pairingError = "Enter a 6-digit code";
      host.render();
      return;
    }
    const relayUrl = s.joinRelayUrl;
    if (!relayUrl) return; // the Join step only renders after enterCode stored the URL
    app.busy = true;
    app.status = "Joining...";
    host.render();
    try {
      const groupId = await invoke<string>("pair_join", { relayUrl, pairingCode: code });
      await invoke("save_relay_config", { relayUrl, groupId });
      s.relayConfig = { relay_url: relayUrl, group_id: groupId };
      s.pairingOpen = false;
      s.pairingMode = undefined;
      app.status = "Paired successfully";
    } catch (err) {
      s.pairingError = String(err);
      app.status = `Join failed: ${String(err)}`;
    }
    app.busy = false;
    host.render();
  }

  async function finishPairing(): Promise<void> {
    const pending = s.pendingRelayConfig;
    if (pending) {
      try {
        await invoke("save_relay_config", { relayUrl: pending.relay_url, groupId: pending.group_id });
        s.relayConfig = pending;
      } catch (err) {
        app.status = `Failed to save relay config: ${String(err)}`;
      }
      s.pendingRelayConfig = undefined;
    }
    s.pairingOpen = false;
    s.pairingMode = undefined;
    host.render();
  }

  return {
    load,
    reset() {
      s = {};
    },
    renderSidebar,
    renderModals: () => renderLogModal() + renderPairingModal(),
    attachHandlers() {
      onClick("syncViewLogBtn", openLog);
      onClick("syncLogClose", closeLog);
      onClick("syncLogExport", exportLog);
      onClick("relayPairBtn", openPairing);
      onClick("relaySyncBtn", syncNow);
      onClick("relayPairCancel", cancelPairing);
      onClick("relayCreateCode", createCode);
      onClick("relayEnterCode", enterCode);
      onClick("relayPairCancel2", cancelJoin);
      onClick("relayJoinSubmit", joinWithCode);
      onClick("relayPairDone", finishPairing);
    },
    closeModals() {
      s.logOpen = false;
      s.pairingOpen = false;
    },
  };
}
