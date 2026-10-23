//! Files the relay sync keeps under `sources/`. They are each device's sync
//! state, not pipeline input, so changing them must not trigger a rebuild or
//! defeat the pipeline's early exit.

use std::path::{Path, PathBuf};

/// The Automerge metadata document the relay merges between devices.
pub const METADATA_FILE: &str = "emuacc-metadata.automerge";
/// This device's relay URL and sync group.
pub const RELAY_CONFIG_FILE: &str = "relay-config.json";
/// The relay blob store (`cas/<2-char prefix>/<rest of hash>`).
pub const CAS_DIR: &str = "cas";

/// Whether `relative`, a path relative to `sources/`, is relay sync state:
/// the metadata document or relay config at the top level, or anything under
/// `cas/`. A file of the same name inside an account folder is not.
pub fn is_sync_state(relative: &Path) -> bool {
    let mut parts = relative.components().map(|c| c.as_os_str());
    match (parts.next(), parts.next()) {
        (Some(first), _) if first == CAS_DIR => true,
        (Some(name), None) => name == METADATA_FILE || name == RELAY_CONFIG_FILE,
        _ => false,
    }
}

/// [`is_sync_state`] for a path that may be absolute: it must lie under
/// `sources_dir`. Anything else (another directory, or a path the watcher
/// reports in a form that doesn't share the prefix) is not sync state.
pub fn is_sync_state_in(sources_dir: &Path, path: &Path) -> bool {
    path.strip_prefix(sources_dir).is_ok_and(is_sync_state)
}

/// Whether a file-watcher event touched only sync state, so it needs no
/// rebuild. An event that names no paths is not "only sync state": the
/// watcher can't tell what changed, so it must rebuild.
pub fn only_sync_state(sources_dir: &Path, paths: &[PathBuf]) -> bool {
    !paths.is_empty() && paths.iter().all(|p| is_sync_state_in(sources_dir, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_sync_files_are_sync_state() {
        assert!(is_sync_state(Path::new(METADATA_FILE)));
        assert!(is_sync_state(Path::new(RELAY_CONFIG_FILE)));
    }

    #[test]
    fn everything_under_cas_is_sync_state() {
        assert!(is_sync_state(Path::new("cas")));
        assert!(is_sync_state(Path::new("cas/ab/cdef0123")));
    }

    #[test]
    fn source_files_are_not_sync_state() {
        assert!(!is_sync_state(Path::new("alex/cash/bank/2025-01.csv")));
        assert!(!is_sync_state(Path::new("alex/cash/bank/_rules.json")));
        assert!(!is_sync_state(Path::new("_prices/BTC.txt")));
    }

    #[test]
    fn same_names_inside_account_folders_are_not_sync_state() {
        assert!(!is_sync_state(Path::new("alex/relay-config.json")));
        assert!(!is_sync_state(Path::new("alex/emuacc-metadata.automerge")));
        assert!(!is_sync_state(Path::new("alex/cas/2025-01.csv")));
    }

    #[test]
    fn absolute_paths_are_judged_relative_to_sources() {
        let sources = PathBuf::from("/vault/sources");
        assert!(is_sync_state_in(&sources, &sources.join(METADATA_FILE)));
        assert!(is_sync_state_in(&sources, &sources.join("cas/ab/cd")));
        assert!(!is_sync_state_in(&sources, &sources.join("alex/a.csv")));
        assert!(!is_sync_state_in(&sources, Path::new("/elsewhere/relay-config.json")));
    }

    #[test]
    fn an_event_is_only_sync_state_when_every_path_is() {
        let sources = PathBuf::from("/vault/sources");
        let doc = sources.join(METADATA_FILE);
        let blob = sources.join("cas/ab/cd");
        let csv = sources.join("alex/a.csv");
        assert!(only_sync_state(&sources, &[doc.clone(), blob]));
        assert!(!only_sync_state(&sources, &[doc, csv.clone()]));
        assert!(!only_sync_state(&sources, &[csv]));
    }

    #[test]
    fn an_event_without_paths_is_not_only_sync_state() {
        assert!(!only_sync_state(Path::new("/vault/sources"), &[]));
    }
}
