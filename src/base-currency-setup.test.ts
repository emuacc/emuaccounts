import { describe, it, expect } from "vitest";
import {
  BASE_CURRENCY_SETUP_TESTID,
  chosenBaseCurrency,
  needsBaseCurrencySetup,
  normalizeBaseCurrencyInput,
  preselectBaseCurrency,
  renderBaseCurrencySetup,
} from "./base-currency-setup";

describe("needsBaseCurrencySetup", () => {
  it("gates when the vault has no base currency", () => {
    expect(needsBaseCurrencySetup(null)).toBe(true);
    expect(needsBaseCurrencySetup(undefined)).toBe(true);
    expect(needsBaseCurrencySetup("")).toBe(true);
  });
  it("passes once the vault has one", () => {
    expect(needsBaseCurrencySetup("AUD")).toBe(false);
  });
});

describe("normalizeBaseCurrencyInput", () => {
  it("trims and upper-cases", () => {
    expect(normalizeBaseCurrencyInput("  usd ")).toBe("USD");
    expect(normalizeBaseCurrencyInput("   ")).toBe("");
  });
});

describe("preselectBaseCurrency", () => {
  it("prefers the set's existing override when it is a candidate", () => {
    expect(preselectBaseCurrency(["AUD", "USD"], "USD")).toBe("USD");
  });
  it("ignores an override that is not priced in this vault", () => {
    expect(preselectBaseCurrency(["AUD", "USD"], "GBP")).toBeUndefined();
  });
  it("picks the only candidate", () => {
    expect(preselectBaseCurrency(["USD"])).toBe("USD");
  });
  it("leaves the choice open otherwise", () => {
    expect(preselectBaseCurrency(["AUD", "USD"])).toBeUndefined();
    expect(preselectBaseCurrency([])).toBeUndefined();
  });
});

describe("chosenBaseCurrency", () => {
  it("typed text wins over the select", () => {
    expect(chosenBaseCurrency("AUD", "eur")).toBe("EUR");
  });
  it("falls back to the select", () => {
    expect(chosenBaseCurrency("AUD", "  ")).toBe("AUD");
  });
  it("is undefined when nothing was chosen", () => {
    expect(chosenBaseCurrency("", "")).toBeUndefined();
    expect(chosenBaseCurrency(undefined, undefined)).toBeUndefined();
  });
});

describe("renderBaseCurrencySetup", () => {
  it("renders the candidates as options with the preselection applied", () => {
    const html = renderBaseCurrencySetup({ appName: "EmuAcc", candidates: ["AUD", "USD"], preselected: "USD" });
    expect(html).toContain(`data-testid="${BASE_CURRENCY_SETUP_TESTID}"`);
    expect(html).toContain('<option value="AUD">AUD</option>');
    expect(html).toContain('<option value="USD" selected>USD</option>');
    expect(html).not.toContain('<option value="" selected>Select…</option>');
    expect(html).toContain('data-testid="base-currency-continue"');
  });
  it("offers a placeholder option when nothing is preselected", () => {
    const html = renderBaseCurrencySetup({ appName: "EmuAcc", candidates: ["AUD", "USD"] });
    expect(html).toContain('<option value="" selected>Select…</option>');
  });
  it("omits the select entirely when the vault has no priced currencies", () => {
    const html = renderBaseCurrencySetup({ appName: "EmuAcc", candidates: [] });
    expect(html).not.toContain("baseCurrencySelect");
    expect(html).toContain('data-testid="base-currency-input"');
  });
  it("shows the error and escapes it", () => {
    const html = renderBaseCurrencySetup({ appName: "EmuAcc", candidates: [], error: "<bad>" });
    expect(html).toContain('data-testid="base-currency-error"');
    expect(html).toContain("&lt;bad&gt;");
  });
});
