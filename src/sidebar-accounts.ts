import type { AccountBalance, AccountFolders } from "./types";
import { accountFoldersForSet, folderRootOf } from "./account-utils";
import { matchesPrefix } from "./meta";

/** The roots the Categories pane lists: money flows, never holdings. */
const CATEGORY_ROOTS = ["income", "expenses"] as const;

/** Every account a source folder backs, in any set. */
export function folderBackedAccounts(accountFoldersMap: AccountFolders): Set<string> {
  return new Set(Object.keys(accountFoldersForSet(accountFoldersMap, undefined)));
}

/** A declared clearing account, or a sub-account of one: segment-aware, like
 *  the backend's `AccountAllowlist` (`assets:transfer` covers
 *  `assets:transfer:cash`, not `assets:transferwise`). */
export function isClearingAccount(account: string, clearingAccounts: readonly string[]): boolean {
  return clearingAccounts.some((c) => matchesPrefix(account, c));
}

/** What "Show all holdings" adds to the Accounts pane: the `assets:*` and
 *  `liabilities:*` accounts the set's transactions own (every set's when none
 *  is selected) that no source folder backs in any set — staking, lending, a
 *  card only rules post to. Clearing accounts are pass-through, not holdings. */
export function nonFolderHoldings(
  accountFoldersMap: AccountFolders,
  accountSetMap: Record<string, string[]>,
  selectedAccountSet: string | undefined,
  clearingAccounts: readonly string[],
): Set<string> {
  const folderBacked = folderBackedAccounts(accountFoldersMap);
  const owned = selectedAccountSet
    ? (accountSetMap[selectedAccountSet] ?? [])
    : Object.values(accountSetMap).flat();
  return new Set(
    owned.filter((a) =>
      folderRootOf(a) !== undefined && !folderBacked.has(a) && !isClearingAccount(a, clearingAccounts)
    ),
  );
}

export type SidebarOptions = {
  /** Global Settings → "Show all holdings". */
  showAllHoldings?: boolean;
  /** The vault's clearing accounts, which neither pane lists. */
  clearingAccounts?: readonly string[];
};

/** The Accounts pane: the folder-backed accounts under a folder root
 *  (`assets:*`, `liabilities:*`) for the active set, plus its
 *  {@link nonFolderHoldings} while "Show all holdings" is on. Clearing
 *  accounts never appear. Parent groups are derived by the tree builder from
 *  these leaf balances. */
export function filterSidebarBalances(
  allBalances: AccountBalance[],
  accountFoldersMap: AccountFolders,
  accountSetMap: Record<string, string[]>,
  selectedAccountSet?: string,
  options: SidebarOptions = {},
): AccountBalance[] {
  const clearingAccounts = options.clearingAccounts ?? [];
  const setFolderBacked = new Set(
    Object.keys(accountFoldersForSet(accountFoldersMap, selectedAccountSet)),
  );
  const owned = selectedAccountSet ? new Set(accountSetMap[selectedAccountSet] ?? []) : undefined;
  const revealed = options.showAllHoldings
    ? nonFolderHoldings(accountFoldersMap, accountSetMap, selectedAccountSet, clearingAccounts)
    : new Set<string>();
  const listed = (account: string) =>
    (setFolderBacked.has(account) && (owned?.has(account) ?? true)) || revealed.has(account);
  return allBalances.filter((b) =>
    folderRootOf(b.account) !== undefined &&
    listed(b.account) &&
    !isClearingAccount(b.account, clearingAccounts)
  );
}

/** The Categories pane: what money was earned and spent on — `income:*` and
 *  `expenses:*`, plus `ignore:*` while "Show Ignored" is on. Holdings live in
 *  the Accounts pane and equity in neither. Cross-set by design: categories
 *  are not partitioned by owner, matching the union query that backs them. */
export function filterCategoryBalances(
  allBalances: AccountBalance[],
  showHidden: boolean,
): AccountBalance[] {
  return allBalances.filter((b) =>
    CATEGORY_ROOTS.some((root) => matchesPrefix(b.account, root)) ||
    (showHidden && matchesPrefix(b.account, "ignore"))
  );
}

/** `balances` plus a row with no totals for each pending account they lack:
 *  a pane lists an account the app is writing at once, and the refresh fills
 *  its totals in. */
export function withPendingBalances(balances: AccountBalance[], pending: readonly string[]): AccountBalance[] {
  const listed = new Set(balances.map((b) => b.account));
  const added = [...new Set(pending)].filter((a) => !listed.has(a));
  return added.length === 0 ? balances : [...balances, ...added.map((account) => ({ account, totals: [] }))];
}

/** `accountSetMap` with the pending accounts owned by the active set, as the
 *  refresh will report them, so "Show all holdings" lists a new holding at once. */
export function withPendingOwned(
  accountSetMap: Record<string, string[]>,
  selectedAccountSet: string | undefined,
  pending: readonly string[],
): Record<string, string[]> {
  if (pending.length === 0) return accountSetMap;
  const set = selectedAccountSet ?? "";
  return { ...accountSetMap, [set]: [...(accountSetMap[set] ?? []), ...pending] };
}

export type TransactionQuery = "query_search" | "query_global";

/** The query that reads a selection's transactions. `query_search` reads only
 *  the ledgers of the folders under the selected account, so it finds nothing
 *  for an account no folder backs; a selection whose subtree in the pane holds
 *  one takes the union `query_global`. With nothing selected the pane decides:
 *  the Categories pane holds no folder accounts at all. */
export function transactionQueryFor(
  selected: string | undefined,
  paneBalances: AccountBalance[],
  folderBacked: Set<string>,
  pane: "accounts" | "categories",
): TransactionQuery {
  if (!selected) return pane === "categories" ? "query_global" : "query_search";
  const holdsNonFolderAccount = paneBalances.some((b) =>
    matchesPrefix(b.account, selected) && !folderBacked.has(b.account)
  );
  return holdsNonFolderAccount ? "query_global" : "query_search";
}

/** Whether `account` counts toward balances: folder-backed, or at or under an
 *  "Accounts included in balance" prefix — the backend's primary-accounts
 *  allowlist, which also decides the header, group totals and Balances report. */
export function isIncludedInBalance(
  account: string,
  folderBacked: Set<string>,
  includedPrefixes: readonly string[],
): boolean {
  return folderBacked.has(account) || includedPrefixes.some((p) => p !== "" && matchesPrefix(account, p));
}

type NodeLike = { fullPath: string; children: NodeLike[] };

/** True when nothing at or beneath `node` counts toward balances: a row or
 *  group only "Show all holdings" put in the Accounts pane. Its primary-scope
 *  total is empty, so it shows its own value instead, marked as outside. */
export function isOutsideBalance(node: NodeLike, included: (account: string) => boolean): boolean {
  return !included(node.fullPath) && node.children.every((c) => isOutsideBalance(c, included));
}
