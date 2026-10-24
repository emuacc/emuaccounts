use crate::terminology::TermOverride;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    pub current_root: Option<String>,
    #[serde(default)]
    pub known_roots: Vec<String>,
    #[serde(default = "default_expense_account")]
    pub default_expense_account: String,
    /// Optional path to an override prompt file for `emuacc-classify`.
    /// If unset, the embedded default is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classify_prompt_path: Option<String>,
    /// When true, the app runs the daily price-backfill plugins on startup.
    #[serde(default)]
    pub update_prices_on_startup: bool,
    /// Extra account-name prefixes unioned into the primary-accounts allowlist
    /// used by the Balances / Performance / Tax-Savings reports. Lets nominal
    /// accounts (e.g. `assets:staking`, `assets:lending`) and sub-accounts of
    /// folder accounts count toward holdings. Each entry matches itself and any
    /// sub-account; empty list = today's folder-accounts-only behaviour.
    #[serde(default)]
    pub extra_primary_account_prefixes: Vec<String>,
    /// OpenAI-compatible chat-completions base URL including `/v1`
    /// (e.g. `http://localhost:1234/v1` for LM Studio). Empty = AI disabled.
    #[serde(default)]
    pub ai_api_url: String,
    /// Bearer token for the AI endpoint. Empty = no Authorization header
    /// (local servers). Stored in plain text in config.json.
    #[serde(default)]
    pub ai_api_key: String,
    /// Model id sent in AI requests (e.g. `qwen2.5-7b-instruct`, `gpt-4o-mini`).
    #[serde(default)]
    pub ai_model: String,
    /// Reasoning effort ("low"/"medium"/"high") sent as `reasoning_effort` on
    /// AI chat requests. Empty = omit the field (model default).
    #[serde(default)]
    pub ai_reasoning_effort: String,
    /// The user's name/context, injected into AI categorisation prompts so the
    /// model can recognise the user's own name or initials in narrations.
    #[serde(default)]
    pub ai_user_name: String,
    /// Display-terminology overrides keyed by catalogue term (see
    /// `terminology.rs`). Omitted when empty: the catalogue defaults apply.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub terms: BTreeMap<String, TermOverride>,
}

fn default_expense_account() -> String {
    crate::FALLBACK_EXPENSE_ACCOUNT.to_string()
}

impl Default for RootConfig {
    fn default() -> Self {
        Self {
            current_root: None,
            known_roots: Vec::new(),
            default_expense_account: default_expense_account(),
            classify_prompt_path: None,
            update_prices_on_startup: false,
            extra_primary_account_prefixes: Vec::new(),
            ai_api_url: String::new(),
            ai_api_key: String::new(),
            ai_model: String::new(),
            ai_reasoning_effort: String::new(),
            ai_user_name: String::new(),
            terms: BTreeMap::new(),
        }
    }
}

/// The bundle identifier (`identifier` in `tauri.conf.json`); it names the
/// platform app-data folder that holds `config.json`.
const APP_IDENTIFIER: &str = "com.emusoftware.emuaccounts";

/// Platform app-data directory (`~/Library/Application Support/<APP_IDENTIFIER>`
/// on macOS). Canonical copy for non-Tauri callers (CLI bins, AI client);
/// the Tauri app itself resolves this via its path resolver.
/// `EMUACC_APP_DATA_DIR` overrides it so a CLI can run against a sandboxed
/// `config.json` (the BDD suite, a scripted vault on another account).
pub fn platform_app_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("EMUACC_APP_DATA_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library/Application Support")
                .join(APP_IDENTIFIER);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(data) = std::env::var("XDG_DATA_HOME").ok().or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{h}/.local/share"))
        }) {
            return PathBuf::from(data).join(APP_IDENTIFIER);
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join(APP_IDENTIFIER);
        }
    }
    PathBuf::from(".")
}

/// Load root config from `config.json` inside `app_data_dir`.
/// Returns a default (empty) config if the file doesn't exist or can't be parsed.
pub fn load_root_config(app_data_dir: &Path) -> RootConfig {
    let config_path = app_data_dir.join("config.json");
    match std::fs::read_to_string(&config_path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => RootConfig::default(),
    }
}

