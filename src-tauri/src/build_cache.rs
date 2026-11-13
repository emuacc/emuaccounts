use crate::ledger_parser::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTransaction {
    pub text: String,
    /// Parsed Transaction struct, stored to avoid re-parsing on cache load.
    /// None for old cache entries (pre-migration); falls back to parse_transactions.
    #[serde(default)]
    pub txn: Option<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub content_hash: String,
    pub transform_hash: String,
    pub transactions: Vec<CachedTransaction>,
}

// --- Per-folder cache file (one per source folder) ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FolderCacheFile {
    pub fingerprint: String,
    /// Per-CSV/OFX file entries within this folder (key: relative path from sources_dir)
    #[serde(default)]
    pub entries: HashMap<String, CacheEntry>,
    /// Manual transactions content hash for this folder
    #[serde(default)]
    pub manual_hash: Option<String>,
    /// Accounts file hash + cached text for this folder
    #[serde(default)]
    pub accounts_hash: Option<String>,
    #[serde(default)]
    pub accounts_text: Option<String>,
}

// --- Lightweight global manifest (always loaded) ---

/// What `fs::metadata` said about an output file right after the pipeline
/// wrote it. A file whose size and mtime still match is taken as unchanged
/// without being read; one that moved is re-hashed against `output_hashes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputStat {
    pub size: u64,
    /// Modification time in nanoseconds since the Unix epoch.
    pub mtime_ns: u64,
}

