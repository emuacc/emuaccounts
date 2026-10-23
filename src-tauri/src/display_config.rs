//! Display config resolution: vault default + per-set override.
//!
//! Two JSON files participate:
//!
//! - `generated/config.json` — the **vault** config. Carries the default
//!   `base_currency` every account set inherits.
//! - `generated/<set>/config.json` — the **set** config. Any key present here
//!   overrides the vault value for that set (`base_currency`, `commodities`,
//!   `default_decimals`, `default_account`, `hidden_accounts`, `tax`, …).
//!
//! Every backend reader of `base_currency` goes through [`resolve_base_currency`]
//! so the set→vault lookup is defined in exactly one place. A vault with no
//! `base_currency` at either level is a *setup-incomplete* state: holdings
//! cannot be valued, and the UI gates on [`vault_base_currency`] being `None`.

use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// Keys the app writes into a fresh set config so per-commodity formatting
/// has something to start from. Kept free of `base_currency` on purpose —
/// that must be an explicit choice (vault-level), never a silent default.
pub fn default_set_config() -> Value {
    serde_json::json!({
        "commodities": {
            "AUD": { "decimals": 2 },
            "USD": { "decimals": 2 }
        },
        "default_decimals": 2
    })
}

fn read_json(path: &Path) -> Result<Option<Map<String, Value>>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    match value {
        Value::Object(map) => Ok(Some(map)),
        _ => Err(format!("{} is not a JSON object", path.display())),
    }
}

fn write_json(path: &Path, map: &Map<String, Value>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let pretty = crate::to_sorted_json_pretty(&Value::Object(map.clone()))
        .map_err(|e| format!("failed to serialize {}: {e}", path.display()))?;
    std::fs::write(path, pretty).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn vault_config_path(generated_dir: &Path) -> std::path::PathBuf {
    generated_dir.join("config.json")
}

/// `generated/<set>/config.json`; an empty set name addresses the vault file.
pub fn set_config_path(generated_dir: &Path, account_set: &str) -> std::path::PathBuf {
    if account_set.is_empty() {
        vault_config_path(generated_dir)
    } else {
        generated_dir.join(account_set).join("config.json")
    }
}

fn base_currency_of(map: &Map<String, Value>) -> Option<String> {
    map.get("base_currency")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// The vault-level default base currency, or `None` when the vault has not
/// been set up yet. Set-level overrides are deliberately NOT consulted: the
/// setup gate asks about the vault default, not about whichever set happens
/// to be selected.
pub fn vault_base_currency(generated_dir: &Path) -> Result<Option<String>, String> {
    Ok(read_json(&vault_config_path(generated_dir))?
        .as_ref()
        .and_then(base_currency_of))
}

/// Write `base_currency` into the vault config, preserving every other key.
/// Creates the file when absent. Rejects blank input.
pub fn set_vault_base_currency(generated_dir: &Path, currency: &str) -> Result<(), String> {
    let currency = currency.trim();
    if currency.is_empty() {
        return Err("base currency must not be blank".to_string());
    }
    let path = vault_config_path(generated_dir);
    let mut map = read_json(&path)?.unwrap_or_default();
    map.insert(
        "base_currency".to_string(),
        Value::String(currency.to_string()),
    );
    write_json(&path, &map)
}

/// The display config a set actually runs with: vault keys, overlaid by the
/// set's own keys. Missing files contribute nothing; a set with no config of
/// its own simply sees the vault config.
pub fn resolve_display_config(generated_dir: &Path, account_set: &str) -> Result<Value, String> {
    let mut merged = read_json(&vault_config_path(generated_dir))?.unwrap_or_default();
    if !account_set.is_empty() {
        if let Some(set_map) = read_json(&set_config_path(generated_dir, account_set))? {
            for (k, v) in set_map {
                merged.insert(k, v);
            }
        }
    }
    Ok(Value::Object(merged))
}

/// Set-then-vault `base_currency` lookup. `None` means the vault has not been
/// set up; callers that cannot proceed without a currency should surface that
/// rather than invent one.
pub fn resolve_base_currency(generated_dir: &Path, account_set: &str) -> Option<String> {
    resolve_display_config(generated_dir, account_set)
        .ok()
        .and_then(|v| v.as_object().and_then(base_currency_of))
}

/// Set-then-vault lookup addressed by a set *directory*, the way the report
/// and issue collectors iterate. `set_dir == generated_dir` (or any path not
/// under it) addresses the vault itself.
pub fn resolve_base_currency_for_dir(generated_dir: &Path, set_dir: &Path) -> Option<String> {
    let set = set_dir
        .strip_prefix(generated_dir)
        .ok()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    resolve_base_currency(generated_dir, set)
}

/// Set-then-vault `base_currency` of each named set, read once per set
/// however often a name repeats. The empty name is the vault itself, the
/// currency of a transaction that belongs to no set.
pub fn resolve_base_currencies<'a>(
    generated_dir: &Path,
    sets: impl IntoIterator<Item = &'a str>,
) -> HashMap<String, Option<String>> {
    let mut out = HashMap::new();
    for set in sets {
        if !out.contains_key(set) {
            out.insert(set.to_string(), resolve_base_currency(generated_dir, set));
        }
    }
    out
}

/// Currencies worth offering as a base: every quote commodity in
/// `sources/_prices/*.txt` (the only currencies holdings can be valued in)
/// plus any commodity the vault config already formats. Sorted, deduplicated.
pub fn base_currency_candidates(sources_dir: &Path, generated_dir: &Path) -> Vec<String> {
    let mut out = BTreeSet::new();
    let prices_dir = sources_dir.join("_prices");
    if let Ok(entries) = std::fs::read_dir(&prices_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("txt") {
                continue;
            }
            if let Ok(contents) = std::fs::read_to_string(&path) {
                for p in crate::ledger_parser::parse_prices(&contents).prices {
                    out.insert(p.quote_commodity);
                }
            }
        }
    }
    if let Ok(Some(map)) = read_json(&vault_config_path(generated_dir)) {
        if let Some(commodities) = map.get("commodities").and_then(Value::as_object) {
            out.extend(commodities.keys().cloned());
        }
    }
    out.into_iter().collect()
}
