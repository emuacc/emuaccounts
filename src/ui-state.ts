/**
 * Per-vault UI state persistence.
 *
 * Nav state and the selected account set live in the VAULT at
 * `generated/.cache/ui-state.json` (via the load_ui_state/save_ui_state
 * commands) — never in localStorage. WebView storage sits in the real
 * `~/Library/WebKit/<app>/`, shared by every instance and vault regardless of
 * $HOME, so persisting there leaks one vault's state into another (the
 * "fresh instance shows the old vault's account" bug).
 *
 * Saves are debounced and best-effort: with no vault selected the backend
 * errors, and the read-only PWA server has no save route — both are swallowed,
 * matching the old try{localStorage} behavior.
 */
import type { SearchPill } from "./smart-search";
import type {
  BalancesSortColumn,
  CgtSortColumn,
  GenericSortState,
  IncomeSortColumn,
  SortState,
} from "./types";

export type NavEntry = {
  sidebarView: "accounts" | "categories" | "reports" | "rule-editor" | "label-editor" | "plugins";
  selectedAccount?: string;
  drillPath?: string[];
  selectedCategory?: string;
  categoryDrillPath?: string[];
  selectedReport?: "cgt" | "income" | "balances" | "performance" | "loss_harvest";
  selectedReportYear?: number;
  search?: string;
  searchPills?: SearchPill[];
  searchText?: string;
  txSort?: SortState;
  cgtSort?: GenericSortState<CgtSortColumn>;
  cgtFilterText?: string;
  cgtExpandedGroups?: Set<string>;
  incomeSort?: GenericSortState<IncomeSortColumn>;
  incomeFilterText?: string;
  incomeExpandedGroups?: Set<string>;
  balancesSort?: GenericSortState<BalancesSortColumn>;
  balancesFilterText?: string;
  balancesExpanded?: Set<string>;
  lossHarvestExpanded?: Set<string>;
  lossHarvestView?: "position" | "parcel";
};

export const UI_STATE_VERSION = 1;

export type PersistedUiState = {
  version: number;
  nav?: NavEntry;
  selectedAccountSet?: string;
};

type InvokeFn = <T = unknown>(cmd: string, args?: Record<string, any>) => Promise<T>;

/** Read the current vault's persisted UI state; undefined when there is none
 *  (no vault, fresh vault, corrupt file, unknown version). */
export async function loadPersistedUiState(
  invoke: InvokeFn,
): Promise<PersistedUiState | undefined> {
  try {
    const raw = await invoke<unknown>("load_ui_state");
    if (!raw || typeof raw !== "object") return undefined;
    const persisted = raw as PersistedUiState;
    if (persisted.version !== UI_STATE_VERSION) return undefined;
    // Reports open on the current FY (loadReport defaults it); only in-session
    // back/forward restores years.
    if (persisted.nav) delete persisted.nav.selectedReportYear;
    return persisted;
  } catch {
    return undefined;
  }
}

export type UiStatePersister = {
  /** Record the latest snapshot; written after the debounce interval. */
  schedule(snapshot: PersistedUiState): void;
  /** Write any pending snapshot immediately (awaits the backend call). */
  flush(): Promise<void>;
  /** Discard pending state and invalidate in-flight timers — call when the
   *  vault changes so the old vault's snapshot can't land in the new one. */
  reset(): void;
};

export function createUiStatePersister(
  invoke: InvokeFn,
  debounceMs = 300,
): UiStatePersister {
  let pending: PersistedUiState | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let epoch = 0;

  const clearTimer = () => {
    if (timer !== undefined) {
      clearTimeout(timer);
      timer = undefined;
    }
  };

  const write = async (snapshot: PersistedUiState, atEpoch: number) => {
    if (atEpoch !== epoch) return;
    try {
      await invoke("save_ui_state", { state: snapshot });
    } catch {
      /* no vault selected, or read-only PWA — persistence is best-effort */
    }
  };

  return {
    schedule(snapshot) {
      pending = snapshot;
      clearTimer();
      const atEpoch = epoch;
      timer = setTimeout(() => {
        timer = undefined;
        const snap = pending;
        pending = undefined;
        if (snap) void write(snap, atEpoch);
      }, debounceMs);
    },
    async flush() {
      clearTimer();
      const snap = pending;
      pending = undefined;
      if (snap) await write(snap, epoch);
    },
    reset() {
      epoch++;
      clearTimer();
      pending = undefined;
    },
  };
}

/**
 * Clear every vault-derived field on the app state when switching vaults,
 * leaving app-scoped fields (rootDir, knownRoots, showVaultPicker, global
 * settings, refresh hooks) untouched. Anything missed here is a cross-vault
 * leak — when adding vault-derived fields to AppState, add their reset here.
 */
