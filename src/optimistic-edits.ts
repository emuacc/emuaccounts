// Optimistic edits to the loaded transaction rows.
//
// An edit made from the transactions table (re-categorise, rename a party,
// hide, unhide) changes the vault's rules and rebuilds the pipeline, which
// takes seconds on a large vault. Until that rebuild lands, the edit is held
// here as an OptimisticEdit and applied at render time over the rows already
// loaded — the same TS mirrors of the rule and label matchers the editors'
// previews use decide which rows it rewrites. The edit is never written into
// the loaded rows: withdrawing it (failure) or retiring it (the rebuilt rows
// have arrived) is just removing it from the list.
//
// When the rebuild lands, `settleOptimisticEdit` re-fetches the window in
// place, works out which rows look different from what was on screen (they
// fade) and which rows the edit was predicted to change but the rebuild did
// not (they flash, and the caller says the change did not apply).
//
// Show Ignored changes no rules, only which rows the queries return:
// `setShowIgnored` drops the ignored rows at once when it is turned off and
// fades them in when it is turned on.
//
// DOM-free so the whole flow is unit-tested; main.ts owns the wiring.

import type { Transaction } from "./types";
import { computeRulePreview, type RuleInfo } from "./rules";
import { computeLabelPreview, labelMatchesFields, type LabelInfo } from "./labels";
import { contraLeg, isTradeShaped } from "./transaction-legs";
import { legIdFromMeta, matchesPrefix, txnIdFromMeta } from "./meta";

/** Where "Hide" sends a transaction's legs — mirrors HIDE_AMOUNT_ACCOUNT in
 *  src-tauri/src/processing_pipeline.rs. */
export const HIDDEN_ACCOUNT = "ignore:hidden";

/** Which loaded rows an edit rewrites. A rule or label applies to the rows
 *  imported into `scopeFolder` (the folder its file lives in) or below it;
 *  "" is the sources root, every row. */
export type EditTarget =
  /** An inline cell edit or a row button: the one row it was made on. The
   *  `leg:` id when the pipeline stamped one (sibling legs share the `txn:`). */
  | { by: "row"; legId: string; txnId: string }
  /** Every row with this `txn:` id (hide / unhide act on the whole txn). */
  | { by: "txn"; txnId: string }
  /** Rows a saved rule categorises: in scope, matched, and not shadowed by a
   *  higher-priority rule in `allRules` (the editor's own preview logic). */
  | { by: "rule"; rule: RuleInfo; allRules: RuleInfo[]; scopeFolder: string }
  /** Rows a saved payee label renames. With the label chain it skips rows an
   *  earlier label already claims; without it every matching row counts. */
  | { by: "label"; label: RuleInfo; scopeFolder: string; chain?: LabelInfo[]; insertIndex?: number };

export type OptimisticEdit = {
  id: number;
  /** category: the categorised (contra) leg's account becomes `value`;
   *  payee: the displayed payee becomes `value`; remove: the rows leave;
   *  hide: every leg moves to ignore:hidden and the row stays (Show Ignored). */
  field: "category" | "payee" | "remove" | "hide";
  value: string;
  target: EditTarget;
  /** How the status line names the change, e.g. `Rule "*COFFEE*"`. */
  label: string;
};

export type NewOptimisticEdit = Omit<OptimisticEdit, "id">;

/** What a render of the table needs to place an edit. */
export type EditView = {
  /** The pane's selection; an edited row with no leg under it has left the view. */
  selectedAccount?: string;
  /** The account a row was imported into. Its contra leg is the leg a rule
   *  categorises — not necessarily the selection's contra (Categories pane). */
  sourceAccountOf: (t: Transaction) => string | undefined;
  /** The source folder a row was imported from (decides rule/label scope). */
  sourceFolderOf: (t: Transaction) => string | undefined;
};

/** Rows the user should notice after a render: `changed` fade in place,
 *  `reverted` flash — the rebuild did not apply the change predicted for them. */
export type RowFlashes = { changed: Set<string>; reverted: Set<string> };

export type OptimisticState = {
  pendingEdits: OptimisticEdit[];
  rowFlashes?: RowFlashes;
};

/** A row's identity across renders and rebuilds — also its `data-expand-key`,
 *  which morphdom keys rows by. Sibling legs of one on-chain txn share the
 *  `txn:` id (and often datetime, amount and narration), so the per-leg `leg:`
 *  id keys the row when the pipeline stamped one. */
export function txnRowKey(t: Pick<Transaction, "meta" | "datetime" | "amount" | "narration">): string {
  const legId = legIdFromMeta(t.meta);
  if (legId) return legId;
  const txnId = txnIdFromMeta(t.meta);
  return txnId ? `${txnId}|${t.datetime}|${t.amount}|${t.narration ?? ""}` : "";
}

