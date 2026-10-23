//! Configurable display terminology.
//!
//! The catalogue of terms — key, default singular/plural, description — is
//! `src-tauri/terminology.json`, embedded at compile time. The frontend
//! imports the same file for its no-backend fallback, so default wording has
//! exactly one source. Per-user overrides persist in the root config
//! (`terms` in `config.json`, see [`crate::root_config::RootConfig`]):
//! [`resolve_terms`] merges the two into what the UI shows, and
//! [`normalise_overrides`] keeps the persisted map down to deliberate changes.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// The embedded catalogue, verbatim.
const CATALOGUE_JSON: &str = include_str!("../terminology.json");

#[derive(Debug, Clone, Deserialize)]
struct CatalogueEntry {
    singular: String,
    plural: String,
    description: String,
}

/// Parsed once; a `BTreeMap` keeps the term order stable (by key).
static CATALOGUE: LazyLock<BTreeMap<String, CatalogueEntry>> = LazyLock::new(|| {
    serde_json::from_str(CATALOGUE_JSON).expect("terminology.json must be a map of term entries")
});

/// A user's override of one term. A blank field means "use the default".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermOverride {
    #[serde(default)]
    pub singular: String,
    #[serde(default)]
    pub plural: String,
}

/// One term as the UI consumes it: the effective wording, the catalogue
/// defaults (so the editor can show what "blank" means) and the description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Term {
    pub key: String,
    pub singular: String,
    pub plural: String,
    pub default_singular: String,
    pub default_plural: String,
    pub description: String,
}

fn effective<'a>(override_text: &'a str, default: &'a str) -> &'a str {
    let trimmed = override_text.trim();
    if trimmed.is_empty() {
        default
    } else {
        trimmed
    }
}

/// Every catalogue term with `overrides` applied. Overrides for keys the
/// catalogue does not know are ignored.
pub fn resolve_terms(overrides: &BTreeMap<String, TermOverride>) -> Vec<Term> {
    CATALOGUE
        .iter()
        .map(|(key, def)| {
            let ov = overrides.get(key);
            Term {
                key: key.clone(),
                singular: effective(ov.map_or("", |o| o.singular.as_str()), &def.singular)
                    .to_string(),
                plural: effective(ov.map_or("", |o| o.plural.as_str()), &def.plural).to_string(),
                default_singular: def.singular.clone(),
                default_plural: def.plural.clone(),
                description: def.description.clone(),
            }
        })
        .collect()
}

/// Trim every field, blank out fields equal to their default, drop unknown
/// keys and entries with nothing left — so `config.json` only ever records
/// deliberate changes.
pub fn normalise_overrides(
    overrides: BTreeMap<String, TermOverride>,
) -> BTreeMap<String, TermOverride> {
    let keep = |text: &str, default: &str| {
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed == default {
            String::new()
        } else {
            trimmed.to_string()
        }
    };
    overrides
        .into_iter()
        .filter_map(|(key, ov)| {
            let def = CATALOGUE.get(&key)?;
            let normalised = TermOverride {
                singular: keep(&ov.singular, &def.singular),
                plural: keep(&ov.plural, &def.plural),
            };
            (normalised != TermOverride::default()).then_some((key, normalised))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overrides(entries: &[(&str, &str, &str)]) -> BTreeMap<String, TermOverride> {
        entries
            .iter()
            .map(|(k, s, p)| {
                (
                    k.to_string(),
                    TermOverride {
                        singular: s.to_string(),
                        plural: p.to_string(),
                    },
                )
            })
            .collect()
    }

    fn asset(terms: &[Term]) -> &Term {
        terms.iter().find(|t| t.key == "asset").expect("asset term")
    }

    #[test]
    fn catalogue_defaults_resolve_with_no_overrides() {
        let terms = resolve_terms(&BTreeMap::new());
        let t = asset(&terms);
        assert_eq!((t.singular.as_str(), t.plural.as_str()), ("asset", "assets"));
        assert_eq!(t.default_singular, t.singular);
        assert_eq!(t.default_plural, t.plural);
        assert!(!t.description.is_empty());
    }

    #[test]
    fn every_catalogue_entry_is_complete() {
        for (key, def) in CATALOGUE.iter() {
            assert!(!def.singular.trim().is_empty(), "{key}: blank singular");
            assert!(!def.plural.trim().is_empty(), "{key}: blank plural");
            assert!(!def.description.trim().is_empty(), "{key}: blank description");
        }
    }

    #[test]
    fn override_replaces_wording_but_keeps_defaults() {
        let terms = resolve_terms(&overrides(&[("asset", "holding", "holdings")]));
        let t = asset(&terms);
        assert_eq!((t.singular.as_str(), t.plural.as_str()), ("holding", "holdings"));
        assert_eq!(
            (t.default_singular.as_str(), t.default_plural.as_str()),
            ("asset", "assets")
        );
    }

    #[test]
    fn blank_or_whitespace_override_field_falls_back_per_field() {
        let terms = resolve_terms(&overrides(&[("asset", "  holding ", "   ")]));
        let t = asset(&terms);
        assert_eq!((t.singular.as_str(), t.plural.as_str()), ("holding", "assets"));
    }

    #[test]
    fn unknown_override_keys_are_ignored() {
        let terms = resolve_terms(&overrides(&[("gadget", "foo", "foos")]));
        assert!(terms.iter().all(|t| t.key != "gadget"));
        assert_eq!(terms.len(), CATALOGUE.len());
    }

    #[test]
    fn normalise_keeps_only_deliberate_changes() {
        let out = normalise_overrides(overrides(&[
            ("asset", " holding ", "assets"),
            ("gadget", "foo", "foos"),
        ]));
        assert_eq!(out, overrides(&[("asset", "holding", "")]));
    }

    #[test]
    fn normalise_drops_entries_that_only_restate_the_default() {
        let out = normalise_overrides(overrides(&[("asset", "asset", " assets ")]));
        assert!(out.is_empty());
        let cleared = normalise_overrides(overrides(&[("asset", "", "")]));
        assert!(cleared.is_empty());
    }
}
