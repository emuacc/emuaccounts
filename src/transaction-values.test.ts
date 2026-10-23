import { describe, expect, it } from "vitest";
import {
  applyLegValues,
  buildLegValueRequests,
  computeTransactionValues,
  describeValueBasis,
  planLegValue,
  valueRateCommodity,
  type ConvertToBase,
  type LegBasis,
  type ValueLoadState,
} from "./transaction-values";
import { txnValueKey } from "./meta";
import type { Posting, Transaction } from "./types";

const WALLET = "assets:crypto:wallet:ethereum:0x6d25";
const BROKER = "assets:equity:broker:ibkr:personal";
const PERP_QTY = 985.0348891476083;
const PERP_AUD = 0.0248; // ~swap-derived AUD/PERP
const USD_AUD = 1.433; // USD.txt on 2026-10-15
const NET_USD = 316.32; // NET.txt daily close on 2026-10-15
const FILL_DT = "2026-10-15 00:00:00";

/** Base-currency rates the stub converter knows, per requested commodity. */
const RATES: Record<string, number> = { PERP: PERP_AUD, USD: USD_AUD, NET: NET_USD * USD_AUD };

/** Stub `convert_to_base_currency`: amount × the commodity's rate, null when unknown. */
const convertStub: ConvertToBase = async (_baseCurrency, requests) =>
  requests.map((r) => (r.commodity in RATES ? r.amount * RATES[r.commodity] : null));

/** A PERP disposal in WALLET (wallet leg + equity:trading:sell contra), no price
 *  annotation — so its value comes from a `convert_to_base_currency` call. */
function perpSale(): Transaction {
  return {
    date: "2026-06-26",
    datetime: "2026-06-26 00:23:23",
    status: "*",
    payee: "Uniswap",
    narration: "token_transfer:trade PERP",
    meta: "txn:0x34ed",
    postings: [
      { account: WALLET, amount: -PERP_QTY, amount_text: `${-PERP_QTY}`, commodity: "PERP" },
      { account: "equity:trading:sell", amount: PERP_QTY, amount_text: `${PERP_QTY}`, commodity: "PERP" },
    ],
    amount: -PERP_QTY,
    amount_commodity: "PERP",
  };
}

/** An IBKR fill as the transform writes it: the shares on the broker account
 *  with the fill's own `@@` total in USD, the cash on the :cash sub-account and
 *  the commission on expenses. With BROKER selected both asset legs are in
 *  scope; `valuationLegs` keeps the headline (shares) side. */
function netSell(txnId: string, shares: number, totalUsd: number): Transaction {
  return {
    date: "2026-10-15",
    datetime: FILL_DT,
    status: "*",
    payee: "Interactive Brokers",
    narration: `Sell -${shares}.0 NET @ ${(totalUsd / shares).toFixed(2)} USD`,
    meta: `txn:${txnId}`,
    postings: [
      {
        account: BROKER,
        amount: -shares,
        amount_text: `-${shares}.0`,
        commodity: "NET",
        price: { is_total: true, amount: totalUsd, amount_text: `${totalUsd}`, commodity: "USD" },
      },
      { account: `${BROKER}:cash`, amount: totalUsd - 0.64, amount_text: `${totalUsd - 0.64}`, commodity: "USD" },
      { account: "expenses:fees:brokerage", amount: 0.64, amount_text: "0.64", commodity: "USD" },
    ],
    amount: -shares,
    amount_commodity: "NET",
  };
}

function accountState(account: string, txns: Transaction[], all = txns): ValueLoadState {
  return {
    displayConfig: { base_currency: "AUD" },
    parse: { transactions: all },
    searchFilteredTransactions: txns,
    sidebarView: "accounts",
    selectedAccount: account,
    tradeLinks: [],
  };
}

const PRICED_USD: Posting = {
  account: BROKER,
  amount: -100,
  commodity: "NET",
  price: { is_total: true, amount: 31036, amount_text: "31036.0", commodity: "USD" },
};
const PRICED_AUD: Posting = {
  account: "assets:property:home",
  amount: 1,
  commodity: "LAND",
  price: { is_total: true, amount: 450000, amount_text: "450000", commodity: "AUD" },
};
const UNPRICED: Posting = { account: WALLET, amount: -PERP_QTY, commodity: "PERP" };

