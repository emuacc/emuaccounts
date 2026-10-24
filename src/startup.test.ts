import { describe, it, expect, vi } from "vitest";
import { normalStartup, type StartupDeps, type StartupState } from "./startup";

// ---------------------------------------------------------------------------
// Mock data returned by each invoke command
// ---------------------------------------------------------------------------

const MOCK_BALANCES = [
  { account: "assets:crypto:wallet:ethereum", totals: [{ commodity: "ETH", amount: 1.5 }] },
];

const MOCK_PIPELINE_METADATA = {
  owner_accounts: { alex: ["assets:crypto:wallet:ethereum"] },
  account_folders: { alex: { "assets:crypto:wallet:ethereum": "alex/crypto/wallet/ethereum" } },
  account_properties: {},
  accounts_with_opening: ["assets:crypto:wallet:ethereum"],
};

const MOCK_RESPONSES: Record<string, unknown> = {
  list_account_sets: ["alex"],
  get_display_config: { commodities: {}, default_decimals: 2, base_currency: "AUD" },
  load_account_tree: MOCK_BALANCES,
  load_pipeline_metadata: MOCK_PIPELINE_METADATA,
  get_pipeline_warnings: [],
  get_trade_links: [],
  get_account_gaps: [],
  get_source_folder_path: "/mock/path",
};

// ---------------------------------------------------------------------------
// Deferred-promise test harness
//
// Every deps.invoke() call returns a deferred promise. The test controls when
// these resolve via flush(). Each flush resolves all currently-pending invoke
// calls, then waits for microtasks to settle so that subsequent invoke calls
// (started by .then() handlers) land in `pending` ready for the next flush.
//
// The number of flush() calls == number of sequential invoke rounds.
// ---------------------------------------------------------------------------

interface Pending {
  cmd: string;
  resolve: (v: unknown) => void;
}

