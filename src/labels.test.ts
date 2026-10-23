import { describe, expect, it } from "vitest";
import {
  buildDeleteLabelArgs,
  buildSaveLabelArgs,
  buildUpdateLabelArgs,
  computeLabelPreview,
  labelInsertIndex,
  labelMatchesFields,
  type LabelInfo,
} from "./labels";
import type { RuleInfo } from "./rules";

function makeLabel(overrides: Partial<RuleInfo> = {}): RuleInfo {
  return { id: "label-1", pattern: "*Cafe*", match_field: "payee", payee: "Cafe", ...overrides };
}

describe("labelMatchesFields", () => {
  it("match_field payee matches the raw payee only — never narration or display", () => {
    const label = makeLabel();
    expect(labelMatchesFields(label, { payee: "Coffee Cafe 123" })).toBe(true);
    // Labels transform the raw payee; the display value is their OUTPUT and
    // must not feed back as an input (unlike rule matching, which falls back
    // to display_payee). The fields type has no display slot at all.
    expect(labelMatchesFields(label, { narration: "Cafe", payee: "unrelated" })).toBe(false);
    expect(labelMatchesFields(label, {})).toBe(false);
  });

  it("no match_field falls back to any of raw payee/narration/meta", () => {
    const label = makeLabel({ match_field: undefined });
    expect(labelMatchesFields(label, { narration: "morning Cafe run" })).toBe(true);
    expect(labelMatchesFields(label, { meta: "txn:cafe-1" })).toBe(true);
    expect(labelMatchesFields(label, { payee: "x", narration: "y", meta: "z" })).toBe(false);
  });

  it("a label without a payee output (commodity rename) never matches", () => {
    const label = makeLabel({ payee: undefined });
    expect(labelMatchesFields(label, { payee: "Coffee Cafe 123" })).toBe(false);
  });

  it("txn-anchored meta patterns use segment-exact matching", () => {
    const label = makeLabel({ pattern: "txn:abc123", match_field: "meta" });
    expect(labelMatchesFields(label, { meta: "txn:abc123, src:kraken" })).toBe(true);
    expect(labelMatchesFields(label, { meta: "txn:abc123456" })).toBe(false);
    const glob = makeLabel({ pattern: "*abc*", match_field: "meta" });
    expect(labelMatchesFields(glob, { meta: "txn:abc123456" })).toBe(true);
  });

  it("amount conditions gate the match like the Rust side", () => {
    const label = makeLabel({ amount_condition: ">0" });
    expect(labelMatchesFields(label, { payee: "Cafe", amount: 5 })).toBe(true);
    expect(labelMatchesFields(label, { payee: "Cafe", amount: -5 })).toBe(false);
    expect(labelMatchesFields(label, { payee: "Cafe" })).toBe(false);
  });
});

describe("labelInsertIndex", () => {
  const chain: LabelInfo[] = [
    { folder: "alex/cash/cba/bta", label: makeLabel({ id: "leaf" }) },
    { folder: "alex/cash/cba", label: makeLabel({ id: "inst" }) },
    { folder: "", label: makeLabel({ id: "root" }) },
  ];

  it("a new institution label lands after the leaf block, before ancestor labels", () => {
    expect(labelInsertIndex(chain, "alex/cash/cba")).toBe(2);
  });

  it("a new leaf label lands after existing leaf labels", () => {
    expect(labelInsertIndex(chain, "alex/cash/cba/bta")).toBe(1);
  });

  it("a root label lands at the end; an empty chain inserts at 0", () => {
    expect(labelInsertIndex(chain, "")).toBe(3);
    expect(labelInsertIndex([], "anything")).toBe(0);
  });
});

