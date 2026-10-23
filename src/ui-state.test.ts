import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createUiStatePersister,
  loadPersistedUiState,
  resetVaultScopedState,
  UI_STATE_VERSION,
  type PersistedUiState,
} from "./ui-state";

const snap = (account: string): PersistedUiState => ({
  version: UI_STATE_VERSION,
  nav: { sidebarView: "accounts", selectedAccount: account },
});

describe("createUiStatePersister", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("debounces: three schedules within the interval yield one save with the last snapshot", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const p = createUiStatePersister(invoke, 300);
    p.schedule(snap("a"));
    p.schedule(snap("b"));
    p.schedule(snap("c"));
    await vi.advanceTimersByTimeAsync(299);
    expect(invoke).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("save_ui_state", { state: snap("c") });
  });

  it("flush writes the pending snapshot immediately", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const p = createUiStatePersister(invoke, 300);
    p.schedule(snap("a"));
    await p.flush();
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("save_ui_state", { state: snap("a") });
    // The cleared timer must not fire a second save later.
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("flush with nothing pending does not invoke", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const p = createUiStatePersister(invoke, 300);
    await p.flush();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("reset discards a pending snapshot", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const p = createUiStatePersister(invoke, 300);
    p.schedule(snap("old-vault"));
    p.reset();
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("a snapshot scheduled before reset never lands after it (epoch guard)", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const p = createUiStatePersister(invoke, 300);
    p.schedule(snap("old-vault"));
    p.reset();
    p.schedule(snap("new-vault"));
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("save_ui_state", { state: snap("new-vault") });
  });

  it("swallows invoke rejections from schedule and flush", async () => {
    const invoke = vi.fn().mockRejectedValue(new Error("No vault selected"));
    const p = createUiStatePersister(invoke, 300);
    p.schedule(snap("a"));
    await vi.advanceTimersByTimeAsync(300);
    p.schedule(snap("b"));
    await expect(p.flush()).resolves.toBeUndefined();
    expect(invoke).toHaveBeenCalledTimes(2);
  });
});

describe("loadPersistedUiState", () => {
  it.each([null, undefined, 42, "str"])("returns undefined for non-object %p", async (raw) => {
    const invoke = vi.fn().mockResolvedValue(raw);
    expect(await loadPersistedUiState(invoke)).toBeUndefined();
  });

  it("returns undefined for an unknown version", async () => {
    const invoke = vi.fn().mockResolvedValue({ version: UI_STATE_VERSION + 1 });
    expect(await loadPersistedUiState(invoke)).toBeUndefined();
  });

  it("returns undefined when the backend errors (no vault selected)", async () => {
    const invoke = vi.fn().mockRejectedValue(new Error("No vault selected"));
    expect(await loadPersistedUiState(invoke)).toBeUndefined();
  });

  it("round-trips a valid state and strips selectedReportYear", async () => {
    const invoke = vi.fn().mockResolvedValue({
      version: UI_STATE_VERSION,
      nav: { sidebarView: "reports", selectedReport: "cgt", selectedReportYear: 2024 },
      selectedAccountSet: "alex",
    });
    const loaded = await loadPersistedUiState(invoke);
    expect(loaded?.nav?.selectedReport).toBe("cgt");
    expect(loaded?.nav?.selectedReportYear).toBeUndefined();
    expect(loaded?.selectedAccountSet).toBe("alex");
  });
});