function createHarness(responseOverrides?: Record<string, unknown>) {
  const responses: Record<string, unknown> = { ...MOCK_RESPONSES, ...responseOverrides };
  const pending: Pending[] = [];
  const rounds: string[][] = [];

  const state: StartupState = {
    accountSets: [],
    busy: false,
    status: undefined,
    drillPath: [],
    pipelineWarnings: [],
    accountSetMap: {},
    accountFoldersMap: {},
    accountPropertiesMap: {},
    accountsWithOpening: [],
    tradeLinks: [],
    tradeSuggestions: [],
    sidebarView: "accounts",
    knownRoots: [],
  } as StartupState & Record<string, unknown>;

  const renderSpy = vi.fn();

  const deps: StartupDeps = {
    invoke: <T>(cmd: string, _args?: Record<string, unknown>): Promise<T> => {
      return new Promise<T>((resolve) => {
        pending.push({ cmd, resolve: resolve as (v: unknown) => void });
      });
    },
    listen: vi.fn().mockResolvedValue(undefined),
    render: renderSpy,
    log: vi.fn(),
    onPipelineEvent: vi.fn(),
  };

  /** Resolve all pending invoke calls and let microtasks settle. */
  const flush = async (): Promise<string[]> => {
    // Let microtasks from previous flush settle (new invokes may have been queued)
    await new Promise((r) => setTimeout(r, 0));

    const batch = pending.splice(0);
    if (batch.length === 0) return [];

    const cmds = batch.map((p) => p.cmd);
    rounds.push(cmds);

    for (const p of batch) {
      p.resolve(responses[p.cmd] ?? undefined);
    }

    // Let .then() handlers and subsequent microtasks run
    await new Promise((r) => setTimeout(r, 0));
    return cmds;
  };

  return { state, deps, renderSpy, flush, rounds, pending };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("normalStartup invoke rounds", () => {
  it("round 1: only list_account_sets", async () => {
    const { state, deps, flush } = createHarness();
    const done = normalStartup(state, deps);

    const r1 = await flush();
    expect(r1).toEqual(["list_account_sets"]);

    // Drain remaining rounds so the promise resolves
    while ((await flush()).length > 0) { /* drain */ }
    await done;
  });

  it("round 2: parallel independent operations", async () => {
    const { state, deps, flush } = createHarness();
    const done = normalStartup(state, deps);

    await flush(); // round 1
    const r2 = await flush();

    expect(r2.sort()).toEqual([
      "get_account_gaps",
      "get_display_config",
      "get_trade_links",
      "load_account_tree",
      "load_pipeline_metadata",
    ]);

    while ((await flush()).length > 0) { /* drain */ }
    await done;
  });

  it("round 3: dependent operations after tree + pipeline metadata", async () => {
    const { state, deps, flush } = createHarness();
    const done = normalStartup(state, deps);

    await flush(); // round 1
    await flush(); // round 2

    const r3 = await flush();

    // These all require results from round 2
    expect(r3).toContain("get_pipeline_warnings");
    expect(r3).toContain("get_source_folder_path");

    while ((await flush()).length > 0) { /* drain */ }
    await done;
  });

  it("completes in at most 3 sequential invoke rounds", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(rounds.length).toBeLessThanOrEqual(3);
  });

  it("makes all expected invoke calls", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    const allCmds = rounds.flat();
    expect(allCmds).toContain("list_account_sets");
    expect(allCmds).toContain("load_account_tree");
    expect(allCmds).toContain("load_pipeline_metadata");
    expect(allCmds).toContain("get_display_config");
    expect(allCmds).toContain("get_account_gaps");
    expect(allCmds).toContain("get_pipeline_warnings");
    expect(allCmds).toContain("get_trade_links");
    expect(allCmds).toContain("get_source_folder_path");
  });

  it("never invokes the multi-device sync commands (only the sync build's src/sync.ts does)", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    const allCmds = rounds.flat();
    for (const cmd of ["init_sync", "get_relay_config", "list_devices", "get_sync_log"]) {
      expect(allCmds, cmd).not.toContain(cmd);
    }
  });

  it("leaves the trade links empty when they cannot be read", async () => {
    const { state, deps, flush } = createHarness();
    state.tradeLinks = [{ id: "stale" }];
    deps.invoke = ((orig) => <T>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
      if (cmd === "get_trade_links") return Promise.reject("unreadable _trade_links.json");
      return orig<T>(cmd, args);
    })(deps.invoke);
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(state.tradeLinks).toEqual([]);
  });

  it("is slim: never rebuilds the pipeline or ships the full ledger", async () => {
    // The backend already rebuilds the vault when it is opened; the fat
    // rebuild_pipeline response carries the entire active ledger (~40MB JSON
    // on a large vault) whose main-thread JSON.parse stalls the WebView.
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) {
      // The startup path must never block the UI behind a busy spinner.
      expect(state.busy).toBe(false);
      expect(state.status).not.toBe("Rebuilding...");
    }
    await done;

    const allCmds = rounds.flat();
    expect(allCmds).not.toContain("rebuild_pipeline");
    expect(allCmds).not.toContain("load_generated_ledger");
    // The full-vault trade-suggestion scan (measured 9.3s on the real vault)
    // belongs to the post-paint heavy refresh, never the startup rounds.
    expect(allCmds).not.toContain("suggest_trade_links_cmd");
  });

  it("registers the pipeline-rebuilt event listener", async () => {
    const { state, deps, flush } = createHarness();
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(deps.listen).toHaveBeenCalledWith("pipeline-rebuilt", expect.any(Function));
  });

  it("calls render after each round", async () => {
    const { state, deps, flush, renderSpy } = createHarness();
    const done = normalStartup(state, deps);

    await flush(); // round 1
    await flush(); // round 2
    // render called after the account tree lands + after round 2 settles
    const afterR2 = renderSpy.mock.calls.length;
    expect(afterR2).toBeGreaterThanOrEqual(2);

    await flush(); // round 3
    while ((await flush()).length > 0) { /* drain */ }
    await done;

    // Final render after round 3
    expect(renderSpy.mock.calls.length).toBeGreaterThan(afterR2);
  });

  it("populates state correctly after startup", async () => {
    const { state, deps, flush } = createHarness();
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(state.accountSets).toEqual(["alex"]);
    expect(state.selectedAccountSet).toBe("alex");
    expect(state.displayConfig?.base_currency).toBe("AUD");
    expect(state.parse?.ok).toBe(true);
    // Slim startup: balances come from load_account_tree, transactions stay
    // empty (the visible window loads via query_search right after startup).
    expect(state.parse?.balances).toEqual(MOCK_BALANCES);
    expect(state.parse?.transactions).toEqual([]);
    expect(state.selectedAccount).toBe("assets:crypto:wallet:ethereum");
    // Not the render fallback's misleading "Parsed (0 transactions)"
    expect(state.status).toBe("Loaded 1 account");
    expect(state.accountSetMap).toEqual(MOCK_PIPELINE_METADATA.owner_accounts);
    expect(state.accountFoldersMap).toEqual(MOCK_PIPELINE_METADATA.account_folders);
    // The unverified-balance hint reads this — the slim parse's own
    // accounts_with_opening is always empty (regression: every account showed
    // "no opening balance set" and invited setting one on the wrong account).
    expect(state.accountsWithOpening).toEqual(["assets:crypto:wallet:ethereum"]);
  });

  it("keeps a pre-set selectedAccountSet that exists in this vault", async () => {
    // The vault's persisted ui-state is applied to state BEFORE startup runs.
    const { state, deps, flush } = createHarness({ list_account_sets: ["alex", "shared"] });
    state.selectedAccountSet = "shared";
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(state.selectedAccountSet).toBe("shared");
  });

  it("falls back to the first set when the pre-set selection no longer exists", async () => {
    // Regression: a selection restored from another vault's era (or a deleted
    // set) must degrade gracefully, not leave startup querying a ghost set.
    const { state, deps, flush } = createHarness();
    state.selectedAccountSet = "no-such-set";
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(state.selectedAccountSet).toBe("alex");
  });

  it("honours display-config default_account when picking the initial account", async () => {
    // load_account_tree and get_display_config resolve in the same round; the
    // default-account fallback must wait for the display config rather than
    // racing it (a lost race silently selects balances[0] instead).
    const { state, deps, flush } = createHarness({
      load_account_tree: [
        { account: "assets:cash:bank", totals: [{ commodity: "AUD", amount: 10 }] },
        ...MOCK_BALANCES,
      ],
      get_display_config: {
        commodities: {},
        default_decimals: 2,
        base_currency: "AUD",
        default_account: "assets:crypto:wallet:ethereum",
      },
    });
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(state.selectedAccount).toBe("assets:crypto:wallet:ethereum");
  });

  it("tolerates a missing pipeline-metadata.json (brand-new vault)", async () => {
    const { state, deps, flush } = createHarness();
    // load_pipeline_metadata rejects when no metadata file exists yet
    deps.invoke = ((orig) => <T>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
      if (cmd === "load_pipeline_metadata") return Promise.reject("No pipeline metadata found");
      return orig<T>(cmd, args);
    })(deps.invoke);
    const done = normalStartup(state, deps);

    while ((await flush()).length > 0) { /* drain */ }
    await done;

    expect(state.parse?.ok).toBe(true);
    expect(state.accountFoldersMap).toEqual({});
  });
});

