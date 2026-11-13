import { describe, expect, it } from "vitest";
import {
  filterCategoryBalances,
  filterSidebarBalances,
  isClearingAccount,
  isIncludedInBalance,
  isOutsideBalance,
  nonFolderHoldings,
  transactionQueryFor,
  withPendingBalances,
  withPendingOwned,
} from "./sidebar-accounts";
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
            "liabilities:card", // owned but not folder-backed — only "Show all holdings" lists it
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

describe("filterSidebarBalances with Show all holdings", () => {
  const holdings: AccountBalance[] = [
    { account: "assets:crypto:wallet:sol", totals: [{ commodity: "SOL", amount: 10 }] }, // folder
    { account: "assets:crypto:staking:lido", totals: [{ commodity: "ETH", amount: 2 }] }, // non-folder holding
    { account: "assets:transfer", totals: [{ commodity: "AUD", amount: -193724.92 }] }, // clearing
    { account: "assets:transfer:cash", totals: [{ commodity: "AUD", amount: 5 }] }, // clearing sub-account
    { account: "assets:bank:suzi", totals: [{ commodity: "AUD", amount: 100 }] }, // suzi's folder
    { account: "liabilities:card:westpac", totals: [{ commodity: "AUD", amount: -900 }] }, // non-folder
    { account: "equity:opening-balance", totals: [{ commodity: "AUD", amount: -1 }] },
    { account: "income:salary", totals: [{ commodity: "AUD", amount: -5000 }] },
  ];
  const folders = {
    alex: { "assets:crypto:wallet:sol": "alex/crypto/wallet/sol" },
    suzi: { "assets:bank:suzi": "suzi/bank" },
  };
  const owners = {
    alex: [
      "assets:crypto:wallet:sol",
      "assets:crypto:staking:lido",
      "assets:transfer",
      "assets:transfer:cash",
      "assets:bank:suzi", // alex paid into Suzi's folder account
      "liabilities:card:westpac",
    ],
    suzi: ["assets:bank:suzi"],
  };
  const listed = (set: string | undefined, showAllHoldings: boolean) =>
    filterSidebarBalances(holdings, folders, owners, set, {
      showAllHoldings,
      clearingAccounts: ["assets:transfer"],
    }).map((b) => b.account);

  it("lists only folder accounts while it is off", () => {
    expect(listed("alex", false)).toEqual(["assets:crypto:wallet:sol"]);
  });

  it("adds the set's non-folder asset and liability holdings, never clearing accounts, other sets' folders or equity", () => {
    expect(listed("alex", true)).toEqual([
      "assets:crypto:wallet:sol",
      "assets:crypto:staking:lido",
      "liabilities:card:westpac",
    ]);
  });

  it("reveals every set's non-folder holdings when no set is selected", () => {
    expect(listed(undefined, true)).toEqual([
      "assets:crypto:wallet:sol",
      "assets:crypto:staking:lido",
      "assets:bank:suzi",
      "liabilities:card:westpac",
    ]);
  });

  it("never lists a clearing account, even a folder-backed one", () => {
    expect(
      filterSidebarBalances(holdings, folders, owners, "alex", { clearingAccounts: ["assets:crypto:wallet"] })
        .map((b) => b.account),
    ).toEqual([]);
  });
});

describe("nonFolderHoldings", () => {
  it("takes the set's owned assets and liabilities that no folder backs in any set", () => {
    const got = nonFolderHoldings(
      { alex: { "assets:cash:bank": "alex/cash/bank" }, suzi: { "assets:cash:savings": "suzi/cash/savings" } },
      { alex: ["assets:cash:bank", "assets:cash:savings", "assets:lending:aave", "assets:transfer:cash"] },
      "alex",
      ["assets:transfer"],
    );
    expect([...got]).toEqual(["assets:lending:aave"]);
  });
});

describe("isClearingAccount", () => {
  it("covers the entry and its sub-accounts by segment", () => {
    expect(isClearingAccount("assets:transfer", ["assets:transfer"])).toBe(true);
    expect(isClearingAccount("assets:transfer:cash", ["assets:transfer"])).toBe(true);
    expect(isClearingAccount("assets:transferwise", ["assets:transfer"])).toBe(false);
  });
});

