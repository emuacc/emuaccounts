// Per-transaction base-currency value computation for the ledger view (the
// "Value" column). Extracted from main.ts so this path is unit-testable: the
// regression it guards against is sourcing values from the wrong transaction
// set, so the Value column renders "—" even though prices resolve fine.
//
// A leg is valued one of three ways (`planLegValue`). A leg carrying its own
// `@`/`@@` price in the base currency is summed directly. A leg priced in
// another currency is that price value converted at the currency's rate on the
// transaction date — a broker fill is worth what it settled for, which is the
// proceeds the CGT report lists, not the day's close × quantity. An unpriced
// leg (a plain holding, a transfer) is its quantity × the commodity's daily
// rate. The same planner values a trade-linked swap partner, and every row
// keeps the basis it was summed from so the Value cell can say so and open
// Set Price for the rate it actually used.
//
// `computeTransactionValues` keeps the former in-main.ts `loadTransactionValues`
// guard and source, with the Tauri `convert_to_base_currency` IPC injected so
// tests can stub it.

import type { Posting, Transaction, TradeLink } from "./types";
import { filterByAccountPrefix } from "./account-search";
import { formatAmount, formatAmountSmart, formatMoney } from "./format";
import { matchesPrefix, swapPartnerRefFromMeta, txnIdFromMeta, txnValueKey } from "./meta";
import { postingPriceValue } from "./posting-value";
import { valuationLegs } from "./transaction-legs";

export type ConversionRequest = { commodity: string; amount: number; datetime: string };

/** Converts a batch of (commodity, amount, datetime) requests to the base
 *  currency. Injected so main.ts wires it to the Tauri `convert_to_base_currency`
 *  command while tests stub it. */
export type ConvertToBase = (
  baseCurrency: string,
  requests: ConversionRequest[],
) => Promise<(number | null)[]>;

/** How one leg was valued: what it held, its own price if it carried one, and
 *  whose rate the value was looked up at. */
export type LegBasis = {
  commodity: string;
  /** The signed leg amount. */
  amount: number;
  /** When the rate was looked up: the leg's own transaction date. */
  datetime: string;
  /** The leg's `@`/`@@` price as a total in its currency, null when unpriced. */
  price: { value: number; commodity: string } | null;
  /** The commodity whose rate the value needed: the price currency of a leg
   *  priced in another currency, the held commodity of an unpriced leg, null
   *  when the price is already in the base currency. */
  rateCommodity: string | null;
  /** False when that rate was missing on `datetime`: a priced leg then shows
   *  its unconverted price value, an unpriced leg contributes nothing. */
  resolved: boolean;
};

/** A transaction's base-currency value (null when no leg resolved) and the
 *  legs it was summed from. */
export type TxnValue = { value: number | null; legs: LegBasis[] };

/** One leg's valuation before the rates come back: already in base (`direct`)
 *  or a rate `request`, with the value to show when that rate is missing. */
export type PlannedLeg = {
  basis: LegBasis;
  /** −1 for a leg leaving the account, +1 for one entering it. */
  sign: number;
  direct: number | null;
  request: ConversionRequest | null;
  fallback: number | null;
};

export function planLegValue(posting: Posting, datetime: string, baseCurrency: string): PlannedLeg {
  const sign = posting.amount < 0 ? -1 : 1;
  const priceValue = postingPriceValue(posting);
  const price = priceValue !== null && posting.price ? { value: priceValue, commodity: posting.price.commodity } : null;
  const basis: LegBasis = { commodity: posting.commodity, amount: posting.amount, datetime, price, rateCommodity: null, resolved: false };
  if (price && price.commodity === baseCurrency) {
    basis.resolved = true;
    return { basis, sign, direct: price.value, request: null, fallback: null };
  }
  basis.rateCommodity = price ? price.commodity : posting.commodity;
  const amount = price ? price.value : Math.abs(posting.amount);
  const request = { commodity: basis.rateCommodity, amount, datetime };
  return { basis, sign, direct: null, request, fallback: price ? price.value : null };
}

/** A leg awaiting its rate; its index matches the plan's `requests`. */
type PendingLeg = { sign: number; fallback: number | null; basis: LegBasis };

/** The signed base-currency contribution of a pending leg once its rate is
 *  back: the converted value, else the unconverted price value, else null.
 *  Records on the basis whether the rate resolved. */
function resolvedContribution(leg: PendingLeg, converted: number | null): number | null {
  leg.basis.resolved = converted !== null;
  const value = converted ?? leg.fallback;
  return value === null ? null : leg.sign * value;
}

type TxnValueAcc = { sum: number; has: boolean; legs: LegBasis[] };

