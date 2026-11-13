// @vitest-environment jsdom
// Architectural fixture tests for the morphdom-based DOM update path.
//
// These prove the non-intrusive update contract: re-applying the same
// HTML produces zero structural changes; the existing DOM nodes keep
// their identity (and any DOM properties / attached state) across
// re-renders. This is the test of record for the "UI works on the
// local DOM" requirement — if a future refactor goes back to wholesale
// innerHTML replacement these will fail loudly.

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { debouncedRender, morphInPlace, playRowFlashes, preserveFocus } from "./dom-update";
import { type SearchPill, attachSmartSearch, renderSmartSearch } from "./smart-search";

function setup(html: string): HTMLElement {
  document.body.innerHTML = `<div id="root">${html}</div>`;
  return document.getElementById("root")!;
}

describe("morphInPlace — non-intrusive DOM updates", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("preserves node identity when the new HTML is identical", () => {
    const root = setup(`<table><tbody><tr id="r1"><td>A</td></tr></tbody></table>`);
    const beforeRow = root.querySelector("#r1");

    morphInPlace(
      root,
      `<div id="root"><table><tbody><tr id="r1"><td>A</td></tr></tbody></table></div>`,
    );

    const afterRow = root.querySelector("#r1");
    expect(afterRow).toBe(beforeRow);
  });

  it("preserves a JS property on a row across an identical render", () => {
    // Identity is the architectural guarantee — runtime DOM properties
    // (e.g. event listeners attached via addEventListener, custom JS
    // expandos) survive a re-render because morphdom keeps the
    // existing element rather than replacing it. Runtime *attributes*
    // not in the new HTML are still synced by morphdom — that's
    // expected; if we want them preserved we'd put them in the HTML.
    const root = setup(`<table><tbody><tr id="r1"><td>A</td></tr></tbody></table>`);
    const row = root.querySelector("#r1") as HTMLElement & { __testMarker?: string };
    row.__testMarker = "preserved";

    morphInPlace(
      root,
      `<div id="root"><table><tbody><tr id="r1"><td>A</td></tr></tbody></table></div>`,
    );

    expect(row.__testMarker).toBe("preserved");
    expect(document.contains(row)).toBe(true);
  });

  it("preserves identity for unchanged rows when only one row's text changes", () => {
    const root = setup(`
      <table><tbody>
        <tr id="r1"><td>A</td></tr>
        <tr id="r2"><td>B</td></tr>
        <tr id="r3"><td>C</td></tr>
      </tbody></table>
    `);
    const r1Before = root.querySelector("#r1");
    const r2Before = root.querySelector("#r2");
    const r3Before = root.querySelector("#r3");

    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr id="r1"><td>A</td></tr>
        <tr id="r2"><td>B-changed</td></tr>
        <tr id="r3"><td>C</td></tr>
      </tbody></table></div>`,
    );

    expect(root.querySelector("#r1")).toBe(r1Before);
    expect(root.querySelector("#r3")).toBe(r3Before);
    // r2's <tr> can survive too — morphdom updates the inner text on the
    // existing element rather than replacing it.
    expect(root.querySelector("#r2")).toBe(r2Before);
    expect(root.querySelector("#r2")?.textContent).toBe("B-changed");
  });

  it("removes a row that's gone from the new HTML", () => {
    const root = setup(`
      <ul>
        <li id="a">A</li>
        <li id="b">B</li>
        <li id="c">C</li>
      </ul>
    `);
    const aBefore = root.querySelector("#a");
    const cBefore = root.querySelector("#c");

    morphInPlace(
      root,
      `<div id="root"><ul>
        <li id="a">A</li>
        <li id="c">C</li>
      </ul></div>`,
    );

    expect(root.querySelector("#a")).toBe(aBefore);
    expect(root.querySelector("#b")).toBeNull();
    expect(root.querySelector("#c")).toBe(cBefore);
  });

  it("keyed rows: removing one txn row removes THAT row, not the last position", () => {
    // Without getNodeKey, morphdom matches children by tag+position. If
    // the new HTML has 4 rows and the old DOM has 5, morphdom would
    // position-update rows 0..3 (their content shifts up to match the
    // new sequence) and remove row 4. The CLICKED row's <tr> survives
    // with the next row's content — visually "the row is still there"
    // and the bottom row vanishes. getNodeKey on data-txn-row-id makes
    // morphdom remove the actual deleted row.
    const root = setup(`
      <table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:a"><td>Alice</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:b"><td>Bob</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:c"><td>Carol</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:d"><td>Dave</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:e"><td>Eve</td></tr>
      </tbody></table>
    `);
    const aliceBefore = root.querySelector('[data-txn-row-id="txn:a"]');
    const carolBefore = root.querySelector('[data-txn-row-id="txn:c"]');
    const eveBefore = root.querySelector('[data-txn-row-id="txn:e"]');

    // New HTML omits "txn:c" — Carol was deleted.
    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:a"><td>Alice</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:b"><td>Bob</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:d"><td>Dave</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:e"><td>Eve</td></tr>
      </tbody></table></div>`,
    );

    // Alice and Eve must survive with their original DOM identity.
    // Carol must be gone. The clicked row IS the one removed.
    expect(root.querySelector('[data-txn-row-id="txn:a"]')).toBe(aliceBefore);
    expect(root.querySelector('[data-txn-row-id="txn:c"]')).toBeNull();
    expect(root.querySelector('[data-txn-row-id="txn:e"]')).toBe(eveBefore);
    // The "carolBefore" reference shouldn't be in the DOM anywhere.
    expect(document.contains(carolBefore!)).toBe(false);
    // Total row count is now 4.
    expect(root.querySelectorAll('[data-testid="txn-row"]').length).toBe(4);
  });

  it("multi-leg txn: each leg is keyed independently by data-expand-key", () => {
    // Live-app reproduction: a single on-chain Solana txn renders as N
    // separate <tr>s (one per posting/leg). They all share the same
    // data-txn-row-id (the meta's txn id), so keying by it collapses
    // the whole group to one node in morphdom's map. Only data-expand-key
    // is unique per leg (txnId|datetime|amount|narration). With it the
    // identity of every leg survives across renders and an "expand
    // leg-2 only" render keeps the right detail row attached.
    const root = setup(`
      <table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:multi" data-expand-key="txn:multi|t|10|leg-out"><td>Out 10</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:multi" data-expand-key="txn:multi|t|-10|leg-in"><td>In -10</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:multi" data-expand-key="txn:multi|t|0.1|leg-fee"><td>Fee 0.1</td></tr>
      </tbody></table>
    `);
    const outBefore = root.querySelector('[data-expand-key="txn:multi|t|10|leg-out"]');
    const inBefore = root.querySelector('[data-expand-key="txn:multi|t|-10|leg-in"]');
    const feeBefore = root.querySelector('[data-expand-key="txn:multi|t|0.1|leg-fee"]');

    // User clicks the "in" leg → only THAT leg expands. New HTML
    // inserts the detail row right after the "in" leg.
    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:multi" data-expand-key="txn:multi|t|10|leg-out"><td>Out 10</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:multi" data-expand-key="txn:multi|t|-10|leg-in"><td>In -10</td></tr>
        <tr class="txRow__detail" data-txn-detail-for="txn:multi|t|-10|leg-in"><td>Detail for in</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:multi" data-expand-key="txn:multi|t|0.1|leg-fee"><td>Fee 0.1</td></tr>
      </tbody></table></div>`,
    );

    // Every leg's identity survives — none of them was reused as a
    // peer or merged.
    expect(root.querySelector('[data-expand-key="txn:multi|t|10|leg-out"]')).toBe(outBefore);
    expect(root.querySelector('[data-expand-key="txn:multi|t|-10|leg-in"]')).toBe(inBefore);
    expect(root.querySelector('[data-expand-key="txn:multi|t|0.1|leg-fee"]')).toBe(feeBefore);

    // The detail row is positioned right after the "in" leg, not
    // floating after the "fee" leg or somewhere else.
    expect(inBefore?.nextElementSibling?.getAttribute("data-txn-detail-for"))
      .toBe("txn:multi|t|-10|leg-in");

    // No leg was duplicated.
    const allLegs = root.querySelectorAll<HTMLTableRowElement>('[data-testid="txn-row"]');
    expect(allLegs.length).toBe(3);
    const expandKeys = Array.from(allLegs).map((r) => r.getAttribute("data-expand-key"));
    expect(new Set(expandKeys).size).toBe(3);
  });

  it("multi-leg swap rows: removing one leg leaves the other intact", () => {
    // Regression: a single on-chain transaction with multiple postings
    // renders as multiple <tr>s, all carrying the SAME data-txn-row-id
    // (the meta's txn id, not the posting). getNodeKey returns
    // `txn:${txnRowId}` for both, which collides in morphdom's keyed
    // map. When one leg leaves the DOM before a render (an optimistic
    // "remove" edit drops a txn's rows, a later render brings legs
    // back), morphdom's diff has to handle the surviving sibling
    // gracefully.
    //
    // Symptom: clicking another row shows a "hidden" row appearing —
    // the surviving sibling that morphdom didn't track right.
    const root = setup(`
      <table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:swap" data-expand-key="txn:swap|leg1"><td>Leg 1</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:swap" data-expand-key="txn:swap|leg2"><td>Leg 2</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:other" data-expand-key="txn:other|x"><td>Other</td></tr>
      </tbody></table>
    `);
    const leg2Before = root.querySelector('[data-expand-key="txn:swap|leg2"]');
    const otherBefore = root.querySelector('[data-expand-key="txn:other|x"]');

    // leg1 leaves the DOM ahead of the render.
    leg2Before!.previousElementSibling!.remove();

    // The next render's HTML still contains the surviving leg2 plus
    // the unrelated other row. morphdom should keep leg2 in place.
    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:swap" data-expand-key="txn:swap|leg2"><td>Leg 2</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:other" data-expand-key="txn:other|x"><td>Other</td></tr>
      </tbody></table></div>`,
    );

    expect(root.querySelector('[data-expand-key="txn:swap|leg1"]')).toBeNull();
    expect(root.querySelector('[data-expand-key="txn:swap|leg2"]')).toBe(leg2Before);
    expect(root.querySelector('[data-expand-key="txn:other|x"]')).toBe(otherBefore);
    expect(root.querySelectorAll("[data-txn-row-id]").length).toBe(2);
  });

  it("multi-leg swap rows: a render that re-adds a deleted leg restores the right row", () => {
    // After an optimistic delete is withdrawn (the rebuild did not
    // apply it), a subsequent render brings the removed siblings back.
    // morphdom must put the new row in the right position next to
    // the surviving sibling and not orphan or duplicate it.
    const root = setup(`
      <table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:swap" data-expand-key="txn:swap|leg2"><td>Leg 2</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:other" data-expand-key="txn:other|x"><td>Other</td></tr>
      </tbody></table>
    `);

    // Render restores leg1 (e.g. user undid, or pipeline rebuilt).
    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:swap" data-expand-key="txn:swap|leg1"><td>Leg 1</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:swap" data-expand-key="txn:swap|leg2"><td>Leg 2</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:other" data-expand-key="txn:other|x"><td>Other</td></tr>
      </tbody></table></div>`,
    );

    const trs = root.querySelectorAll<HTMLTableRowElement>("[data-txn-row-id]");
    expect(trs.length).toBe(3);
    expect(trs[0].getAttribute("data-expand-key")).toBe("txn:swap|leg1");
    expect(trs[1].getAttribute("data-expand-key")).toBe("txn:swap|leg2");
    expect(trs[2].getAttribute("data-expand-key")).toBe("txn:other|x");
  });

  it("detail row sticks with its owning row when earlier rows disappear", () => {
    // Live-app reproduction: rowA is expanded (so DOM has [rowA,
    // detail-A, rowB, detail-B]). The user deletes rowA via the X
    // button: the pending "remove" edit makes the next render exclude
    // rowA and its detail row. New HTML is
    // [rowB, detail-B]. morphdom must NOT leave detail-A's slot
    // "stuck" between rows so detail-B drifts to the wrong row.
    //
    // Without keying detail rows by data-txn-detail-for, the unkeyed
    // detail row from a previous render would morph in place against
    // whatever <tr> happened to be in that slot now — surfacing as a
    // detail row sitting under an unrelated row.
    const root = setup(`
      <table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:a"><td>Alice</td></tr>
        <tr class="txRow__detail" data-txn-detail-for="txn:a"><td>Detail A</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:b"><td>Bob</td></tr>
        <tr class="txRow__detail" data-txn-detail-for="txn:b"><td>Detail B</td></tr>
      </tbody></table>
    `);
    const detailBBefore = root.querySelector('[data-txn-detail-for="txn:b"]');

    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:b"><td>Bob</td></tr>
        <tr class="txRow__detail" data-txn-detail-for="txn:b"><td>Detail B</td></tr>
      </tbody></table></div>`,
    );

    expect(root.querySelector('[data-txn-row-id="txn:a"]')).toBeNull();
    expect(root.querySelector('[data-txn-detail-for="txn:a"]')).toBeNull();
    expect(root.querySelector('[data-txn-detail-for="txn:b"]')).toBe(detailBBefore);
    // Critical: detail-B must follow row-B, not be a stranded sibling.
    const rowB = root.querySelector('[data-txn-row-id="txn:b"]');
    expect(rowB?.nextElementSibling).toBe(detailBBefore);
    expect(root.querySelectorAll("tr").length).toBe(2);
  });

  it("re-ordered rows: detail row moves with its owning row", () => {
    // After a sort change the row order flips, but detail rows must
    // stay attached to their txn row, not to whatever sat at their
    // previous index.
    const root = setup(`
      <table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:a"><td>Alice</td></tr>
        <tr class="txRow__detail" data-txn-detail-for="txn:a"><td>Detail A</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:b"><td>Bob</td></tr>
      </tbody></table>
    `);

    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-testid="txn-row" data-txn-row-id="txn:b"><td>Bob</td></tr>
        <tr data-testid="txn-row" data-txn-row-id="txn:a"><td>Alice</td></tr>
        <tr class="txRow__detail" data-txn-detail-for="txn:a"><td>Detail A</td></tr>
      </tbody></table></div>`,
    );

    const trs = root.querySelectorAll<HTMLTableRowElement>("tr");
    expect(trs.length).toBe(3);
    expect(trs[0].getAttribute("data-txn-row-id")).toBe("txn:b");
    expect(trs[1].getAttribute("data-txn-row-id")).toBe("txn:a");
    expect(trs[2].getAttribute("data-txn-detail-for")).toBe("txn:a");
  });

  it("does NOT clobber the value of a focused input mid-edit", () => {
    const root = setup(`<form><input id="search" value="" /></form>`);
    const input = root.querySelector<HTMLInputElement>("#search")!;
    input.focus();
    input.value = "user typed this";
    expect(document.activeElement).toBe(input);

    // A render fires while the user is typing — the new HTML wants to
    // restore value="" because the renderer doesn't know the user is
    // editing. morphdom's onBeforeElUpdated should skip the focused
    // input.
    morphInPlace(
      root,
      `<div id="root"><form><input id="search" value="" /></form></div>`,
    );

    expect(input.value).toBe("user typed this");
    expect(document.activeElement).toBe(input);
  });

  it("preserves an injected inline cell editor across a background render", () => {
    // Live-app reproduction: the user clicks a payee cell, startCellEdit
    // replaces the cell content with an injected input, and a background
    // render (pipeline auto-rebuild, reports-rebuilt, heavy refresh)
    // lands while they type. The freshly built HTML still describes the
    // ORIGINAL cell content — without the data-cell-editing guard,
    // morphdom discards the editor mid-keystroke (and removing a focused
    // node fires no blur, so nothing is saved).
    const root = setup(`
      <table><tbody>
        <tr data-expand-key="k1"><td class="cell-clickable" data-rule-type="payee"><span class="copyable">Old Payee</span></td></tr>
      </tbody></table>
    `);
    const td = root.querySelector("td")!;
    // What startCellEdit does: inject wrapper+input, mark the cell.
    const wrapper = document.createElement("div");
    wrapper.className = "cell-edit-wrapper";
    const input = document.createElement("input");
    input.className = "cell-edit-input";
    input.value = "SP"; // the user got two letters in
    wrapper.appendChild(input);
    td.innerHTML = "";
    td.appendChild(wrapper);
    td.setAttribute("data-cell-editing", "true");
    input.focus();

    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-expand-key="k1"><td class="cell-clickable" data-rule-type="payee"><span class="copyable">Old Payee</span></td></tr>
      </tbody></table></div>`,
    );

    const after = root.querySelector<HTMLInputElement>(".cell-edit-input");
    expect(after).toBe(input);
    expect(after!.value).toBe("SP");
    expect(document.activeElement).toBe(input);
  });

  it("reclaims the cell once the inline edit is finished (mark removed)", () => {
    // After save/cancel the editing mark is removed — the next render
    // must be able to morph the cell back to rendered content, or the
    // input would linger forever after a save.
    const root = setup(`
      <table><tbody>
        <tr data-expand-key="k1"><td class="cell-clickable"><div class="cell-edit-wrapper"><input class="cell-edit-input" /></div></td></tr>
      </tbody></table>
    `);

    morphInPlace(
      root,
      `<div id="root"><table><tbody>
        <tr data-expand-key="k1"><td class="cell-clickable"><span class="copyable">SPAM</span></td></tr>
      </tbody></table></div>`,
    );

    expect(root.querySelector(".cell-edit-input")).toBeNull();
    expect(root.querySelector("td")?.textContent).toBe("SPAM");
  });
});