describe("filterCategoryBalances", () => {
  const catBalances: AccountBalance[] = [
    { account: "assets:cash:bank:savings", totals: [{ commodity: "USD", amount: 10 }] }, // folder-backed
    { account: "assets:staking", totals: [{ commodity: "LFNTY", amount: 11296 }] }, // holding
    { account: "assets:transfer", totals: [{ commodity: "USD", amount: 500 }] }, // clearing
    { account: "income:crypto:staking", totals: [{ commodity: "SOL", amount: -3 }] },
    { account: "expenses:fees:trading", totals: [{ commodity: "USD", amount: -2 }] },
    { account: "expensesfoo", totals: [{ commodity: "USD", amount: 1 }] },
    { account: "equity:opening-balance", totals: [{ commodity: "USD", amount: -100 }] },
    { account: "liabilities:custody:james", totals: [{ commodity: "BTC", amount: -1 }] },
    { account: "ignore:spam", totals: [{ commodity: "SCAM", amount: 1000 }] },
  ];

  it("lists only income and expenses, never assets, liabilities or equity", () => {
    expect(filterCategoryBalances(catBalances, false).map((b) => b.account)).toEqual([
      "income:crypto:staking",
      "expenses:fees:trading",
    ]);
  });

  it("adds ignore:* while Show Ignored is on", () => {
    expect(filterCategoryBalances(catBalances, true).map((b) => b.account)).toEqual([
      "income:crypto:staking",
      "expenses:fees:trading",
      "ignore:spam",
    ]);
  });
});

describe("withPendingBalances", () => {
  it("adds a row with no totals for each pending account the balances lack", () => {
    expect(withPendingBalances(balances, ["expenses:pets", "expenses:pets", "liabilities:card"]))
      .toEqual([...balances, { account: "expenses:pets", totals: [] }]);
  });

  it("returns the balances themselves when nothing is pending or all are listed", () => {
    expect(withPendingBalances(balances, [])).toBe(balances);
    expect(withPendingBalances(balances, ["assets:cash"])).toBe(balances);
  });
});

describe("withPendingOwned", () => {
  it("adds the pending accounts to the active set's owned accounts", () => {
    expect(withPendingOwned({ alex: ["assets:a"], sam: ["assets:b"] }, "alex", ["assets:new"]))
      .toEqual({ alex: ["assets:a", "assets:new"], sam: ["assets:b"] });
  });

  it("returns the map itself when nothing is pending", () => {
    const map = { alex: ["assets:a"] };
    expect(withPendingOwned(map, "alex", [])).toBe(map);
  });
});

describe("transactionQueryFor", () => {
  const folderBacked = new Set(["assets:crypto:wallet:sol"]);
  const pane: AccountBalance[] = [
    { account: "assets:crypto:wallet:sol", totals: [] },
    { account: "assets:crypto:staking:lido", totals: [] },
    { account: "assets:cash:bank", totals: [] },
  ];

  it("reads a folder-only subtree through the folder-pruned query_search", () => {
    expect(transactionQueryFor("assets:crypto:wallet:sol", pane, folderBacked, "accounts")).toBe("query_search");
  });

  it("reads a revealed holding, or a group holding one, through query_global", () => {
    expect(transactionQueryFor("assets:crypto:staking:lido", pane, folderBacked, "accounts")).toBe("query_global");
    expect(transactionQueryFor("assets:crypto", pane, folderBacked, "accounts")).toBe("query_global");
  });

  it("reads a category through query_global", () => {
    const categories: AccountBalance[] = [{ account: "income:salary", totals: [] }];
    expect(transactionQueryFor("income", categories, folderBacked, "categories")).toBe("query_global");
  });

  it("lets the pane decide when nothing is selected", () => {
    expect(transactionQueryFor(undefined, pane, folderBacked, "accounts")).toBe("query_search");
    expect(transactionQueryFor(undefined, pane, folderBacked, "categories")).toBe("query_global");
  });
});

describe("isIncludedInBalance / isOutsideBalance", () => {
  const folderBacked = new Set(["assets:crypto:wallet:sol"]);
  const included = (a: string) => isIncludedInBalance(a, folderBacked, ["assets:crypto:lending"]);
  const leaf = (fullPath: string) => ({ fullPath, children: [] });

  it("counts folder accounts and accounts at or under an included prefix", () => {
    expect(included("assets:crypto:wallet:sol")).toBe(true);
    expect(included("assets:crypto:lending:aave")).toBe(true);
    expect(included("assets:crypto:lendingpool")).toBe(false);
    expect(included("assets:crypto:staking:lido")).toBe(false);
  });

  it("marks a revealed row, and a group holding only revealed rows, as outside the balance", () => {
    const staking = { fullPath: "assets:crypto:staking", children: [leaf("assets:crypto:staking:lido")] };
    const crypto = { fullPath: "assets:crypto", children: [leaf("assets:crypto:wallet:sol"), staking] };
    expect(isOutsideBalance(leaf("assets:crypto:staking:lido"), included)).toBe(true);
    expect(isOutsideBalance(staking, included)).toBe(true);
    expect(isOutsideBalance(crypto, included)).toBe(false);
    expect(isOutsideBalance(leaf("assets:crypto:lending:aave"), included)).toBe(false);
  });
});