export type LegValuePlan = {
  requests: ConversionRequest[];
  /** One per request, in order. Each `basis` is the object already listed in
   *  its transaction's `legs`, so resolving it updates the row's basis too. */
  legReqs: (PendingLeg & { txnIdx: number })[];
  totals: Map<string, TxnValueAcc>;
};

/** Plan each transaction's base-currency value from its in-scope legs. Legs
 *  already priced in the base currency (e.g. a property's `@@ … AUD` land /
 *  building legs) are summed immediately; the rest become conversion requests
 *  the caller folds back in. A co-held multi-commodity holding sums to the full
 *  value; in-scope transfers and swaps (e.g. a broker trade's shares + cash
 *  sub-account) value a single side — see `valuationLegs`. `has` stays false
 *  until a leg yields a value, so a fully-unpriced transaction records `null`,
 *  not a misleading 0. */
export function buildLegValueRequests(
  txns: Transaction[],
  account: string,
  baseCurrency: string,
): LegValuePlan {
  const requests: LegValuePlan["requests"] = [];
  const legReqs: LegValuePlan["legReqs"] = [];
  const totals: LegValuePlan["totals"] = new Map();
  txns.forEach((t, i) => {
    const legs = valuationLegs(t, account);
    if (legs.length === 0) return;
    const key = txnValueKey(t);
    const acc = totals.get(key) ?? { sum: 0, has: false, legs: [] };
    totals.set(key, acc);
    for (const posting of legs) {
      const plan = planLegValue(posting, t.datetime, baseCurrency);
      acc.legs.push(plan.basis);
      if (plan.direct !== null) {
        acc.sum += plan.sign * plan.direct;
        acc.has = true;
      } else if (plan.request) {
        requests.push(plan.request);
        legReqs.push({ txnIdx: i, sign: plan.sign, fallback: plan.fallback, basis: plan.basis });
      }
    }
  });
  return { requests, legReqs, totals };
}

/** Fold the converted leg values back into per-transaction totals and produce
 *  the value map (txn key → summed base-currency value, or null if none). */
export function applyLegValues(
  plan: LegValuePlan,
  txns: Transaction[],
  values: (number | null)[],
): Map<string, TxnValue> {
  const { legReqs, totals } = plan;
  values.forEach((v, j) => {
    const leg = legReqs[j];
    const contribution = resolvedContribution(leg, v);
    const acc = totals.get(txnValueKey(txns[leg.txnIdx]));
    if (acc && contribution !== null) {
      acc.sum += contribution;
      acc.has = true;
    }
  });
  const valMap = new Map<string, TxnValue>();
  for (const [key, { sum, has, legs }] of totals) {
    valMap.set(key, { value: has ? sum : null, legs });
  }
  return valMap;
}

/** For trade-linked rows, overwrite each leg's value with the partner's
 *  counterparty value so a swap shows the same value whether displayed linked or
 *  individually. The partner leg is valued like any row leg (`planLegValue`);
 *  when nothing resolves the row keeps its own value. Mutates `valMap`. */
async function applyTradeLinkPartnerValues(
  valMap: Map<string, TxnValue>,
  txns: Transaction[],
  account: string,
  baseCurrency: string,
  allTxns: Transaction[],
  tradeLinks: TradeLink[],
  convert: ConvertToBase,
): Promise<void> {
  const linkMap = new Map<string, string>();
  for (const link of tradeLinks) {
    linkMap.set(link.txn_id_a, link.txn_id_b);
    linkMap.set(link.txn_id_b, link.txn_id_a);
  }

  const requests: ConversionRequest[] = [];
  const pending: (PendingLeg & { key: string })[] = [];
  for (const t of txns) {
    const txnId = txnIdFromMeta(t.meta);
    // Saved-link gate (matches the swap-row merge gate in resolveSwapPartner).
    if (!txnId || !linkMap.has(txnId)) continue;
    // Rust-stamped partner reference: (hash, commodity) is unique per ledger row.
    const swapRef = swapPartnerRefFromMeta(t.meta);
    if (!swapRef) continue;
    const partner = allTxns.find(
      (pt) =>
        pt !== t &&
        txnIdFromMeta(pt.meta) === swapRef.partnerTxnId &&
        pt.amount_commodity === swapRef.partnerCommodity,
    );
    if (!partner) continue;
    const own = t.postings.find((p) => matchesPrefix(p.account, account));
    const pp = partner.postings.find((p) => matchesPrefix(p.account, account));
    if (!pp || pp.commodity === own?.commodity) continue;
    const sign = own && own.amount < 0 ? -1 : 1;
    const key = txnValueKey(t);
    const plan = planLegValue(pp, partner.datetime, baseCurrency);
    if (plan.direct !== null) {
      valMap.set(key, { value: sign * plan.direct, legs: [plan.basis] });
    } else if (plan.request) {
      requests.push(plan.request);
      pending.push({ key, sign, fallback: plan.fallback, basis: plan.basis });
    }
  }

  if (pending.length === 0) return;
  const partnerValues = await convert(baseCurrency, requests);
  partnerValues.forEach((pv, j) => {
    const leg = pending[j];
    const contribution = resolvedContribution(leg, pv);
    if (contribution !== null) valMap.set(leg.key, { value: contribution, legs: [leg.basis] });
  });
}

