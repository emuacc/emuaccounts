import { describe, it, expect } from "vitest";
import {
  HIDDEN_ACCOUNT,
  addOptimisticEdit,
  applyOptimisticEdits,
  changedRowKeys,
  predictedRows,
  revertOptimisticEdit,
  postsToHiddenAccount,
  rowsNotApplied,
  setShowIgnored,
  settleOptimisticEdit,
  txnRowKey,
  type EditView,
  type NewOptimisticEdit,
  type OptimisticState,
  type ShowIgnoredDeps,
  type ShowIgnoredState,
} from "./optimistic-edits";
import type { RuleInfo } from "./rules";
import type { Transaction } from "./types";

const BANK = "assets:bank:everyday";

function txn(id: string, payee: string, narration: string, contra: string, amount = -10, extraMeta = ""): Transaction {
  return {
    date: "2026-02-01",
    datetime: `2026-02-01T09:00:0${id.length % 10}Z`,
    payee,
    narration,
    meta: `txn:${id}${extraMeta}`,
    postings: [
      { account: BANK, amount, commodity: "AUD" },
      { account: contra, amount: -amount, commodity: "AUD" },
    ],
    amount,
    amount_commodity: "AUD",
  };
}

const coffee = txn("coffee", "Coffee Club", "Card purchase COFFEE CLUB", "expenses:unknown", -4.5);
const grocer = txn("grocer", "Grocer", "Card purchase GROCER", "expenses:unknown", -52.1);
const salary = txn("salary", "Employer", "Salary", "income:salary", 2000);
const rows = [coffee, grocer, salary];

const FOLDER = "e2e/bank/everyday";
const accountView: EditView = { selectedAccount: BANK, sourceAccountOf: () => BANK, sourceFolderOf: () => FOLDER };

function categoryOf(t: Transaction | undefined): string | undefined {
  return t?.postings.find((p) => p.account !== BANK)?.account;
}

function byPayee(list: Transaction[], payee: string): Transaction | undefined {
  return list.find((t) => t.payee === payee);
}

const recategoriseCoffee: NewOptimisticEdit = {
  field: "category", value: "expenses:food", label: "Category", target: { by: "row", legId: "", txnId: "txn:coffee" },
};

describe("txnRowKey", () => {
  it("prefers the per-leg id", () => {
    expect(txnRowKey({ ...coffee, meta: "txn:coffee,leg:L1" })).toBe("leg:L1");
  });

  it("falls back to txn id + datetime + amount + narration", () => {
    expect(txnRowKey(coffee)).toBe(`txn:coffee|${coffee.datetime}|-4.5|Card purchase COFFEE CLUB`);
  });

  it("is empty for a row with no id", () => {
    expect(txnRowKey({ ...coffee, meta: "" })).toBe("");
  });
});

