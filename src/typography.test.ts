import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

// The typography contract: one UI family and one monospace family, and every
// size and weight from the type scale, all set from CSS variables; form controls
// inherit the page font instead of the platform's control font. The rendered
// main screen, dialogs and a report are checked against the same tokens by
// features/ui/typography.feature.

const STYLESHEETS = ["style.css", "sync.css"].map((name) => ({
  name,
  css: readFileSync(new URL(`./${name}`, import.meta.url), "utf8"),
}));

const FAMILY_VALUES = ["var(--font-ui)", "var(--font-mono)", "var(--font-brand)", "inherit"];
const SIZE_VALUE = /^(var\(--text-(xs|sm|md|lg|xl|brand)\)|inherit)$/;
const WEIGHT_VALUE = /^var\(--weight-(regular|semibold|bold)\)$/;

/** The render code: every non-test TypeScript file under src/. */
const SOURCES = readdirSync(new URL(".", import.meta.url))
  .filter((name) => name.endsWith(".ts") && !name.endsWith(".test.ts"))
  .map((name) => ({ name, ts: readFileSync(new URL(`./${name}`, import.meta.url), "utf8") }));

/** The values of every `prop` declaration, @font-face blocks aside (custom
 *  properties such as `--font-mono:` are not `font` declarations). */
function declarations(css: string, prop: string): string[] {
  const rules = css.replace(/\/\*[\s\S]*?\*\//g, "").replace(/@font-face\s*\{[^}]*\}/g, "");
  return [...rules.matchAll(new RegExp(`(?<![-\\w])${prop}\\s*:\\s*([^;}]+)`, "g"))].map((m) => m[1].trim());
}

/** The body of the rule whose selector list is exactly `selectors`. */
function ruleBody(css: string, selectors: string[]): string | undefined {
  for (const m of css.matchAll(/([^{}]+)\{([^}]*)\}/g)) {
    const list = m[1].replace(/\/\*[\s\S]*?\*\//g, "").split(",").map((s) => s.trim());
    if (list.join(",") === selectors.join(",")) return m[2];
  }
  return undefined;
}

describe("typography", () => {
  it.each(STYLESHEETS)("$name takes every font family from a typography variable", ({ css }) => {
    const families = declarations(css, "font-family");
    expect(families.length).toBeGreaterThan(0);
    expect(families.filter((f) => !FAMILY_VALUES.includes(f))).toEqual([]);
    expect(declarations(css, "font").filter((f) => f !== "inherit")).toEqual([]);
  });

  it.each(STYLESHEETS)("$name takes every font size and weight from the type scale", ({ css }) => {
    expect(declarations(css, "font-size").filter((v) => !SIZE_VALUE.test(v))).toEqual([]);
    expect(declarations(css, "font-weight").filter((v) => !WEIGHT_VALUE.test(v))).toEqual([]);
  });

  it("sets the page's base size and the element defaults from the tokens", () => {
    const { css } = STYLESHEETS[0];
    expect(declarations(ruleBody(css, ["body"]) ?? "", "font-size")).toEqual(["var(--text-md)"]);
    expect(declarations(ruleBody(css, ["code", "kbd", "pre", "samp"]) ?? "", "font-family")).toEqual(["var(--font-mono)"]);
    expect(declarations(ruleBody(css, ["b", "strong"]) ?? "", "font-weight")).toEqual(["var(--weight-bold)"]);
  });

  it("keeps fonts in the render code on the tokens", () => {
    const literals = SOURCES.flatMap(({ name, ts }) => [
      // Inline styles in rendered markup: font-size:12px, font-family:monospace …
      ...[...ts.matchAll(/font-(?:size|weight|family)\s*:\s*([^;"'`<>]+)/g)]
        .filter((m) => !m[1].trim().startsWith("var(--"))
        .map((m) => `${name}: ${m[0]}`),
      // Option objects handed to libraries: fontSize: "12px", fontFamily: "…"
      ...[...ts.matchAll(/font(?:Size|Weight|Family)\s*:\s*["'`]/g)].map((m) => `${name}: ${m[0]}`),
    ]);
    expect(literals).toEqual([]);
  });

  it("defines one UI family and one monospace family, and the page uses the UI one", () => {
    const { css } = STYLESHEETS[0];
    const root = ruleBody(css, [":root"]) ?? "";
    expect(root.match(/--font-ui\s*:/g)).toHaveLength(1);
    expect(root.match(/--font-mono\s*:/g)).toHaveLength(1);
    expect(declarations(root, "font-family")).toEqual(["var(--font-ui)"]);
  });

  it("makes form controls inherit the page font", () => {
    const body = ruleBody(STYLESHEETS[0].css, ["button", "input", "select", "textarea"]) ?? "";
    expect(declarations(body, "font-family")).toEqual(["inherit"]);
    expect(declarations(body, "font-size")).toEqual(["var(--text-md)"]);
  });
});