describe("planLegValue", () => {
  it("asks for the price currency's rate on the price value when a leg is priced in another currency", () => {
    const plan = planLegValue(PRICED_USD, FILL_DT, "AUD");
    expect(plan.direct).toBeNull();
    expect(plan.request).toEqual({ commodity: "USD", amount: 31036, datetime: FILL_DT });
    expect(plan.fallback).toBe(31036);
    expect(plan.sign).toBe(-1);
    expect(plan.basis).toMatchObject({
      commodity: "NET",
      amount: -100,
      datetime: FILL_DT,
      price: { value: 31036, commodity: "USD" },
      rateCommodity: "USD",
    });
  });

  it("needs no rate for a leg priced in the base currency", () => {
    const plan = planLegValue(PRICED_AUD, "2024-03-01 00:00:00", "AUD");
    expect(plan.direct).toBe(450000);
    expect(plan.request).toBeNull();
    expect(plan.basis).toMatchObject({ price: { value: 450000, commodity: "AUD" }, rateCommodity: null, resolved: true });
  });

  it("asks for the held commodity's rate on the quantity when a leg is unpriced", () => {
    const plan = planLegValue(UNPRICED, "2026-06-26 00:23:23", "AUD");
    expect(plan.direct).toBeNull();
    expect(plan.request).toEqual({ commodity: "PERP", amount: PERP_QTY, datetime: "2026-06-26 00:23:23" });
    expect(plan.fallback).toBeNull();
    expect(plan.basis).toMatchObject({ price: null, rateCommodity: "PERP" });
  });
});

describe("buildLegValueRequests", () => {
  it("emits one conversion request for an unpriced in-scope leg", () => {
    const plan = buildLegValueRequests([perpSale()], WALLET, "AUD");
    expect(plan.requests).toEqual([
      { commodity: "PERP", amount: PERP_QTY, datetime: "2026-06-26 00:23:23" },
    ]);
  });

  it("requests the fill currency on the price value, not the share quantity, for a USD-priced fill", () => {
    const plan = buildLegValueRequests([netSell("ibkr-tx-964750631", 100, 31036)], BROKER, "AUD");
    expect(plan.requests).toEqual([{ commodity: "USD", amount: 31036, datetime: FILL_DT }]);
  });
});

describe("applyLegValues", () => {
  it("values a fill at its own price converted at the day's currency rate (sell → negative)", () => {
    const txns = [netSell("ibkr-tx-964750631", 100, 31036)];
    const plan = buildLegValueRequests(txns, BROKER, "AUD");
    const result = applyLegValues(plan, txns, [31036 * USD_AUD]).get(txnValueKey(txns[0]));
    expect(result?.value).toBeCloseTo(-44474.59, 2);
    expect(result?.legs[0]).toMatchObject({ rateCommodity: "USD", resolved: true });
  });

  it("falls back to the unconverted price value when the currency rate is missing", () => {
    const txns = [netSell("ibkr-tx-964750631", 100, 31036)];
    const plan = buildLegValueRequests(txns, BROKER, "AUD");
    const result = applyLegValues(plan, txns, [null]).get(txnValueKey(txns[0]));
    expect(result?.value).toBe(-31036);
    expect(result?.legs[0].resolved).toBe(false);
  });

  it("records null, not 0, for an unpriced leg whose rate is missing", () => {
    const txns = [perpSale()];
    const plan = buildLegValueRequests(txns, WALLET, "AUD");
    const result = applyLegValues(plan, txns, [null]).get(txnValueKey(txns[0]));
    expect(result?.value).toBeNull();
    expect(result?.legs[0].resolved).toBe(false);
  });
});

describe("computeTransactionValues", () => {
  it("values an account's rows via the injected converter (disposal → negative)", async () => {
    const txns = [perpSale()];
    const result = await computeTransactionValues(accountState(WALLET, txns), convertStub);
    expect(result).toBeDefined();
    expect(result?.get(txnValueKey(txns[0]))?.value).toBeCloseTo(-PERP_QTY * PERP_AUD, 3);
  });

  it("returns undefined when no account is selected", async () => {
    const state: ValueLoadState = {
      displayConfig: { base_currency: "AUD" },
      parse: { transactions: [perpSale()] },
      sidebarView: "accounts",
      selectedAccount: undefined,
    };
    expect(await computeTransactionValues(state, convertStub)).toBeUndefined();
  });

  // REGRESSION GUARD for the empty "Value" column. In a windowed account/search
  // view `state.parse` is NOT populated — the rendered rows live in
  // `state.searchFilteredTransactions`, which is where the value-load now sources
  // from. Before the fix this failed (the load sourced from `state.parse` and
  // bailed on `!state.parse`), leaving every Value cell empty.
  it(
    "populates values from the on-screen (searchFiltered) rows when state.parse is empty",
    async () => {
      const txns = [perpSale()];
      const state: ValueLoadState = { ...accountState(WALLET, txns), parse: undefined };
      const result = await computeTransactionValues(state, convertStub);
      expect(result).toBeDefined();
      expect(result?.get(txnValueKey(txns[0]))?.value).toBeCloseTo(-PERP_QTY * PERP_AUD, 3);
    },
  );

  // REGRESSION GUARD for TASK-095: two 100-share fills on the same day at
  // different prices read the same 45,328.66 AUD (100 × the NET daily close ×
  // USD→AUD) instead of their own proceeds, which the CGT report lists as
  // 44,739.69 and 44,474.59 AUD.
  it("values two same-day fills at different prices as their own proceeds, not the daily close", async () => {
    const txns = [netSell("ibkr-tx-964748880", 100, 31221), netSell("ibkr-tx-964750631", 100, 31036)];
    const result = await computeTransactionValues(accountState(BROKER, txns), convertStub);
    expect(result?.get(txnValueKey(txns[0]))?.value).toBeCloseTo(-44739.69, 2);
    expect(result?.get(txnValueKey(txns[1]))?.value).toBeCloseTo(-44474.59, 2);
  });

  it("values a linked swap partner through the same leg planner (its own price, converted)", async () => {
    const sell: Transaction = {
      ...perpSale(),
      meta: "txn:A,swap:txn:B,swap_partner_commodity:ETH",
    };
    const buy: Transaction = {
      date: "2026-06-26",
      datetime: "2026-06-26 00:23:23",
      status: "*",
      payee: "Uniswap",
      narration: "token_transfer:trade ETH",
      meta: "txn:B,swap:txn:A,swap_partner_commodity:PERP",
      postings: [
        {
          account: WALLET,
          amount: 2,
          amount_text: "2",
          commodity: "ETH",
          price: { is_total: true, amount: 6000, amount_text: "6000", commodity: "USD" },
        },
        { account: "equity:trading:buy", amount: -2, amount_text: "-2", commodity: "ETH" },
      ],
      amount: 2,
      amount_commodity: "ETH",
    };
    const state: ValueLoadState = {
      ...accountState(WALLET, [sell], [sell, buy]),
      tradeLinks: [{ id: "l1", txn_id_a: "txn:A", txn_id_b: "txn:B" }],
    };
    // The stub knows no ETH rate: a quantity request for the partner would be
    // null and the sell would keep its own PERP value. Its USD price converts.
    const result = await computeTransactionValues(state, convertStub);
    const value = result?.get(txnValueKey(sell));
    expect(value?.value).toBeCloseTo(-6000 * USD_AUD, 2);
    expect(value?.legs).toHaveLength(1);
    expect(value?.legs[0]).toMatchObject({ commodity: "ETH", rateCommodity: "USD", resolved: true });
  });
});