describe("applyOptimisticEdits", () => {
  it("returns the rows untouched when nothing is pending", () => {
    expect(applyOptimisticEdits(rows, [], accountView)).toBe(rows);
  });

  it("re-categorises the edited row only, without mutating the loaded rows", () => {
    const out = applyOptimisticEdits(rows, [{ ...recategoriseCoffee, id: 1 }], accountView);
    expect(categoryOf(byPayee(out, "Coffee Club"))).toBe("expenses:food");
    expect(categoryOf(byPayee(out, "Grocer"))).toBe("expenses:unknown");
    expect(categoryOf(coffee)).toBe("expenses:unknown");
  });

  it("targets one leg by its leg id when sibling legs share a txn id", () => {
    const a = { ...coffee, meta: "txn:swap,leg:A" };
    const b = { ...grocer, meta: "txn:swap,leg:B" };
    const out = applyOptimisticEdits([a, b], [{ ...recategoriseCoffee, id: 1, target: { by: "row", legId: "leg:B", txnId: "txn:swap" } }], accountView);
    expect(categoryOf(out[0])).toBe("expenses:unknown");
    expect(categoryOf(out[1])).toBe("expenses:food");
  });

  it("re-categorises every in-scope row a saved rule matches", () => {
    const rule: RuleInfo = { id: "", pattern: "*Card purchase*", match_field: "narration", amount_account: "expenses:shopping" };
    const out = applyOptimisticEdits(rows, [{
      id: 1, field: "category", value: "expenses:shopping", label: "Rule",
      target: { by: "rule", rule, allRules: [], scopeFolder: FOLDER },
    }], accountView);
    expect(categoryOf(byPayee(out, "Coffee Club"))).toBe("expenses:shopping");
    expect(categoryOf(byPayee(out, "Grocer"))).toBe("expenses:shopping");
    expect(categoryOf(byPayee(out, "Employer"))).toBe("income:salary");
  });

  it("leaves rows a higher-priority rule already claims", () => {
    const claimed: RuleInfo = { id: "r1", pattern: "*GROCER*", match_field: "narration", amount_account: "expenses:groceries" };
    const rule: RuleInfo = { id: "", pattern: "*Card purchase*", match_field: "narration", amount_account: "expenses:shopping" };
    const out = applyOptimisticEdits(rows, [{
      id: 1, field: "category", value: "expenses:shopping", label: "Rule",
      target: { by: "rule", rule, allRules: [claimed], scopeFolder: FOLDER },
    }], accountView);
    expect(categoryOf(byPayee(out, "Coffee Club"))).toBe("expenses:shopping");
    expect(categoryOf(byPayee(out, "Grocer"))).toBe("expenses:unknown");
  });

  it("applies a rule only to rows imported into its folder or below", () => {
    const rule: RuleInfo = { id: "", pattern: "*", match_field: "narration", amount_account: "expenses:shopping" };
    const edit = (scopeFolder: string) => [{
      id: 1, field: "category" as const, value: "expenses:shopping", label: "Rule",
      target: { by: "rule" as const, rule, allRules: [], scopeFolder },
    }];
    expect(applyOptimisticEdits(rows, edit("e2e/bank/other"), accountView).map(categoryOf)).toEqual(rows.map(categoryOf));
    // "e2e/bank/every" is a name prefix of the row's folder, not a parent of it.
    expect(applyOptimisticEdits(rows, edit("e2e/bank/every"), accountView).map(categoryOf)).toEqual(rows.map(categoryOf));
    expect(categoryOf(applyOptimisticEdits(rows, edit("e2e/bank"), accountView)[0])).toBe("expenses:shopping");
    expect(categoryOf(applyOptimisticEdits(rows, edit(""), accountView)[0])).toBe("expenses:shopping");
  });

  it("matches an id-anchored meta rule on its meta segment", () => {
    const rule: RuleInfo = { id: "r1", pattern: "txn:grocer", match_field: "meta", amount_account: "expenses:food" };
    const out = applyOptimisticEdits(rows, [{
      id: 1, field: "category", value: "expenses:food", label: "Rule",
      target: { by: "rule", rule, allRules: [rule], scopeFolder: "" },
    }], accountView);
    expect(categoryOf(byPayee(out, "Grocer"))).toBe("expenses:food");
    expect(categoryOf(byPayee(out, "Coffee Club"))).toBe("expenses:unknown");
  });

  it("renames the party on every row a payee label matches", () => {
    const label: RuleInfo = { id: "", pattern: "*Grocer*", match_field: "payee", payee: "Local Grocer" };
    const out = applyOptimisticEdits(rows, [{
      id: 1, field: "payee", value: "Local Grocer", label: "Label", target: { by: "label", label, scopeFolder: "e2e/bank" },
    }], accountView);
    expect(byPayee(out, "Grocer")?.display_payee).toBe("Local Grocer");
    expect(byPayee(out, "Coffee Club")?.display_payee).toBeUndefined();
  });

  it("skips rows an earlier label in the chain already renames", () => {
    const earlier: RuleInfo = { id: "l0", pattern: "*Grocer*", match_field: "payee", payee: "Corner Shop" };
    const label: RuleInfo = { id: "", pattern: "*Grocer*", match_field: "payee", payee: "Local Grocer" };
    const out = applyOptimisticEdits(rows, [{
      id: 1, field: "payee", value: "Local Grocer", label: "Label",
      target: { by: "label", label, scopeFolder: "", chain: [{ folder: "e2e/bank/everyday", label: earlier }], insertIndex: 1 },
    }], accountView);
    expect(byPayee(out, "Grocer")?.display_payee).toBeUndefined();
  });

  it("drops every row of a removed (hidden) txn", () => {
    const out = applyOptimisticEdits(rows, [{
      id: 1, field: "remove", value: "", label: "Hide", target: { by: "txn", txnId: "txn:grocer" },
    }], accountView);
    expect(out.map((t) => t.payee)).toEqual(["Coffee Club", "Employer"]);
  });

  it("under Show Ignored keeps a hidden row, every leg on ignore:hidden", () => {
    const out = applyOptimisticEdits(rows, [{
      id: 1, field: "hide", value: "", label: "Hide", target: { by: "txn", txnId: "txn:grocer" },
    }], accountView);
    expect(out).toHaveLength(3);
    expect(byPayee(out, "Grocer")?.postings.map((p) => p.account)).toEqual([HIDDEN_ACCOUNT, HIDDEN_ACCOUNT]);
  });

  it("rewrites the imported account's contra, and drops the row once it leaves the category viewed", () => {
    // Categories pane on expenses:unknown: the row's displayed contra is the
    // bank leg, but the rule rewrites the expense leg — so the row moves away.
    const view: EditView = { selectedAccount: "expenses:unknown", sourceAccountOf: () => BANK, sourceFolderOf: () => FOLDER };
    const edits = [{ ...recategoriseCoffee, id: 1 }];
    expect(applyOptimisticEdits(rows, edits, view).map((t) => t.payee)).toEqual(["Grocer", "Employer"]);
    const [patched] = predictedRows(edits[0], rows, view).values();
    expect(patched?.postings.map((p) => p.account)).toEqual([BANK, "expenses:food"]);
  });

  it("leaves a trade-shaped row alone", () => {
    const trade: Transaction = {
      ...coffee,
      postings: [
        { account: "assets:broker", amount: 10, commodity: "VAS" },
        { account: "assets:broker", amount: -1000, commodity: "AUD" },
        { account: "expenses:fees:brokerage", amount: 10, commodity: "AUD" },
      ],
    };
    const view: EditView = { selectedAccount: "assets:broker", sourceAccountOf: () => "assets:broker", sourceFolderOf: () => "e2e/broker" };
    const out = applyOptimisticEdits([trade], [{ ...recategoriseCoffee, id: 1 }], view);
    expect(out[0]).toBe(trade);
  });
});