describe("playRowFlashes", () => {
  function table(): HTMLElement {
    return setup(`<table><tbody>
      <tr data-expand-key="leg:a"><td>a1</td><td>a2</td></tr>
      <tr data-expand-key="txn:b|2026|1|n"><td>b1</td></tr>
      <tr data-expand-key="leg:c"><td>c1</td></tr>
    </tbody></table>`);
  }

  function stubAnimate(root: HTMLElement): Map<string, Keyframe[][]> {
    const calls = new Map<string, Keyframe[][]>();
    root.querySelectorAll("td").forEach((td) => {
      (td as unknown as { animate: unknown }).animate = (frames: Keyframe[]) => {
        const list = calls.get(td.textContent ?? "") ?? [];
        list.push(frames);
        calls.set(td.textContent ?? "", list);
        return {} as Animation;
      };
    });
    return calls;
  }

  it("animates every cell of a changed row and of a reverted row, in their own colours", () => {
    const root = table();
    const calls = stubAnimate(root);
    playRowFlashes(root, { changed: new Set(["leg:a"]), reverted: new Set(["txn:b|2026|1|n"]) });
    expect([...calls.keys()].sort()).toEqual(["a1", "a2", "b1"]);
    expect(String(calls.get("a1")![0][0].boxShadow)).toContain("59, 130, 246");
    expect(String(calls.get("b1")![0][0].boxShadow)).toContain("245, 158, 11");
  });

  it("ends every flash fully transparent and leaves the markup alone", () => {
    const root = table();
    const calls = stubAnimate(root);
    const before = root.innerHTML;
    playRowFlashes(root, { changed: new Set(["leg:c"]), reverted: new Set() });
    expect(String(calls.get("c1")![0][1].boxShadow)).toMatch(/, 0\)$/);
    expect(root.innerHTML).toBe(before);
  });

  it("skips keys with no rendered row and tolerates no Web Animations support", () => {
    const root = table();
    expect(() => playRowFlashes(root, { changed: new Set(["leg:gone", "leg:a"]), reverted: new Set() })).not.toThrow();
    expect(() => playRowFlashes(root, undefined)).not.toThrow();
  });
});

