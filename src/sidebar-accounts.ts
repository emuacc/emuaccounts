import type { AccountBalance, AccountFolders } from "./types";
import { accountFoldersForSet, folderRootOf } from "./account-utils";

/** The sidebar only exposes folder-backed accounts under a folder root
 * (`assets:*`, `liabilities:*`) for the active set. Parent groups are derived
 * by the tree builder from these leaf balances. */
export function filterSidebarBalances(
  allBalances: AccountBalance[],
  accountFoldersMap: AccountFolders,
  accountSetMap: Record<string, string[]>,
  selectedAccountSet?: string,
): AccountBalance[] {
  const folderBackedAccounts = new Set(
    Object.keys(accountFoldersForSet(accountFoldersMap, selectedAccountSet)),
  );
  const rootedBalances = allBalances.filter((b) => folderRootOf(b.account) !== undefined);

  if (!selectedAccountSet) {
    return rootedBalances.filter((b) => folderBackedAccounts.has(b.account));
  }

  const ownedAccounts = new Set(accountSetMap[selectedAccountSet] ?? []);
  return rootedBalances.filter((b) =>
    ownedAccounts.has(b.account) && folderBackedAccounts.has(b.account)
  );
}

/** The complement of {@link filterSidebarBalances}: the "Categories" pane exposes
 * every account NOT backed by a source folder — `income:*`, `expenses:*`,
 * `equity:*`, `liabilities:*` without a folder of their own, and the synthetic
 * `assets` contras (`assets:staking`, `assets:lending`, `assets:transfer`,
 * `assets:crypto:bridge`, …). `ignore:*` is
 * excluded unless "Show Ignored" is on. Cross-set by design — these nominal/contra
 * accounts are not partitioned by owner, matching the union query that backs them. */
export function filterCategoryBalances(
  allBalances: AccountBalance[],
  accountFoldersMap: AccountFolders,
  showHidden: boolean,
): AccountBalance[] {
  const folderBackedAccounts = new Set(
    Object.keys(accountFoldersForSet(accountFoldersMap, undefined)),
  );
  return allBalances.filter((b) =>
    !folderBackedAccounts.has(b.account) &&
    (showHidden || !b.account.startsWith("ignore:"))
  );
}
