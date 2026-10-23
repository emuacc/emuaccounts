import { describe, expect, it } from "vitest";
import { loadTreeBaseTotals, treeBaseTotalsFromWire } from "./currency-totals";

const wire = {
  as_of_date: "2026-09-14",
  base_currency: "AUD",
  primary: { "assets": 79150, "assets:cash": 29000, "assets:crypto": 50000 },
  all: { "assets": 579150, "assets:crypto": 550000, "income": -500 },
};

describe("treeBaseTotalsFromWire", () => {
  it("keeps the primary and unrestricted maps apart", () => {
    const totals = treeBaseTotalsFromWire(wire);
    expect(totals.asOfDate).toBe("2026-09-14");
    expect(totals.baseCurrency).toBe("AUD");
    expect(totals.primary.get("assets")).toBe(79150);
    expect(totals.primary.has("income")).toBe(false);
    expect(totals.all.get("assets:crypto")).toBe(550000);
    expect(totals.all.get("income")).toBe(-500);
  });
});

describe("loadTreeBaseTotals", () => {
  it("asks the backend for the account set's totals in the base currency", async () => {
    const calls: Array<{ cmd: string; args: Record<string, unknown> }> = [];
    const invoker = async <T,>(cmd: string, args: Record<string, unknown>): Promise<T> => {
      calls.push({ cmd, args });
      return wire as unknown as T;
    };
    const totals = await loadTreeBaseTotals("AUD", "alex", invoker);
    expect(calls).toEqual([{ cmd: "load_tree_base_totals", args: { accountSet: "alex", baseCurrency: "AUD" } }]);
    expect(totals?.primary.get("assets:cash")).toBe(29000);
  });

  it("returns undefined without a base currency (no backend call)", async () => {
    let called = false;
    const invoker = async <T,>(): Promise<T> => { called = true; return wire as unknown as T; };
    expect(await loadTreeBaseTotals(undefined, "alex", invoker)).toBeUndefined();
    expect(called).toBe(false);
  });

  it("returns undefined when the backend rejects", async () => {
    const invoker = async <T,>(): Promise<T> => { throw new Error("no vault"); };
    expect(await loadTreeBaseTotals("AUD", "alex", invoker)).toBeUndefined();
  });
});
