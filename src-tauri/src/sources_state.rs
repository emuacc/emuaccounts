//! Files the app keeps under `sources/` as its own state rather than pipeline
//! input: the trade links ([`TRADE_LINKS_FILE`]), the Automerge metadata
//! document ([`METADATA_FILE`]: the pre-TASK-133 trade links and the sync
//! build's metadata) and, in the sync build, the relay sync state. Changing
//! them must not trigger a rebuild or defeat the pipeline's early exit.

use crate::trade_links::{METADATA_FILE, TRADE_LINKS_FILE};
use std::path::{Path, PathBuf};

/// Whether `relative`, a path relative to `sources/`, is app state: the
/// trade-link file or the metadata document at the top level (a file of
/// either name inside an account folder is not), or the sync build's relay
/// state.
pub fn is_app_state(relative: &Path) -> bool {
    is_top_level_state(relative) || is_sync_state(relative)
}

fn is_top_level_state(relative: &Path) -> bool {
    let mut parts = relative.components();
    matches!(
        (parts.next(), parts.next()),
        (Some(name), None) if name.as_os_str() == TRADE_LINKS_FILE || name.as_os_str() == METADATA_FILE
    )
}

/// The sync build's relay state (docs/SYNC.md); a default build writes none.
#[cfg(feature = "sync")]
use crate::sync::state::is_sync_state;
#[cfg(not(feature = "sync"))]
fn is_sync_state(_relative: &Path) -> bool {
    false
}

/// [`is_app_state`] for a path that may be absolute: it must lie under
/// `sources_dir`. Anything else (another directory, or a path the watcher
/// reports in a form that doesn't share the prefix) is not app state.
pub fn is_app_state_in(sources_dir: &Path, path: &Path) -> bool {
    path.strip_prefix(sources_dir).is_ok_and(is_app_state)
}

/// Whether a file-watcher event touched only app state, so it needs no
/// rebuild. An event that names no paths is not "only app state": the
/// watcher can't tell what changed, so it must rebuild.
pub fn only_app_state(sources_dir: &Path, paths: &[PathBuf]) -> bool {
    !paths.is_empty() && paths.iter().all(|p| is_app_state_in(sources_dir, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_top_level_trade_links_and_metadata_doc_are_app_state() {
        assert!(is_app_state(Path::new(TRADE_LINKS_FILE)));
        assert!(is_app_state(Path::new(METADATA_FILE)));
    }

    #[test]
    fn source_files_are_not_app_state() {
        assert!(!is_app_state(Path::new("alex/cash/bank/2025-01.csv")));
        assert!(!is_app_state(Path::new("alex/cash/bank/_rules.json")));
        assert!(!is_app_state(Path::new("_prices/BTC.txt")));
    }

    #[test]
    fn the_same_names_inside_an_account_folder_are_not_app_state() {
        assert!(!is_app_state(Path::new("alex/emuacc-metadata.automerge")));
        assert!(!is_app_state(Path::new("alex/_trade_links.json")));
    }

    #[cfg(not(feature = "sync"))]
    #[test]
    fn a_default_build_keeps_no_relay_state() {
        assert!(!is_app_state(Path::new("relay-config.json")));
        assert!(!is_app_state(Path::new("cas/ab/cdef0123")));
    }

    #[cfg(feature = "sync")]
    #[test]
    fn the_sync_build_adds_the_relay_state() {
        assert!(is_app_state(Path::new("relay-config.json")));
        assert!(is_app_state(Path::new("cas/ab/cdef0123")));
    }

    #[test]
    fn absolute_paths_are_judged_relative_to_sources() {
        let sources = PathBuf::from("/vault/sources");
        assert!(is_app_state_in(&sources, &sources.join(METADATA_FILE)));
        assert!(!is_app_state_in(&sources, &sources.join("alex/a.csv")));
        assert!(!is_app_state_in(
            &sources,
            Path::new("/elsewhere/emuacc-metadata.automerge")
        ));
    }

    #[test]
    fn an_event_is_only_app_state_when_every_path_is() {
        let sources = PathBuf::from("/vault/sources");
        let doc = sources.join(METADATA_FILE);
        let csv = sources.join("alex/a.csv");
        assert!(only_app_state(&sources, std::slice::from_ref(&doc)));
        assert!(!only_app_state(&sources, &[doc, csv.clone()]));
        assert!(!only_app_state(&sources, &[csv]));
    }

    #[test]
    fn an_event_without_paths_is_not_only_app_state() {
        assert!(!only_app_state(Path::new("/vault/sources"), &[]));
    }
}
