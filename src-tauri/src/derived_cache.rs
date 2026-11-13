//! Persisted single-entry caches for expensive derived scans (issues panel,
//! trade suggestions). Each result is stored as a JSON envelope keyed on the
//! inputs it was computed from — a sources fingerprint plus whatever else the
//! computation reads (trade-links hash, base currency) and a code-version
//! const so logic changes invalidate stale artifacts (the MATCHER_VERSION
//! pattern). A key mismatch, missing file, or unreadable envelope all fall
//! through to a fresh compute; persisting is best-effort and never fails the
//! command.

use serde::{de::DeserializeOwned, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize)]
struct Envelope<T> {
    key: String,
    data: T,
}

/// Return the value cached at `path` when its stored key matches `key`;
/// otherwise run `compute`, persist the result under the new key, and return it.
pub fn get_or_compute<T, F>(path: &Path, key: &str, compute: F) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Result<T, String>,
{
    if let Ok(text) = std::fs::read_to_string(path) {
        if let Ok(env) = serde_json::from_str::<Envelope<T>>(&text) {
            if env.key == key {
                return Ok(env.data);
            }
        }
    }
    let data = compute()?;
    let env = Envelope { key: key.to_string(), data };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(&env) {
        let _ = std::fs::write(path, json);
    }
    Ok(env.data)
}

/// What the derived scans read of the vault: the pipeline's outputs, by the
/// hash of each generated file the cache manifest recorded, and the price
/// files under sources. A rule save whose rebuild rewrites no output leaves
/// it unchanged, so the scans stay cached; a source edit reaches it through
/// the outputs the rebuild rewrote. The sources fingerprint it replaces
/// changed on every rule save, which recomputed every scan.
pub fn outputs_fingerprint(sources_dir: &Path, generated_dir: &Path) -> Result<String, String> {
    let manifest = crate::build_cache::load_cache_filtered(generated_dir, Some(&HashSet::new()));
    let outputs: BTreeMap<String, String> = manifest.output_hashes.into_iter().collect();
    Ok(format!(
        "{}:{}",
        value_hash(&outputs),
        crate::build_cache::dir_fingerprint(&sources_dir.join("_prices"))?
    ))
}

/// The content hash of one file, or empty when it cannot be read.
pub fn file_fingerprint(path: &Path) -> String {
    crate::build_cache::file_hash(path).unwrap_or_default()
}

