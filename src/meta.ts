/**
 * Pure helpers for working with the comma-separated `meta` string that the
 * pipeline stamps onto every transaction (e.g. "txn:HASH,rule:R1,swap:txn:OTHER").
 *
 * These are extracted from src/main.ts so they can be imported and unit-tested
 * directly (main.ts has no exports because it owns mutable module-level state).
 */

/** True when `acct` equals `prefix` or is a child (e.g. "a:b:c" matches "a:b"). */
export function matchesPrefix(acct: string, prefix: string): boolean {
  return acct === prefix || acct.startsWith(prefix + ":");
}

/**
 * The search term matching the transactions with a leg on `account` or a child
 * of it, as {@link matchesPrefix} does. The query engine compiles an `account:`
 * pattern as an unanchored, case-insensitive regex — `account:income` also
 * matches `expenses:tax:income` — so the account is escaped, anchored at a
 * segment boundary and matched case-sensitively.
 */
export function accountSubtreeTerm(account: string): string {
  return `account:(?-i)^${account.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}(:|$)`;
}

/** Extract the `rule:<id>` value, or undefined when absent. */
export function extractRuleId(meta?: string | null): string | undefined {
  if (!meta) return undefined;
  const match = meta.split(",").map((s) => s.trim()).find((s) => s.startsWith("rule:"));
  return match ? match.slice(5) : undefined;
}

/** Extract the `txn:<id>` entry from a comma-separated meta string. Returns "" when absent. */
export function txnIdFromMeta(meta: string | null | undefined): string {
  return (meta ?? "").split(",").map((p) => p.trim()).find((p) => p.startsWith("txn:")) ?? "";
}

/**
 * Extract the `leg:<id>` entry — the per-leg id the pipeline stamps when several
 * legs of one on-chain transaction share a `txn:` id (swap / wrap / multi-hop).
 * Returns "" when absent (single-leg rows are uniquely identified by `txn:`).
 */
export function legIdFromMeta(meta: string | null | undefined): string {
  return (meta ?? "").split(",").map((p) => p.trim()).find((p) => p.startsWith("leg:")) ?? "";
}

/** Port of rules.rs `id_anchor_prefix`: a bare `txn:<id>` / `leg:<id>`
 *  pattern with no glob/alternation/whitespace is id-anchored. */
export function isIdAnchored(pattern: string): boolean {
  const p = pattern.trim();
  for (const prefix of ["leg:", "txn:"]) {
    if (p.startsWith(prefix)) {
      const rest = p.slice(prefix.length);
      if (rest && !rest.includes("*") && !rest.includes("|") && !rest.includes(",") && !/\s/.test(rest)) {
        return true;
      }
    }
  }
  return false;
}

/** Port of rules.rs `meta_segment_match`: split meta on `,`, trim, and
 *  require one segment to equal the pattern case-insensitively. */
export function metaSegmentMatch(pattern: string, meta: string): boolean {
  const p = pattern.trim().toLowerCase();
  if (!p) return false;
  return meta.split(",").some((seg) => seg.trim().toLowerCase() === p);
}

/**
 * Extract a `key:value` segment from comma-separated meta. Returns the part
 * after the prefix (without the prefix itself), or undefined when absent.
 * For prefix `swap:` this returns the partner's `txn:<HASH>` segment.
 */
export function metaSegmentValue(meta: string | null | undefined, prefix: string): string | undefined {
  if (!meta) return undefined;
  for (const seg of meta.split(",")) {
    const trimmed = seg.trim();
    if (trimmed.startsWith(prefix)) return trimmed.slice(prefix.length);
  }
  return undefined;
}

/**
 * Read the Rust-stamped swap partner reference from a transaction's meta.
 * Returns `{ partnerTxnId, partnerCommodity }` when both segments are present —
 * the pipeline writes them together for every paired transaction so consumers
 * can find the partner unambiguously even when several legs share an on-chain
 * hash. Returns undefined for transactions that aren't part of a swap pair.
 */
export function swapPartnerRefFromMeta(meta: string | null | undefined):
  | { partnerTxnId: string; partnerCommodity: string }
  | undefined {
  const swapRef = metaSegmentValue(meta, "swap:");
  const partnerCommodity = metaSegmentValue(meta, "swap_partner_commodity:");
  if (!swapRef || !partnerCommodity) return undefined;
  return { partnerTxnId: swapRef, partnerCommodity };
}

/** Stable key for a transaction that works across different deserialization
 *  sources. Sibling legs of one transaction share the `txn:` id, and two
 *  identical fills (1 EWU @ 32.88 twice on one day) share the datetime, amount
 *  and narration as well, so the per-leg `leg:` id keys the row when the
 *  pipeline stamped one — otherwise their values collide and double-count. */
export function txnValueKey(
  t: { meta?: string | null; datetime: string; amount: number; narration?: string | null },
): string {
  return `${legIdFromMeta(t.meta) || txnIdFromMeta(t.meta)}|${t.datetime}|${t.amount}|${t.narration ?? ""}`;
}
