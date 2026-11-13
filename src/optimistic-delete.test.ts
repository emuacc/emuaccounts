// End-to-end test for the optimistic hide/delete flow.
//
// Simulates the click handler (a "remove" edit queued over the loaded rows)
// and the queue's drain without involving the DOM, and asserts:
//   1. After the click the txn is gone from the rendered rows before the
//      pipeline returns.
//   2. When the rebuild lands without the txn, it stays gone and the edit
//      is retired.
//   3. When the rebuild still has the txn (the hide did not apply), the row
//      comes back, flashes, and the flow reports it.

import { describe, it, expect, vi } from "vitest";
import { MutationQueue } from "./mutation-queue";
import {
  addOptimisticEdit,
  applyOptimisticEdits,
  settleOptimisticEdit,
  txnRowKey,
  type EditView,
  type OptimisticState,
} from "./optimistic-edits";
import type { Transaction } from "./types";

function row(meta: string, payee: string): Transaction {
  return {
    date: "2026-01-01", datetime: "2026-01-01T00:00:00Z", payee, narration: payee, meta,
    postings: [{ account: "assets:bank", amount: -1, commodity: "AUD" }, { account: "expenses:unknown", amount: 1, commodity: "AUD" }],
    amount: -1, amount_commodity: "AUD",
  };
}

const original = row("txn:abc, ofx_id:foo", "Original Row");
const other = row("txn:other", "Other Row");
const view: EditView = { selectedAccount: "assets:bank", sourceAccountOf: () => "assets:bank", sourceFolderOf: () => "e2e/bank" };

interface SimState extends OptimisticState {
  pendingMutations: number;
  status?: string;
  searchFilteredTransactions: Transaction[];
}

function makeState(): SimState {
  return { pendingMutations: 0, pendingEdits: [], searchFilteredTransactions: [original, other] };
}

/** What computeTransactionViewState renders from. */
function visibleRows(state: SimState): string[] {
  return applyOptimisticEdits(state.searchFilteredTransactions, state.pendingEdits, view).map((t) => t.payee ?? "");
}

/** Click handler + queued mutation; `rebuilt` is what the in-place refresh returns. */
function hide(state: SimState, queue: MutationQueue, backend: Promise<void>, rebuilt: Transaction[]) {
  const edit = addOptimisticEdit(state, {
    field: "remove", value: "", label: "Hide", target: { by: "txn", txnId: "txn:abc" },
  }, state.searchFilteredTransactions, view);
  let missed = -1;
  const done = queue.enqueue(state, "Hiding...", async () => {
    await backend;
    ({ missed } = await settleOptimisticEdit(state, edit, {
      rows: () => state.searchFilteredTransactions,
      view: () => view,
      viewKey: () => "assets:bank",
      refresh: async () => { state.searchFilteredTransactions = rebuilt; },
    }));
  });
  return { done, missed: () => missed };
}

describe("optimistic hide end-to-end", () => {
  it("removes the row before the pipeline returns and keeps it removed after", async () => {
    const state = makeState();
    const snapshots: string[][] = [];
    const queue = new MutationQueue({ render: () => snapshots.push(visibleRows(state)) });
    let backendDone: () => void = () => {};
    const backend = new Promise<void>((res) => { backendDone = res; });

    const flow = hide(state, queue, backend, [other]);
    // Renders are deferred (requestIdleCallback / setTimeout); flush.
    await new Promise((r) => setTimeout(r, 20));
    expect(snapshots[0]).toEqual(["Other Row"]);
    expect(state.searchFilteredTransactions).toContain(original);

    backendDone();
    await flow.done;
    await new Promise((r) => setTimeout(r, 20));
    expect(snapshots[snapshots.length - 1]).toEqual(["Other Row"]);
    expect(state.pendingEdits).toEqual([]);
    expect(flow.missed()).toBe(0);
  });

  it("brings the row back and reports it when the rebuild did not hide it", async () => {
    const state = makeState();
    const queue = new MutationQueue({ render: vi.fn() });
    const flow = hide(state, queue, Promise.resolve(), [original, other]);
    expect(visibleRows(state)).toEqual(["Other Row"]);

    await flow.done;
    expect(visibleRows(state)).toEqual(["Original Row", "Other Row"]);
    expect(flow.missed()).toBe(1);
    expect([...state.rowFlashes!.reverted]).toEqual([txnRowKey(original)]);
  });
});
