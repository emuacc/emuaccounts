// Display terminology — the configurable nouns the UI uses ("asset", …).
//
// The catalogue (key → default singular/plural + description) is
// src-tauri/terminology.json. The Rust backend embeds that same file, so
// there is exactly one source of default wording. At runtime `get_terms`
// returns the catalogue merged with the user's overrides (`terms` in the app
// config.json); `defaultTerms()` covers the moment before that answer lands
// and contexts with no backend (unit tests).
//
// Renderers never spell these nouns out — they call `term()` / `termTitle()`
// so a change in Global Settings → Terminology reaches every screen at once.

import catalogue from "../src-tauri/terminology.json";

export type TermKey = keyof typeof catalogue;

/** A resolved term as `get_terms` returns it. */
export type Term = {
  key: string;
  singular: string;
  plural: string;
  default_singular: string;
  default_plural: string;
  description: string;
};

/** A user's override of one term; a blank field means "use the default". */
export type TermOverride = { singular: string; plural: string };

/** The catalogue as resolved terms, with no overrides applied. */
export function defaultTerms(): Term[] {
  return Object.entries(catalogue).map(([key, def]) => ({
    key,
    singular: def.singular,
    plural: def.plural,
    default_singular: def.singular,
    default_plural: def.plural,
    description: def.description,
  }));
}

function lookup(terms: Term[] | undefined, key: TermKey): Term {
  const found = terms?.find((t) => t.key === key);
  if (found) return found;
  // `key` is a catalogue key by type, so the fallback always exists.
  return defaultTerms().find((t) => t.key === key) as Term;
}

/** The wording for `key`: singular when `count` is exactly 1, plural otherwise. */
export function term(terms: Term[] | undefined, key: TermKey, count = 1): string {
  const t = lookup(terms, key);
  return count === 1 ? t.singular : t.plural;
}

/** `term()` with its first letter capitalised — for headings and column titles. */
export function termTitle(terms: Term[] | undefined, key: TermKey, count = 1): string {
  const word = term(terms, key, count);
  return word.charAt(0).toUpperCase() + word.slice(1);
}

/**
 * The overrides as the Terminology editor shows them: the override text where
 * one is in force, blank where the default applies.
 */
export function overridesOf(terms: Term[]): Record<string, TermOverride> {
  return Object.fromEntries(
    terms.map((t) => [
      t.key,
      {
        singular: t.singular === t.default_singular ? "" : t.singular,
        plural: t.plural === t.default_plural ? "" : t.plural,
      },
    ]),
  );
}
