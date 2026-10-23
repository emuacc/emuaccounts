// Client-side mirror of payee-label matching for the party (label) editor
// preview. Ports `find_payee_rename` from src-tauri/src/rules.rs: labels
// match RAW payee/narration/meta only (display values are the OUTPUT of
// labels, never an input) and are first-match-wins across the merged
// closest-folder-first chain — no priority tiers, no rule shadowing.
//
// DOM-free (like save-rule-args.ts) so contract tests can import it
// without main.ts's bootstrap.

import type { RuleInfo } from "./rules";
import { amountMatches, parseAmountCondition, wildcardMatch } from "./rules";

/** One label plus the folder whose _labels.json owns it ("" = sources root).
 *  Mirrors the Rust `LabelInfo` returned by the `get_labels` command; array
 *  order is the pipeline's first-match-wins order. */
export type LabelInfo = { folder: string; label: RuleInfo };

export type LabelMatchFields = {
  payee?: string | null;
  narration?: string | null;
  meta?: string | null;
  amount?: number;
};

/** Port of rules.rs `id_anchor_prefix`: a bare `txn:<id>` / `leg:<id>`
 *  pattern with no glob/alternation/whitespace is id-anchored. */
function isIdAnchored(pattern: string): boolean {
  const p = pattern.trim();
  for (const prefix of ["leg:", "txn:"]) {
    if (p.startsWith(prefix)) {
      const rest = p.slice(prefix.length);
      if (rest && !rest.includes("*") && !rest.includes("|") && !rest.includes(",") && !/\s/.test(rest)) {
        return true;
      }
    }
  }
  return false;
}

/** Port of rules.rs `meta_segment_match`: split meta on `,`, trim, and
 *  require one segment to equal the pattern case-insensitively. */
function metaSegmentMatch(pattern: string, meta: string): boolean {
  const p = pattern.trim().toLowerCase();
  if (!p) return false;
  return meta.split(",").some((seg) => seg.trim().toLowerCase() === p);
}

/** Mirror of the per-label predicate inside `find_payee_rename`
 *  (src-tauri/src/rules.rs). Only payee-rename labels participate. */
export function labelMatchesFields(label: RuleInfo, fields: LabelMatchFields): boolean {
  if (label.payee == null) return false;

  let patternMatched: boolean;
  if (label.match_field) {
    const target =
      label.match_field === "meta" ? fields.meta :
      label.match_field === "payee" ? fields.payee :
      label.match_field === "narration" ? fields.narration :
      undefined;
    if (label.match_field === "meta" && isIdAnchored(label.pattern)) {
      patternMatched = target != null && metaSegmentMatch(label.pattern, target);
    } else {
      patternMatched = target != null && wildcardMatch(label.pattern, target);
    }
  } else {
    patternMatched = [fields.payee, fields.narration, fields.meta].some(
      (f) => f != null && wildcardMatch(label.pattern, f),
    );
  }
  if (!patternMatched) return false;

  if (label.amount_condition) {
    const cond = parseAmountCondition(label.amount_condition);
    if (cond) {
      if (fields.amount === undefined) return false;
      if (!amountMatches(cond, fields.amount)) return false;
    }
  }
  return true;
}

/** Where a NEW label appended to `targetFolder`'s _labels.json lands in the
 *  merged chain: after every entry from `targetFolder` itself or a deeper
 *  (closer-to-leaf) folder, before entries from its ancestors. Every folder
 *  in the chain lies on the leaf→root path, so the first strict ancestor of
 *  `targetFolder` marks the insertion point. */
export function labelInsertIndex(chain: LabelInfo[], targetFolder: string): number {
  for (let i = 0; i < chain.length; i++) {
    const f = chain[i].folder;
    const isStrictAncestor = f === "" ? targetFolder !== "" : targetFolder.startsWith(`${f}/`);
    if (isStrictAncestor) return i;
  }
  return chain.length;
}

export type LabelPreviewMatch = {
  transactionIndex: number;
  shadowed: boolean;
  winnerPattern?: string;
  winnerPayee?: string;
};

export type LabelDraftInput = {
  pattern: string;
  /** "" = any-field matching (payee/narration/meta) */
  matchField: string;
  /** "" = a new label */
  labelId: string;
  /** The draft's position in the merged chain: its own index when editing,
   *  labelInsertIndex(...) when new. Only labels BEFORE it can shadow it. */
  insertIndex: number;
};

export function computeLabelPreview(
  draft: LabelDraftInput,
  chain: LabelInfo[],
  transactions: ReadonlyArray<LabelMatchFields>,
): LabelPreviewMatch[] {
  const draftLabel: RuleInfo = {
    id: draft.labelId,
    pattern: draft.pattern,
    match_field: draft.matchField || undefined,
    payee: "preview",
  };
  const earlier = chain
    .slice(0, draft.insertIndex)
    .filter((entry) => !draft.labelId || entry.label.id !== draft.labelId);

  const matches: LabelPreviewMatch[] = [];
  transactions.forEach((txn, transactionIndex) => {
    if (!labelMatchesFields(draftLabel, txn)) return;
    const winner = earlier.find((entry) => labelMatchesFields(entry.label, txn));
    matches.push(
      winner
        ? {
            transactionIndex,
            shadowed: true,
            winnerPattern: winner.label.pattern,
            winnerPayee: winner.label.payee,
          }
        : { transactionIndex, shadowed: false },
    );
  });
  return matches;
}

/** Args for the `save_label` Tauri command — keys must mirror the Rust
 *  signature in src-tauri/src/main.rs (Tauri converts camelCase→snake_case). */
export function buildSaveLabelArgs(
  pattern: string,
  party: string,
  matchField: string,
  accountFolder: string,
  accountSet: string,
  nowYyyymm: string,
) {
  return {
    nowYyyymm,
    accountFolder,
    pattern,
    payee: party,
    commodity: null,
    matchField: matchField || null,
    accountSet,
  };
}

/** Args for the `update_label` Tauri command. */
export function buildUpdateLabelArgs(
  labelId: string,
  pattern: string,
  party: string,
  matchField: string,
  folder: string,
  accountSet: string,
  nowYyyymm: string,
) {
  return {
    nowYyyymm,
    folder,
    labelId,
    pattern,
    matchField: matchField || null,
    payee: party,
    accountSet,
  };
}

/** Args for the `delete_label` Tauri command. */
export function buildDeleteLabelArgs(
  folder: string,
  labelId: string,
  accountSet: string,
  nowYyyymm: string,
) {
  return { nowYyyymm, folder, labelId, accountSet };
}