describe("resetVaultScopedState", () => {
  // The direct regression for "stale ibkr account survives into another vault".
  it("clears vault-derived fields", () => {
    const state: Record<string, unknown> = {
      parse: { ok: true },
      selectedAccount: "assets:equity:broker:ibkr",
      drillPath: ["equity", "broker", "ibkr"],
      _drillInitialized: true,
      selectedCategory: "expenses",
      categoryDrillPath: ["expenses"],
      accountSets: ["alex"],
      selectedAccountSet: "alex",
      accountSetMap: { alex: ["assets:x"] },
      allAccounts: ["assets:equity:broker:ibkr"],
      accountFoldersMap: { alex: { "assets:x": "x" } },
      accountPropertiesMap: { "assets:x": {} },
      accountsWithOpening: ["assets:x"],
      displayConfig: { base_currency: "AUD" },
      search: "ibkr",
      searchPills: [{ key: "account", value: "ibkr" }],
      searchText: "ibkr",
      searchFilteredTransactions: [{}],
      prefixQuery: {},
      treeBaseTotals: new Map(),
      categoryBaseTotals: new Map(),
      transactionValues: new Map(),
      accountTotalValue: { total: 1, currency: "AUD", bycommodity: [] },
      metadataReady: true,
      devices: [{}],
      syncLog: [{}],
      relayConfig: {},
      tradeLinks: [{}],
      tradeSuggestions: [{}],
      tradeLinkSelection: "t1",
      sidebarView: "reports",
      selectedReport: "cgt",
      selectedReportYear: 2024,
      cgtReport: {},
      incomeReport: {},
      balancesReport: {},
      performanceReport: {},
      lossHarvestReport: {},
      taxConfig: {},
      cgtSort: { column: "sell_date", direction: "desc" },
      pendingMutations: 3,
      pendingDeletes: new Set(["x"]),
      pendingAdds: [{}],
      pipelineWarnings: ["w"],
      status: "Loaded",
      manualDraft: {},
      addAccountDraft: {},
      transformDraft: {},
      docImportDraft: {},
      openingBalanceDraft: {},
      setPriceDraft: {},
      ruleEditorDraft: {},
      aiSuggest: {},
      pluginsList: [{}],
      pluginLog: "log",
      pluginReadmeOpen: "ibkr-flex-sync",
      pluginReadmes: { "ibkr-flex-sync": "# Setup" },
      dailySummary: {},
      issueFilter: "ibkr",
      txExpandedGroups: new Set(["g"]),
      accountGaps: [{}],
      sourceFolderPaths: { x: "/x" },
      txWindowStart: 100,
      reportPageSize: 200,
    };

    resetVaultScopedState(state);

    expect(state.parse).toBeUndefined();
    expect(state.selectedAccount).toBeUndefined();
    expect(state.drillPath).toEqual([]);
    expect(state._drillInitialized).toBe(false);
    expect(state.selectedCategory).toBeUndefined();
    expect(state.categoryDrillPath).toEqual([]);
    expect(state.accountSets).toEqual([]);
    expect(state.selectedAccountSet).toBeUndefined();
    expect(state.accountSetMap).toEqual({});
    expect(state.allAccounts).toEqual([]);
    expect(state.accountFoldersMap).toEqual({});
    expect(state.accountsWithOpening).toEqual([]);
    expect(state.displayConfig).toBeUndefined();
    expect(state.search).toBe("");
    expect(state.searchPills).toEqual([]);
    expect(state.searchText).toBe("");
    expect(state.searchFilteredTransactions).toBeUndefined();
    expect(state.prefixQuery).toBeUndefined();
    expect(state.treeBaseTotals).toBeUndefined();
    expect(state.categoryBaseTotals).toBeUndefined();
    expect(state.transactionValues).toBeUndefined();
    expect(state.accountTotalValue).toBeUndefined();
    expect(state.metadataReady).toBeUndefined();
    expect(state.pluginReadmeOpen).toBeUndefined();
    expect(state.pluginReadmes).toBeUndefined();
    expect(state.devices).toBeUndefined();
    expect(state.syncLog).toBeUndefined();
    expect(state.relayConfig).toBeUndefined();
    expect(state.tradeLinks).toEqual([]);
    expect(state.tradeSuggestions).toEqual([]);
    expect(state.tradeLinkSelection).toBeUndefined();
    expect(state.sidebarView).toBe("accounts");
    expect(state.selectedReport).toBeUndefined();
    expect(state.selectedReportYear).toBeUndefined();
    expect(state.cgtReport).toBeUndefined();
    expect(state.incomeReport).toBeUndefined();
    expect(state.balancesReport).toBeUndefined();
    expect(state.performanceReport).toBeUndefined();
    expect(state.lossHarvestReport).toBeUndefined();
    expect(state.taxConfig).toBeUndefined();
    expect(state.cgtSort).toBeUndefined();
    expect(state.txSort).toEqual({ column: "date", direction: "asc" });
    expect(state.pendingMutations).toBe(0);
    expect(state.pendingDeletes).toEqual(new Set());
    expect(state.pendingAdds).toEqual([]);
    expect(state.pipelineWarnings).toEqual([]);
    expect(state.status).toBeUndefined();
    expect(state.manualDraft).toBeUndefined();
    expect(state.addAccountDraft).toBeUndefined();
    expect(state.transformDraft).toBeUndefined();
    expect(state.docImportDraft).toBeUndefined();
    expect(state.openingBalanceDraft).toBeUndefined();
    expect(state.setPriceDraft).toBeUndefined();
    expect(state.ruleEditorDraft).toBeUndefined();
    expect(state.aiSuggest).toBeUndefined();
    expect(state.pluginsList).toBeUndefined();
    expect(state.pluginLog).toBeUndefined();
    expect(state.dailySummary).toBeUndefined();
    expect(state.issueFilter).toBeUndefined();
    expect(state.txExpandedGroups).toBeUndefined();
    expect(state.accountGaps).toBeUndefined();
    expect(state.sourceFolderPaths).toBeUndefined();
    expect(state.txWindowStart).toBe(0);
    expect(state.reportPageSize).toBe(50);
  });

  it("preserves app-scoped fields", () => {
    const refreshView = async () => {};
    const state: Record<string, unknown> = {
      busy: true,
      rootDir: "/vault/new",
      knownRoots: ["/vault/old", "/vault/new"],
      showVaultPicker: false,
      showHidden: true,
      updatePricesOnStartup: true,
      extraPrimaryAccountPrefixes: ["loans:"],
      globalSettingsOpen: true,
      globalSettingsDraft: ["loans:"],
      aiSettingsDraft: { apiUrl: "http://localhost:1234", apiKey: "", model: "m" },
      refreshView,
    };

    resetVaultScopedState(state);

    expect(state.busy).toBe(true);
    expect(state.rootDir).toBe("/vault/new");
    expect(state.knownRoots).toEqual(["/vault/old", "/vault/new"]);
    expect(state.showVaultPicker).toBe(false);
    expect(state.showHidden).toBe(true);
    expect(state.updatePricesOnStartup).toBe(true);
    expect(state.extraPrimaryAccountPrefixes).toEqual(["loans:"]);
    expect(state.globalSettingsOpen).toBe(true);
    expect(state.globalSettingsDraft).toEqual(["loans:"]);
    expect(state.aiSettingsDraft).toEqual({ apiUrl: "http://localhost:1234", apiKey: "", model: "m" });
    expect(state.refreshView).toBe(refreshView);
  });
});
