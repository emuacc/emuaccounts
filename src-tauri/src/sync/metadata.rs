//! The sync build's metadata document (`sources/`[`METADATA_FILE`]):
//! the device-sync metadata the relay merges between devices — transaction
//! refs, the file manifest, rules, the sync log and the known devices.
//!
//! Trade links are not part of it. Every build keeps them in
//! `sources/`[`crate::trade_links::TRADE_LINKS_FILE`], which the sync carries
//! like any other source file. A document from before TASK-133 still holds
//! its links in a `trade_links` map; nothing here reads or writes that key
//! (struct reconcile leaves the keys it does not name untouched), so the
//! migration in [`crate::trade_links`] can always read it.

use automerge::AutoCommit;
use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::content_store::sha256_hex;
use crate::ledger_parser::parse_transactions;
use crate::rules::RulesFile;
#[cfg(doc)]
use crate::trade_links::METADATA_FILE;
use crate::trade_links::{extract_txn_id, get_device_id, now};

// ── The document ────────────────────────────────────────────────────

pub struct MetadataStore {
    doc: AutoCommit,
    device_id: String,
    metadata_path: PathBuf,
}

impl std::fmt::Debug for MetadataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataStore")
            .field("device_id", &self.device_id)
            .field("metadata_path", &self.metadata_path)
            .finish()
    }
}

impl MetadataStore {
    /// Open the document at `metadata_path`, or start an empty one in memory
    /// (written by the first [`MetadataStore::save`]).
    pub fn new(metadata_path: PathBuf) -> Result<Self, String> {
        let doc = if metadata_path.exists() {
            let bytes =
                fs::read(&metadata_path).map_err(|e| format!("Failed to read metadata: {}", e))?;
            AutoCommit::load(&bytes).map_err(|e| format!("Failed to load Automerge doc: {}", e))?
        } else {
            AutoCommit::new()
        };
        Ok(Self {
            doc,
            device_id: get_device_id(),
            metadata_path,
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn metadata_path(&self) -> &Path {
        &self.metadata_path
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.metadata_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {}", e))?;
        }
        let bytes = self.doc.clone().save();
        fs::write(&self.metadata_path, bytes).map_err(|e| format!("Save failed: {}", e))?;
        Ok(())
    }
}

// ── Metadata schema ─────────────────────────────────────────────────

/// The whole metadata document. Any field may be missing: a document a
/// default build started before TASK-133 holds only `trade_links`, which this
/// schema leaves alone.
#[derive(Reconcile, Hydrate, Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct EmuAccMetadata {
    #[autosurgeon(missing = "Default::default")]
    pub transaction_refs: Vec<TxnRef>,
    #[autosurgeon(missing = "Default::default")]
    pub accounts: HashMap<String, AccountMeta>,
    #[autosurgeon(missing = "Default::default")]
    pub rules: HashMap<String, RuleMeta>,
    #[autosurgeon(missing = "Default::default")]
    pub file_manifest: HashMap<String, FileEntry>,
    #[autosurgeon(missing = "Default::default")]
    pub sync_log: Vec<SyncEvent>,
    #[autosurgeon(missing = "Default::default")]
    pub devices: HashMap<String, DeviceInfo>,
}

#[derive(Reconcile, Hydrate, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TxnRef {
    pub id: String,
    pub content_hash: String,
    pub file_path: String,
    pub datetime: String,
    pub device_origin: String,
    pub created_at: i64,
}

#[derive(Reconcile, Hydrate, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AccountMeta {
    pub path: String,
    pub default_commodity: Option<String>,
    pub device_origin: String,
}

#[derive(Reconcile, Hydrate, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RuleMeta {
    pub id: String,
    pub pattern: String,
    pub payee: Option<String>,
    pub postings: Vec<String>,
    pub account_folder: String,
    pub device_origin: String,
}

#[derive(Reconcile, Hydrate, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub content_hash: String,
    pub relative_path: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub device_origin: String,
    pub uploaded_at: i64,
}

#[derive(Reconcile, Hydrate, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SyncEvent {
    pub timestamp: i64,
    pub device_id: String,
    pub event_type: String,
    pub target_id: String,
    pub details: String,
}

#[derive(Reconcile, Hydrate, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub last_seen: i64,
}

// ── Sync operations on the store ────────────────────────────────────

