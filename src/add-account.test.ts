import { describe, expect, it } from "vitest";
import {
  ACCOUNT_KINDS,
  NEW_ENTITY,
  buildAccountName,
  deriveWalletLabel,
  isCryptoKind,
  kindPrefix,
  normalizeWalletAddress,
  resolveEntityChoice,
  shouldPromptImport,
  validateEntityName,
  initialEntityChoice,
} from "./add-account";

describe("ACCOUNT_KINDS", () => {
  it("offers the seven kinds, folder-backed first, with correct prefixes", () => {
    expect(ACCOUNT_KINDS.map((k) => k.value)).toEqual([
      "bank", "crypto", "broker", "asset", "liabilities", "expenses", "equity",
    ]);
    expect(kindPrefix("bank")).toBe("assets:cash:bank");
    expect(kindPrefix("crypto")).toBe("assets:crypto:wallet");
    expect(kindPrefix("broker")).toBe("assets:equity:broker");
    expect(kindPrefix("asset")).toBe("assets");
    expect(kindPrefix("expenses")).toBe("expenses");
    expect(kindPrefix("equity")).toBe("equity");
    expect(kindPrefix("liabilities")).toBe("liabilities");
  });
});

describe("buildAccountName", () => {
  it("prefixes the kind for plain names", () => {
    expect(buildAccountName("bank", { accountName: "cba:smartaccess" })).toBe(
      "assets:cash:bank:cba:smartaccess",
    );
    expect(buildAccountName("broker", { accountName: "commsec:personal" })).toBe(
      "assets:equity:broker:commsec:personal",
    );
    expect(buildAccountName("expenses", { accountName: "office:supplies" })).toBe(
      "expenses:office:supplies",
    );
  });

  it("honours a typed name that already starts with the kind's top-level category", () => {
    expect(buildAccountName("bank", { accountName: "assets:bank:savings" })).toBe(
      "assets:bank:savings",
    );
    expect(buildAccountName("expenses", { accountName: "expenses:office" })).toBe(
      "expenses:office",
    );
  });

  it("throws on an empty name", () => {
    expect(() => buildAccountName("bank", { accountName: "  " })).toThrow();
  });

  it("builds crypto names from chain + normalized address", () => {
    expect(
      buildAccountName("crypto", {
        accountName: "",
        chain: "ethereum",
        address: " 0xAbCdEf1234567890aBcDeF1234567890AbCdEf12 ",
      }),
    ).toBe("assets:crypto:wallet:ethereum:0xabcdef1234567890abcdef1234567890abcdef12");
    expect(
      buildAccountName("crypto", {
        accountName: "",
        chain: "solana",
        address: "9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc",
      }),
    ).toBe("assets:crypto:wallet:solana:9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc");
  });

  it("throws on a missing crypto address", () => {
    expect(() =>
      buildAccountName("crypto", { accountName: "", chain: "ethereum", address: " " }),
    ).toThrow();
  });
});

describe("normalizeWalletAddress", () => {
  it("lower-cases only 0x/0X addresses, mirroring crypto-sync models.py", () => {
    expect(normalizeWalletAddress("0XABCDEF")).toBe("0xabcdef");
    expect(normalizeWalletAddress(" 0xAbC ")).toBe("0xabc");
    expect(normalizeWalletAddress("9Abc9Abc9Abc")).toBe("9Abc9Abc9Abc");
    expect(normalizeWalletAddress("osmo12Grf7P9")).toBe("osmo12Grf7P9");
  });
});

describe("deriveWalletLabel", () => {
  it("builds chain-<first6>…<last4> from the normalized address", () => {
    expect(deriveWalletLabel("ethereum", "0xAbCdEf1234567890aBcDeF1234567890AbCdEf12")).toBe(
      "ethereum-0xabcd…ef12",
    );
    expect(deriveWalletLabel("solana", "9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc")).toBe(
      "solana-9Abc9A…9Abc",
    );
  });

  it("uses the whole address when it is short", () => {
    expect(deriveWalletLabel("cosmos", "osmo1abc")).toBe("cosmos-osmo1abc");
  });

  it("is empty-chain/address safe", () => {
    expect(deriveWalletLabel("ethereum", "")).toBe("ethereum-");
  });
});

describe("shouldPromptImport", () => {
  it("prompts only for folder-backed, manually imported kinds", () => {
    expect(shouldPromptImport("bank")).toBe(true);
    expect(shouldPromptImport("broker")).toBe(true);
    expect(shouldPromptImport("asset")).toBe(true);
    expect(shouldPromptImport("liabilities")).toBe(true);
    expect(shouldPromptImport("crypto")).toBe(false);
    expect(shouldPromptImport("expenses")).toBe(false);
    expect(shouldPromptImport("equity")).toBe(false);
  });
});

describe("isCryptoKind", () => {
  it("identifies the crypto kind", () => {
    expect(isCryptoKind("crypto")).toBe(true);
    expect(isCryptoKind("bank")).toBe(false);
  });
});

describe("resolveEntityChoice", () => {
  it("returns the picked entity, or the typed name for the new-entity sentinel", () => {
    expect(resolveEntityChoice("alex", "", "fallback")).toBe("alex");
    expect(resolveEntityChoice(NEW_ENTITY, " Acme ", "fallback")).toBe("Acme");
    expect(resolveEntityChoice("", "", "fallback")).toBe("fallback");
  });
});

describe("validateEntityName", () => {
  it("accepts clean names", () => {
    expect(validateEntityName("Acme", ["alex", "sam"])).toBeNull();
  });

  it("rejects empties, path/separator chars, hyphens, and leading underscore/dot", () => {
    for (const bad of ["", "  ", "a/b", "a\\b", "a:b", "my-co", "_x", ".x"]) {
      expect(validateEntityName(bad, []), bad).not.toBeNull();
    }
  });

  it("rejects case-insensitive collisions with existing entities", () => {
    expect(validateEntityName("Alex", ["alex"])).not.toBeNull();
  });
});

describe("initialEntityChoice", () => {
  it("starts on the selected account set, else the first one", () => {
    expect(initialEntityChoice("acme", ["personal", "acme"])).toBe("acme");
    expect(initialEntityChoice(undefined, ["personal", "acme"])).toBe("personal");
  });

  it("starts on 'new entity' in a vault with no account sets, so the name input is shown", () => {
    // Regression: an empty default left the single-option select unable to
    // fire a change event, so the very first account could never be added.
    expect(initialEntityChoice(undefined, [])).toBe(NEW_ENTITY);
  });
});
