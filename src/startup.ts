/**
 * Startup orchestration — extracted for testability.
 *
 * The key invariant is that independent backend calls run in parallel.
 * The test suite verifies this by counting sequential "invoke rounds":
 *
 *   Round 1: list_account_sets
 *   Round 2: load_account_tree ‖ load_pipeline_metadata ‖ get_display_config
 *            ‖ get_trade_links ‖ get_account_gaps
 *   Round 3: get_pipeline_warnings ‖ get_source_folder_path×N
 *
 * Startup deliberately does NOT invoke `rebuild_pipeline`. The backend already
 * rebuilds the vault when it is opened (a cached no-op when sources are
 * unchanged), and main.ts waits for that behind the "Rebuilding cache" screen
 * (vault-rebuild.ts) before calling this. The fat `PipelineResponse` ships
 * the entire active ledger (~40MB JSON on a large vault) whose main-thread
 * `JSON.parse` stalls the WebView — the same reason mutations use
 * `build_mutation_response`. Startup
 * takes the same slim path: `load_account_tree` for sidebar balances,
 * `load_pipeline_metadata` for the account maps, `get_pipeline_warnings` for
 * warnings the launch rebuild stashed. The caller (main.ts) loads the visible
 * transaction window + value column via `query_search` right after.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

import type { AccountFolders } from "./types";

export interface StartupDeps {
  invoke: <T = unknown>(cmd: string, args?: Record<string, any>) => Promise<T>;
  listen: <T>(event: string, handler: (e: { payload: T }) => void) => Promise<unknown>;
  render: () => void;
  /** Timeline sink for `[startup] <phase> +<ms>` marks (pushDebug in the app). */
  log: (message: string) => void;
  onPipelineEvent: (payload: any) => void;
}

export interface StartupState {
  accountSets: string[];
  selectedAccountSet?: string;
  displayConfig?: { base_currency?: string; default_account?: string; commodities?: Record<string, any>; default_decimals?: number };
  busy: boolean;
  status?: string;
  parse?: {
    ok: boolean;
    transactions: Array<{
      date: string;
      datetime: string;
      meta?: string | null;
      amount: number;
      narration?: string | null;
      postings: Array<{ account: string; commodity: string; amount: number }>;
    }>;
    balances: Array<{ account: string; totals: Array<{ commodity: string; amount: number }> }>;
    accounts_with_opening: string[];
    account_properties: Record<string, any>;
    diagnostics: any[];
  };
  selectedAccount?: string;
  drillPath: string[];
  pipelineWarnings: string[];
  accountSetMap: Record<string, string[]>;
  accountFoldersMap: AccountFolders;
  accountPropertiesMap: Record<string, any>;
  accountsWithOpening: string[];
  tradeLinks: any[];
  tradeSuggestions: any[];
  accountGaps?: any[];
  sourceFolderPaths?: Record<string, string>;
}

// ---------------------------------------------------------------------------
// Main startup function
// ---------------------------------------------------------------------------