impl MetadataStore {
    /// The whole document.
    pub fn get_metadata(&self) -> Result<EmuAccMetadata, String> {
        autosurgeon::hydrate(&self.doc).map_err(|e| format!("Hydrate failed: {}", e))
    }

    /// Whether this device has registered itself
    /// ([`MetadataStore::build_from_sources`]).
    pub fn knows_this_device(&self) -> Result<bool, String> {
        Ok(self.get_metadata()?.devices.contains_key(&self.device_id))
    }

    /// Register this device and add the transactions, files and rules under
    /// `sources_dir`. Everything already in the document — other devices'
    /// entries, the sync log, a pre-TASK-133 document's trade links — is kept.
    pub fn build_from_sources(&mut self, sources_dir: &Path) -> Result<(), String> {
        let mut metadata = self.get_metadata()?;

        self.scan_transactions(sources_dir, &mut metadata)?;
        self.scan_files(sources_dir, &mut metadata)?;
        self.scan_rules(sources_dir, &mut metadata)?;

        metadata.devices.insert(
            self.device_id.clone(),
            DeviceInfo {
                device_id: self.device_id.clone(),
                device_name: get_hostname(),
                last_seen: now(),
            },
        );

        autosurgeon::reconcile(&mut self.doc, &metadata)
            .map_err(|e| format!("Reconcile failed: {}", e))?;

        self.log_sync_event("metadata_built", "", "Built from sources")?;

        Ok(())
    }

    fn scan_transactions(
        &self,
        sources_dir: &Path,
        metadata: &mut EmuAccMetadata,
    ) -> Result<(), String> {
        // Collect all manual.transactions files recursively
        let mut manual_files: Vec<std::path::PathBuf> = Vec::new();
        for entry in WalkDir::new(sources_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file()
                && path.file_name().and_then(|n| n.to_str()) == Some("manual.transactions")
            {
                manual_files.push(path.to_path_buf());
            }
        }

        for manual_path in &manual_files {
            let rel_path = manual_path
                .strip_prefix(sources_dir)
                .map(|p| format!("sources/{}", p.display()))
                .unwrap_or_else(|_| format!("{}", manual_path.display()));

            let content =
                fs::read_to_string(manual_path).map_err(|e| format!("Read failed: {}", e))?;

            let parse_result = parse_transactions(&content);

            for (idx, txn) in parse_result.transactions.iter().enumerate() {
                let txn_id = extract_txn_id(&txn.meta).unwrap_or_else(|| format!("manual-{}", idx));
                let txn_text = format!("{} {}", txn.datetime, txn.payee.as_deref().unwrap_or(""));
                metadata.transaction_refs.push(TxnRef {
                    id: txn_id,
                    content_hash: sha256_hex(txn_text.as_bytes()),
                    file_path: rel_path.clone(),
                    datetime: txn.datetime.clone(),
                    device_origin: self.device_id.clone(),
                    created_at: now(),
                });
            }
        }
        Ok(())
    }

    fn scan_files(&self, sources_dir: &Path, metadata: &mut EmuAccMetadata) -> Result<(), String> {
        if !sources_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(sources_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let file_type = match ext {
                "csv" => "csv",
                "rhai" => "transform",
                "json" => "json",
                "transactions" => "transactions",
                "pdf" => "pdf",
                _ => continue,
            };

            let content = fs::read(path).map_err(|e| format!("Read failed: {}", e))?;
            let hash = sha256_hex(&content);

            let relative = path
                .strip_prefix(sources_dir)
                .map_err(|e| format!("Path error: {}", e))?;

            metadata.file_manifest.insert(
                hash.clone(),
                FileEntry {
                    content_hash: hash,
                    relative_path: relative.to_string_lossy().to_string(),
                    file_type: file_type.to_string(),
                    size_bytes: content.len() as u64,
                    device_origin: self.device_id.clone(),
                    uploaded_at: now(),
                },
            );
        }

        Ok(())
    }

    fn scan_rules(&self, sources_dir: &Path, metadata: &mut EmuAccMetadata) -> Result<(), String> {
        if !sources_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(sources_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) != Some("_rules.json") {
                continue;
            }

            let folder = path.parent().unwrap_or(sources_dir);
            let account_folder = folder
                .strip_prefix(sources_dir)
                .unwrap_or(Path::new(""))
                .to_string_lossy()
                .to_string();

            let rules_file = RulesFile::load(folder);
            for rule in &rules_file.rules {
                metadata.rules.insert(
                    rule.id.clone(),
                    RuleMeta {
                        id: rule.id.clone(),
                        pattern: rule.pattern.clone(),
                        payee: rule.payee.clone(),
                        postings: rule.postings.clone(),
                        account_folder: account_folder.clone(),
                        device_origin: self.device_id.clone(),
                    },
                );
            }
        }