/// Stable hex digest of any serializable value — used to fold secondary
/// inputs (e.g. the current trade links) into a cache key.
pub fn value_hash<T: Serialize>(value: &T) -> String {
    use sha2::{Digest, Sha256};
    let json = serde_json::to_string(value).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    hex::encode(hasher.finalize())[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn counted<'a>(calls: &'a Cell<u32>, result: Vec<i32>) -> impl FnOnce() -> Result<Vec<i32>, String> + 'a {
        move || {
            calls.set(calls.get() + 1);
            Ok(result)
        }
    }

    #[test]
    fn miss_computes_persists_then_hits_without_recompute() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".cache/derived.json");
        let calls = Cell::new(0);

        let v: Vec<i32> = get_or_compute(&path, "k1", counted(&calls, vec![1, 2, 3])).unwrap();
        assert_eq!(v, vec![1, 2, 3]);
        assert_eq!(calls.get(), 1);
        assert!(path.exists(), "envelope should be persisted");

        // Same key: served from disk, compute not called again.
        let v2: Vec<i32> = get_or_compute(&path, "k1", counted(&calls, vec![9])).unwrap();
        assert_eq!(v2, vec![1, 2, 3]);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn key_change_recomputes_and_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("derived.json");
        let calls = Cell::new(0);

        let _: Vec<i32> = get_or_compute(&path, "k1", counted(&calls, vec![1])).unwrap();
        let v: Vec<i32> = get_or_compute(&path, "k2", counted(&calls, vec![2])).unwrap();
        assert_eq!(v, vec![2]);
        assert_eq!(calls.get(), 2);

        // New key now persisted: k2 hits, k1 misses.
        let v: Vec<i32> = get_or_compute(&path, "k2", counted(&calls, vec![3])).unwrap();
        assert_eq!(v, vec![2]);
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn corrupt_or_mismatched_envelope_recomputes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("derived.json");
        std::fs::write(&path, "not json at all").unwrap();
        let calls = Cell::new(0);

        let v: Vec<i32> = get_or_compute(&path, "k1", counted(&calls, vec![7])).unwrap();
        assert_eq!(v, vec![7]);
        assert_eq!(calls.get(), 1);

        // Valid JSON but wrong shape for the envelope: also a miss.
        std::fs::write(&path, r#"{"unexpected": true}"#).unwrap();
        let v: Vec<i32> = get_or_compute(&path, "k1", counted(&calls, vec![8])).unwrap();
        assert_eq!(v, vec![8]);
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn compute_error_propagates_and_nothing_is_persisted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("derived.json");
        let err = get_or_compute::<Vec<i32>, _>(&path, "k1", || Err("boom".to_string()));
        assert_eq!(err.unwrap_err(), "boom");
        assert!(!path.exists());
    }

    /// A vault whose pipeline has run: a manifest recording `outputs` as the
    /// generated files, a price file and a rules file under sources.
    fn vault_with_outputs(root: &Path, outputs: &[(&str, &str)]) -> (std::path::PathBuf, std::path::PathBuf) {
        let sources = root.join("sources");
        let generated = root.join("generated");
        std::fs::create_dir_all(sources.join("_prices")).unwrap();
        std::fs::create_dir_all(sources.join("bank")).unwrap();
        std::fs::write(sources.join("_prices/AUD.csv"), "2025-01-01,1\n").unwrap();
        std::fs::write(sources.join("bank/_rules.json"), r#"{"rules":[]}"#).unwrap();
        let cache = crate::build_cache::BuildCache {
            output_hashes: outputs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        crate::build_cache::save_cache(&generated, &cache, None).unwrap();
        (sources, generated)
    }

    #[test]
    fn outputs_fingerprint_ignores_a_rule_save_that_rewrites_no_output() {
        let dir = tempfile::tempdir().unwrap();
        let (sources, generated) = vault_with_outputs(dir.path(), &[("bank/ledger.transactions", "h1")]);
        let before = outputs_fingerprint(&sources, &generated).unwrap();
        std::fs::write(sources.join("bank/_rules.json"), r#"{"rules":[{"id":"r","pattern":"*x*"}]}"#).unwrap();
        assert_eq!(outputs_fingerprint(&sources, &generated).unwrap(), before);
    }

    #[test]
    fn outputs_fingerprint_changes_with_a_rewritten_output_or_a_price_file() {
        let dir = tempfile::tempdir().unwrap();
        let (sources, generated) = vault_with_outputs(dir.path(), &[("bank/ledger.transactions", "h1")]);
        let before = outputs_fingerprint(&sources, &generated).unwrap();

        let cache = crate::build_cache::BuildCache {
            output_hashes: [("bank/ledger.transactions".to_string(), "h2".to_string())].into(),
            ..Default::default()
        };
        crate::build_cache::save_cache(&generated, &cache, None).unwrap();
        let rewritten = outputs_fingerprint(&sources, &generated).unwrap();
        assert_ne!(rewritten, before);

        std::fs::write(sources.join("_prices/BTC.csv"), "2025-01-01,100000\n").unwrap();
        assert_ne!(outputs_fingerprint(&sources, &generated).unwrap(), rewritten);
    }

    #[test]
    fn file_fingerprint_is_empty_for_a_missing_file_and_tracks_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("_trade_links.json");
        assert_eq!(file_fingerprint(&path), "");
        std::fs::write(&path, "[]").unwrap();
        let empty = file_fingerprint(&path);
        std::fs::write(&path, r#"[{"id":"l1"}]"#).unwrap();
        assert_ne!(file_fingerprint(&path), empty);
    }

    #[test]
    fn value_hash_is_stable_and_input_sensitive() {
        let a = value_hash(&vec!["x", "y"]);
        assert_eq!(a, value_hash(&vec!["x", "y"]));
        assert_ne!(a, value_hash(&vec!["y", "x"]));
        assert_eq!(a.len(), 16);
    }
}
