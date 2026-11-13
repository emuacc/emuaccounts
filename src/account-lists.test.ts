// TASK-148: the sidebar's Accounts and Categories panes and the account
// dropdowns follow a change on both paths — the main refresh re-reads them from
// disk, and an account the app is writing is listed at once, before it.

import { describe, expect, it, vi } from "vitest";
import { creatingAccounts, loadAccountLists, type AccountListsState } from "./account-lists";
import { collectAccountSuggestions } from "./account-suggestions";
import { MutationQueue } from "./mutation-queue";
import { filterCategoryBalances, filterSidebarBalances, withPendingBalances, withPendingOwned } from "./sidebar-accounts";
import type { AccountBalance } from "./types";

vi.mock("./issues", () => ({ refreshIssuesCache: vi.fn().mockResolvedValue(undefined) }));

const aud = (amount: number) => [{ commodity: "AUD", amount }];

/** What load_account_tree reads off disk, per set ("" = the whole vault). */
type Vault = Record<string, AccountBalance[]>;

function fakeInvoke(vault: () => Vault, failing: string[] = []) {
  return vi.fn(async (cmd: string, args?: Record<string, unknown>): Promise<any> => {
    if (cmd === "load_account_tree") {
      const set = String(args?.accountSet ?? "");
      if (failing.includes(set)) throw new Error(`cannot read ${set || "vault"}`);
      return vault()[set] ?? [];
    }
    if (cmd === "load_pipeline_metadata") return { owner_accounts: { alex: ["assets:bank"] } };
    if (cmd === "get_hidden_accounts") {
      if (failing.includes("hidden")) throw new Error("cannot read config.json");
      return args?.accountSet === "alex" ? ["ignore:"] : [];
    }
    throw new Error(`unexpected ${cmd}`);
  });
}

function makeState(balances: AccountBalance[], allAccounts: string[]): AccountListsState & { pendingMutations: number } {
  return {
    parse: { ok: true, diagnostics: [], transactions: [], balances, accounts_with_opening: [], account_properties: {} },
    selectedAccountSet: "alex",
    selectedAccount: "assets:bank",
    allAccounts,
    pendingAccounts: [],
    hiddenAccounts: [],
    pipelineWarnings: [],
    accountSetMap: { alex: ["assets:bank"] },
    accountFoldersMap: { alex: { "assets:bank": "alex/bank" } },
    accountPropertiesMap: {},
    accountsWithOpening: [],
    pendingMutations: 0,
  };
}

/** The Categories pane and the account dropdowns, read the way main.ts reads them. */
function categoriesPane(state: AccountListsState): string[] {
  return filterCategoryBalances(withPendingBalances(state.parse?.balances ?? [], state.pendingAccounts), false)
    .map((b) => b.account);
}
function dropdown(state: AccountListsState): string[] {
  return collectAccountSuggestions({
    accountSetMap: state.accountSetMap,
    balances: state.parse?.balances,
    allAccounts: state.allAccounts,
    pendingAccounts: state.pendingAccounts,
  });
}

describe("the main refresh (loadAccountLists)", () => {
  it("lists a new account and drops a renamed one in the sidebar and the dropdowns", async () => {
    // A rule, a sync or an edited source file renamed expenses:coffee to expenses:cafe.
    const state = makeState(
      [{ account: "assets:bank", totals: aud(100) }, { account: "expenses:coffee", totals: aud(4.5) }],
      ["assets:bank", "expenses:coffee"],
    );
    const onDisk: AccountBalance[] = [
      { account: "assets:bank", totals: aud(100) },
      { account: "expenses:cafe", totals: aud(4.5) },
    ];

    await loadAccountLists(state, fakeInvoke(() => ({ alex: onDisk, "": onDisk })));

    expect(categoriesPane(state)).toEqual(["expenses:cafe"]);
    expect(dropdown(state)).toContain("expenses:cafe");
    expect(dropdown(state)).not.toContain("expenses:coffee");
  });

  it("after a mutation reads the active set's tree once and keeps the pool current from it", async () => {
    const state = makeState([{ account: "assets:bank", totals: aud(100) }], ["assets:bank", "assets:other-set"]);
    const onDisk: AccountBalance[] = [
      { account: "assets:bank", totals: aud(100) },
      { account: "expenses:cafe", totals: aud(4.5) },
    ];
    const invoke = fakeInvoke(() => ({ alex: onDisk, "": onDisk }));

    await loadAccountLists(state, invoke, { pool: false });

    expect(invoke.mock.calls.filter(([cmd]) => cmd === "load_account_tree")).toHaveLength(1);
    expect(dropdown(state)).toContain("expenses:cafe");
    // Another set's account is not this set's to drop.
    expect(state.allAccounts).toContain("assets:other-set");
  });

  it("reads the active set's tree for the sidebar and the whole vault's for the dropdowns", async () => {
    const state = makeState([], []);
    const invoke = fakeInvoke(() => ({
      alex: [{ account: "assets:bank", totals: aud(1) }],
      "": [{ account: "assets:bank", totals: aud(1) }, { account: "income:dividends", totals: aud(-2) }],
    }));

    await loadAccountLists(state, invoke);

    expect(invoke).toHaveBeenCalledWith("load_account_tree", { accountSet: "alex" });
    expect(invoke).toHaveBeenCalledWith("load_account_tree", { accountSet: "" });
    expect(state.parse?.balances.map((b) => b.account)).toEqual(["assets:bank"]);
    expect(state.allAccounts).toEqual(["assets:bank", "income:dividends"]);
  });

  it("keeps the previous dropdown pool when only the pool can't be read", async () => {
    const state = makeState([], ["expenses:old"]);

    await loadAccountLists(state, fakeInvoke(() => ({ alex: [{ account: "assets:bank", totals: aud(1) }] }), [""]));

    expect(state.allAccounts).toEqual(["expenses:old"]);
    expect(state.parse?.balances.map((b) => b.account)).toEqual(["assets:bank"]);
  });

  it("reads the accounts the active set hides, keeping the previous ones when they can't be read", async () => {
    const state = makeState([], []);
    const vault = () => ({ alex: [{ account: "assets:bank", totals: aud(1) }] });

    await loadAccountLists(state, fakeInvoke(vault));
    expect(state.hiddenAccounts).toEqual(["ignore:"]);

    await loadAccountLists(state, fakeInvoke(vault, ["hidden"]));
    expect(state.hiddenAccounts).toEqual(["ignore:"]);
  });

  it("throws when the sidebar tree can't be read, for the caller to report", async () => {
    const state = makeState([], []);
    await expect(loadAccountLists(state, fakeInvoke(() => ({}), ["alex"]))).rejects.toThrow("cannot read alex");
  });
});