/** Money for the basis text: thousands separators, but a dust-sized total
 *  keeps its significant digits instead of rounding to 0.00. */
function money(value: number): string {
  return Math.abs(value) >= 0.01 ? formatMoney(value) : formatAmountSmart(value);
}

function describeLegBasis(leg: LegBasis, baseCurrency: string): string {
  const quantity = `${formatAmount(Math.abs(leg.amount), leg.commodity)} ${leg.commodity}`;
  const day = leg.datetime.slice(0, 10);
  if (leg.price) {
    const priced = `${quantity} priced at ${money(leg.price.value)} ${leg.price.commodity}`;
    if (!leg.rateCommodity) return priced;
    return leg.resolved
      ? `${priced}, converted to ${baseCurrency} at the ${leg.rateCommodity} rate on ${day}`
      : `${priced}; no ${leg.rateCommodity} rate on ${day}, so shown unconverted`;
  }
  return leg.resolved
    ? `${quantity} × the ${leg.commodity} price on ${day}`
    : `${quantity}; no ${leg.commodity} price on ${day}`;
}

/** The basis a value was summed from, as the Value cell's tooltip and the Set
 *  Price modal state it: one clause per leg. */
export function describeValueBasis(txnValue: TxnValue, baseCurrency: string): string {
  return txnValue.legs.map((leg) => describeLegBasis(leg, baseCurrency)).join(" + ");
}

/** The commodity whose price Set Price should edit for this value: the rate
 *  the first leg converted through, else the commodity it holds. Null when
 *  nothing was valued. */
export function valueRateCommodity(txnValue: TxnValue): string | null {
  const first = txnValue.legs[0];
  return first ? (first.rateCommodity ?? first.commodity) : null;
}

/** The subset of AppState the value-load reads. Structural so main.ts's AppState
 *  satisfies it without a circular import. */
export interface ValueLoadState {
  displayConfig?: { base_currency?: string } | undefined;
  parse?: { transactions: Transaction[] } | undefined;
  searchFilteredTransactions?: Transaction[] | undefined;
  sidebarView?: string | undefined;
  selectedCategory?: string | undefined;
  selectedAccount?: string | undefined;
  tradeLinks?: TradeLink[] | undefined;
}

/** Rows whose Value the ledger view should compute: the windowed rows actually
 *  on screen. `state.parse` is NOT populated in account/search views, so
 *  sourcing from it left every Value cell empty. */
function rowsForValuation(state: ValueLoadState): Transaction[] {
  return state.searchFilteredTransactions ?? [];
}

/** Where to resolve a trade-link swap partner: the full ledger when loaded, else
 *  the windowed rows on screen. */
function partnerLookupRows(state: ValueLoadState): Transaction[] {
  return state.parse?.transactions ?? state.searchFilteredTransactions ?? [];
}

/** Compute the per-transaction base-currency value map for the selected account,
 *  or undefined when there is nothing to value (no base currency / account, or
 *  the account's own commodity already IS the base currency). */
export async function computeTransactionValues(
  state: ValueLoadState,
  convert: ConvertToBase,
): Promise<Map<string, TxnValue> | undefined> {
  const baseCurrency = state.displayConfig?.base_currency;
  const selected = state.sidebarView === "categories" ? state.selectedCategory : state.selectedAccount;
  if (!baseCurrency || !selected) {
    return undefined;
  }

  const account = selected;
  const txns = filterByAccountPrefix(rowsForValuation(state), account);
  // Selected account's commodity already is the base currency → no conversion.
  const samplePosting = txns
    .find((t) => t.postings.some((p) => matchesPrefix(p.account, account)))
    ?.postings.find((p) => matchesPrefix(p.account, account));
  if (samplePosting && samplePosting.commodity === baseCurrency) {
    return undefined;
  }

  const plan = buildLegValueRequests(txns, account, baseCurrency);

  try {
    const values = plan.requests.length > 0 ? await convert(baseCurrency, plan.requests) : [];
    const valMap = applyLegValues(plan, txns, values);

    // Trade-linked rows take their partner's counterparty value. Resolve the
    // partner in the full ledger when loaded, else the windowed rows on screen.
    if (state.tradeLinks?.length) {
      await applyTradeLinkPartnerValues(
        valMap,
        txns,
        account,
        baseCurrency,
        partnerLookupRows(state),
        state.tradeLinks,
        convert,
      );
    }

    return valMap;
  } catch {
    return undefined;
  }
}