function basis(overrides: Partial<LegBasis>): LegBasis {
  return {
    commodity: "NET",
    amount: -100,
    datetime: FILL_DT,
    price: { value: 31036, commodity: "USD" },
    rateCommodity: "USD",
    resolved: true,
    ...overrides,
  };
}

describe("describeValueBasis", () => {
  it("states a priced leg's own price and the currency rate it converted at", () => {
    expect(describeValueBasis({ value: -44474.59, legs: [basis({})] }, "AUD")).toBe(
      "100.00 NET priced at 31,036.00 USD, converted to AUD at the USD rate on 2026-10-15",
    );
  });

  it("says when the currency rate was missing and the price value is shown unconverted", () => {
    expect(describeValueBasis({ value: -31036, legs: [basis({ resolved: false })] }, "AUD")).toBe(
      "100.00 NET priced at 31,036.00 USD; no USD rate on 2026-10-15, so shown unconverted",
    );
  });

  it("states a base-priced leg's price with no rate", () => {
    const leg = basis({ commodity: "LAND", amount: 1, price: { value: 450000, commodity: "AUD" }, rateCommodity: null });
    expect(describeValueBasis({ value: 450000, legs: [leg] }, "AUD")).toBe("1.00 LAND priced at 450,000.00 AUD");
  });

  it("states an unpriced leg's quantity times its daily price, or that the price is missing", () => {
    const leg = basis({ commodity: "PERP", amount: -PERP_QTY, datetime: "2026-06-26 00:23:23", price: null, rateCommodity: "PERP" });
    expect(describeValueBasis({ value: -24.43, legs: [leg] }, "AUD")).toBe("985.03 PERP × the PERP price on 2026-06-26");
    expect(describeValueBasis({ value: null, legs: [{ ...leg, resolved: false }] }, "AUD")).toBe(
      "985.03 PERP; no PERP price on 2026-06-26",
    );
  });

  it("joins the legs of a co-held multi-commodity holding", () => {
    const land = basis({ commodity: "LAND", amount: 1, price: { value: 450000, commodity: "AUD" }, rateCommodity: null });
    const building = basis({ commodity: "BUILDING", amount: 1, price: { value: 300000, commodity: "AUD" }, rateCommodity: null });
    expect(describeValueBasis({ value: 750000, legs: [land, building] }, "AUD")).toBe(
      "1.00 LAND priced at 450,000.00 AUD + 1.00 BUILDING priced at 300,000.00 AUD",
    );
  });
});

describe("valueRateCommodity", () => {
  it("is the currency a priced leg converted through, the held commodity otherwise", () => {
    expect(valueRateCommodity({ value: -44474.59, legs: [basis({})] })).toBe("USD");
    expect(valueRateCommodity({ value: null, legs: [basis({ price: null, rateCommodity: "NET" })] })).toBe("NET");
    expect(valueRateCommodity({ value: 450000, legs: [basis({ commodity: "LAND", rateCommodity: null })] })).toBe("LAND");
    expect(valueRateCommodity({ value: null, legs: [] })).toBeNull();
  });
});
