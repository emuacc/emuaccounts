import { describe, expect, it } from "vitest";
import { startEditTimeline } from "./edit-timeline";

describe("startEditTimeline", () => {
  it("logs each mark with the time since the click and counts the renders", () => {
    const lines: string[] = [];
    let t = 1000;
    const timeline = startEditTimeline('Category "x"', (line) => lines.push(line), () => t);

    t = 1030;
    timeline.mark("write landed");
    t = 1042;
    timeline.rendered();
    t = 1100;
    timeline.rendered();
    t = 1101;
    timeline.end("settled");

    expect(lines).toEqual([
      '[edit] Category "x": click +0ms',
      '[edit] Category "x": write landed +30ms',
      '[edit] Category "x": render #1 +42ms',
      '[edit] Category "x": render #2 +100ms',
      '[edit] Category "x": settled, 2 renders +101ms',
    ]);
  });

  it("says render, singular, for one", () => {
    const lines: string[] = [];
    const timeline = startEditTimeline("Hide", (line) => lines.push(line), () => 0);
    timeline.rendered();
    timeline.end("settled");
    expect(lines.at(-1)).toBe("[edit] Hide: settled, 1 render +0ms");
  });
});
