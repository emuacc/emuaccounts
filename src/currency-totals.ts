/** Base-currency totals for the account tree.
 *
 * The backend's `load_tree_base_totals` runs the SAME holdings computation as
 * the Balances report (openings folded in, primary-accounts allowlist, hidden
 * accounts dropped, as of today) and rolls each account's value up to every
 * ancestor. `primary["assets"]` is therefore exactly the Balances report total,
 * so the sidebar rows, the folder header and the report can never disagree. */

import type { TreeBaseTotals } from "./types";

type TreeBaseTotalsWire = {
  as_of_date: string;
  base_currency: string;
  primary: Record<string, number>;
  all: Record<string, number>;
};

export function treeBaseTotalsFromWire(wire: TreeBaseTotalsWire): TreeBaseTotals {
  return {
    asOfDate: wire.as_of_date,
    baseCurrency: wire.base_currency,
    primary: new Map(Object.entries(wire.primary)),
    all: new Map(Object.entries(wire.all)),
  };
}

/** Load the tree totals for `accountSet`. Returns undefined when there is no
 * base currency yet or the backend can't compute them. */
export async function loadTreeBaseTotals(
  baseCurrency: string | undefined,
  accountSet: string,
  invoker: <T>(cmd: string, args: Record<string, unknown>) => Promise<T>,
): Promise<TreeBaseTotals | undefined> {
  if (!baseCurrency) return undefined;
  try {
    const wire = await invoker<TreeBaseTotalsWire>("load_tree_base_totals", { accountSet, baseCurrency });
    return treeBaseTotalsFromWire(wire);
  } catch {
    return undefined;
  }
}