describe("computeLabelPreview", () => {
  const txns = [
    { payee: "ANTHROPIC ANTHROPIC.COM CA US Card xx1 Value Date: 01/03/2026", amount: -25 },
    { payee: "ANTHROPIC HTTPSWWW.ANTH CA US Card xx1 Value Date: 02/03/2026", amount: -50 },
    { payee: "XERO AU INV-1 HAWTHORN AU Card xx1 Value Date: 03/03/2026", amount: -70 },
  ];

  it("a widened pattern matches both merchant shapes but not other payees", () => {
    const matches = computeLabelPreview(
      { pattern: "*ANTHROPIC*", matchField: "payee", labelId: "", insertIndex: 0 },
      [],
      txns,
    );
    expect(matches.map((m) => m.transactionIndex)).toEqual([0, 1]);
    expect(matches.every((m) => !m.shadowed)).toBe(true);
  });

  it("a narrower pattern drops the other shape", () => {
    const matches = computeLabelPreview(
      { pattern: "*ANTHROPIC ANTHROPIC.COM CA US*", matchField: "payee", labelId: "", insertIndex: 0 },
      [],
      txns,
    );
    expect(matches.map((m) => m.transactionIndex)).toEqual([0]);
  });

  it("an earlier (closer-folder) label wins and shadows the draft's rows", () => {
    const chain: LabelInfo[] = [
      { folder: "a/b", label: makeLabel({ id: "leaf", pattern: "*ANTHROPIC.COM*", payee: "Anthropic US" }) },
      { folder: "", label: makeLabel({ id: "root", pattern: "*ANTHROPIC*", payee: "Anthropic Root" }) },
    ];
    // New institution ("a") label: leaf sits before it, root after it.
    const matches = computeLabelPreview(
      { pattern: "*ANTHROPIC*", matchField: "payee", labelId: "", insertIndex: labelInsertIndex(chain, "a") },
      chain,
      txns,
    );
    expect(matches).toHaveLength(2);
    const first = matches.find((m) => m.transactionIndex === 0)!;
    expect(first.shadowed).toBe(true);
    expect(first.winnerPattern).toBe("*ANTHROPIC.COM*");
    expect(first.winnerPayee).toBe("Anthropic US");
    // The root label comes after the draft in the chain — it must NOT shadow.
    const second = matches.find((m) => m.transactionIndex === 1)!;
    expect(second.shadowed).toBe(false);
  });

  it("editing an existing label is not shadowed by itself or by later labels", () => {
    const chain: LabelInfo[] = [
      { folder: "a", label: makeLabel({ id: "mine", pattern: "*ANTHROPIC*", payee: "Anthropic" }) },
      { folder: "", label: makeLabel({ id: "root", pattern: "*ANTHROPIC*", payee: "Root" }) },
    ];
    const matches = computeLabelPreview(
      { pattern: "*ANTHROPIC*", matchField: "payee", labelId: "mine", insertIndex: 0 },
      chain,
      txns,
    );
    expect(matches).toHaveLength(2);
    expect(matches.every((m) => !m.shadowed)).toBe(true);
  });
});

// Locks the JS↔Rust IPC contract for the label commands — same rationale as
// save-rule-contract.test.ts: camelCase keys drifting from the Rust
// signatures make fields silently disappear on save.
describe("label IPC contracts", () => {
  it("buildSaveLabelArgs matches save_label's signature", () => {
    const args = buildSaveLabelArgs("*ANTHROPIC*", "Anthropic", "payee", "alex/cash/cba", "default", "202608");
    expect(Object.keys(args).sort()).toEqual(
      ["accountFolder", "accountSet", "commodity", "matchField", "nowYyyymm", "pattern", "payee"],
    );
    expect(args.payee).toBe("Anthropic");
    expect(args.commodity).toBe(null);
  });

  it("buildSaveLabelArgs sends matchField null when the row had no source payee", () => {
    const args = buildSaveLabelArgs("*x*", "X", "", "f", "default", "202608");
    expect(args.matchField).toBe(null);
  });

  it("buildUpdateLabelArgs matches update_label's signature", () => {
    const args = buildUpdateLabelArgs("rule-abc", "*ANTHROPIC*", "Anthropic", "payee", "alex/cash/cba", "default", "202608");
    expect(Object.keys(args).sort()).toEqual(
      ["accountSet", "folder", "labelId", "matchField", "nowYyyymm", "pattern", "payee"],
    );
    expect(args.labelId).toBe("rule-abc");
  });

  it("buildDeleteLabelArgs matches delete_label's signature", () => {
    const args = buildDeleteLabelArgs("alex/cash/cba", "rule-abc", "default", "202608");
    expect(Object.keys(args).sort()).toEqual(["accountSet", "folder", "labelId", "nowYyyymm"]);
  });
});