describe("changedRowKeys", () => {
  it("reports rows whose content changed and rows new to the window", () => {
    const recategorised = { ...grocer, postings: [grocer.postings[0], { ...grocer.postings[1], account: "expenses:food" }] };
    const extra = txn("new", "Bakery", "Card purchase BAKERY", "expenses:food");
    const changed = changedRowKeys([coffee, grocer], [coffee, extra, recategorised]);
    expect([...changed].sort()).toEqual([txnRowKey(extra), txnRowKey(grocer)].sort());
  });

  it("does not count rows the re-fetch pages in after the last row the window had", () => {
    // Grocer left (hidden); the re-fetched window is filled from the next page.
    const nextPage = txn("older", "Bakery", "Card purchase BAKERY", "expenses:food");
    expect(changedRowKeys([coffee, salary], [coffee, salary, nextPage]).size).toBe(0);
  });

  it("reports nothing when the window was empty before", () => {
    expect(changedRowKeys([], rows).size).toBe(0);
  });
});

describe("rowsNotApplied", () => {
  const edit = { ...recategoriseCoffee, id: 1 };

  it("is empty when the rebuilt row carries the new category", () => {
    const predicted = predictedRows(edit, rows, accountView);
    const rebuilt = applyOptimisticEdits(rows, [edit], accountView);
    expect(rowsNotApplied(edit, predicted, rebuilt, accountView).size).toBe(0);
  });

  it("reports a row the rebuild left uncategorised", () => {
    const predicted = predictedRows(edit, rows, accountView);
    expect([...rowsNotApplied(edit, predicted, rows, accountView)]).toEqual([txnRowKey(coffee)]);
  });

  it("does not count a row that left the rebuilt window", () => {
    const predicted = predictedRows(edit, rows, accountView);
    expect(rowsNotApplied(edit, predicted, [grocer, salary], accountView).size).toBe(0);
  });

  it("reports a removed row that is still there", () => {
    const hide = { id: 2, field: "remove" as const, value: "", label: "Hide", target: { by: "txn" as const, txnId: "txn:grocer" } };
    const predicted = predictedRows(hide, rows, accountView);
    expect([...rowsNotApplied(hide, predicted, rows, accountView)]).toEqual([txnRowKey(grocer)]);
    expect(rowsNotApplied(hide, predicted, [coffee, salary], accountView).size).toBe(0);
  });

  it("checks that a hidden row now posts to ignore:hidden", () => {
    const hide = { id: 4, field: "hide" as const, value: "", label: "Hide", target: { by: "txn" as const, txnId: "txn:grocer" } };
    const predicted = predictedRows(hide, rows, accountView);
    const hidden = applyOptimisticEdits(rows, [hide], accountView);
    expect(rowsNotApplied(hide, predicted, hidden, accountView).size).toBe(0);
    expect([...rowsNotApplied(hide, predicted, rows, accountView)]).toEqual([txnRowKey(grocer)]);
  });

  it("checks the displayed party for a payee edit", () => {
    const label: RuleInfo = { id: "", pattern: "*Grocer*", match_field: "payee", payee: "Local Grocer" };
    const rename = { id: 3, field: "payee" as const, value: "Local Grocer", label: "Label", target: { by: "label" as const, label, scopeFolder: "" } };
    const predicted = predictedRows(rename, rows, accountView);
    expect(rowsNotApplied(rename, predicted, [{ ...grocer, display_payee: "Local Grocer" }], accountView).size).toBe(0);
    expect(rowsNotApplied(rename, predicted, [grocer], accountView).size).toBe(1);
  });
});

