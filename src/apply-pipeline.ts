import type {
  AccountFolders,
  AccountProperties,
  ParseResponse,
  PipelineResponse,
  Transaction,
} from "./types";
import { refreshIssuesCache } from "./issues";
import { matchesPrefix, txnIdFromMeta } from "./meta";

/** Subset of AppState consumed by applyParse/applyPipelineResponse. */
export interface ApplyTarget {
  parse?: ParseResponse;
  selectedAccount?: string;
  displayConfig?: { default_account?: string } | undefined;
  accountPropertiesMap: Record<string, AccountProperties>;
  accountSetMap: Record<string, string[]>;
  accountFoldersMap: AccountFolders;
  /** Accounts whose declarations carry an opening balance. Sourced from
   *  pipeline metadata (slim paths) or the fat parse — NOT from the slim
   *  synthetic ParseResponse, whose accounts_with_opening is always empty. */
  accountsWithOpening: string[];
  pipelineWarnings: string[];
  reportsBuilding?: boolean;
  /** After a pipeline response, whose parse carries the account tree: re-read
   *  the rest — the vault-wide account pool, the header and the window. */
  refreshView?: () => Promise<void>;
  /** The main refresh, after a slim mutation response that carries nothing:
   *  re-read the account lists (sidebar tree, vault-wide pool, pipeline
   *  metadata), the header and the window, and schedule the heavy totals. */
  refreshLedger?: () => Promise<void>;
  /** Set of txn ids that appeared in this pipeline rebuild but were NOT
   *  in the previous parse. The renderer reads this once to add a
   *  fade-in class, then it's cleared. */
  justAddedTxnIds?: Set<string>;
}

function collectTxnIds(transactions: Transaction[] | undefined): Set<string> {
  const ids = new Set<string>();
  if (!transactions) return ids;
  for (const t of transactions) {
    const id = txnIdFromMeta(t.meta);
    if (id) ids.add(id);
  }
  return ids;
}

export function applyParse(state: ApplyTarget, response: ParseResponse): void {
  state.parse = response;

  if (response.account_properties) {
    state.accountPropertiesMap = { ...state.accountPropertiesMap, ...response.account_properties };
  }

  const nextBalances = response.balances ?? [];

  if (!state.selectedAccount) {
    const defaultAccount = state.displayConfig?.default_account;
    state.selectedAccount = (defaultAccount && nextBalances.find((b) => matchesPrefix(b.account, defaultAccount))?.account)
      ?? nextBalances[0]?.account;
  }
}

/** The main refresh after a slim mutation: a rule, a label, an import or a
 *  hide can add, rename or empty an account, so the account lists (sidebar,
 *  dropdowns) are re-read along with the header and the transaction window.
 *  The window refreshes IN PLACE — runSearchFilter re-fetches the whole loaded
 *  span from offset 0 (see `searchWindowFetch`) and swaps the rows only once
 *  the new ones arrive — so the table never empties, keeps the pages the user
 *  scrolled to, and morphdom updates just the rows that changed. The issues
 *  cache is part of the heavy tail the main refresh schedules (a full-vault
 *  scan when it misses), so it is not refreshed here. */
export async function refreshAfterRebuild(state: ApplyTarget): Promise<void> {
  await state.refreshLedger?.();
}

/** Slim mutation response — backend confirmed the write succeeded but
 *  didn't re-send the full ledger, avoiding the multi-second main-thread
 *  JSON deserialize stall on large vaults. Everything the change can alter is
 *  re-read from disk instead, by the main refresh. */
export interface MutationResponse {
  ok: boolean;
  warnings: string[];
  output_files_written: number;
}

/** Record a slim mutation's outcome (warnings) without refreshing — the
 *  optimistic flow refreshes through settleOptimisticEdit. A mutation never
 *  rebuilds the report files: the Reports view computes a report live when
 *  its cached file is stale. */
export function noteMutationResponse(state: ApplyTarget, response: MutationResponse): void {
  state.pipelineWarnings = response.warnings ?? [];
}

export async function applyMutationResponse(
  state: ApplyTarget,
  response: MutationResponse,
): Promise<void> {
  noteMutationResponse(state, response);
  await refreshAfterRebuild(state);
}

/** Record a full pipeline response (parse, account maps, warnings) without
 *  refreshing the window — see `noteMutationResponse`. */
export function notePipelineResponse(state: ApplyTarget, response: PipelineResponse): void {
  // Snapshot the prev set BEFORE applyParse mutates state.parse so we can
  // diff against the new transactions and surface a fade-in for the rows
  // that genuinely arrived in this rebuild.
  const prevTxnIds = collectTxnIds(state.parse?.transactions);
  applyParse(state, response.parse);
  const nextTxnIds = collectTxnIds(state.parse?.transactions);
  const justAdded = new Set<string>();
  for (const id of nextTxnIds) {
    if (!prevTxnIds.has(id)) justAdded.add(id);
  }
  state.justAddedTxnIds = justAdded.size > 0 ? justAdded : undefined;
  if (response.parse.accounts_with_opening) {
    state.accountsWithOpening = response.parse.accounts_with_opening;
  }
  state.pipelineWarnings = response.warnings ?? [];
  if (response.result.owner_accounts) {
    state.accountSetMap = response.result.owner_accounts;
  }
  if (response.result.account_folders) {
    state.accountFoldersMap = response.result.account_folders;
  }
  if (response.result.account_properties) {
    state.accountPropertiesMap = response.result.account_properties;
  }
}

export async function applyPipelineResponse(state: ApplyTarget, response: PipelineResponse): Promise<void> {
  notePipelineResponse(state, response);
  // A full rebuild can change which issues exist; refresh them before the
  // window so the render that follows shows both.
  try { await refreshIssuesCache(); } catch { /* ignore */ }
  await state.refreshView?.();
}