/// Persist root config to `config.json` inside `app_data_dir`.
pub fn save_root_config(app_data_dir: &Path, config: &RootConfig) -> Result<(), String> {
    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| format!("failed to create app data dir: {e}"))?;
    let config_path = app_data_dir.join("config.json");
    let json = crate::to_sorted_json_pretty(config)
        .map_err(|e| format!("failed to serialize config: {e}"))?;
    std::fs::write(&config_path, json).map_err(|e| format!("failed to write config: {e}"))?;
    Ok(())
}

/// Set the current root, add it to known_roots, and create the directory structure.
/// Returns the updated config.
pub fn set_root(app_data_dir: &Path, root_path: &Path) -> Result<RootConfig, String> {
    // Canonicalize to prevent duplicates from symlinks or trailing slashes.
    // Fall back to the original path if canonicalization fails (e.g. path doesn't exist yet).
    let canonical = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.to_path_buf());
    let root_str = canonical.to_string_lossy().to_string();
    let mut config = load_root_config(app_data_dir);
    config.current_root = Some(root_str.clone());
    if !config.known_roots.contains(&root_str) {
        config.known_roots.push(root_str);
    }

    // Create sources/ and generated/ under the root
    let sources = canonical.join("sources");
    let generated = canonical.join("generated");
    std::fs::create_dir_all(&sources).map_err(|e| format!("failed to create sources dir: {e}"))?;
    std::fs::create_dir_all(&generated)
        .map_err(|e| format!("failed to create generated dir: {e}"))?;

    // A brand-new generated/ gets an ignore-all .gitignore so a vault kept in
    // git never commits build output by accident. One that already has files
    // is left alone: in the known case (TASK-093) they were tracked in git, and
    // an ignore-all file there hid untracked duplicate ledger folders from
    // `git status`.
    if dir_is_empty(&generated) {
        std::fs::write(generated.join(".gitignore"), "*\n!.gitignore\n")
            .map_err(|e| format!("failed to write .gitignore: {e}"))?;
    }

    save_root_config(app_data_dir, &config)?;
    Ok(config)
}

fn dir_is_empty(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false)
}

/// Resolve sources directory: env var > config root > app_data_dir fallback.
pub fn resolve_sources(
    env_override: Option<&str>,
    config: &RootConfig,
    app_data_dir: &Path,
) -> PathBuf {
    if let Some(custom) = env_override {
        return PathBuf::from(custom);
    }
    if let Some(ref root) = config.current_root {
        return PathBuf::from(root).join("sources");
    }
    app_data_dir.join("sources")
}

/// Resolve generated directory: env var > config root > app_data_dir fallback.
pub fn resolve_generated(
    env_override: Option<&str>,
    config: &RootConfig,
    app_data_dir: &Path,
) -> PathBuf {
    if let Some(custom) = env_override {
        return PathBuf::from(custom);
    }
    if let Some(ref root) = config.current_root {
        return PathBuf::from(root).join("generated");
    }
    app_data_dir.join("generated")
}

/// Strict variant for the GUI app: env var > config root, no app-data
/// fallback. The app-data dir must only ever hold config.json, so with no
/// vault selected every data command errors instead of silently reading or
/// materializing app_data/sources.
pub fn resolve_sources_strict(
    env_override: Option<&str>,
    config: &RootConfig,
) -> Result<PathBuf, String> {
    if let Some(custom) = env_override {
        return Ok(PathBuf::from(custom));
    }
    match config.current_root {
        Some(ref root) => Ok(PathBuf::from(root).join("sources")),
        None => Err("No vault selected — open or create a data folder first".to_string()),
    }
}

/// Strict variant for the GUI app: see [`resolve_sources_strict`].
pub fn resolve_generated_strict(
    env_override: Option<&str>,
    config: &RootConfig,
) -> Result<PathBuf, String> {
    if let Some(custom) = env_override {
        return Ok(PathBuf::from(custom));
    }
    match config.current_root {
        Some(ref root) => Ok(PathBuf::from(root).join("generated")),
        None => Err("No vault selected — open or create a data folder first".to_string()),
    }
}