        Ok(())
    }

    pub fn merge_from_file(&mut self, remote_path: &Path) -> Result<(), String> {
        let remote_bytes =
            fs::read(remote_path).map_err(|e| format!("Read remote failed: {}", e))?;
        self.merge_from_bytes(&remote_bytes)?;
        self.log_sync_event(
            "merged_remote",
            "",
            &format!("Merged from {:?}", remote_path),
        )?;
        Ok(())
    }

    /// Merge from raw Automerge bytes. Tolerates "duplicate seq" errors
    /// which occur when merging a doc that already contains our own changes
    /// (e.g. when syncing through a relay that has the merged result).
    pub fn merge_from_bytes(&mut self, remote_bytes: &[u8]) -> Result<(), String> {
        let mut remote_doc =
            AutoCommit::load(remote_bytes).map_err(|e| format!("Load remote failed: {}", e))?;

        match self.doc.merge(&mut remote_doc) {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("duplicate seq") {
                    // Benign: the remote doc contains changes we already have
                    Ok(())
                } else {
                    Err(format!("Merge failed: {}", e))
                }
            }
        }
    }

    pub fn log_sync_event(
        &mut self,
        event_type: &str,
        target_id: &str,
        details: &str,
    ) -> Result<(), String> {
        let mut metadata: EmuAccMetadata =
            autosurgeon::hydrate(&self.doc).map_err(|e| format!("Hydrate failed: {}", e))?;

        metadata.sync_log.push(SyncEvent {
            timestamp: now(),
            device_id: self.device_id.clone(),
            event_type: event_type.to_string(),
            target_id: target_id.to_string(),
            details: details.to_string(),
        });

        autosurgeon::reconcile(&mut self.doc, &metadata)
            .map_err(|e| format!("Reconcile failed: {}", e))?;

        Ok(())
    }

    /// Register a file in the metadata file manifest.
    pub fn register_file(
        &mut self,
        content_hash: &str,
        relative_path: &str,
        file_type: &str,
        size_bytes: u64,
    ) -> Result<(), String> {
        let mut metadata: EmuAccMetadata =
            autosurgeon::hydrate(&self.doc).map_err(|e| format!("Hydrate failed: {}", e))?;

        metadata.file_manifest.insert(
            content_hash.to_string(),
            FileEntry {
                content_hash: content_hash.to_string(),
                relative_path: relative_path.to_string(),
                file_type: file_type.to_string(),
                size_bytes,
                device_origin: self.device_id.clone(),
                uploaded_at: now(),
            },
        );

        autosurgeon::reconcile(&mut self.doc, &metadata)
            .map_err(|e| format!("Reconcile failed: {}", e))?;

        Ok(())
    }

    /// Recover metadata entries that were lost during an Automerge merge due to
    /// conflicting map objects created independently by different devices.
    /// Returns true if any entries were recovered.
    pub fn recover_from_snapshot(&mut self, snapshot: &EmuAccMetadata) -> Result<bool, String> {
        let mut current: EmuAccMetadata =
            autosurgeon::hydrate(&self.doc).map_err(|e| format!("Hydrate failed: {}", e))?;

        let mut changed = false;

        // Recover file_manifest entries
        for (hash, entry) in &snapshot.file_manifest {
            if !current.file_manifest.contains_key(hash) {
                current.file_manifest.insert(hash.clone(), entry.clone());
                changed = true;
            }
        }

        // Recover sync_log entries (match by all fields to avoid false dedup)
        for event in &snapshot.sync_log {
            let exists = current.sync_log.iter().any(|e| {
                e.timestamp == event.timestamp
                    && e.device_id == event.device_id
                    && e.event_type == event.event_type
                    && e.target_id == event.target_id
                    && e.details == event.details
            });
            if !exists {
                current.sync_log.push(event.clone());
                changed = true;
            }
        }

        // Recover device entries
        for (id, info) in &snapshot.devices {
            if !current.devices.contains_key(id) {
                current.devices.insert(id.clone(), info.clone());
                changed = true;
            }
        }

        if changed {
            autosurgeon::reconcile(&mut self.doc, &current)
                .map_err(|e| format!("Reconcile failed: {}", e))?;
        }

        Ok(changed)
    }
}