describe("the optimistic edit lifecycle", () => {
  function setup(initial: Transaction[]) {
    const state: OptimisticState = { pendingEdits: [] };
    let loaded = initial;
    let next: Transaction[] | Error = initial;
    let viewKey = "assets:bank:everyday";
    let nextViewKey = viewKey;
    const deps = {
      rows: () => loaded,
      view: () => accountView,
      viewKey: () => viewKey,
      refresh: async () => {
        viewKey = nextViewKey;
        if (next instanceof Error) throw next;
        loaded = next;
      },
    };
    return {
      state, deps,
      rebuildTo: (r: Transaction[] | Error) => { next = r; },
      navigateDuringRebuild: (key: string) => { nextViewKey = key; },
    };
  }

  it("shows the edit at once and flashes the row it changes", () => {
    const { state } = setup(rows);
    const edit = addOptimisticEdit(state, recategoriseCoffee, rows, accountView);
    expect(state.pendingEdits).toEqual([edit]);
    expect([...state.rowFlashes!.changed]).toEqual([txnRowKey(coffee)]);
    expect(categoryOf(byPayee(applyOptimisticEdits(rows, state.pendingEdits, accountView), "Coffee Club"))).toBe("expenses:food");
  });

  it("settles a correct prediction without flashing the row again", async () => {
    const { state, deps, rebuildTo } = setup(rows);
    const edit = addOptimisticEdit(state, recategoriseCoffee, rows, accountView);
    state.rowFlashes = undefined; // the immediate flash was rendered
    rebuildTo(applyOptimisticEdits(rows, [edit], accountView));
    expect(await settleOptimisticEdit(state, edit, deps)).toEqual({ missed: 0, checked: 1 });
    expect(state.pendingEdits).toEqual([]);
    expect(state.rowFlashes).toBeUndefined();
  });

  it("fades rows the rebuild changed beyond the prediction", async () => {
    const { state, deps, rebuildTo } = setup(rows);
    const edit = addOptimisticEdit(state, recategoriseCoffee, rows, accountView);
    state.rowFlashes = undefined;
    const alsoGrocer = applyOptimisticEdits(rows, [edit, { ...recategoriseCoffee, id: 9, target: { by: "txn", txnId: "txn:grocer" } }], accountView);
    rebuildTo(alsoGrocer);
    expect((await settleOptimisticEdit(state, edit, deps)).missed).toBe(0);
    expect([...state.rowFlashes!.changed]).toEqual([txnRowKey(grocer)]);
  });

  it("flashes and counts a row the rebuild did not change", async () => {
    const { state, deps, rebuildTo } = setup(rows);
    const edit = addOptimisticEdit(state, recategoriseCoffee, rows, accountView);
    state.rowFlashes = undefined;
    rebuildTo(rows); // the rule lost to another one
    expect(await settleOptimisticEdit(state, edit, deps)).toEqual({ missed: 1, checked: 1 });
    expect([...state.rowFlashes!.reverted]).toEqual([txnRowKey(coffee)]);
    expect(state.rowFlashes!.changed.size).toBe(0);
    expect(state.pendingEdits).toEqual([]);
  });

  it("neither flashes nor checks when the user moved to another list meanwhile", async () => {
    const { state, deps, rebuildTo, navigateDuringRebuild } = setup(rows);
    const edit = addOptimisticEdit(state, recategoriseCoffee, rows, accountView);
    state.rowFlashes = undefined;
    navigateDuringRebuild("income:salary");
    rebuildTo([salary]);
    expect(await settleOptimisticEdit(state, edit, deps)).toEqual({ missed: 0, checked: 0 });
    expect(state.rowFlashes).toBeUndefined();
    expect(state.pendingEdits).toEqual([]);
  });

  it("still retires the edit when the in-place refresh throws", async () => {
    const { state, deps, rebuildTo } = setup(rows);
    const edit = addOptimisticEdit(state, recategoriseCoffee, rows, accountView);
    rebuildTo(new Error("query failed"));
    await expect(settleOptimisticEdit(state, edit, deps)).rejects.toThrow("query failed");
    expect(state.pendingEdits).toEqual([]);
  });

  it("withdraws a failed edit and flashes the row that goes back", async () => {
    const { state, deps } = setup(rows);
    const edit = addOptimisticEdit(state, recategoriseCoffee, rows, accountView);
    state.rowFlashes = undefined;
    await revertOptimisticEdit(state, edit, deps);
    expect(state.pendingEdits).toEqual([]);
    expect([...state.rowFlashes!.reverted]).toEqual([txnRowKey(coffee)]);
  });

  it("brings a hidden row back when hiding fails, even if the refresh fails too", async () => {
    const { state, deps, rebuildTo } = setup(rows);
    const hide = addOptimisticEdit(state, { field: "remove", value: "", label: "Hide", target: { by: "txn", txnId: "txn:grocer" } }, rows, accountView);
    expect(applyOptimisticEdits(rows, state.pendingEdits, accountView)).toHaveLength(2);
    rebuildTo(new Error("offline"));
    await revertOptimisticEdit(state, hide, deps);
    expect(applyOptimisticEdits(rows, state.pendingEdits, accountView)).toHaveLength(3);
    expect([...state.rowFlashes!.reverted]).toEqual([txnRowKey(grocer)]);
  });
});