describe("an account the app is writing (creatingAccounts)", () => {
  it("is listed in the sidebar and the dropdowns at once, and gets its totals from the refresh", async () => {
    const state = makeState([{ account: "assets:bank", totals: aud(100) }], ["assets:bank"]);
    const onDisk: AccountBalance[] = [{ account: "assets:bank", totals: aud(100) }];
    const invoke = fakeInvoke(() => ({ alex: onDisk, "": onDisk }));
    let finishRebuild = () => {};
    const rebuilt = new Promise<void>((resolve) => { finishRebuild = resolve; });
    const queue = new MutationQueue({ render: () => {} });

    // The inline category edit: save a rule posting to a category nothing uses yet.
    const done = queue.enqueue(state, "Saving rule...", creatingAccounts(state, ["expenses:pets"], async () => {
      await rebuilt;
      onDisk.push({ account: "expenses:pets", totals: aud(30) });
      await loadAccountLists(state, invoke);
    }));

    // Before the rebuild: listed, with no totals yet.
    expect(categoriesPane(state)).toEqual(["expenses:pets"]);
    expect(withPendingBalances(state.parse!.balances, state.pendingAccounts))
      .toContainEqual({ account: "expenses:pets", totals: [] });
    expect(dropdown(state)).toContain("expenses:pets");

    finishRebuild();
    await done;

    // After the refresh: the real row, with its totals, and nothing pending.
    expect(state.pendingAccounts).toEqual([]);
    expect(state.parse?.balances).toContainEqual({ account: "expenses:pets", totals: aud(30) });
    expect(categoriesPane(state)).toEqual(["expenses:pets"]);
    expect(dropdown(state)).toContain("expenses:pets");
  });

  it("is taken away again when the mutation fails", async () => {
    const state = makeState([{ account: "assets:bank", totals: aud(100) }], ["assets:bank"]);
    const queue = new MutationQueue({ render: () => {} });

    const failed = queue.enqueue(state, "Adding transaction…", creatingAccounts(state, ["assets:bank", "expenses:pets"], async () => {
      throw new Error("pipeline rebuild failed");
    }));
    expect(categoriesPane(state)).toEqual(["expenses:pets"]);

    await expect(failed).rejects.toThrow("pipeline rebuild failed");
    expect(state.pendingAccounts).toEqual([]);
    expect(categoriesPane(state)).toEqual([]);
    expect(dropdown(state)).not.toContain("expenses:pets");
  });

  it("stays listed while another mutation in flight still writes it", async () => {
    const state = makeState([], []);
    let finishSecond = () => {};
    const second = new Promise<void>((resolve) => { finishSecond = resolve; });

    const first = creatingAccounts(state, ["expenses:pets"], async () => {});
    const other = creatingAccounts(state, ["expenses:pets"], () => second);
    await first();
    expect(state.pendingAccounts).toEqual(["expenses:pets"]);

    finishSecond();
    await other();
    expect(state.pendingAccounts).toEqual([]);
  });

  it("ignores blank names: a rule with no fee account, an emptied cell", () => {
    const state = makeState([], []);
    creatingAccounts(state, [" expenses:pets ", null, "", undefined, "expenses:pets"], async () => {});
    expect(state.pendingAccounts).toEqual(["expenses:pets"]);
  });

  it("shows a new holding in the Accounts pane at once while Show all holdings is on", () => {
    // A manual transaction posting to a holding no source folder backs.
    const state = makeState([{ account: "assets:bank", totals: aud(100) }], ["assets:bank"]);
    creatingAccounts(state, ["assets:loans:bob"], async () => {});

    const pane = filterSidebarBalances(
      withPendingBalances(state.parse!.balances, state.pendingAccounts),
      state.accountFoldersMap,
      withPendingOwned(state.accountSetMap, state.selectedAccountSet, state.pendingAccounts),
      state.selectedAccountSet,
      { showAllHoldings: true },
    );

    expect(pane.map((b) => b.account)).toEqual(["assets:bank", "assets:loans:bob"]);
  });
});
