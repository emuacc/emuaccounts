//! Files the relay sync keeps under `sources/` besides the metadata document
//! ([`crate::trade_links::METADATA_FILE`]). They are each device's
//! sync state, not pipeline input, so changing them must not trigger a rebuild
//! or defeat the pipeline's early exit — [`crate::sources_state`] asks
//! [`is_sync_state`].

use std::path::Path;

/// This device's relay URL and sync group.
pub const RELAY_CONFIG_FILE: &str = "relay-config.json";
/// The relay blob store (`cas/<2-char prefix>/<rest of hash>`).
pub const CAS_DIR: &str = "cas";

/// Whether `relative`, a path relative to `sources/`, is relay sync state:
/// the relay config at the top level, or anything under `cas/`. A file of the
/// same name inside an account folder is not.
pub fn is_sync_state(relative: &Path) -> bool {
    let mut parts = relative.components().map(|c| c.as_os_str());
    match (parts.next(), parts.next()) {
        (Some(first), _) if first == CAS_DIR => true,
        (Some(name), None) => name == RELAY_CONFIG_FILE,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_top_level_relay_config_is_sync_state() {
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
        assert!(!is_sync_state(Path::new("alex/cas/2025-01.csv")));
    }
}