describe("Show Ignored, in place (setShowIgnored)", () => {
  const hidden = txn("grocer", "Grocer", "Card purchase GROCER", HIDDEN_ACCOUNT, -52.1);
  const ignoredFee = txn("fee", "Bank", "Fee", "ignore:fees", -1);

  /** The backend: which rows its queries return for the current setting. */
  function setup(onDisk: Transaction[], loaded: Transaction[]) {
    const calls: string[] = [];
    let backendShows = false;
    const state: ShowIgnoredState = {
      pendingEdits: [],
      showHidden: loaded.some((t) => postsToHiddenAccount(t, ["ignore:"])),
      hiddenAccounts: ["ignore:"],
      searchFilteredTransactions: loaded,
      searchFilteredCount: loaded.length,
    };
    const rendered: string[][] = [];
    let key = "assets:bank:everyday";
    const deps: ShowIgnoredDeps = {
      apply: async (show) => { calls.push(`apply ${show}`); backendShows = show; },
      refresh: async () => {
        calls.push("refresh");
        const rows = backendShows ? onDisk : onDisk.filter((t) => !postsToHiddenAccount(t, state.hiddenAccounts));
        state.searchFilteredTransactions = rows;
        state.searchFilteredCount = rows.length;
      },
      render: () => rendered.push((state.searchFilteredTransactions ?? []).map((t) => t.payee ?? "")),
      viewKey: () => `${key}|${state.showHidden}`,
    };
    return { state, deps, calls, rendered, navigate: (next: string) => { key = next; } };
  }

  it("matches a row on any leg under a hidden prefix, as the backend's queries do", () => {
    expect(postsToHiddenAccount(hidden, ["ignore:"])).toBe(true);
    expect(postsToHiddenAccount(coffee, ["ignore:"])).toBe(false);
    expect(postsToHiddenAccount(ignoredFee, ["ignore:hidden"])).toBe(false);
    expect(postsToHiddenAccount(hidden, [])).toBe(false);
  });

  it("drops the ignored rows in the first render when turned off, before the backend hears of it", async () => {
    const onDisk = [coffee, hidden, salary, ignoredFee];
    const { state, deps, calls, rendered } = setup(onDisk, onDisk);

    await setShowIgnored(state, false, deps);

    expect(rendered[0]).toEqual(["Coffee Club", "Employer"]);
    expect(calls).toEqual(["apply false", "refresh"]);
    expect(state.showHidden).toBe(false);
    expect(state.searchFilteredTransactions).toEqual([coffee, salary]);
    expect(state.rowFlashes).toBeUndefined();
  });

  it("lowers the row count with the rows it drops", () => {
    const { state, deps } = setup([coffee, hidden, salary], [coffee, hidden, salary]);
    state.searchFilteredCount = 250;
    void setShowIgnored(state, false, deps);
    expect(state.searchFilteredCount).toBe(249);
  });

  it("fades in the ignored rows the re-query brings when turned on, and nothing else", async () => {
    const onDisk = [coffee, hidden, salary];
    const { state, deps, rendered } = setup(onDisk, [coffee, salary]);

    await setShowIgnored(state, true, deps);

    expect(rendered[0]).toEqual(["Coffee Club", "Employer"]);
    expect(state.searchFilteredTransactions).toEqual(onDisk);
    expect([...state.rowFlashes!.changed]).toEqual([txnRowKey(hidden)]);
  });

  it("fades nothing when the user moved to another list while the re-query ran", async () => {
    const { state, deps, navigate } = setup([coffee, hidden], [coffee]);
    const apply = deps.apply;
    deps.apply = async (show) => { navigate("expenses:food"); await apply(show); };

    await setShowIgnored(state, true, deps);

    expect(state.rowFlashes).toBeUndefined();
  });
});