export async function normalStartup(state: StartupState, deps: StartupDeps): Promise<void> {
  const t0 = performance.now();
  const mark = (label: string) => deps.log(`[startup] ${label} +${(performance.now() - t0).toFixed(0)}ms`);

  // ── Round 1 ── list_account_sets (everything depends on selectedAccountSet)
  try {
    const sets = await deps.invoke<string[]>("list_account_sets");
    state.accountSets = sets;
    // A pre-set selection comes from the vault's persisted ui-state (applied
    // before startup); fall back to the first set when it's absent or no
    // longer exists in this vault.
    if (sets.length > 0 && (!state.selectedAccountSet || !sets.includes(state.selectedAccountSet))) {
      state.selectedAccountSet = sets[0];
    }
    mark(`account sets listed (${sets.length})`);
  } catch { /* not available in non-Tauri contexts */ }

  const accountSet = state.selectedAccountSet ?? "";

  // ── Round 2 ── independent operations, all started concurrently
  const displayConfigP = (async () => {
    try {
      state.displayConfig = await deps.invoke<any>("get_display_config", { accountSet });
    } catch { /* fallback to defaults */ }
  })();

  const treeP = (async () => {
    try {
      const balances = await deps.invoke<
        Array<{ account: string; totals: Array<{ commodity: string; amount: number }> }>
      >("load_account_tree", { accountSet });
      // The default-account fallback below reads displayConfig; wait for it so
      // the initial selection doesn't race the config fetch.
      await displayConfigP;
      applyParse(state, {
        ok: true,
        diagnostics: [],
        transactions: [],
        balances,
        accounts_with_opening: [],
        account_properties: {},
      });
      // Explicit status: the render fallback would otherwise report the slim
      // parse's empty transaction list as "Parsed (0 transactions)".
      state.status = `Loaded ${balances.length} account${balances.length === 1 ? "" : "s"}`;
      mark(`account tree loaded (${balances.length} accounts)`);
    } catch (err) {
      state.status = `Error: ${String(err)}`;
    } finally {
      deps.render();
    }
  })();

  const metadataMapsP = (async () => {
    try {
      const meta = await deps.invoke<{
        owner_accounts?: Record<string, string[]>;
        account_folders?: AccountFolders;
        account_properties?: Record<string, any>;
        accounts_with_opening?: string[];
      }>("load_pipeline_metadata");
      if (meta.owner_accounts) state.accountSetMap = meta.owner_accounts;
      if (meta.account_folders) state.accountFoldersMap = meta.account_folders;
      if (meta.account_properties) {
        state.accountPropertiesMap = { ...state.accountPropertiesMap, ...meta.account_properties };
      }
      if (meta.accounts_with_opening) state.accountsWithOpening = meta.accounts_with_opening;
      mark("pipeline metadata loaded");
    } catch { /* brand-new vault — no pipeline-metadata.json yet */ }
  })();

  const tradeLinksP = (async () => {
    state.tradeLinks = await deps.invoke<any[]>("get_trade_links").catch(() => []);
    mark("trade links loaded");
  })();

  const gapsP = (async () => {
    try {
      state.accountGaps = await deps.invoke<any[]>("get_account_gaps");
    } catch { /* gaps not available */ }
  })();

  // listen registration is fast (just sets up a callback)
  deps.listen("pipeline-rebuilt", (event: any) => {
    deps.onPipelineEvent(event.payload);
  }).catch(() => {});

  await Promise.allSettled([treeP, metadataMapsP, displayConfigP, tradeLinksP, gapsP]);
  mark("round 2 settled");
  deps.render();

  // ── Round 3 ── operations that depend on round 2 results
  const warningsP = (async () => {
    try {
      const w = await deps.invoke<string[]>("get_pipeline_warnings");
      if (w.length > 0) state.pipelineWarnings = w;
      mark("pipeline warnings loaded");
    } catch { /* warnings not available */ }
  })();

  // Trade suggestions are deliberately NOT fetched here: the full-vault scan
  // measured 9.3s on the real vault and was blocking the visible window from
  // loading. The caller (main.ts) fetches them via refreshHeavy after the
  // first data render, same as the post-mutation path.

  const foldersP = (async () => {
    const uniqueFolders = [
      ...new Set(Object.values(state.accountFoldersMap).flatMap((bySet) => Object.values(bySet))),
    ];
    if (uniqueFolders.length === 0) return;
    const paths: Record<string, string> = {};
    await Promise.all(uniqueFolders.map(async (folder) => {
      try {
        paths[folder] = await deps.invoke<string>("get_source_folder_path", { folderName: folder });
      } catch { /* folder not found */ }
    }));
    state.sourceFolderPaths = paths;
    mark("source folder paths loaded");
  })();

  // The issues cache (collect_issues_cmd) is deliberately NOT refreshed here:
  // it measured 9.5s on the real vault — like trade suggestions, it runs
  // post-paint from the caller (main.ts).

  await Promise.allSettled([warningsP, foldersP]);
  mark("round 3 settled");
  deps.render();
}

// ---------------------------------------------------------------------------
// State helpers
// ---------------------------------------------------------------------------

function applyParse(state: StartupState, response: NonNullable<StartupState["parse"]>): void {
  state.parse = response;
  if (response.account_properties) {
    state.accountPropertiesMap = { ...state.accountPropertiesMap, ...response.account_properties };
  }
  const nextBalances = response.balances ?? [];
  if (!state.selectedAccount) {
    const matchesPfx = (acct: string, pfx: string) => acct === pfx || acct.startsWith(pfx + ":");
    const defaultAccount = state.displayConfig?.default_account;
    state.selectedAccount =
      (defaultAccount && nextBalances.find((b) => matchesPfx(b.account, defaultAccount))?.account)
        ?? nextBalances[0]?.account;
  }
}