describe("preserveFocus — a render never moves the user's caret", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  function focusField(root: HTMLElement, start: number, end = start, direction: "forward" | "backward" = "forward") {
    const input = root.querySelector<HTMLInputElement>("#q")!;
    input.focus();
    input.setSelectionRange(start, end, direction);
    return input;
  }

  it("leaves a field morphInPlace kept, with the caret the user has now", () => {
    const root = setup(`<input id="q" value="office" />`);
    const input = focusField(root, 6);
    input.setSelectionRange(2, 2); // moved after the work was scheduled

    preserveFocus(() => morphInPlace(root, `<div id="root"><input id="q" value="of" /></div>`));

    expect(root.querySelector("#q")).toBe(input);
    expect([input.value, input.selectionStart, input.selectionEnd]).toEqual(["office", 2, 2]);
    expect(document.activeElement).toBe(input);
  });

  it("gives a field the update replaced the focus and the selection back", () => {
    const root = setup(`<input id="q" value="office" />`);
    focusField(root, 2, 4, "backward");

    preserveFocus(() => {
      root.innerHTML = `<input id="q" value="office" />`;
    });

    const next = root.querySelector<HTMLInputElement>("#q")!;
    expect(document.activeElement).toBe(next);
    expect([next.selectionStart, next.selectionEnd, next.selectionDirection]).toEqual([2, 4, "backward"]);
  });

  it("clamps the selection to the replaced field's value", () => {
    const root = setup(`<input id="q" value="office" />`);
    focusField(root, 6);

    preserveFocus(() => {
      root.innerHTML = `<input id="q" value="off" />`;
    });

    const next = root.querySelector<HTMLInputElement>("#q")!;
    expect([next.selectionStart, next.selectionEnd]).toEqual([3, 3]);
  });

  it("keeps a focus the update moved on purpose", () => {
    const root = setup(`<input id="q" value="office" />`);
    focusField(root, 6);

    preserveFocus(() => {
      root.innerHTML = `<input id="q" value="office" /><input id="other" />`;
      root.querySelector<HTMLInputElement>("#other")!.focus();
    });

    expect(document.activeElement?.id).toBe("other");
  });

  it("just runs the update when no text field has the focus", () => {
    const root = setup(`<button id="b">go</button>`);
    root.querySelector<HTMLButtonElement>("#b")!.focus();
    const update = vi.fn();

    preserveFocus(update);

    expect(update).toHaveBeenCalledOnce();
    expect(document.activeElement?.id).toBe("b");
  });
});

