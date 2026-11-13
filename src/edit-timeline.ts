// The timeline of one table edit, from the click to the rebuilt rows landing.
//
// Each mark is one line in the debug log, `[edit] <label>: <mark> +<ms>`, with
// the time since the click; renders are counted as they happen. It is what
// says where an edit's wait goes — the write's round trip, the refresh
// queries, the renders — so a change to the path is measured in the app, not
// guessed. DOM-free; main.ts supplies the log and the clock.

export type EditTimeline = {
  /** Log `what` with the time since the click. */
  mark(what: string): void;
  /** A render happened: log it as `render #n`. */
  rendered(): void;
  /** Log the end and the totals. */
  end(what: string): void;
};

export function startEditTimeline(
  label: string,
  log: (line: string) => void,
  now: () => number = () => performance.now(),
): EditTimeline {
  const t0 = now();
  let renders = 0;
  const line = (what: string) => log(`[edit] ${label}: ${what} +${(now() - t0).toFixed(0)}ms`);
  line("click");
  return {
    mark: line,
    rendered() {
      renders += 1;
      line(`render #${renders}`);
    },
    end(what) {
      line(`${what}, ${renders} render${renders === 1 ? "" : "s"}`);
    },
  };
}
