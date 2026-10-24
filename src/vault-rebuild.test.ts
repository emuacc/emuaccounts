import { describe, it, expect, vi } from "vitest";
import { renderVaultRebuilding, VAULT_REBUILDING_TESTID, waitForVaultRebuild } from "./vault-rebuild";

/** An invoke whose `await_vault_rebuild` stays pending until `finish()`. */
function backend(running: boolean) {
  let finish!: () => void;
  const finished = new Promise<void>((resolve) => { finish = resolve; });
  const invoke = vi.fn(async (cmd: string) => {
    if (cmd === "vault_rebuild_running") return running;
    if (cmd === "await_vault_rebuild") return finished;
    throw new Error(`unexpected command ${cmd}`);
  }) as any;
  return { invoke, finish };
}

describe("waitForVaultRebuild", () => {
  it("passes straight through when no rebuild is running", async () => {
    const { invoke } = backend(false);
    const render = vi.fn();
    const state: { rebuildingCache?: boolean } = {};

    await waitForVaultRebuild(state, { invoke, render });

    expect(render).not.toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalledWith("await_vault_rebuild");
    expect(state.rebuildingCache).toBe(false);
  });

  it("shows the rebuilding screen until the rebuild ends", async () => {
    const { invoke, finish } = backend(true);
    const state: { rebuildingCache?: boolean } = {};
    const rendered: Array<boolean | undefined> = [];
    const render = vi.fn(() => rendered.push(state.rebuildingCache));

    const waiting = waitForVaultRebuild(state, { invoke, render });
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("await_vault_rebuild"));
    expect(rendered).toEqual([true]);
    expect(state.rebuildingCache).toBe(true);

    finish();
    await waiting;
    expect(state.rebuildingCache).toBe(false);
  });

  it("passes straight through outside Tauri, where invoke rejects", async () => {
    const invoke = vi.fn(async () => { throw new Error("no Tauri"); }) as any;
    const render = vi.fn();
    const state: { rebuildingCache?: boolean } = {};

    await waitForVaultRebuild(state, { invoke, render });

    expect(render).not.toHaveBeenCalled();
    expect(state.rebuildingCache).toBe(false);
  });
});

describe("renderVaultRebuilding", () => {
  it("tells the user the cache is rebuilding and may take a while", () => {
    const html = renderVaultRebuilding("EmuAcc");
    expect(html).toContain(`data-testid="${VAULT_REBUILDING_TESTID}"`);
    expect(html).toContain("Rebuilding cache");
    expect(html).toContain("This can take a while for large ledgers.");
  });

  it("escapes the app name", () => {
    expect(renderVaultRebuilding("<b>x</b>")).toContain("&lt;b&gt;x&lt;/b&gt;");
  });
});