describe("debouncedRender — typing faster than the debounce plus the search (TASK-150)", () => {
  const DEBOUNCE_MS = 300;
  const SEARCH_MS = 800;
  const KEY_GAP_MS = 400; // longer than the debounce, shorter than the debounce plus the search
  const CONFIG = { keywords: ["payee"] };

  beforeEach(() => {
    vi.useFakeTimers();
    document.body.innerHTML = "";
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  /** The accounts search the way main.ts wires it: every render morphs the
   *  page from `state` and re-attaches the smart search bar, and each change
   *  schedules the search (SEARCH_MS in the backend), then a render. */
  function mountSearchPage() {
    const state = { pills: [] as SearchPill[], text: "", results: "", searches: 0 };
    const scheduleSearch = debouncedRender(DEBOUNCE_MS);
    document.body.innerHTML = `<div id="app"></div>`;
    const app = document.getElementById("app")!;
    let detach = () => {};
    function render() {
      morphInPlace(
        app,
        `<div id="app">${renderSmartSearch("search", state.pills, state.text, CONFIG)}` +
          `<p id="results">${state.results}</p><input id="other" /></div>`,
      );
      detach();
      detach = attachSmartSearch("search", state.pills, CONFIG, (change) => {
        state.pills = change.pills;
        state.text = change.text;
        scheduleSearch(async () => {
          const query = state.text;
          state.searches += 1;
          await new Promise((resolve) => setTimeout(resolve, SEARCH_MS));
          state.results = `results for ${query}`;
        }, render);
      });
    }
    render();
    const input = document.getElementById("searchInput") as HTMLInputElement;
    input.focus();
    return { state, input, results: () => document.getElementById("results")!.textContent };
  }

  /** Type one character at the caret of the focused field, as the browser does. */
  function typeKey(ch: string): void {
    const field = document.activeElement as HTMLInputElement;
    field.setRangeText(ch, field.selectionStart ?? 0, field.selectionEnd ?? 0, "end");
    field.dispatchEvent(new Event("input", { bubbles: true }));
  }

  async function typeFast(text: string): Promise<void> {
    for (const [i, ch] of [...text].entries()) {
      if (i > 0) await vi.advanceTimersByTimeAsync(KEY_GAP_MS);
      typeKey(ch);
    }
  }

  it("keeps every character in order and the caret where the user is typing", async () => {
    const { state, input, results } = mountSearchPage();

    await typeFast("office");

    expect(state.results).not.toBe(""); // searches landed while the user typed
    expect([input.value, input.selectionStart, input.selectionEnd]).toEqual(["office", 6, 6]);

    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + SEARCH_MS);

    expect(results()).toBe("results for office");
    expect(document.activeElement).toBe(input);
    expect([input.value, input.selectionStart, input.selectionEnd]).toEqual(["office", 6, 6]);
  });

  it("keeps a caret move the user makes while a search is running", async () => {
    const { input, results } = mountSearchPage();

    await typeFast("office");
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS); // the last key's search is now running
    input.setSelectionRange(2, 2);
    await vi.advanceTimersByTimeAsync(SEARCH_MS);

    expect(results()).toBe("results for office");
    expect([input.value, input.selectionStart, input.selectionEnd]).toEqual(["office", 2, 2]);
  });

  it("leaves the focus where the user moved it while a search was running", async () => {
    const { results } = mountSearchPage();

    await typeFast("office");
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS);
    document.getElementById("other")!.focus();
    await vi.advanceTimersByTimeAsync(SEARCH_MS);

    expect(results()).toBe("results for office");
    expect(document.activeElement?.id).toBe("other");
  });

  it("runs one search for a burst of keys inside the debounce", async () => {
    const { state, input, results } = mountSearchPage();

    for (const ch of "office") typeKey(ch);
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + SEARCH_MS);

    expect(state.searches).toBe(1);
    expect(results()).toBe("results for office");
    expect([input.value, input.selectionStart]).toEqual(["office", 6]);
  });
});
