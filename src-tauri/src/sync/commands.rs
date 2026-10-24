//! The sync build's Tauri commands (docs/SYNC.md). This file is a module of
//! the app binary — `main.rs` declares it with `#[path]` — not of the library,
//! because the commands use the app's vault resolution.

use super::resolve_sources_dir;
use emuaccounts::sync::content_store::ContentStore;
use emuaccounts::sync::metadata::{DeviceInfo, MetadataStore, SyncEvent};
use emuaccounts::sync::protocol::full_sync;
use emuaccounts::sync::relay_client;
use emuaccounts::sync::state::CAS_DIR;
use emuaccounts::trade_links::METADATA_FILE;
use parking_lot::Mutex;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Expands `$app_handler!` (main.rs's command handler) with the sync commands
/// appended, so the app registers them only in the sync build.
macro_rules! with_sync_commands {
    ($app_handler:ident) => {
        $app_handler!(
            , sync_commands::init_sync
            , sync_commands::get_sync_log
            , sync_commands::merge_metadata
            , sync_commands::list_devices
            , sync_commands::sync_with_remote
            , sync_commands::pair_initiate
            , sync_commands::pair_join
            , sync_commands::sync_with_relay_cmd
            , sync_commands::save_relay_config
            , sync_commands::get_relay_config
        )
    };
}
pub(crate) use with_sync_commands;

/// The current vault's metadata document, opened by the first sync command
/// that needs it.
#[derive(Default)]
pub struct MetadataState(Mutex<Option<MetadataStore>>);

impl MetadataState {
    /// Run `f` on the current vault's document, opening it first when none is
    /// open or the open one belongs to the vault before a switch.
    fn with_store<T>(
        &self,
        app: &tauri::AppHandle,
        f: impl FnOnce(&mut MetadataStore, &Path) -> Result<T, String>,
    ) -> Result<T, String> {
        let sources_dir = resolve_sources_dir(app)?;
        let path = sources_dir.join(METADATA_FILE);
        let mut open = self.0.lock();
        let store = match open.take() {
            Some(store) if store.metadata_path() == path => open.insert(store),
            _ => open.insert(MetadataStore::new(path)?),
        };
        f(store, &sources_dir)
    }
}

/// Register this device in the metadata document and add the vault's sources
/// to its manifest, unless this device already did. The sync UI calls it once
/// at startup.
#[tauri::command(async)]
pub fn init_sync(app: tauri::AppHandle, state: tauri::State<MetadataState>) -> Result<(), String> {
    state.with_store(&app, |store, sources_dir| {
        if !store.knows_this_device()? {
            store.build_from_sources(sources_dir)?;
            store.save()?;
        }
        Ok(())
    })
}

#[tauri::command(async)]
pub fn get_sync_log(
    app: tauri::AppHandle,
    state: tauri::State<MetadataState>,
) -> Result<Vec<SyncEvent>, String> {
    state.with_store(&app, |store, _| Ok(store.get_metadata()?.sync_log))
}

#[tauri::command(async)]
pub fn merge_metadata(
    app: tauri::AppHandle,
    remote_path: String,
    state: tauri::State<MetadataState>,
) -> Result<(), String> {
    state.with_store(&app, |store, _| {
        store.merge_from_file(Path::new(&remote_path))?;
        store.save()
    })
}

#[tauri::command(async)]
pub fn list_devices(
    app: tauri::AppHandle,
    state: tauri::State<MetadataState>,
) -> Result<Vec<DeviceInfo>, String> {
    state.with_store(&app, |store, _| {
        Ok(store.get_metadata()?.devices.into_values().collect())
    })
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    files_transferred: usize,
    metadata_merged: bool,
}

#[tauri::command(async)]
pub fn sync_with_remote(
    app: tauri::AppHandle,
    remote_metadata_path: String,
    remote_cas_path: String,
    state: tauri::State<MetadataState>,
) -> Result<SyncResponse, String> {
    state.with_store(&app, |store, sources_dir| {
        let local_cas = ContentStore::new(sources_dir.join(CAS_DIR));
        let remote_cas = ContentStore::new(PathBuf::from(&remote_cas_path));

        let result = full_sync(
            store,
            &local_cas,
            Path::new(&remote_metadata_path),
            &remote_cas,
        )?;

        store.save()?;

        Ok(SyncResponse {
            files_transferred: result.files_transferred,
            metadata_merged: result.metadata_merged,
        })
    })
}

#[derive(Debug, Serialize)]
pub struct PairInitiateResult {
    group_id: String,
    pairing_code: String,
    expires_in: u64,
}

#[tauri::command(async)]
pub fn pair_initiate(relay_url: String) -> Result<PairInitiateResult, String> {
    let result = relay_client::pair_initiate(&relay_url)?;
    Ok(PairInitiateResult {
        group_id: result.group_id,
        pairing_code: result.pairing_code,
        expires_in: result.expires_in,
    })
}

#[tauri::command(async)]
pub fn pair_join(relay_url: String, pairing_code: String) -> Result<String, String> {
    relay_client::pair_join(&relay_url, &pairing_code)
}

#[derive(Debug, Serialize)]
pub struct RelaySyncResponse {
    metadata_merged: bool,
    blobs_uploaded: usize,
    blobs_downloaded: usize,
}

#[tauri::command(async)]
pub fn sync_with_relay_cmd(
    app: tauri::AppHandle,
    state: tauri::State<MetadataState>,
) -> Result<RelaySyncResponse, String> {
    state.with_store(&app, |store, sources_dir| {
        let local_cas = ContentStore::new(sources_dir.join(CAS_DIR));

        let config = relay_client::load_relay_config(sources_dir)?
            .ok_or("Relay not configured. Set relay URL and pair first.")?;

        let result = relay_client::sync_with_relay(store, &local_cas, &config)?;

        Ok(RelaySyncResponse {
            metadata_merged: result.metadata_merged,
            blobs_uploaded: result.blobs_uploaded,
            blobs_downloaded: result.blobs_downloaded,
        })
    })
}

#[tauri::command(async)]
pub fn save_relay_config(
    app: tauri::AppHandle,
    relay_url: String,
    group_id: String,
) -> Result<(), String> {
    let sources_dir = resolve_sources_dir(&app)?;
    let config = relay_client::RelayConfig {
        relay_url,
        group_id,
    };
    relay_client::save_relay_config(&sources_dir, &config)
}

#[tauri::command(async)]
pub fn get_relay_config(
    app: tauri::AppHandle,
) -> Result<Option<relay_client::RelayConfig>, String> {
    let sources_dir = resolve_sources_dir(&app)?;
    relay_client::load_relay_config(&sources_dir)
}