/** What a row renders from: its party, notes, date and every leg. Two rows
 *  with the same signature look the same in the table. */
function displaySignature(t: Transaction): string {
  const legs = t.postings
    .map((p) => `${p.account}|${p.amount}|${p.commodity}`)
    .join(";");
  return [t.datetime, t.payee ?? "", t.display_payee ?? "", t.narration ?? "", t.amount_commodity, legs].join("¦");
}

function inScope(t: Transaction, scopeFolder: string, view: EditView): boolean {
  if (!scopeFolder) return true;
  const folder = view.sourceFolderOf(t);
  return !!folder && (folder === scopeFolder || folder.startsWith(`${scopeFolder}/`));
}

/** Indices into `txns` of the rows `target` selects. */
function targetIndices(target: EditTarget, txns: Transaction[], view: EditView): number[] {
  switch (target.by) {
    case "row":
      return indicesWhere(txns, (t) =>
        target.legId ? legIdFromMeta(t.meta) === target.legId : txnIdFromMeta(t.meta) === target.txnId);
    case "txn":
      return indicesWhere(txns, (t) => txnIdFromMeta(t.meta) === target.txnId);
    case "rule": {
      const scoped = indicesWhere(txns, (t) => inScope(t, target.scopeFolder, view));
      const r = target.rule;
      const matches = computeRulePreview(
        r.pattern, r.match_field ?? "", r.id, target.allRules, scoped.map((i) => txns[i]),
        r.amount_condition, r.fee_condition, view.selectedAccount, r.payee_condition,
        r.narration_condition, r.commodity_condition, r.meta_condition,
      );
      return matches.filter((m) => !m.shadowed).map((m) => scoped[m.transactionIndex]);
    }
    case "label": {
      const scoped = indicesWhere(txns, (t) => inScope(t, target.scopeFolder, view));
      if (target.chain) {
        return computeLabelPreview(
          { pattern: target.label.pattern, matchField: target.label.match_field ?? "", labelId: target.label.id, insertIndex: target.insertIndex ?? 0 },
          target.chain, scoped.map((i) => txns[i]),
        ).filter((m) => !m.shadowed).map((m) => scoped[m.transactionIndex]);
      }
      return scoped.filter((i) => labelMatchesFields(target.label, txns[i]));
    }
  }
}

function indicesWhere(txns: Transaction[], pred: (t: Transaction) => boolean): number[] {
  const out: number[] = [];
  txns.forEach((t, i) => { if (pred(t)) out.push(i); });
  return out;
}

/** The leg a categorisation rewrites: the contra of the account the row was
 *  imported into. A trade-shaped row has no single category to rewrite. */
function categorisedLeg(t: Transaction, view: EditView) {
  const source = view.sourceAccountOf(t);
  if (!source || isTradeShaped(t, source)) return undefined;
  return contraLeg(t, source);
}

/** `t` as the edit will leave it, or null when the edit removes it. Rows the
 *  edit cannot rewrite (no categorised leg) come back unchanged. */
function patchRow(edit: OptimisticEdit, t: Transaction, view: EditView): Transaction | null {
  switch (edit.field) {
    case "remove":
      return null;
    case "hide":
      return { ...t, postings: t.postings.map((p) => ({ ...p, account: HIDDEN_ACCOUNT })) };
    case "payee":
      return { ...t, display_payee: edit.value };
    case "category": {
      const leg = categorisedLeg(t, view);
      if (!leg) return t;
      return { ...t, postings: t.postings.map((p) => (p === leg ? { ...p, account: edit.value } : p)) };
    }
  }
}

/** The rows `edit` rewrites in `txns`, by row key, as the edit predicts them
 *  (null = the row leaves). */
export function predictedRows(edit: OptimisticEdit, txns: Transaction[], view: EditView): Map<string, Transaction | null> {
  const out = new Map<string, Transaction | null>();
  for (const i of targetIndices(edit.target, txns, view)) {
    const key = txnRowKey(txns[i]);
    if (key) out.set(key, patchRow(edit, txns[i], view));
  }
  return out;
}

/** The loaded rows as the user should see them: each pending edit applied in
 *  order over `txns`. A row an edit removes — or moves out of the selected
 *  account (re-categorised away from the category being viewed) — is dropped. */
