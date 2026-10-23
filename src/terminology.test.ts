import { describe, expect, it } from "vitest";
import catalogue from "../src-tauri/terminology.json";
import { defaultTerms, overridesOf, term, termTitle, type Term } from "./terminology";

const overridden: Term[] = [
  {
    key: "asset",
    singular: "holding",
    plural: "holdings",
    default_singular: "asset",
    default_plural: "assets",
    description: "x",
  },
];

describe("terminology catalogue", () => {
  it("every catalogue entry has singular, plural and description", () => {
    for (const [key, def] of Object.entries(catalogue)) {
      expect(def.singular, key).not.toBe("");
      expect(def.plural, key).not.toBe("");
      expect(def.description, key).not.toBe("");
    }
  });

  it("defaultTerms mirrors the catalogue with defaults equal to the effective wording", () => {
    const asset = defaultTerms().find((t) => t.key === "asset");
    expect(asset).toEqual({
      key: "asset",
      singular: "asset",
      plural: "assets",
      default_singular: "asset",
      default_plural: "assets",
      description: catalogue.asset.description,
    });
  });
});

describe("term", () => {
  it("falls back to the catalogue when no terms are loaded", () => {
    expect(term(undefined, "asset")).toBe("asset");
    expect(term(undefined, "asset", 1)).toBe("asset");
    expect(term(undefined, "asset", 2)).toBe("assets");
    expect(term(undefined, "asset", 0)).toBe("assets");
  });

  it("uses the resolved wording when terms are loaded", () => {
    expect(term(overridden, "asset")).toBe("holding");
    expect(term(overridden, "asset", 3)).toBe("holdings");
  });

  it("falls back per key when the loaded list lacks the term", () => {
    expect(term([], "asset", 2)).toBe("assets");
  });
});

describe("termTitle", () => {
  it("capitalises the first letter only", () => {
    expect(termTitle(undefined, "asset")).toBe("Asset");
    expect(termTitle(undefined, "asset", 2)).toBe("Assets");
    const multiWord: Term[] = [{ ...overridden[0], singular: "crypto coin" }];
    expect(termTitle(multiWord, "asset")).toBe("Crypto coin");
  });
});

describe("overridesOf", () => {
  it("is blank where the default applies and carries the override text otherwise", () => {
    expect(overridesOf(defaultTerms())).toEqual({ asset: { singular: "", plural: "" } });
    const singularOnly: Term[] = [{ ...overridden[0], plural: "assets" }];
    expect(overridesOf(singularOnly)).toEqual({ asset: { singular: "holding", plural: "" } });
  });
});
