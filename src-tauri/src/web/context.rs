use std::path::{Path, PathBuf};

use crate::root_config::{CliVault, RootConfig};

/// Read-only request context for the web server. Resolves the active vault
/// (the same one the desktop app uses) without any Tauri `AppHandle`, through
/// [`CliVault`] like every CLI tool.
#[derive(Debug, Clone)]
pub struct WebCtx {
    pub config: RootConfig,
    pub sources_dir: PathBuf,
    pub generated_dir: PathBuf,
    /// When false (the default, matching the desktop app), accounts marked
    /// hidden in the vault are filtered out of ledger/query results.
    pub show_hidden: bool,
}

impl WebCtx {
    /// Resolve from the platform app-data dir (the same `config.json` the
    /// desktop app writes) plus the `EMUACC_SOURCES_DIR` /
    /// `EMUACC_GENERATED_DIR` env overrides the Tauri commands already honor.
    pub fn from_app_data_dir(app_data_dir: &Path) -> Self {
        let CliVault {
            config,
            sources_dir,
            generated_dir,
        } = CliVault::from_app_data_dir(app_data_dir);
        Self {
            config,
            sources_dir,
            generated_dir,
            show_hidden: false,
        }
    }

    /// Construct directly from resolved directories (used by tests).
    pub fn from_dirs(sources_dir: PathBuf, generated_dir: PathBuf) -> Self {
        Self {
            config: RootConfig::default(),
            sources_dir,
            generated_dir,
            show_hidden: false,
        }
    }

    /// The generated directory for a given account set (empty set = root).
    /// Mirrors the desktop `resolve_set_dir`.
    pub fn set_dir(&self, account_set: &str) -> PathBuf {
        if account_set.is_empty() {
            self.generated_dir.clone()
        } else {
            self.generated_dir.join(account_set)
        }
    }
}
