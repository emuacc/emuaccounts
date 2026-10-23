import { describe, expect, it } from "vitest";
import { filterSidebarBalances, filterCategoryBalances } from "./sidebar-accounts";
import type { AccountBalance } from "./types";

const balances: AccountBalance[] = [
  { account: "assets:cash:bank:savings", totals: [{ commodity: "USD", amount: 10 }] },
  { account: "assets:cash:bank:checking", totals: [{ commodity: "USD", amount: 20 }] },
  { account: "assets:cash", totals: [{ commodity: "USD", amount: 30 }] },
  { account: "liabilities:card", totals: [{ commodity: "USD", amount: -5 }] },
];

describe("filterSidebarBalances", () => {
  it("keeps only folder-backed asset accounts for the active set", () => {
    expect(
      filterSidebarBalances(
        balances,
        {
          alex: {
            "assets:cash:bank:savings": "alex/cash/bank/savings",
            "assets:cash:bank:checking": "alex/cash/bank/checking",
          },
        },
        {
          alex: [
            "assets:cash:bank:savings",
            "assets:cash:bank:checking",
            "assets:cash",
          ],
        },
        "alex",
      ).map((b) => b.account),
    ).toEqual([
      "assets:cash:bank:savings",
      "assets:cash:bank:checking",
    ]);
  });

  it("filters explicit-layout folder mappings identically", () => {
    expect(
      filterSidebarBalances(
        [{ account: "assets:crypto:wallet:ethereum", totals: [{ commodity: "ETH", amount: 1 }] }],
        { alex: { "assets:crypto:wallet:ethereum": "alex/assets/crypto/wallet/ethereum" } },
        { alex: ["assets:crypto:wallet:ethereum"] },
        "alex",
      ).map((b) => b.account),
    ).toEqual(["assets:crypto:wallet:ethereum"]);
  });

  it("keeps a declaration-only account with empty totals when folder-backed and owned", () => {
    const withEmpty: AccountBalance[] = [
      ...balances,
      { account: "assets:crypto:wallet:ethereum", totals: [] },
    ];
    expect(
      filterSidebarBalances(
        withEmpty,
        { alex: { "assets:crypto:wallet:ethereum": "alex/crypto/wallet/ethereum" } },
        { alex: ["assets:crypto:wallet:ethereum"] },
        "alex",
      ).map((b) => b.account),
    ).toEqual(["assets:crypto:wallet:ethereum"]);
  });

  it("keeps folder-backed liabilities beside the asset folders, never other roots", () => {
    const withLiabilities: AccountBalance[] = [
      ...balances,
      { account: "liabilities:credit-card:amex", totals: [{ commodity: "AUD", amount: -1200 }] },
      { account: "expenses:groceries", totals: [{ commodity: "AUD", amount: 1200 }] },
    ];
    expect(
      filterSidebarBalances(
        withLiabilities,
        {
          alex: {
            "assets:cash:bank:savings": "alex/cash/bank/savings",
            "liabilities:credit-card:amex": "alex/liabilities/credit-card/amex",
            // A folder mapping can never make a declaration-only root navigable.
            "expenses:groceries": "alex/_accounts",
          },
        },
        {
          alex: [
            "assets:cash:bank:savings",
            "liabilities:credit-card:amex",
            "liabilities:card", // owned but not folder-backed — stays in Categories
            "expenses:groceries",
          ],
        },
        "alex",
      ).map((b) => b.account),
    ).toEqual(["assets:cash:bank:savings", "liabilities:credit-card:amex"]);
  });

  it("falls back to every set's folder-backed assets when no set is selected", () => {
    expect(
      filterSidebarBalances(
        balances,
        {
          alex: { "assets:cash:bank:savings": "alex/cash/bank/savings" },
          acme: { "assets:cash:bank:checking": "acme/cash/bank/checking" },
        },
        {},
      ).map((b) => b.account),
    ).toEqual(["assets:cash:bank:savings", "assets:cash:bank:checking"]);
  });
});

describe("filterCategoryBalances", () => {
  const catBalances: AccountBalance[] = [
    { account: "assets:cash:bank:savings", totals: [{ commodity: "USD", amount: 10 }] }, // folder-backed
    { account: "assets:staking", totals: [{ commodity: "LFNTY", amount: 11296 }] }, // contra
    { account: "assets:lending", totals: [{ commodity: "USDC", amount: 500 }] }, // contra
    { account: "income:crypto:staking", totals: [{ commodity: "SOL", amount: -3 }] },
    { account: "expenses:fees:trading", totals: [{ commodity: "USD", amount: -2 }] },
    { account: "equity:opening-balance", totals: [{ commodity: "USD", amount: -100 }] },
    { account: "liabilities:custody:james", totals: [{ commodity: "BTC", amount: -1 }] },
    { account: "ignore:spam", totals: [{ commodity: "SCAM", amount: 1000 }] },
  ];
  const folders = { alex: { "assets:cash:bank:savings": "alex/cash/bank/savings" } };

  it("keeps non-folder-backed accounts and excludes ignore:* by default", () => {
    expect(
      filterCategoryBalances(catBalances, folders, false).map((b) => b.account),
    ).toEqual([
      "assets:staking",
      "assets:lending",
      "income:crypto:staking",
      "expenses:fees:trading",
      "equity:opening-balance",
      "liabilities:custody:james",
    ]);
  });

  it("includes ignore:* when showHidden is on", () => {
    expect(
      filterCategoryBalances(catBalances, folders, true).map((b) => b.account),
    ).toContain("ignore:spam");
  });

  it("never includes folder-backed asset accounts (the Accounts pane owns them)", () => {
    expect(
      filterCategoryBalances(catBalances, folders, true).map((b) => b.account),
    ).not.toContain("assets:cash:bank:savings");
  });
});