export function applyOptimisticEdits(txns: Transaction[], edits: OptimisticEdit[], view: EditView): Transaction[] {
  if (edits.length === 0) return txns;
  let rows = txns;
  for (const edit of edits) {
    const indices = new Set(targetIndices(edit.target, rows, view));
    if (indices.size === 0) continue;
    const next: Transaction[] = [];
    rows.forEach((t, i) => {
      if (!indices.has(i)) { next.push(t); return; }
      const patched = patchRow(edit, t, view);
      // Show Ignored keeps a hidden row in its account's view (query.rs relaxes
      // the account predicate for hidden rows), so a hidden row never leaves.
      if (patched && (edit.field === "hide" || !leftView(t, patched, view.selectedAccount))) next.push(patched);
    });
    rows = next;
  }
  return rows;
}

function leftView(before: Transaction, after: Transaction, selected: string | undefined): boolean {
  if (!selected) return false;
  const under = (t: Transaction) => t.postings.some((p) => matchesPrefix(p.account, selected));
  return under(before) && !under(after);
}

/** Keys of the rows in `after` that look different from `before`: content
 *  changed, or the row is new to a window that already had rows. A new row
 *  past the last row the window already had is not a change: the re-fetch
 *  pages it in to fill the space a row that left made. */
export function changedRowKeys(before: Transaction[], after: Transaction[]): Set<string> {
  const prev = new Map<string, string>();
  for (const t of before) {
    const key = txnRowKey(t);
    if (key) prev.set(key, displaySignature(t));
  }
  const changed = new Set<string>();
  if (before.length === 0) return changed;
  const keys = after.map(txnRowKey);
  let lastKnown = -1;
  keys.forEach((key, i) => { if (prev.has(key)) lastKnown = i; });
  keys.forEach((key, i) => {
    if (!key) return;
    const was = prev.get(key);
    if (was === undefined ? i < lastKnown : was !== displaySignature(after[i])) changed.add(key);
  });
  return changed;
}

/** The keys of predicted rows the rebuilt window does not show as predicted.
 *  A removal is missed while the row is still there; a rewrite is missed when
 *  the rebuilt row carries a different value. A rewritten row the rebuilt
 *  window no longer holds (it moved out of the view) is not counted. */
export function rowsNotApplied(
  edit: OptimisticEdit,
  predicted: Map<string, Transaction | null>,
  rebuilt: Transaction[],
  view: EditView,
): Set<string> {
  const byKey = new Map<string, Transaction>();
  for (const t of rebuilt) byKey.set(txnRowKey(t), t);
  const missed = new Set<string>();
  for (const [key, expected] of predicted) {
    const actual = byKey.get(key);
    if (actual && !showsEdit(edit, expected, actual, view)) missed.add(key);
  }
  return missed;
}

function showsEdit(edit: OptimisticEdit, expected: Transaction | null, actual: Transaction, view: EditView): boolean {
  if (expected === null) return false; // a removed row is still there
  if (edit.field === "payee") return (actual.display_payee || actual.payee || "") === edit.value;
  if (edit.field === "hide") return actual.postings.some((p) => p.account === HIDDEN_ACCOUNT);
  // A row with no categorised leg was predicted unchanged; nothing to check.
  if (categorisedLeg(expected, view)?.account !== edit.value) return true;
  return categorisedLeg(actual, view)?.account === edit.value;
}

let editSeq = 0;

function addFlashes(state: OptimisticState, changed: Set<string>, reverted: Set<string> = new Set()): void {
  if (changed.size === 0 && reverted.size === 0) return;
  const flashes = state.rowFlashes ?? { changed: new Set<string>(), reverted: new Set<string>() };
  for (const k of changed) if (!reverted.has(k)) flashes.changed.add(k);
  for (const k of reverted) { flashes.reverted.add(k); flashes.changed.delete(k); }
  state.rowFlashes = flashes;
}

/** Show `edit` at once: queue it over the loaded rows and flash the rows it
 *  changes. Returns the edit with its id, for settle/revert. */
export function addOptimisticEdit(
  state: OptimisticState,
  edit: NewOptimisticEdit,
  rows: Transaction[],
  view: EditView,
): OptimisticEdit {
  editSeq += 1;
  const pending: OptimisticEdit = { ...edit, id: editSeq };
  const before = applyOptimisticEdits(rows, state.pendingEdits, view);
  state.pendingEdits = [...state.pendingEdits, pending];
  addFlashes(state, changedRowKeys(before, applyOptimisticEdits(rows, state.pendingEdits, view)));
  return pending;
}

export type SettleDeps = {
  /** Re-fetch the loaded window in place (rows replaced only once they arrive). */
  refresh: () => Promise<void>;
  /** The loaded rows, before any optimistic edit. */
  rows: () => Transaction[];
  view: () => EditView;
  /** Identifies what the table lists (selection, search, sort). When it
   *  changes while the rebuild lands, the rows before and after belong to
   *  different lists, so nothing fades and nothing is checked. */
  viewKey: () => string;
};

