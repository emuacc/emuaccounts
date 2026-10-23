import type { AccountBalance, CommodityAmount } from "./types";
import { formatMoneyWhole } from "./format";
import { folderRootOf } from "./account-utils";

/** The entity's root folder: the `assets` tree root. Its header total is the
 * Balances report total. Other folder roots (`liabilities`) sit beside the
 * asset folders at level 0 of the Accounts pane as groups of their own. */
export const ROOT_FOLDER_PATH = "assets";

/** Drill-path segments to reach `account` in the Accounts pane. The entity
 * root (`assets`) is implicit and stripped — `assets:cash:cba` → ["cash", "cba"]
 * — while any other folder root keeps its root segment, so the `liabilities`
 * group is a level-0 entry: `liabilities:credit-card` → ["liabilities", "credit-card"]. */
export function accountDrillSegments(account: string): string[] {
  const parts = account.split(":");
  return parts[0] === ROOT_FOLDER_PATH ? parts.slice(1) : parts;
}

/** Inverse of {@link accountDrillSegments}: the group account a drill path
 * selects. An empty path is the entity root folder. */
export function accountFromDrillPath(drillPath: string[]): string {
  if (drillPath.length === 0) return ROOT_FOLDER_PATH;
  const [head] = drillPath;
  const root = folderRootOf(head);
  if (root !== undefined && root !== ROOT_FOLDER_PATH) return drillPath.join(":");
  return `${ROOT_FOLDER_PATH}:${drillPath.join(":")}`;
}

type TreeLike = {
  name: string;
  fullPath: string;
  isLeaf: boolean;
  children: TreeLike[];
};

function find(tree: TreeLike[], fullPath: string): TreeLike | undefined {
  const parts = fullPath.split(":");
  let cur: TreeLike | undefined = tree.find((r) => r.name === parts[0]);
  for (let i = 1; cur && i < parts.length; i++) {
    cur = cur.children.find((c) => c.name === parts[i]);
  }
  return cur;
}

export function isFolderSelection(selectedAccount: string | undefined, tree: TreeLike[]): boolean {
  if (!selectedAccount) return false;
  const node = find(tree, selectedAccount);
  if (!node) return false;
  return node.children.length > 0 || !node.isLeaf;
}

export function folderDisplayName(folderPath: string, accountSetLabel: string): string {
  if (folderPath === ROOT_FOLDER_PATH) return accountSetLabel;
  const parts = folderPath.split(":");
  return parts[parts.length - 1];
}

/** Header / row total for an account path: its primary-scope base-currency
 * value from the backend holdings computation (rolled up over every primary
 * account beneath it). `assets` is the Balances report total. Undefined when
 * the path holds nothing in the primary scope. */
export function folderTotalAud(
  folderPath: string | undefined,
  treeBaseTotals: Map<string, number>,
): number | undefined {
  if (!folderPath) return undefined;
  return treeBaseTotals.get(folderPath);
}

const CURRENCY_SYMBOLS: Record<string, string> = {
  AUD: "$", USD: "$", NZD: "$", CAD: "$", SGD: "$", HKD: "$",
  EUR: "€", GBP: "£", JPY: "¥", CNY: "¥", CNH: "¥",
};

/** Formats a folder-level balance with thousand separators and a leading
 * currency symbol when the commodity is a known fiat code. Unknown commodities
 * get the number alone — callers render the commodity ticker separately. */
export function formatFolderAmount(amount: number, commodity: string): string {
  const symbol = CURRENCY_SYMBOLS[commodity] ?? "";
  const sign = amount < 0 ? "-" : "";
  return `${sign}${symbol}${formatMoneyWhole(Math.abs(amount))}`;
}

/** Synthesizes an AccountBalance for a folder node by summing commodity totals
 * across every leaf account beneath the folder prefix. Returns undefined when
 * the folder has no descendant leaves (i.e. the path is itself a leaf account). */
export function syntheticFolderBalance(
  folderPath: string,
  balances: AccountBalance[],
): AccountBalance | undefined {
  const prefix = folderPath + ":";
  const totals = new Map<string, number>();
  let matched = false;
  for (const b of balances) {
    if (b.account === folderPath) continue;
    if (!b.account.startsWith(prefix)) continue;
    matched = true;
    for (const t of b.totals) {
      totals.set(t.commodity, (totals.get(t.commodity) ?? 0) + t.amount);
    }
  }
  if (!matched) return undefined;
  const result: CommodityAmount[] = [...totals.entries()].map(([commodity, amount]) => ({ commodity, amount }));
  return { account: folderPath, totals: result };
}
