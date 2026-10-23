/** Add Account kinds: the single source of truth for the modal's dropdown and
 * name building. Bank/Broker/Crypto expand to the ACCOUNTS.md conventions;
 * "asset" is the free-form assets fallback; "liabilities" (credit cards,
 * loans) is folder-backed like the asset kinds — the backend creates
 * <set>/liabilities/<rest>/ — while expenses and equity are declaration-only
 * (routed to <set>/_accounts by the backend). */

export interface AccountKind {
  value: string;
  label: string;
  prefix: string;
}

export const ACCOUNT_KINDS: AccountKind[] = [
  { value: "bank", label: "Bank", prefix: "assets:cash:bank" },
  { value: "crypto", label: "Crypto Wallet", prefix: "assets:crypto:wallet" },
  { value: "broker", label: "Broker", prefix: "assets:equity:broker" },
  { value: "asset", label: "Other Asset", prefix: "assets" },
  { value: "liabilities", label: "Liabilities", prefix: "liabilities" },
  { value: "expenses", label: "Expenses", prefix: "expenses" },
  { value: "equity", label: "Equity", prefix: "equity" },
];

export function kindPrefix(kind: string): string {
  return ACCOUNT_KINDS.find((k) => k.value === kind)?.prefix ?? "assets";
}

/** Sentinel value of the entity <select> meaning "create a new entity". */
export const NEW_ENTITY = "__new__";

/** What the entity select starts on when the Add Account form opens: the
 *  current account set, else the first one, else — in a vault with no account
 *  sets yet — "new entity", so the name input is on screen. With no sets the
 *  select has that single option and can never fire a change event, so an
 *  empty default left the very first account impossible to add. */
export function initialEntityChoice(selectedAccountSet: string | undefined, accountSets: string[]): string {
  return selectedAccountSet ?? accountSets[0] ?? NEW_ENTITY;
}

/** The effective entity for a submission: the picked one, the typed name for
 *  the new-entity sentinel, or the fallback (current tab) when unset. */
export function resolveEntityChoice(
  entity: string,
  newEntityName: string,
  fallback: string,
): string {
  if (entity === NEW_ENTITY) return newEntityName.trim();
  return entity || fallback;
}

/** Validate a new entity name; returns an error message or null when valid.
 *  Hyphens are rejected because set listing truncates names at the first "-". */
export function validateEntityName(name: string, existing: string[]): string | null {
  const trimmed = name.trim();
  if (!trimmed) return "Entity name is required.";
  if (/[/\\:]/.test(trimmed)) return "Entity name cannot contain / \\ or :";
  if (trimmed.includes("-")) return "Entity name cannot contain hyphens.";
  if (trimmed.startsWith("_") || trimmed.startsWith(".")) {
    return "Entity name cannot start with _ or .";
  }
  if (existing.some((e) => e.toLowerCase() === trimmed.toLowerCase())) {
    return `Entity "${trimmed}" already exists.`;
  }
  return null;
}

export function isCryptoKind(kind: string): boolean {
  return kind === "crypto";
}

/** Only folder-backed, manually-imported kinds get the post-add CSV/OFX
 * prompt. Crypto folders are populated by the crypto-sync plugin; expenses
 * and equity have no source folder at all. */
export function shouldPromptImport(kind: string): boolean {
  return kind === "bank" || kind === "broker" || kind === "asset" || kind === "liabilities";
}

/** Mirror of crypto-sync's normalization (plugins/crypto-sync/
 * sync_crypto/models.py): trim, and lower-case only 0x/0X addresses.
 * Solana and cosmos addresses are case-sensitive and stay verbatim. */
export function normalizeWalletAddress(address: string): string {
  const trimmed = address.trim();
  return trimmed.slice(0, 2).toLowerCase() === "0x" ? trimmed.toLowerCase() : trimmed;
}

/** Default wallet label: `<chain>-<first6>…<last4>` of the normalized
 * address; short addresses are used whole. */
export function deriveWalletLabel(chain: string, address: string): string {
  const a = normalizeWalletAddress(address);
  const short = a.length > 10 ? `${a.slice(0, 6)}…${a.slice(-4)}` : a;
  return `${chain}-${short}`;
}

export function buildAccountName(
  kind: string,
  draft: { accountName: string; chain?: string; address?: string },
): string {
  if (isCryptoKind(kind)) {
    const address = normalizeWalletAddress(draft.address ?? "");
    if (!address) {
      throw new Error("Wallet address is required.");
    }
    return `${kindPrefix(kind)}:${draft.chain}:${address}`;
  }
  const rawName = draft.accountName.trim();
  if (!rawName) {
    throw new Error("Account name is required.");
  }
  // Honour a typed name that already starts with the kind's top-level
  // category (e.g. "assets:bank:savings" under the Bank kind).
  const topLevel = kindPrefix(kind).split(":")[0];
  return rawName.startsWith(`${topLevel}:`) ? rawName : `${kindPrefix(kind)}:${rawName}`;
}