/// The size and mtime of `path`, or None when it cannot be read.
pub fn file_stat(path: &Path) -> Option<OutputStat> {
    let meta = fs::metadata(path).ok()?;
    let mtime_ns = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos() as u64;
    Some(OutputStat {
        size: meta.len(),
        mtime_ns,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheManifest {
    pub inputs_hash: Option<String>,
    /// Quick lookup: folder relative path → fingerprint hash
    #[serde(default)]
    pub folder_fingerprints: HashMap<String, String>,
    /// Layer 4: SHA256 of each generated output file
    #[serde(default)]
    pub output_hashes: HashMap<String, String>,
    /// Size and mtime of each output file as the pipeline last wrote it, so
    /// the early exit can trust an unchanged file without re-hashing it.
    #[serde(default)]
    pub output_stats: HashMap<String, OutputStat>,
    /// Root-level _ignored.txt
    #[serde(default)]
    pub ignored_hash: Option<String>,
    #[serde(default)]
    pub ignored_ids: Vec<String>,
    /// Root-level accounts.transactions (key "" for root)
    #[serde(default)]
    pub root_accounts_hash: Option<String>,
    #[serde(default)]
    pub root_accounts_text: Option<String>,
}

// --- In-memory working struct (hydrated from manifest + folder files) ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildCache {
    /// Per-file cache entries (Layer 3: content_hash + transform_hash → skip re-transform)
    pub entries: HashMap<String, CacheEntry>,
    #[serde(default)]
    pub inputs_hash: Option<String>,
    #[serde(default)]
    pub folder_hashes: HashMap<String, String>,
    #[serde(default)]
    pub output_hashes: HashMap<String, String>,
    #[serde(default)]
    pub manual_hashes: HashMap<String, String>,
    #[serde(default)]
    pub accounts_hashes: HashMap<String, String>,
    #[serde(default)]
    pub accounts_cached: HashMap<String, String>,
    #[serde(default)]
    pub ignored_hash: Option<String>,
    #[serde(default)]
    pub ignored_ids: Vec<String>,
}

// --- Path helpers ---

fn cache_dir(generated_dir: &Path) -> PathBuf {
    generated_dir.join(".cache")
}

fn manifest_path(generated_dir: &Path) -> PathBuf {
    cache_dir(generated_dir).join("manifest.json")
}

fn folder_cache_filename(folder_rel: &str) -> String {
    format!("{}.json", folder_rel.replace('/', "--"))
}

fn folder_cache_path(generated_dir: &Path, folder_rel: &str) -> PathBuf {
    cache_dir(generated_dir).join(folder_cache_filename(folder_rel))
}

fn legacy_cache_path(generated_dir: &Path) -> PathBuf {
    generated_dir.join("build-cache.json")
}

// --- Load ---

/// Load full cache (all folders). Used by file watcher and full rebuilds.
pub fn load_cache(generated_dir: &Path) -> BuildCache {
    load_cache_filtered(generated_dir, None)
}

/// Load cache with an optional folder filter.
/// When `only_folders` is Some, only those folders' cache files are loaded.
/// The manifest (fingerprints, output hashes, etc.) is always loaded in full.
pub fn load_cache_filtered(
    generated_dir: &Path,
    only_folders: Option<&HashSet<String>>,
) -> BuildCache {
    let manifest_p = manifest_path(generated_dir);
    if manifest_p.exists() {
        return load_from_folder_cache(generated_dir, only_folders);
    }
    // Fall back to legacy monolithic build-cache.json (migration path)
    let legacy_p = legacy_cache_path(generated_dir);
    if legacy_p.exists() {
        let contents = match fs::read_to_string(&legacy_p) {
            Ok(c) => c,
            Err(_) => return BuildCache::default(),
        };
        return serde_json::from_str(&contents).unwrap_or_default();
    }
    BuildCache::default()
}

/// The manifest alone, without any folder cache: enough to decide the early
/// exit. None when there is no readable manifest (a first run, or a legacy
/// build-cache.json a full load still has to migrate).
pub fn load_manifest(generated_dir: &Path) -> Option<CacheManifest> {
    let text = fs::read_to_string(manifest_path(generated_dir)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Does every output file in `manifest.output_hashes` still hold what the
/// pipeline last wrote? Without this check an externally modified generated
/// file (git checkout, manual edit, another tool) would be invisible: the
/// sources fingerprint matches, the pipeline returns at once and never writes
/// the corrected file. A file whose size and mtime match `output_stats` is
/// trusted without being read; one that moved, or has no recorded stat, is
/// re-hashed; a missing file is a divergence.
pub fn outputs_intact(generated_dir: &Path, manifest: &CacheManifest) -> bool {
    manifest.output_hashes.iter().all(|(rel_key, expected)| {
        let path = generated_dir.join(rel_key);
        if let (Some(recorded), Some(now)) = (manifest.output_stats.get(rel_key), file_stat(&path))
        {
            if recorded == &now {
                return true;
            }
        }
        match fs::read_to_string(&path) {
            Ok(text) => &string_hash(&text) == expected,
            Err(_) => false,
        }
    })
}

/// The size and mtime of each of `rel_keys` under `generated_dir`, for those
/// that can be read.
fn output_stats<'a>(
    generated_dir: &Path,
    rel_keys: impl Iterator<Item = &'a String>,
) -> HashMap<String, OutputStat> {
    rel_keys
        .filter_map(|rel_key| {
            file_stat(&generated_dir.join(rel_key)).map(|stat| (rel_key.clone(), stat))
        })
        .collect()
}

fn save_manifest(generated_dir: &Path, manifest: &CacheManifest) -> Result<(), String> {
    fs::create_dir_all(cache_dir(generated_dir)).map_err(|e| e.to_string())?;
    let json = crate::to_sorted_json_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(manifest_path(generated_dir), json).map_err(|e| e.to_string())
}

/// Record the size and mtime of every output file whose stat the manifest
/// lacks and write the manifest back, so a manifest saved before stats
/// existed takes the fast path from its next run. Nothing is written when
/// every output already has a stat.
pub fn complete_output_stats(
    generated_dir: &Path,
    manifest: &mut CacheManifest,
) -> Result<(), String> {
    if manifest
        .output_hashes
        .keys()
        .all(|rel_key| manifest.output_stats.contains_key(rel_key))
    {
        return Ok(());
    }
    manifest.output_stats = output_stats(generated_dir, manifest.output_hashes.keys());
    save_manifest(generated_dir, manifest)
}

fn load_from_folder_cache(
    generated_dir: &Path,
    only_folders: Option<&HashSet<String>>,
) -> BuildCache {
    let manifest: CacheManifest = match fs::read_to_string(manifest_path(generated_dir)) {
        Ok(c) => serde_json::from_str(&c).unwrap_or_default(),
        Err(_) => return BuildCache::default(),
    };

    let mut cache = BuildCache {
        inputs_hash: manifest.inputs_hash,
        folder_hashes: manifest.folder_fingerprints.clone(),
        output_hashes: manifest.output_hashes,
        ignored_hash: manifest.ignored_hash,
        ignored_ids: manifest.ignored_ids,
        ..Default::default()
    };

    // Root-level accounts
    if let Some(h) = manifest.root_accounts_hash {
        cache.accounts_hashes.insert(String::new(), h);
    }
    if let Some(t) = manifest.root_accounts_text {
        cache.accounts_cached.insert(String::new(), t);
    }

    // Determine which folder cache files to load.
    let mut folder_rels: Vec<String> = manifest.folder_fingerprints.keys().cloned().collect();
    // Also scan .cache/ dir for orphaned cache files (fingerprints may have been cleared)
    if only_folders.is_none() {
        if let Ok(dir_entries) = fs::read_dir(cache_dir(generated_dir)) {
            for entry in dir_entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "manifest.json" {
                    continue;
                }
                if let Some(stem) = name
                    .strip_suffix(".json")
                    .or_else(|| name.strip_suffix(".bin"))
                {
                    let rel = stem.replace("--", "/");
                    if !folder_rels.contains(&rel) {
                        folder_rels.push(rel);
                    }
                }
            }
        }
    }

    // Load folder cache files (filtered if requested)
    for folder_rel in &folder_rels {
        if let Some(filter) = only_folders {
            if !filter.contains(folder_rel) {
                continue;
            }
        }
        hydrate_folder_cache(&mut cache, generated_dir, folder_rel);
    }

    cache
}

/// Load a single folder's cache file and merge into the BuildCache.
/// Can be called after initial load to lazily load additional folders.
pub fn hydrate_folder_cache(cache: &mut BuildCache, generated_dir: &Path, folder_rel: &str) {
    let path = folder_cache_path(generated_dir, folder_rel);
    let fc: FolderCacheFile = match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => return,
    };
    for (k, v) in fc.entries {
        cache.entries.insert(k, v);
    }
    if let Some(h) = fc.manual_hash {
        let manual_key = if folder_rel.is_empty() {
            FILE_MANUAL.to_string()
        } else {
            format!("{folder_rel}/{FILE_MANUAL}")
        };
        cache.manual_hashes.insert(manual_key, h);
    }
    if let Some(h) = fc.accounts_hash {
        cache.accounts_hashes.insert(folder_rel.to_string(), h);
    }
    if let Some(t) = fc.accounts_text {
        cache.accounts_cached.insert(folder_rel.to_string(), t);
    }
}

const FILE_MANUAL: &str = "manual.transactions";

// --- Save ---

/// Save cache to per-folder files + manifest.
/// Only writes folder cache files for folders in `changed_folders`.
/// Pass None to write all folders (e.g. first run or migration).
pub fn save_cache(
    generated_dir: &Path,
    cache: &BuildCache,
    changed_folders: Option<&HashSet<String>>,
) -> Result<(), String> {
    let dir = cache_dir(generated_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Build manifest
    let manifest = CacheManifest {
        inputs_hash: cache.inputs_hash.clone(),
        folder_fingerprints: cache.folder_hashes.clone(),
        output_hashes: cache.output_hashes.clone(),
        output_stats: output_stats(generated_dir, cache.output_hashes.keys()),
        ignored_hash: cache.ignored_hash.clone(),
        ignored_ids: cache.ignored_ids.clone(),
        root_accounts_hash: cache.accounts_hashes.get("").cloned(),
        root_accounts_text: cache.accounts_cached.get("").cloned(),
    };
    save_manifest(generated_dir, &manifest)?;

    // Determine which folders to write
    let folders_to_write: HashSet<String> = match changed_folders {
        Some(changed) => cache
            .folder_hashes
            .keys()
            .filter(|k| changed.contains(*k))
            .cloned()
            .collect(),
        None => {
            // Write all folders: collect from all sources (entries, folder_hashes, etc.)
            let mut all: HashSet<String> = cache.folder_hashes.keys().cloned().collect();
            // Also collect folders from per-file entries
            for key in cache.entries.keys() {
                if let Some(pos) = key.rfind('/') {
                    all.insert(key[..pos].to_string());
                }
            }
            all
        }
    };

    for folder_rel in &folders_to_write {
        let fc = extract_folder_cache(cache, folder_rel);
        let json = serde_json::to_string(&fc).map_err(|e| e.to_string())?;
        fs::write(folder_cache_path(generated_dir, folder_rel), json).map_err(|e| e.to_string())?;
    }

    // Prune stale folder cache files (folders that no longer exist on disk).
    //
    // Critically, "stale" is "folder is gone" — NOT "folder wasn't changed
    // this run". With `changed_folders = Some({a,b,c})`, `folders_to_write`
    // only contains the changed folders. Pruning against that set deletes
    // every OTHER folder's cache file on every partial regen, dropping the
    // cache from ~24 files to a handful. Each subsequent regen then has to
    // re-transform those folders from scratch (cache miss → CSV re-runs).
    //
    // The "exists" set is the union of: (1) folder_hashes (every folder
    // we've ever fingerprinted), (2) parent folders of cache.entries keys
    // (every folder with a cached CSV/OFX). A cache file outside that set
    // is one we wrote in a prior run for a folder that has since been
    // deleted from sources/ — those are safe to remove.
    let known_folders: HashSet<String> = {
        let mut s: HashSet<String> = cache.folder_hashes.keys().cloned().collect();
        for key in cache.entries.keys() {
            if let Some(pos) = key.rfind('/') {
                s.insert(key[..pos].to_string());
            }
        }
        s
    };
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "manifest.json" {
                continue;
            }
            // Reverse the filename → folder_rel mapping
            if let Some(folder_rel) = name
                .strip_suffix(".bin")
                .or_else(|| name.strip_suffix(".json"))
            {
                let folder_rel = folder_rel.replace("--", "/");
                if !known_folders.contains(&folder_rel) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    // Remove legacy cache if it exists (migration complete)
    let legacy = legacy_cache_path(generated_dir);
    if legacy.exists() {
        let _ = fs::remove_file(&legacy);
    }

    Ok(())
}

/// Extract a FolderCacheFile from the in-memory BuildCache for a given folder.
fn extract_folder_cache(cache: &BuildCache, folder_rel: &str) -> FolderCacheFile {
    // Collect entries belonging to this folder (entries whose path starts with folder_rel/)
    let prefix = if folder_rel.is_empty() {
        String::new()
    } else {
        format!("{folder_rel}/")
    };
    let entries: HashMap<String, CacheEntry> = cache
        .entries
        .iter()
        .filter(|(k, _)| {
            if folder_rel.is_empty() {
                !k.contains('/')
            } else {
                k.starts_with(&prefix) && !k[prefix.len()..].contains('/')
            }
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let manual_key = if folder_rel.is_empty() {
        FILE_MANUAL.to_string()
    } else {
        format!("{folder_rel}/{FILE_MANUAL}")
    };

    FolderCacheFile {
        fingerprint: cache
            .folder_hashes
            .get(folder_rel)
            .cloned()
            .unwrap_or_default(),
        entries,
        manual_hash: cache.manual_hashes.get(&manual_key).cloned(),
        accounts_hash: cache.accounts_hashes.get(folder_rel).cloned(),
        accounts_text: cache.accounts_cached.get(folder_rel).cloned(),
    }
}

pub fn lookup<'a>(
    cache: &'a BuildCache,
    relative_path: &str,
    content_hash: &str,
    transform_hash: &str,
) -> Option<&'a [CachedTransaction]> {
    let entry = cache.entries.get(relative_path)?;
    if entry.content_hash == content_hash && entry.transform_hash == transform_hash {
        Some(&entry.transactions)
    } else {
        None
    }
}

pub fn file_hash(path: &Path) -> Result<String, String> {
    let contents = fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

pub fn string_hash(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Git-style tree fingerprint from file metadata (no content reads).
/// Hash of sorted (relative_path, mtime_secs, size) tuples.
pub fn dir_fingerprint(dir: &Path) -> Result<String, String> {
    fingerprint_where(dir, |_| true)
}

/// [`dir_fingerprint`] of a `sources/` directory without the app's own state
/// ([`crate::sources_state`]): saving a trade link rewrites those files but
/// changes no pipeline input, so it must not defeat the global early exit.
pub fn sources_fingerprint(sources_dir: &Path) -> Result<String, String> {
    fingerprint_where(sources_dir, |relative| {
        !crate::sources_state::is_app_state(relative)
    })
}

/// The tree fingerprint over the files whose path relative to `dir` passes
/// `include`.
fn fingerprint_where(dir: &Path, include: impl Fn(&Path) -> bool) -> Result<String, String> {
    if !dir.exists() {
        return Ok(String::new());
    }
    let mut entries: Vec<(String, u64, u64)> = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        // Skip hidden files (e.g. .DS_Store)
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }
        let relative_path = path
            .strip_prefix(dir)
            .map_err(|e| format!("strip_prefix failed: {e}"))?;
        if !include(relative_path) {
            continue;
        }
        let relative = relative_path.to_string_lossy().to_string();
        let meta = fs::metadata(path)
            .map_err(|e| format!("metadata failed for {}: {e}", path.display()))?;
        let mtime = meta
            .modified()
            .map_err(|e| format!("mtime failed for {}: {e}", path.display()))?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let size = meta.len();
        entries.push((relative, mtime, size));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha256::new();
    for (name, mtime, size) in &entries {
        hasher.update(name.as_bytes());
        hasher.update(b"\0");
        hasher.update(mtime.to_le_bytes());
        hasher.update(size.to_le_bytes());
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    /// A manifest recording `content` at `rel` as the pipeline wrote it.
    fn manifest_for(generated: &Path, rel: &str, content: &str) -> CacheManifest {
        let path = generated.join(rel);
        fs::write(&path, content).unwrap();
        let mut manifest = CacheManifest::default();
        manifest
            .output_hashes
            .insert(rel.to_string(), string_hash(content));
        manifest
            .output_stats
            .insert(rel.to_string(), file_stat(&path).unwrap());
        manifest
    }

    /// Rewrite `path` with `content` and a later mtime, as an external tool
    /// would, whatever the filesystem's timestamp granularity.
    fn rewrite_later(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
        let later = SystemTime::now() + Duration::from_secs(5);
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(later)
            .unwrap();
    }

    #[test]
    fn outputs_intact_trusts_a_file_whose_stats_have_not_moved() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = manifest_for(dir.path(), "ledger.transactions", "2025-01-01 * \"a\"\n");
        assert!(outputs_intact(dir.path(), &manifest));
    }

    #[test]
    fn outputs_intact_rehashes_a_moved_file_and_accepts_the_same_content() {
        let dir = tempfile::tempdir().unwrap();
        let content = "2025-01-01 * \"a\"\n";
        let manifest = manifest_for(dir.path(), "ledger.transactions", content);
        rewrite_later(&dir.path().join("ledger.transactions"), content);
        assert!(outputs_intact(dir.path(), &manifest));
    }

    #[test]
    fn outputs_intact_detects_a_moved_file_with_other_content() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = manifest_for(dir.path(), "ledger.transactions", "2025-01-01 * \"a\"\n");
        // Same length as the original: only the mtime and the bytes differ.
        rewrite_later(&dir.path().join("ledger.transactions"), "2025-01-01 * \"b\"\n");
        assert!(!outputs_intact(dir.path(), &manifest));
    }

    #[test]
    fn outputs_intact_rehashes_when_no_stat_was_recorded() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = manifest_for(dir.path(), "ledger.transactions", "x\n");
        manifest.output_stats.clear();
        assert!(outputs_intact(dir.path(), &manifest));
        fs::write(dir.path().join("ledger.transactions"), "y\n").unwrap();
        assert!(!outputs_intact(dir.path(), &manifest));
    }

    #[test]
    fn outputs_intact_fails_on_a_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = manifest_for(dir.path(), "ledger.transactions", "x\n");
        fs::remove_file(dir.path().join("ledger.transactions")).unwrap();
        assert!(!outputs_intact(dir.path(), &manifest));
    }

    #[test]
    fn complete_output_stats_records_missing_stats_and_writes_the_manifest_back() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = manifest_for(dir.path(), "ledger.transactions", "x\n");
        manifest.output_stats.clear();
        complete_output_stats(dir.path(), &mut manifest).unwrap();
        let expected = file_stat(&dir.path().join("ledger.transactions")).unwrap();
        assert_eq!(manifest.output_stats.get("ledger.transactions"), Some(&expected));
        let reloaded = load_manifest(dir.path()).unwrap();
        assert_eq!(reloaded.output_stats.get("ledger.transactions"), Some(&expected));
    }

    #[test]
    fn load_manifest_is_none_without_a_manifest_and_reads_a_saved_one() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_manifest(dir.path()).is_none());
        fs::write(dir.path().join("out.txt"), "out\n").unwrap();
        let cache = BuildCache {
            inputs_hash: Some("fp".to_string()),
            output_hashes: HashMap::from([("out.txt".to_string(), string_hash("out\n"))]),
            ..Default::default()
        };
        save_cache(dir.path(), &cache, None).unwrap();
        let manifest = load_manifest(dir.path()).unwrap();
        assert_eq!(manifest.inputs_hash.as_deref(), Some("fp"));
        assert_eq!(
            manifest.output_stats.get("out.txt"),
            file_stat(&dir.path().join("out.txt")).as_ref()
        );
    }
}