// ---------------------------------------------------------------------------
// Efficiency constraints — guard against regressions
// ---------------------------------------------------------------------------

describe("application is efficient", () => {
  /** Count how many times each command was invoked across all rounds. */
  function countInvokes(rounds: string[][]): Map<string, number> {
    const counts = new Map<string, number>();
    for (const cmds of rounds) {
      for (const cmd of cmds) {
        counts.set(cmd, (counts.get(cmd) ?? 0) + 1);
      }
    }
    return counts;
  }

  it("when loading the application, the pipeline is never rebuilt (backend setup covers it)", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);
    while ((await flush()).length > 0) { /* drain */ }
    await done;

    const counts = countInvokes(rounds);
    expect(counts.get("rebuild_pipeline")).toBeUndefined();
    expect(counts.get("load_account_tree")).toBe(1);
    expect(counts.get("load_pipeline_metadata")).toBe(1);
  });

  it("when loading the application, account sets are listed once", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);
    while ((await flush()).length > 0) { /* drain */ }
    await done;

    const counts = countInvokes(rounds);
    expect(counts.get("list_account_sets")).toBe(1);
  });

  it("when loading the application, config and trade links are fetched once each", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);
    while ((await flush()).length > 0) { /* drain */ }
    await done;

    const counts = countInvokes(rounds);
    expect(counts.get("get_display_config")).toBe(1);
    expect(counts.get("get_trade_links")).toBe(1);
  });

  it("when loading the application, render is called at most 5 times", async () => {
    const { state, deps, flush, renderSpy } = createHarness();
    const done = normalStartup(state, deps);
    while ((await flush()).length > 0) { /* drain */ }
    await done;

    // Allowed renders: after the account tree lands, after round 2,
    // after trade suggestions, after round 3
    expect(renderSpy.mock.calls.length).toBeLessThanOrEqual(5);
  });

  it("when loading the application, trade links are fetched once (suggestions deferred to heavy refresh)", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);
    while ((await flush()).length > 0) { /* drain */ }
    await done;

    const counts = countInvokes(rounds);
    expect(counts.get("get_trade_links")).toBe(1);
    expect(counts.get("suggest_trade_links_cmd")).toBeUndefined();
  });

  it("no expensive command is called more than twice during startup", async () => {
    const { state, deps, flush, rounds } = createHarness();
    const done = normalStartup(state, deps);
    while ((await flush()).length > 0) { /* drain */ }
    await done;

    const counts = countInvokes(rounds);
    // get_source_folder_path may be called per-folder — allow N
    const UNLIMITED = new Set(["get_source_folder_path"]);
    for (const [cmd, count] of counts) {
      if (!UNLIMITED.has(cmd)) {
        expect(count, `${cmd} called ${count} times`).toBeLessThanOrEqual(2);
      }
    }
  });
});
