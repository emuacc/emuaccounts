// Non-intrusive DOM update: applies a new HTML description to an existing
// element via morphdom, only swapping the parts that actually changed.
//
// Architectural contract: any subtree whose new HTML is byte-identical to
// the existing DOM is left untouched. Focused inputs (mid-edit) are
// explicitly preserved — morphdom will still update other elements but
// won't replace the focused input's value or selection.

import morphdom from "morphdom";

export function morphInPlace(target: Element, html: string): void {
  morphdom(target, html, {
    // Key txn rows by data-expand-key (unique per posting) rather than
    // data-txn-row-id (which collides — one on-chain txn renders as N
    // rows, one per posting/leg, and they all carry the same txnId).
    // Without unique-per-row keys, morphdom collapses the group into a
    // single node and the surviving 5/6 are matched by position,
    // shuffling content across rows and surfacing duplicates plus
    // orphaned detail rows. Detail rows (the expanded sub-row inserted
    // by toggleTxnExpand) carry data-txn-detail-for, which IS the
    // expand-key — same uniqueness, same scheme. Anything else with a
    // stable id attribute gets keyed by id.
    getNodeKey: (node) => {
      if (node.nodeType !== 1) return undefined;
      const el = node as Element;
      const expandKey = el.getAttribute("data-expand-key");
      if (expandKey) return `row:${expandKey}`;
      const detailFor = el.getAttribute("data-txn-detail-for");
      if (detailFor) return `detail:${detailFor}`;
      // Fallback for non-txn rows that still want stable identity
      // (e.g. the <tr> for a manual transaction without an expand key,
      // or other elements that just carry data-txn-row-id).
      const txnRowId = el.getAttribute("data-txn-row-id");
      if (txnRowId) return `txn:${txnRowId}`;
      return el.id || undefined;
    },
    onBeforeElUpdated: (fromEl, toEl) => {
      // Leave imperatively-managed subtrees (e.g. the ApexCharts mount) alone —
      // morphdom must not reconcile a widget's injected children against our
      // empty placeholder. The widget owns everything inside the marked node.
      if (fromEl instanceof Element && fromEl.hasAttribute("data-morph-preserve")) {
        return false;
      }
      // An inline cell editor (startCellEdit) is injected imperatively —
      // freshly-built HTML still describes the original cell content.
      // While the edit is live the cell carries data-cell-editing;
      // skipping it keeps the user's in-progress input (value, focus,
      // caret) alive when a background render lands mid-edit (pipeline
      // auto-rebuild, reports-rebuilt, heavy refresh). The mark is
      // removed on save/cancel so the next render reclaims the cell.
      if (fromEl instanceof Element && fromEl.hasAttribute("data-cell-editing")) {
        return false;
      }
      if (fromEl.isEqualNode(toEl)) return false;
      if (
        document.activeElement === fromEl &&
        (fromEl instanceof HTMLInputElement || fromEl instanceof HTMLTextAreaElement)
      ) {
        return false;
      }
      return true;
    },
  });
}

/** Highlight colours for `playRowFlashes`: a row the rebuild changed fades
 *  from a soft blue; a row whose change did not apply flashes amber. */
const ROW_FLASH = {
  changed: { color: "59, 130, 246", alpha: 0.18, ms: 900 },
  reverted: { color: "245, 158, 11", alpha: 0.4, ms: 1800 },
} as const;

/** Fade the transaction rows keyed (`data-expand-key`) in `flashes`. Web
 *  Animations rather than a CSS class: the animation lives outside the
 *  markup, so a render that lands mid-fade (morphdom reconciling the row's
 *  class attribute) cannot cut it short. An inset box-shadow overlays the
 *  cell's own background (uncategorised rows are tinted) and ends fully
 *  transparent, so nothing jumps when the animation finishes. */
export function playRowFlashes(
  root: ParentNode,
  flashes: { changed: Set<string>; reverted: Set<string> } | undefined,
): void {
  if (!flashes || (flashes.changed.size === 0 && flashes.reverted.size === 0)) return;
  if (globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches) return;
  // Rows outside the rendered window, or inside a collapsed group, are skipped.
  const rows = new Map<string, Element>();
  for (const row of Array.from(root.querySelectorAll("tr[data-expand-key]"))) {
    rows.set(row.getAttribute("data-expand-key") ?? "", row);
  }
  for (const kind of ["changed", "reverted"] as const) {
    const { color, alpha, ms } = ROW_FLASH[kind];
    for (const key of flashes[kind]) {
      const row = rows.get(key);
      if (!row) continue;
      for (const cell of Array.from(row.children)) {
        if (typeof (cell as HTMLElement).animate !== "function") continue;
        (cell as HTMLElement).animate(
          [
            { boxShadow: `inset 0 0 0 9999px rgba(${color}, ${alpha})` },
            { boxShadow: `inset 0 0 0 9999px rgba(${color}, 0)` },
          ],
          { duration: ms, easing: "ease-out" },
        );
      }
    }
  }
}

type InputField = HTMLInputElement | HTMLTextAreaElement;

function isInputField(el: Element | null): el is InputField {
  return el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement;
}

/** Run `update` (a render) without taking the caret from the user.
 *
 *  The focused field and its selection are read immediately before the
 *  update, never earlier: keys typed and caret moves made while a debounce
 *  or a backend query was pending belong to the user, and restoring a caret
 *  captured back then puts the next keys into the middle of the word
 *  (TASK-150). morphInPlace leaves a focused field alone, so normally there
 *  is nothing to restore. Only when the update replaced or moved the field,
 *  dropping the focus, does the field now carrying its id get the focus and
 *  the selection back; a focus the update moved on purpose is kept. */
export function preserveFocus(update: () => void): void {
  const field = document.activeElement;
  if (!isInputField(field) || !field.id) {
    update();
    return;
  }
  const { id, selectionStart, selectionEnd, selectionDirection } = field;
  update();
  const active = document.activeElement;
  if (active && active !== document.body) return;
  const next = document.getElementById(id);
  if (!isInputField(next)) return;
  next.focus();
  if (selectionStart === null || selectionEnd === null) return;
  const length = next.value.length;
  next.setSelectionRange(
    Math.min(selectionStart, length),
    Math.min(selectionEnd, length),
    selectionDirection ?? undefined,
  );
}

/** A debounced "load, then render" for a field that every render
 *  re-attaches, such as a search bar. Each call replaces the pending one;
 *  `delayMs` after the last call, `load` runs (a backend query) and then
 *  `render`, inside preserveFocus.
 *
 *  Create one per field at module scope, not in the render closure: the
 *  pending timer has to outlive the render that scheduled it, or the next
 *  render (a landing search's own) orphans it and it still fires, a second
 *  query mid-typing. */
export function debouncedRender(
  delayMs: number,
): (load: () => Promise<void> | void, render: () => void) => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return (load, render) => {
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(async () => {
      timer = undefined;
      await load();
      preserveFocus(render);
    }, delayMs);
  };
}
