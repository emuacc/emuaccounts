//! Per-vault UI state (nav entry, selected account set) persisted at
//! `generated/.cache/ui-state.json`. The JSON is opaque here — the frontend
//! owns the schema. Written atomically (temp + rename) because a second
//! instance on the same vault may read concurrently; semantics are
//! last-writer-wins.

use std::path::{Path, PathBuf};

fn state_path(generated_dir: &Path) -> PathBuf {
    generated_dir.join(".cache").join("ui-state.json")
}

/// Read the persisted UI state. A missing or corrupt file is simply
/// "no saved state" — never an error.
pub fn load_ui_state(generated_dir: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(state_path(generated_dir)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Persist the UI state atomically (temp file + rename).
pub fn save_ui_state(generated_dir: &Path, value: &serde_json::Value) -> Result<(), String> {
    let path = state_path(generated_dir);
    let cache_dir = path.parent().expect("state path always has a parent");
    std::fs::create_dir_all(cache_dir)
        .map_err(|e| format!("failed to create {}: {e}", cache_dir.display()))?;
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("failed to serialize ui state: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| format!("failed to replace {}: {e}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_returns_none_when_file_missing() {
        let dir = TempDir::new().unwrap();
        assert_eq!(load_ui_state(dir.path()), None);
    }

    #[test]
    fn load_returns_none_on_corrupt_json() {
        let dir = TempDir::new().unwrap();
        let cache = dir.path().join(".cache");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("ui-state.json"), "{not json").unwrap();
        assert_eq!(load_ui_state(dir.path()), None);
    }

    #[test]
    fn save_then_load_roundtrips_value() {
        let dir = TempDir::new().unwrap();
        let value = serde_json::json!({
            "version": 1,
            "nav": { "selectedAccount": "assets:cash" },
            "selectedAccountSet": "alex",
        });
        save_ui_state(dir.path(), &value).unwrap();
        assert!(
            dir.path().join(".cache").join("ui-state.json").exists(),
            "state must land at generated/.cache/ui-state.json"
        );
        assert_eq!(load_ui_state(dir.path()), Some(value));
    }

    #[test]
    fn save_creates_cache_dir_when_generated_is_bare() {
        // A freshly created vault has generated/ but no .cache/ yet.
        let dir = TempDir::new().unwrap();
        assert!(!dir.path().join(".cache").exists());
        save_ui_state(dir.path(), &serde_json::json!({"version": 1})).unwrap();
        assert!(dir.path().join(".cache").join("ui-state.json").exists());
    }

    #[test]
    fn save_is_atomic_replace() {
        let dir = TempDir::new().unwrap();
        save_ui_state(dir.path(), &serde_json::json!({"gen": "a"})).unwrap();
        save_ui_state(dir.path(), &serde_json::json!({"gen": "b"})).unwrap();
        assert_eq!(
            load_ui_state(dir.path()),
            Some(serde_json::json!({"gen": "b"}))
        );
        let leftovers: Vec<_> = std::fs::read_dir(dir.path().join(".cache"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n != "ui-state.json")
            .collect();
        assert!(leftovers.is_empty(), "no temp files left behind: {leftovers:?}");
    }
}