fn get_hostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_empty_store() {
        let tmp = std::env::temp_dir().join("emuacc-test-empty-store.automerge");
        let _ = fs::remove_file(&tmp);
        let store = MetadataStore::new(tmp.clone()).unwrap();
        let meta = store.get_metadata().unwrap();
        assert!(meta.transaction_refs.is_empty());
        assert!(meta.file_manifest.is_empty());
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_save_and_reload() {
        let tmp = std::env::temp_dir().join("emuacc-test-save-reload.automerge");
        let _ = fs::remove_file(&tmp);

        {
            let mut store = MetadataStore::new(tmp.clone()).unwrap();
            store
                .log_sync_event("test_event", "target-1", "testing save")
                .unwrap();
            store.save().unwrap();
        }

        {
            let store = MetadataStore::new(tmp.clone()).unwrap();
            let meta = store.get_metadata().unwrap();
            assert!(!meta.sync_log.is_empty());
            assert_eq!(meta.sync_log[0].event_type, "test_event");
        }

        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_merge_two_docs() {
        let tmp_a = std::env::temp_dir().join("emuacc-test-merge-a.automerge");
        let tmp_b = std::env::temp_dir().join("emuacc-test-merge-b.automerge");
        let _ = fs::remove_file(&tmp_a);
        let _ = fs::remove_file(&tmp_b);

        // Device A creates the original doc
        {
            let mut store_a = MetadataStore::new(tmp_a.clone()).unwrap();
            store_a.log_sync_event("event_a", "", "from A").unwrap();
            store_a.save().unwrap();
        }

        // Device B loads from A's doc (shared origin), then adds its own event
        fs::copy(&tmp_a, &tmp_b).unwrap();
        {
            let mut store_b = MetadataStore::new(tmp_b.clone()).unwrap();
            store_b.log_sync_event("event_b", "", "from B").unwrap();
            store_b.save().unwrap();
        }

        // Device A adds more work concurrently
        {
            let mut store_a = MetadataStore::new(tmp_a.clone()).unwrap();
            store_a
                .log_sync_event("event_a2", "", "more from A")
                .unwrap();
            store_a.save().unwrap();
        }

        // Merge B into A
        {
            let mut store_a = MetadataStore::new(tmp_a.clone()).unwrap();
            store_a.merge_from_file(&tmp_b).unwrap();
            let meta = store_a.get_metadata().unwrap();
            let event_types: Vec<&str> = meta
                .sync_log
                .iter()
                .map(|e| e.event_type.as_str())
                .collect();
            assert!(
                event_types.contains(&"event_a"),
                "missing event_a; got: {:?}",
                event_types
            );
            assert!(
                event_types.contains(&"event_b"),
                "missing event_b; got: {:?}",
                event_types
            );
            assert!(
                event_types.contains(&"event_a2"),
                "missing event_a2; got: {:?}",
                event_types
            );
        }

        let _ = fs::remove_file(&tmp_a);
        let _ = fs::remove_file(&tmp_b);
    }

    #[test]
    fn a_pre_task_133_documents_trade_links_survive_the_sync_writes() {
        let sources = tempfile::tempdir().unwrap();
        let doc = sources.path().join(crate::trade_links::METADATA_FILE);
        fs::write(
            &doc,
            include_bytes!("../../features/fixtures/trade_links_doc.automerge"),
        )
        .unwrap();
        let links = crate::trade_links::read(sources.path()).unwrap();
        assert_eq!(links.len(), 2);
        fs::create_dir_all(sources.path().join("bank")).unwrap();
        fs::write(
            sources.path().join("bank/2025-01.csv"),
            "Date,Amount\n2025-01-01,1\n",
        )
        .unwrap();

        let mut store = MetadataStore::new(doc).unwrap();
        store.build_from_sources(sources.path()).unwrap();
        store.register_file("abc123", "a.csv", "csv", 3).unwrap();
        let snapshot = store.get_metadata().unwrap();
        store.recover_from_snapshot(&snapshot).unwrap();
        store.save().unwrap();

        assert!(store.knows_this_device().unwrap());
        assert_eq!(
            crate::trade_links::read(sources.path()).unwrap(),
            links,
            "the document's legacy trade links are left as they were"
        );
    }

    /// The pre-TASK-133 store's schema, which wrote the documents the default
    /// build's reader ([`crate::trade_links`]) migrates.
    #[derive(Reconcile, Hydrate, Clone, PartialEq, Debug)]
    struct LegacyLink {
        id: String,
        txn_id_a: String,
        txn_id_b: String,
        device_origin: String,
        created_at: i64,
    }

    #[derive(Reconcile, Hydrate, Default)]
    struct LegacyLinks {
        #[autosurgeon(missing = "HashMap::new")]
        trade_links: HashMap<String, LegacyLink>,
    }

    /// One random edit as the pre-TASK-133 store made them: link a pair from a
    /// small pool (re-linking replaces), or delete a link.
    fn random_edit(doc: &mut AutoCommit, rng: &mut impl rand::Rng, pool: usize) {
        let mut links: LegacyLinks = autosurgeon::hydrate(doc).unwrap();
        let pair = rng.gen_range(0..pool);
        let id =
            crate::trade_links::trade_link_id(&format!("txn:a{pair}"), &format!("txn:b{pair}"));
        if rng.gen_bool(0.25) {
            links.trade_links.remove(&id);
        } else {
            let created_at = match rng.gen_range(0..4) {
                0 => i64::MIN,
                1 => i64::MAX,
                2 => -rng.gen_range(0..1_000_000_000),
                _ => rng.gen_range(1_600_000_000..1_800_000_000),
            };
            let device_origin = ["device-aaaa1111", "", "dévice-ü", "device-unknown"]
                [rng.gen_range(0..4)]
            .to_string();
            links.trade_links.insert(
                id.clone(),
                LegacyLink {
                    id,
                    txn_id_a: format!("txn:a{pair}"),
                    txn_id_b: format!("txn:b{pair}"),
                    device_origin,
                    created_at,
                },
            );
        }
        autosurgeon::reconcile(doc, &links).unwrap();
    }

    #[test]
    fn the_default_builds_reader_agrees_with_automerge_on_the_trade_links() {
        use automerge::ActorId;
        use rand::{Rng, SeedableRng};

        for seed in 0..30u64 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            // Small pools conflict often; 40 links deflate the key columns.
            let pool = [3, 12, 40][seed as usize % 3];
            let mut a =
                AutoCommit::new().with_actor(ActorId::from(rng.gen::<[u8; 16]>().as_slice()));
            autosurgeon::reconcile(&mut a, LegacyLinks::default()).unwrap();
            let mut b = AutoCommit::load(&a.save())
                .unwrap()
                .with_actor(ActorId::from(rng.gen::<[u8; 16]>().as_slice()));
            for _ in 0..rng.gen_range(1..4) {
                for _ in 0..rng.gen_range(0..pool.min(20)) {
                    random_edit(&mut a, &mut rng, pool);
                }
                for _ in 0..rng.gen_range(0..pool.min(20)) {
                    random_edit(&mut b, &mut rng, pool);
                }
                let mut sync = EmuAccMetadata::default();
                sync.sync_log.push(SyncEvent {
                    timestamp: rng.gen(),
                    device_id: "device-bbbb2222".into(),
                    event_type: "relay_sync".into(),
                    target_id: String::new(),
                    details: String::new(),
                });
                autosurgeon::reconcile(&mut b, &sync).unwrap();
                a.merge(&mut b).unwrap();
                if rng.gen_bool(0.5) {
                    b.merge(&mut a).unwrap();
                }
            }

            let sources = tempfile::tempdir().unwrap();
            fs::write(
                sources.path().join(crate::trade_links::METADATA_FILE),
                a.save(),
            )
            .unwrap();
            let read = crate::trade_links::read(sources.path()).unwrap();
            let hydrated: LegacyLinks = autosurgeon::hydrate(&a).unwrap();
            let mut expected: Vec<crate::trade_links::TradeLink> = hydrated
                .trade_links
                .into_values()
                .map(|l| crate::trade_links::TradeLink {
                    id: l.id,
                    txn_id_a: l.txn_id_a,
                    txn_id_b: l.txn_id_b,
                    device_origin: l.device_origin,
                    created_at: l.created_at,
                })
                .collect();
            expected.sort_by(|x, y| x.id.cmp(&y.id));
            assert_eq!(read, expected, "seed {seed}");
        }
    }
}