/** How the rebuilt rows compare with the prediction: `missed` of the
 *  `checked` predicted rows still in the window did not change as shown. */
export type SettleResult = { missed: number; checked: number };

/** The rebuild for `edit` (undefined: a mutation with no prediction) has
 *  succeeded: refresh the window in place, retire the edit, fade the rows that
 *  now look different and flash the rows the rebuild did not change as
 *  predicted. */
export async function settleOptimisticEdit(
  state: OptimisticState,
  edit: OptimisticEdit | undefined,
  deps: SettleDeps,
): Promise<SettleResult> {
  const real = deps.rows();
  const key = deps.viewKey();
  const predicted = edit ? predictedRows(edit, real, deps.view()) : new Map<string, Transaction | null>();
  const before = applyOptimisticEdits(real, state.pendingEdits, deps.view());
  try {
    await deps.refresh();
  } finally {
    if (edit) state.pendingEdits = state.pendingEdits.filter((e) => e.id !== edit.id);
  }
  if (deps.viewKey() !== key) return { missed: 0, checked: 0 };
  const rebuilt = deps.rows();
  const view = deps.view();
  const missed = edit ? rowsNotApplied(edit, predicted, rebuilt, view) : new Set<string>();
  addFlashes(state, changedRowKeys(before, applyOptimisticEdits(rebuilt, state.pendingEdits, view)), missed);
  const present = new Set(rebuilt.map(txnRowKey));
  return { missed: missed.size, checked: [...predicted.keys()].filter((k) => present.has(k)).length };
}

/** The mutation for `edit` failed: withdraw the edit, show the rows as the
 *  vault now has them (best effort — the write may have half-landed), and
 *  flash every row that goes back. */
export async function revertOptimisticEdit(
  state: OptimisticState,
  edit: OptimisticEdit | undefined,
  deps: SettleDeps,
): Promise<void> {
  const key = deps.viewKey();
  const before = applyOptimisticEdits(deps.rows(), state.pendingEdits, deps.view());
  if (edit) state.pendingEdits = state.pendingEdits.filter((e) => e.id !== edit.id);
  try { await deps.refresh(); } catch { /* keep the rows we have */ }
  if (deps.viewKey() !== key) return;
  const after = applyOptimisticEdits(deps.rows(), state.pendingEdits, deps.view());
  // A removed row coming back is new to the window, so it flashes too.
  addFlashes(state, new Set(), changedRowKeys(before, after));
}

/** The table state Show Ignored changes. */
export type ShowIgnoredState = OptimisticState & {
  showHidden: boolean;
  /** The active set's hidden account prefixes (config.json `hidden_accounts`). */
  hiddenAccounts: string[];
  searchFilteredTransactions?: Transaction[];
  searchFilteredCount?: number;
};

export type ShowIgnoredDeps = {
  /** Tell the backend, whose queries leave out the hidden accounts' rows. */
  apply: (show: boolean) => Promise<unknown>;
  /** Re-read the account lists and re-fetch the loaded window in place. */
  refresh: () => Promise<void>;
  render: () => void;
  /** As SettleDeps.viewKey; it includes the Show Ignored setting. */
  viewKey: () => string;
};

/** Does `t` post to a hidden account? Mirrors scoped_query in
 *  src-tauri/src/generated_store.rs: one leg whose account starts with a
 *  hidden prefix leaves the whole row out while Show Ignored is off. */
export function postsToHiddenAccount(t: Transaction, hiddenAccounts: string[]): boolean {
  return t.postings.some((p) => hiddenAccounts.some((prefix) => p.account.startsWith(prefix)));
}

/** Turn Show Ignored on or off without reloading the table. Off: the ignored
 *  rows leave in the same frame as the click, before the re-query. On: they
 *  are not loaded yet, so they fade in when the re-query lands. Either way the
 *  rows around them keep their place. */
export async function setShowIgnored(state: ShowIgnoredState, show: boolean, deps: ShowIgnoredDeps): Promise<void> {
  state.showHidden = show;
  const rows = state.searchFilteredTransactions;
  if (!show && rows) {
    const kept = rows.filter((t) => !postsToHiddenAccount(t, state.hiddenAccounts));
    state.searchFilteredTransactions = kept;
    if (state.searchFilteredCount !== undefined) state.searchFilteredCount -= rows.length - kept.length;
  }
  deps.render();
  const key = deps.viewKey();
  const before = state.searchFilteredTransactions ?? [];
  await deps.apply(show);
  await deps.refresh();
  if (deps.viewKey() === key) addFlashes(state, changedRowKeys(before, state.searchFilteredTransactions ?? []));
}
