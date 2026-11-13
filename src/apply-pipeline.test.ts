import { describe, it, expect, vi } from "vitest";
import { applyPipelineResponse, applyMutationResponse, noteMutationResponse, type ApplyTarget } from "./apply-pipeline";
import type { PipelineResponse } from "./types";

function makeState(extra: Partial<ApplyTarget> = {}): ApplyTarget {
  return {
    pipelineWarnings: [],
    accountSetMap: {},
    accountFoldersMap: {},
    accountPropertiesMap: {},
    accountsWithOpening: [],
    ...extra,
  };
}

function makeResponse(): PipelineResponse {
  return {
    result: {
      owner_accounts: {},
      account_folders: {},
      account_properties: {},
      output_files_written: 1,
      total_written: 3,
    } as any,
    parse: {
      ok: true,
      diagnostics: [],
      transactions: [],
      balances: [],
      accounts_with_opening: [],
      account_properties: {},
    },
    warnings: [],
  };
}

describe("applyPipelineResponse", () => {
  it("invokes state.refreshView so the search view reloads after a mutation", async () => {
    const refreshView = vi.fn().mockResolvedValue(undefined);
    const state = makeState({ refreshView });

    await applyPipelineResponse(state, makeResponse());

    expect(refreshView).toHaveBeenCalledTimes(1);
  });

  it("refreshes AFTER applying parse state (so the refresh can read fresh state)", async () => {
    const order: string[] = [];
    const state = makeState({
      refreshView: async () => {
        order.push(`refresh:parseOk=${state.parse?.ok}`);
      },
    });

    await applyPipelineResponse(state, makeResponse());

    expect(order).toEqual(["refresh:parseOk=true"]);
  });

  it("populates accountsWithOpening from the fat parse (unverified-balance hint source)", async () => {
    const state = makeState({ refreshView: vi.fn().mockResolvedValue(undefined) });
    const response = makeResponse();
    response.parse.accounts_with_opening = ["assets:bank:test"];

    await applyPipelineResponse(state, response);

    expect(state.accountsWithOpening).toEqual(["assets:bank:test"]);
  });

  it("does not throw when refreshView is unset", async () => {
    const state = makeState();
    await expect(applyPipelineResponse(state, makeResponse())).resolves.toBeUndefined();
  });

  it("populates justAddedTxnIds with txns present after but not before the rebuild", async () => {
    const state = makeState({
      parse: {
        ok: true,
        diagnostics: [],
        transactions: [
          { meta: "txn:keep, payee:Foo", date: "", datetime: "", amount: 0, narration: null, postings: [] },
        ],
        balances: [],
        accounts_with_opening: [],
        account_properties: {},
      } as any,
    });

    const resp = makeResponse();
    resp.parse.transactions = [
      { meta: "txn:keep, payee:Foo", date: "", datetime: "", amount: 0, narration: null, postings: [] },
      { meta: "txn:new, payee:Bar", date: "", datetime: "", amount: 0, narration: null, postings: [] },
    ] as any;

    await applyPipelineResponse(state, resp);

    expect(state.justAddedTxnIds).toBeInstanceOf(Set);
    expect(state.justAddedTxnIds?.has("txn:new")).toBe(true);
    expect(state.justAddedTxnIds?.has("txn:keep")).toBe(false);
  });

  it("leaves justAddedTxnIds undefined when no new ids arrived", async () => {
    const state = makeState({
      parse: {
        ok: true,
        diagnostics: [],
        transactions: [
          { meta: "txn:a", date: "", datetime: "", amount: 0, narration: null, postings: [] },
        ],
        balances: [],
        accounts_with_opening: [],
        account_properties: {},
      } as any,
    });

    const resp = makeResponse();
    // Pipeline returned the same set of txns (e.g. a no-op rebuild)
    resp.parse.transactions = [
      { meta: "txn:a", date: "", datetime: "", amount: 0, narration: null, postings: [] },
    ] as any;

    await applyPipelineResponse(state, resp);

    expect(state.justAddedTxnIds).toBeUndefined();
  });
});

describe("applyMutationResponse", () => {
  it("runs the main refresh, so a rule or label change reaches the account lists", async () => {
    // TASK-148: a slim response carries no account tree, so refreshing only the
    // header and the window (refreshView) left the sidebar and the account
    // dropdowns without the account a rule had just created.
    const refreshLedger = vi.fn().mockResolvedValue(undefined);
    const refreshView = vi.fn().mockResolvedValue(undefined);
    const state = makeState({ refreshLedger, refreshView });

    await applyMutationResponse(state, { ok: true, warnings: [], output_files_written: 1 });

    expect(refreshLedger).toHaveBeenCalledTimes(1);
    expect(refreshView).not.toHaveBeenCalled();
  });

  it("records warnings without refreshing, and never marks the reports as rebuilding (noteMutationResponse)", () => {
    const refreshLedger = vi.fn().mockResolvedValue(undefined);
    const refreshView = vi.fn().mockResolvedValue(undefined);
    const state = makeState({ refreshLedger, refreshView });

    noteMutationResponse(state, { ok: true, warnings: ["w1"], output_files_written: 2 });

    expect(state.pipelineWarnings).toEqual(["w1"]);
    // A mutation does not regenerate the report files; the Reports view
    // computes a stale report live, so there is nothing to wait for.
    expect(state.reportsBuilding).toBeUndefined();
    expect(refreshLedger).not.toHaveBeenCalled();
    expect(refreshView).not.toHaveBeenCalled();
  });

  it("does not throw when refreshLedger is unset", async () => {
    const state = makeState();
    await expect(
      applyMutationResponse(state, { ok: true, warnings: [], output_files_written: 0 }),
    ).resolves.toBeUndefined();
  });
});
