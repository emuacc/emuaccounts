// The account lists: the sidebar's Accounts and Categories panes (the active
// set's account tree) and the vault-wide pool behind every account dropdown.
// Two paths keep them current:
//
// - The main refresh re-reads both from disk after every change, wherever it
//   came from (a source file, a sync, a rule, a manual transaction):
//   `loadAccountLists`.
// - An account the app is writing itself — a manual transaction's postings, a
//   rule's accounts, an inline category edit — is listed at once, before the
//   rebuild finishes: `creatingAccounts` holds it as pending while its
//   mutation runs, and the refresh inside that mutation brings its totals.
//   The sidebar merges the pending accounts in with `withPendingBalances` /
//   `withPendingOwned` (./sidebar-accounts), the dropdowns with
//   `collectAccountSuggestions` / `knownAccounts` (./account-suggestions).

import { applyParse, type ApplyTarget } from "./apply-pipeline";
import type { AccountBalance, AccountFolders, AccountProperties } from "./types";

type InvokeFn = <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

export interface AccountListsState extends ApplyTarget {
  selectedAccountSet?: string;
  /** Every account across the whole vault (load_account_tree("")). */
  allAccounts: string[];
  /** Accounts a mutation in flight is writing, one entry per mutation that
   *  names them; the account lists show them until it settles. */
  pendingAccounts: string[];
  /** The account prefixes the active set hides while Show Ignored is off
   *  (its config.json `hidden_accounts`, each ending in ":"). */
  hiddenAccounts: string[];
}

/** The account half of the main refresh: the active set's account tree, the
 *  vault-wide pool and the pipeline metadata (which accounts a folder backs,
 *  which set owns them) and the accounts the set hides, read together so the
 *  sidebar and the dropdowns agree. Throws when the tree can't be read; a
 *  failed pool, metadata or hidden-accounts read keeps the previous one.
 *
 *  With `pool: false` the vault-wide pool is not re-read: a mutation in the
 *  active set can add an account only to that set, so its tree keeps the pool
 *  current without a second read of every set's summaries. An account the set
 *  no longer holds stays in the pool until the next full read — a stale
 *  dropdown suggestion, nothing worse. */
export async function loadAccountLists(
  state: AccountListsState,
  invoke: InvokeFn,
  opts?: { pool?: boolean },
): Promise<void> {
  const accountSet = state.selectedAccountSet ?? "";
  const pool = opts?.pool ?? true;
  const [balances, meta, hidden] = await Promise.all([
    invoke<AccountBalance[]>("load_account_tree", { accountSet }),
    invoke<{
      owner_accounts?: Record<string, string[]>;
      account_folders?: AccountFolders;
      account_properties?: Record<string, AccountProperties>;
      accounts_with_opening?: string[];
    }>("load_pipeline_metadata").catch(() => null),
    invoke<string[]>("get_hidden_accounts", { accountSet }).catch(() => null),
    pool ? loadAllAccounts(state, invoke) : Promise.resolve(),
  ]);
  if (!pool) {
    state.allAccounts = [...new Set([...state.allAccounts, ...balances.map((b) => b.account)])];
  }
  if (hidden) state.hiddenAccounts = hidden;
  if (meta) {
    if (meta.owner_accounts) state.accountSetMap = meta.owner_accounts;
    if (meta.account_folders) state.accountFoldersMap = meta.account_folders;
    if (meta.account_properties) {
      state.accountPropertiesMap = { ...state.accountPropertiesMap, ...meta.account_properties };
    }
    if (meta.accounts_with_opening) state.accountsWithOpening = meta.accounts_with_opening;
  }
  // The tree alone, no transactions: the visible window is paged in separately.
  applyParse(state, {
    ok: true,
    diagnostics: [],
    transactions: [],
    balances,
    accounts_with_opening: [],
    account_properties: {},
  });
}

/** Re-read the vault-wide pool behind every account dropdown. The selected set
 *  doesn't change it, so switching sets doesn't call this. */
export async function loadAllAccounts(
  state: Pick<AccountListsState, "allAccounts">,
  invoke: InvokeFn,
): Promise<void> {
  try {
    const all = await invoke<{ account: string }[]>("load_account_tree", { accountSet: "" });
    state.allAccounts = all.map((b) => b.account);
  } catch (e) {
    console.warn("load all accounts failed:", e);
  }
}

/** Wrap a queued mutation that writes `accounts`: they are pending from now
 *  until it settles, so the account lists show them before its rebuild
 *  finishes. The refresh inside `work` brings their totals; a failed mutation
 *  takes them away again. Blank names (an empty fee account) are ignored. */
export function creatingAccounts(
  state: Pick<AccountListsState, "pendingAccounts">,
  accounts: readonly (string | null | undefined)[],
  work: () => Promise<void>,
): () => Promise<void> {
  const names = [...new Set(accounts.map((a) => a?.trim() ?? "").filter((a) => a !== ""))];
  state.pendingAccounts = [...state.pendingAccounts, ...names];
  return async () => {
    try {
      await work();
    } finally {
      // One entry each: another mutation in flight may name the same account.
      const left = [...state.pendingAccounts];
      for (const name of names) {
        const i = left.indexOf(name);
        if (i >= 0) left.splice(i, 1);
      }
      state.pendingAccounts = left;
    }
  };
}
