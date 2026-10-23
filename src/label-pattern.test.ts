import { describe, expect, it } from "vitest";
import { payeeLabelPattern } from "./label-pattern";

describe("payeeLabelPattern", () => {
  it("strips the CBA card-feed tail so the label covers all of the merchant's rows", () => {
    expect(
      payeeLabelPattern("Google Workspace_acme.c Sydney AU Card xx1234 Value Date: 02/08/2026"),
    ).toBe("*Google Workspace_acme.c Sydney AU*");
  });

  it("strips a Value Date tail without a Card segment", () => {
    expect(payeeLabelPattern("Direct Debit ACME Value Date: 01/02/2026")).toBe(
      "*Direct Debit ACME*",
    );
  });

  it("wildcards invoice numbers so per-invoice payees share one label", () => {
    expect(
      payeeLabelPattern("XERO AU INV-54875839 HAWTHORN AU Card xx1234 Value Date: 04/08/2026"),
    ).toBe("*XERO AU INV-* HAWTHORN AU*");
  });

  it("wildcards standalone date and reference tokens mid-string", () => {
    expect(payeeLabelPattern("PAYMENT 12345678 RECEIVED 01/02/26 THANKS")).toBe(
      "*PAYMENT * RECEIVED * THANKS*",
    );
  });

  it("leaves digits embedded in alphanumeric tokens alone (addresses stay exact)", () => {
    expect(payeeLabelPattern("0xAbCdEf1234567890aBcDeF1234567890AbCdEf12")).toBe(
      "*0xAbCdEf1234567890aBcDeF1234567890AbCdEf12*",
    );
  });

  it("keeps a plain payee unchanged", () => {
    expect(payeeLabelPattern("Coffee Shop")).toBe("*Coffee Shop*");
  });

  it("falls back to the full payee when stripping removes everything", () => {
    expect(payeeLabelPattern("01/02/2026")).toBe("*01/02/2026*");
  });
});