export function resetVaultScopedState(state: Record<string, unknown>): void {
  // Data + selection
  state.parse = undefined;
  state.selectedAccount = undefined;
  state.drillPath = [];
  state._drillInitialized = false;
  state.selectedCategory = undefined;
  state.categoryDrillPath = [];
  state._categoryDrillInitialized = false;
  state.accountSets = [];
  state.selectedAccountSet = undefined;
  state.accountSetMap = {};
  state.allAccounts = [];
  state.accountFoldersMap = {};
  state.accountPropertiesMap = {};
  state.accountsWithOpening = [];
  state.displayConfig = undefined;
  state.showBaseCurrencySetup = undefined;
  state.baseCurrencyCandidates = undefined;
  state.baseCurrencySetupError = undefined;
  state.sourceFolderPaths = undefined;
  state.accountConfigs = undefined;
  state.accountGaps = undefined;
  // Search
  state.search = "";
  state.searchPills = [];
  state.searchText = "";
  state.searchError = undefined;
  state.searchFilteredTransactions = undefined;
  state.searchFilteredCount = undefined;
  state.searchFilteredOffset = undefined;
  state.dateFilter = "all";
  // Derived/query caches
  state.prefixQuery = undefined;
  state.treeBaseTotals = undefined;
  state.categoryBaseTotals = undefined;
  state.transactionValues = undefined;
  state.accountTotalValue = undefined;
  // Metadata-backed state
  state.metadataReady = undefined;
  state.devices = undefined;
  state.syncLog = undefined;
  state.syncLogOpen = undefined;
  state.relayConfig = undefined;
  state.relayPairingOpen = undefined;
  state.relayPairingMode = undefined;
  state.relayPairingCode = undefined;
  state.relayPairingError = undefined;
  state.pendingRelayConfig = undefined;
  state.tradeLinks = [];
  state.tradeSuggestions = [];
  state.tradeLinkSelection = undefined;
  // Reports
  state.sidebarView = "accounts";
  state.selectedReport = undefined;
  state.selectedReportYear = undefined;
  state.reportYears = undefined;
  state.reportMarkdown = undefined;
  state.cgtReport = undefined;
  state.incomeReport = undefined;
  state.balancesReport = undefined;
  state.performanceReport = undefined;
  state.lossHarvestReport = undefined;
  state.reportLoading = undefined;
  state.taxConfig = undefined;
  state.taxSettingsOpen = undefined;
  state.reportsBuilding = undefined;
  state.reportDateMode = undefined;
  state.reportDateFrom = undefined;
  state.reportDateTo = undefined;
  state.reportBaseScope = undefined;
  state.availableReportAccounts = undefined;
  // Sorts, filters, expansion
  state.txSort = { column: "date", direction: "asc" };
  state.cgtSort = undefined;
  state.cgtFilterText = undefined;
  state.cgtExpandedGroups = undefined;
  state.incomeSort = undefined;
  state.incomeFilterText = undefined;
  state.incomeExpandedGroups = undefined;
  state.balancesSort = undefined;
  state.balancesFilterText = undefined;
  state.balancesExpanded = undefined;
  state.lossHarvestExpanded = undefined;
  state.lossHarvestView = undefined;
  state.issueFilter = undefined;
  state.issuesPanelOpen = undefined;
  state.txExpandedGroups = undefined;
  state.txExpandedRows = undefined;
  state.txWindowStart = 0;
  state.reportPageSize = 50;
  // Optimistic mutation state
  state.pendingMutations = 0;
  state.pendingDeletes = new Set<string>();
  state.pendingAdds = [];
  state.justAddedTxnIds = undefined;
  state.pipelineWarnings = [];
  state.status = undefined;
  // Drafts and modals
  state.manualDraft = undefined;
  state.addAccountDraft = undefined;
  state.transformDraft = undefined;
  state.docImportDraft = undefined;
  state.docImportQueue = undefined;
  state.openingBalanceDraft = undefined;
  state.setPriceDraft = undefined;
  state.ruleEditorDraft = undefined;
  state.labelEditorDraft = undefined;
  // AI + plugins
  state.aiSuggest = undefined;
  state.aiAppliedPatterns = undefined;
  state.pluginsList = undefined;
  state.availablePlugins = undefined;
  state.addPluginsModalOpen = undefined;
  state.pluginInstalling = undefined;
  state.pluginRunning = undefined;
  state.pluginResult = undefined;
  state.pluginLog = undefined;
  state.pluginConfigOpen = undefined;
  state.pluginConfigValues = undefined;
  state.pluginSecretValues = undefined;
  state.pluginConfigSaving = undefined;
  state.pluginConfigSaved = undefined;
  state.pluginReadmeOpen = undefined;
  state.pluginReadmes = undefined;
  state.dailySummary = undefined;
}
