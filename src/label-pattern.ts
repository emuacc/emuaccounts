/** Build a payee-label glob pattern from a raw payee string.
 *
 * Bank feeds cram volatile details into the payee/memo — card numbers,
 * value dates, invoice numbers ("XERO AU INV-54875839 HAWTHORN AU Card
 * xx1234 Value Date: 04/08/2026"). A pattern built from the full string
 * matches exactly one transaction, so renaming a Party would never cover
 * the merchant's other rows. Strip the volatile tail and swap volatile
 * tokens for wildcards so one label renames every row of the merchant.
 */
export function payeeLabelPattern(source: string): string {
  let s = source;
  // Card-feed tail (CBA-style): " Card xx1234 Value Date: 02/08/2026"
  s = s.replace(/\s+Card\s+x*\d+.*$/i, "");
  s = s.replace(/\s+Value Date:.*$/i, "");
  // Date tokens anywhere in the remainder
  s = s.replace(/\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b/g, "*");
  // Standalone digit runs (invoice/reference numbers). Digits embedded in
  // alphanumeric tokens (crypto addresses, "xx1234") are left alone.
  s = s.replace(/(^|[^A-Za-z0-9])\d{4,}(?![A-Za-z0-9])/g, "$1*");
  s = s.replace(/\*{2,}/g, "*").trim();
  // Everything stripped (payee was pure noise) — fall back to exact match
  if (s.replace(/\*/g, "").trim() === "") return `*${source}*`;
  return `*${s}*`;
}
