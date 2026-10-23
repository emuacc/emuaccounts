use crate::build_cache::{
    dir_fingerprint, file_hash, load_cache, load_cache_filtered, lookup, save_cache,
    sources_fingerprint, string_hash, BuildCache, CacheEntry, CachedTransaction,
};
use crate::csv_transform::{
    apply_labels_and_rules, apply_rules_with_accounts, resolve_transform,
    transform_csv_with_default,
};
use crate::generated_store::{
    append_text, normalize_blank_lines, posting_to_text, quote, ManualTransactionInput,
};
use crate::ledger_parser::{
    parse_prices, parse_prices_csv, parse_transactions, price_file_stem, AccountBalance,
    AccountProperties, CommodityAmount, ParseResult, Posting, PriceDirective, Transaction,
};
use crate::ofx_parser::{ofx_to_transactions_with_default, parse_ofx};
use crate::rules::{generate_rule_id, LabelsFile, Rule, RulesFile};
use crate::FALLBACK_ASSET_ACCOUNT;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Process-global lock: at most one `run_pipeline` call executes at a time.
/// Concurrent callers queue and run serially, preventing races on
/// `generated/` ledger files and build-cache mtimes.
static PIPELINE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn pipeline_lock() -> &'static Mutex<()> {
    PIPELINE_LOCK.get_or_init(|| Mutex::new(()))
}

const FILE_LEDGER: &str = "ledger.transactions";
const FILE_MANUAL: &str = "manual.transactions";
const FILE_ACCOUNTS: &str = "accounts.transactions";
const FILE_METADATA: &str = "pipeline-metadata.json";
const FILE_TRANSFORM: &str = "_transform.rhai";
const FILE_IGNORED: &str = "_ignored.txt";
const DIR_ARCHIVE: &str = "archive";
const DIR_IMPORTS: &str = "imports";
const DIR_PRICES: &str = "_prices";
const FILE_SUMMARY: &str = "summary.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderSummary {
    pub transaction_count: usize,
    pub balances: Vec<AccountBalance>,
}

pub fn validate_folder_name(folder: &str) -> Result<(), String> {
    if folder.contains("..") || folder.starts_with('/') || folder.starts_with('\\') {
        return Err(format!("invalid folder name: {folder}"));
    }
    Ok(())
}

pub struct PipelineConfig {
    pub sources_dir: PathBuf,
    pub generated_dir: PathBuf,
    pub now_yyyymm: String,
    /// When true, skip all cache layers and force a full rebuild.
    pub force: bool,
    /// Default expense account used when no rule matches a transaction.
    pub default_expense_account: String,
    /// When set, only these source folders are considered changed.
    /// The pipeline skips global fingerprinting and only processes these folders,
    /// loading cached data for everything else.
    pub changed_folder_hint: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineResult {
    pub csv_transformed: usize,
    pub csv_cached: usize,
    pub ofx_transformed: usize,
    pub ofx_cached: usize,
    pub manual_count: usize,
    pub total_written: usize,
    pub warnings: Vec<String>,
    pub owner_accounts: HashMap<String, Vec<String>>,
    pub account_folders: AccountFolders,
    pub account_properties: HashMap<String, AccountProperties>,
    /// True when the pipeline detected no changes and returned cached metadata.
    #[serde(default)]
    pub early_exit: bool,
    /// Number of output files actually written to disk (content changed).
    #[serde(default)]
    pub output_files_written: usize,
    /// Number of output files skipped because content hash matched.
    #[serde(default)]
    pub output_files_skipped: usize,
    /// Source folders whose ledger files were rewritten this run, sorted.
    #[serde(default)]
    pub changed_folders: Vec<String>,
    /// In-memory data for constructing ParseResult without re-reading from disk.
    /// Boxed to minimize PipelineResult stack footprint (cucumber async state machines are stack-sensitive).
    #[serde(skip)]
    pub in_memory: Option<Box<PipelineInMemory>>,
}

/// Heap-allocated pipeline data used to construct ParseResult without disk re-read.
#[derive(Debug, Clone)]
pub struct PipelineInMemory {
    /// (account_set, folder_rel, transaction). `folder_rel` is `None` for
    /// root-level inputs (manuals/CSVs/OFX directly under sources/) which
    /// exist only in memory and are never written to a per-folder ledger.
    pub tagged_txns: Vec<(Option<String>, Option<String>, Transaction)>,
    pub accounts_text_by_set: HashMap<String, String>,
}

impl PipelineResult {
    /// Build a ParseResult from in-memory pipeline data for a specific account set,
    /// avoiding re-reading and re-parsing the generated ledger from disk.
    /// Returns None if in_memory data is absent (e.g. early exit), in which case
    /// the caller should fall back to load_active_ledger.
    pub fn parse_result_for_set(&self, account_set: &str) -> Option<ParseResult> {
        let mem = self.in_memory.as_ref()?;

        let set_filter: Option<&str> = if account_set.is_empty() {
            None
        } else {
            Some(account_set)
        };
        let transactions: Vec<Transaction> = mem
            .tagged_txns
            .iter()
            .filter(|(set, _, _)| set.as_deref() == set_filter)
            .map(|(_, _, txn)| txn.clone())
            .collect();

        // Parse just the small accounts text for opening balances and account declarations
        let accounts_text = mem
            .accounts_text_by_set
            .get(account_set)
            .cloned()
            .unwrap_or_default();
        let accounts_result = parse_transactions(&accounts_text);

        // Compute balances from transactions + account declaration openings
        let mut balances_by_account: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
        for txn in &transactions {
            for posting in &txn.postings {
                let entry = balances_by_account
                    .entry(posting.account.clone())
                    .or_default()
                    .entry(posting.commodity.clone())
                    .or_insert(0.0);
                *entry += posting.amount;
            }
        }
        // Merge account declaration balances (opening balances, declared accounts)
        for ab in &accounts_result.balances {
            for ca in &ab.totals {
                let entry = balances_by_account
                    .entry(ab.account.clone())
                    .or_default()
                    .entry(ca.commodity.clone())
                    .or_insert(0.0);
                *entry += ca.amount;
            }
        }
        // Ensure declared accounts with no transactions still appear
        for ab in &accounts_result.balances {
            let _ = balances_by_account.entry(ab.account.clone()).or_default();
        }

        let balances: Vec<AccountBalance> = balances_by_account
            .into_iter()
            .map(|(account, by_commodity)| AccountBalance {
                account,
                totals: by_commodity
                    .into_iter()
                    .map(|(commodity, amount)| CommodityAmount { commodity, amount })
                    .collect(),
            })
            .collect();

        Some(ParseResult {
            ok: true,
            diagnostics: Vec::new(),
            transactions,
            balances,
            opening_balances: accounts_result.opening_balances,
            accounts_with_opening: accounts_result.accounts_with_opening,
            account_properties: self.account_properties.clone(),
        })
    }
}

/// The source folder behind each folder-backed account, by account set:
/// `set → account → folder` (relative to `sources/`). Keyed by set because
/// [`folder_to_account_name`] drops the entity segment, so `alex/cash/bank/cba`
/// and `acme/cash/bank/cba` both derive `assets:cash:bank:cba` and a flat map
/// kept only the folder processed last (TASK-107).
pub type AccountFolders = HashMap<String, HashMap<String, String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetadata {
    pub owner_accounts: HashMap<String, Vec<String>>,
    pub account_folders: AccountFolders,
    #[serde(default)]
    pub account_properties: HashMap<String, AccountProperties>,
    /// Accounts whose declarations carry an `opening` line. The frontend's
    /// "Unverified balance" hint reads this — the slim startup/mutation paths
    /// never ship the full parse that used to carry it.
    #[serde(default)]
    pub accounts_with_opening: Vec<String>,
}

impl PipelineMetadata {
    /// Every account a source folder backs, across all sets: the reports'
    /// primary-account allowlist.
    pub fn folder_backed_accounts(&self) -> HashSet<String> {
        self.account_folders
            .values()
            .flat_map(|by_account| by_account.keys().cloned())
            .collect()
    }

    /// Every source folder the pipeline writes a ledger for, across all sets.
    pub fn written_folders(&self) -> impl Iterator<Item = &str> {
        self.account_folders
            .values()
            .flat_map(|by_account| by_account.values().map(String::as_str))
    }

    pub fn load(generated_dir: &Path) -> Option<PipelineMetadata> {
        let path = generated_dir.join(FILE_METADATA);
        if !path.exists() {
            return None;
        }
        let contents = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    fn save(&self, generated_dir: &Path) -> Result<(), String> {
        let path = generated_dir.join(FILE_METADATA);
        let json = crate::to_sorted_json_pretty(self)
            .map_err(|e| format!("failed to serialize pipeline metadata: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("failed to write pipeline metadata: {e}"))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub files_processed: Vec<String>,
    pub files_skipped: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountGap {
    pub account: String,
    pub first_month: String,
    pub last_month: String,
    pub missing_months: Vec<String>,
}

fn generated_archive_dir(base_dir: &Path) -> PathBuf {
    base_dir.join(DIR_ARCHIVE)
}

fn yyyymm_from_date(date: &str) -> Option<String> {
    if date.len() < 7 {
        return None;
    }
    let year = &date[0..4];
    let month = &date[5..7];
    if year.chars().all(|c| c.is_ascii_digit()) && month.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("{year}{month}"))
    } else {
        None
    }
}

pub fn transaction_to_text(txn: &Transaction) -> String {
    let meta = txn.meta.as_deref().unwrap_or("");

    let mut header = txn.datetime.clone();
    header.push(' ');

    if let Some(status) = txn.status {
        header.push(status);
        header.push(' ');
    }

    // Write effective payee (display if set, otherwise raw source)
    let effective_payee = txn.display_payee.as_deref().or(txn.payee.as_deref());
    if let Some(payee) = effective_payee {
        header.push_str(&quote(payee));
        header.push(' ');
    } else if txn.narration.is_some() {
        // Write empty payee to preserve narration position in the two-string format
        header.push_str(&quote(""));
        header.push(' ');
    }
    if let Some(narration) = txn.narration.as_deref() {
        header.push_str(&quote(narration));
        header.push(' ');
    }

    header.push_str("; ");
    header.push_str(meta);
    // Persist source values in meta when display transforms were applied
    if txn.display_payee.is_some() {
        if let Some(ref raw) = txn.payee {
            if !meta.is_empty() {
                header.push_str(", ");
            }
            header.push_str("source_payee:");
            header.push_str(raw);
        }
    }
    if txn.display_amount_commodity.is_some() {
        if !header.ends_with("; ") {
            header.push_str(", ");
        }
        header.push_str("source_commodity:");
        header.push_str(&txn.amount_commodity);
    }

    let mut lines = Vec::with_capacity(1 + txn.postings.len() + 1);
    lines.push(header);
    for posting in &txn.postings {
        lines.push(posting_to_text(posting));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn scan_csv_files(sources_dir: &Path) -> Result<Vec<PathBuf>, String> {
    scan_source_files(sources_dir, "csv", true)
}

fn scan_ofx_files(sources_dir: &Path) -> Result<Vec<PathBuf>, String> {
    scan_source_files(sources_dir, "ofx", false)
}

fn scan_source_files(
    sources_dir: &Path,
    extension: &str,
    skip_underscore_prefix: bool,
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    scan_source_files_recursive(sources_dir, extension, skip_underscore_prefix, &mut files)?;
    files.sort();
    Ok(files)
}

fn scan_source_files_recursive(
    dir: &Path,
    extension: &str,
    skip_underscore_prefix: bool,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let entries =
        fs::read_dir(dir).map_err(|e| format!("failed to read dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str());
            if name == Some(DIR_IMPORTS) {
                continue;
            }
            scan_source_files_recursive(&path, extension, skip_underscore_prefix, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            if skip_underscore_prefix {
                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if fname.starts_with('_') {
                    continue;
                }
            }
            out.push(path);
        }
    }
    Ok(())
}

/// Collect all leaf-level source folders that contain data files (CSV/OFX).
/// Returns relative paths from sources_dir, e.g. ["alex/savings", "alex/bitcoin"].
fn collect_source_folders(sources_dir: &Path) -> Result<Vec<String>, String> {
    let mut folders = std::collections::BTreeSet::new();
    for entry in walkdir::WalkDir::new(sources_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_manual = name == FILE_MANUAL;
        if ext != "csv" && ext != "ofx" && !is_manual {
            continue;
        }
        // Skip files in imports/ subdirectories and prices/ folder
        if path.components().any(|c| c.as_os_str() == DIR_IMPORTS) {
            continue;
        }
        // Skip underscore-prefixed files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('_') {
                continue;
            }
        }
        if let Some(parent) = path.parent() {
            if let Ok(rel) = parent.strip_prefix(sources_dir) {
                let rel_str = rel.to_string_lossy().to_string();
                if !rel_str.is_empty() {
                    folders.insert(rel_str);
                }
            }
        }
    }
    Ok(folders.into_iter().collect())
}

/// Extract the account set (entity) from a relative folder path.
/// E.g. "alex-savings/2025-01.csv" → Some("alex")
/// E.g. "alex/CBA/savings/2025-01.csv" → Some("alex")
pub fn extract_account_set(relative_path: &str) -> Option<String> {
    let first_segment = relative_path.split('/').next()?;
    // If segment contains '-', the account set is the part before the first '-'
    if let Some(pos) = first_segment.find('-') {
        Some(first_segment[..pos].to_string())
    } else {
        Some(first_segment.to_string())
    }
}

/// `folder_rel` matches `hint` when it is the exact hinted folder or one of its
/// descendant source folders. Callers like `save_trade_link` pass the directory
/// that owns `_rules.json`; source folders are always leaves, so exact match
/// alone would miss every child wallet under a parent rules folder.
fn hint_matches(hint: &std::collections::HashSet<String>, folder_rel: &str) -> bool {
    hint.iter()
        .any(|h| folder_rel == h || folder_rel.starts_with(&format!("{h}/")))
}

/// Append `tag` (a complete `key:value` segment) to `meta` if it isn't already
/// present, preserving the comma-separated convention.
fn append_unique_meta_tag(meta: &mut Option<String>, tag: &str) {
    let already_has = meta
        .as_deref()
        .map(|m| m.split(',').any(|p| p.trim() == tag))
        .unwrap_or(false);
    if already_has {
        return;
    }
    match meta {
        Some(existing) if !existing.is_empty() => {
            existing.push_str(", ");
            existing.push_str(tag);
        }
        _ => *meta = Some(tag.to_string()),
    }
}

/// True for commodities that act as a price denominator in swaps: the base
/// currency itself, plus stablecoins that peg ~1:1 to the base. When a swap
/// has the base-like side on one leg and a "real" commodity on the other,
/// the base-like side's value (looked up via the price graph) is authoritative
/// and the other side gets priced from it. Used by both passes of
/// `auto_link_equity_swaps`.
fn is_base_like(c: &str, base: &str) -> bool {
    c == base
        || matches!(
            c,
            "USD" | "USDC" | "USDT" | "DAI" | "BUSD" | "FDUSD" | "TUSD"
        )
}

/// The base-currency value of one side of a swap, or `None` when that value
/// is zero. A zero (a 0-amount contract-interaction leg, a dead token's price
/// feed) prices nothing: stamped as `@ 0`, it would give the CGT report a
/// zero cost base or zero proceeds. Used by every pricing pass of
/// `auto_link_equity_swaps`.
fn swap_side_value(
    pg: &crate::ledger_parser::PriceGraph,
    commodity: &str,
    amount: f64,
    datetime: &str,
    base: &str,
) -> Option<f64> {
    pg.convert_to_base(commodity, amount, datetime, base)
        .filter(|value| *value > 0.0)
}

/// The `assets:` posting of a leg: the side of the swap the leg moves.
fn asset_posting(txn: &Transaction) -> Option<&Posting> {
    txn.postings
        .iter()
        .find(|p| p.account.starts_with("assets:"))
}

/// The value of the first `prefix…` segment of a comma-separated meta string:
/// `swap:` on `txn:abc, swap:txn:def` is `txn:def`.
fn meta_tag_value<'a>(meta: &'a Option<String>, prefix: &str) -> Option<&'a str> {
    meta.as_deref()?
        .split(',')
        .map(str::trim)
        .find_map(|segment| segment.strip_prefix(prefix))
}

/// Resolve the `swap:txn:HASH` reference on leg `i` to its partner leg.
///
/// Several ledger legs share one on-chain hash — the sell leg, the buy leg
/// and any 0-amount contract-interaction sibling — so the hash alone is
/// ambiguous. The partner is the first other leg under that hash whose asset
/// posting moves a non-zero amount of the stamped `swap_partner_commodity`,
/// or, when the leg carries no such tag, of any commodity but its own.
fn resolve_swap_ref(
    i: usize,
    tagged_txns: &[(Option<String>, Option<String>, Transaction)],
    txn_id_index: &HashMap<String, Vec<usize>>,
) -> Option<usize> {
    let txn = &tagged_txns[i].2;
    let own = asset_posting(txn)?;
    let partner_commodity = meta_tag_value(&txn.meta, "swap_partner_commodity:");
    let legs = txn_id_index.get(meta_tag_value(&txn.meta, "swap:")?)?;
    legs.iter().copied().find(|&j| {
        j != i
            && asset_posting(&tagged_txns[j].2).is_some_and(|p| {
                p.amount.abs() > 1e-9
                    && match partner_commodity {
                        Some(c) => p.commodity == c,
                        None => p.commodity != own.commodity,
                    }
            })
    })
}

/// A plain `equity:trading` leg of one same-second group: its index in
/// `tagged_txns`, the commodity its asset posting moves and the absolute
/// amount moved.
struct SwapLeg {
    idx: usize,
    commodity: String,
    amount: f64,
}

/// Pair the sells of one group with its buys, each pair in two different
/// commodities, linking as many legs as possible. Returns `(sell, buy)`
/// index pairs in sell rank order.
///
/// Both sides are ranked by (commodity, amount, index) and each sell in rank
/// order takes the first free buy of another commodity, so a multi-fill of
/// one pair (n BTC sells for n ETH buys) pairs smallest with smallest and the
/// result is the same on every run. A sell whose compatible buys are all
/// taken moves an earlier sell on to another buy it is compatible with (an
/// augmenting path): a two-hop convert (sell BTC, buy USDT, sell USDT, buy
/// ETH) thereby links BTC with USDT and USDT with ETH instead of BTC with
/// ETH and the two USDT legs stranded, and opposite-direction fills at one
/// second link within each direction. Two legs of one commodity are never
/// paired.
fn pair_swap_legs(mut sells: Vec<SwapLeg>, mut buys: Vec<SwapLeg>) -> Vec<(usize, usize)> {
    /// Give sell `s` a buy, moving that buy's current sell elsewhere if it
    /// can be. `seen` holds the buys already tried on this sell's path.
    fn claim_buy(
        s: usize,
        sells: &[SwapLeg],
        buys: &[SwapLeg],
        sell_of_buy: &mut [Option<usize>],
        seen: &mut [bool],
    ) -> bool {
        let compatible: Vec<usize> = (0..buys.len())
            .filter(|&b| !seen[b] && buys[b].commodity != sells[s].commodity)
            .collect();
        if let Some(&b) = compatible.iter().find(|&&b| sell_of_buy[b].is_none()) {
            sell_of_buy[b] = Some(s);
            return true;
        }
        for b in compatible {
            if seen[b] {
                continue;
            }
            seen[b] = true;
            let Some(other) = sell_of_buy[b] else {
                continue;
            };
            if claim_buy(other, sells, buys, sell_of_buy, seen) {
                sell_of_buy[b] = Some(s);
                return true;
            }
        }
        false
    }

    let rank = |a: &SwapLeg, b: &SwapLeg| {
        a.commodity
            .cmp(&b.commodity)
            .then(a.amount.total_cmp(&b.amount))
            .then(a.idx.cmp(&b.idx))
    };
    sells.sort_by(rank);
    buys.sort_by(rank);
    let mut sell_of_buy: Vec<Option<usize>> = vec![None; buys.len()];
    for s in 0..sells.len() {
        let mut seen = vec![false; buys.len()];
        claim_buy(s, &sells, &buys, &mut sell_of_buy, &mut seen);
    }
    sells
        .iter()
        .enumerate()
        .filter_map(|(s, sell)| {
            let b = sell_of_buy.iter().position(|&taken| taken == Some(s))?;
            Some((sell.idx, buys[b].idx))
        })
        .collect()
}

/// Auto-detect and link equity:trading swap pairs, pricing every leg in
/// `base_currency`: for the transactions of one account set, whose base
/// currency the caller has resolved. A vault of several sets goes through
/// [`link_equity_swaps_per_set`].
pub fn auto_link_equity_swaps(
    tagged_txns: &mut [(Option<String>, Option<String>, Transaction)],
    price_graph: Option<&crate::ledger_parser::PriceGraph>,
    base_currency: Option<&str>,
) {
    link_equity_swaps_per_set(tagged_txns, price_graph, &|_| base_currency);
}

/// Auto-detect and link equity:trading swap pairs.
///
/// Finds transactions with plain `equity:trading` (no `:sell`/`:buy` suffix)
/// of one account set at the same datetime with the same narration and asset
/// account, where one side disposes a commodity and the other receives a
/// different commodity. Within a group the sells are paired with the buys by
/// `pair_swap_legs`, so a group holding more than two commodities (a two-hop
/// convert, fills in both directions at one second) links each leg with one
/// of another commodity. Applies `:sell`/`:buy` suffixes and adds
/// `swap:txn:xxx` cross-references so the CGT engine can precisely match the
/// two legs.
///
/// Legs never pair across account sets: a set is one entity's books, and two
/// sets may hold same-named accounts (`folder_to_account_name` drops the
/// entity). Each leg is priced in the base currency of its own set,
/// `base_currency_of(set)` (`None` for a transaction that belongs to no set);
/// a set without one keeps its legs unpriced.
pub fn link_equity_swaps_per_set<'b>(
    tagged_txns: &mut [(Option<String>, Option<String>, Transaction)],
    price_graph: Option<&crate::ledger_parser::PriceGraph>,
    base_currency_of: &dyn Fn(Option<&str>) -> Option<&'b str>,
) {
    // Fast-reject: if no transactions have any equity:trading postings, skip entirely
    let has_any_equity_trading = tagged_txns.iter().any(|(_, _, txn)| {
        txn.postings
            .iter()
            .any(|p| p.account.starts_with("equity:trading"))
    });
    if !has_any_equity_trading {
        return;
    }

    // Collect indices of transactions with plain equity:trading
    let mut candidates: Vec<usize> = Vec::new();
    for (i, (_, _, txn)) in tagged_txns.iter().enumerate() {
        let has_plain_eq = txn.postings.iter().any(|p| p.account == "equity:trading");
        if has_plain_eq {
            candidates.push(i);
        }
    }
    // Note: even if no plain equity:trading candidates, we still run the price
    // annotation pass below for pre-linked :sell/:buy pairs.

    // Group candidates by (account set, datetime, asset_account, narration)
    let mut groups: HashMap<(Option<String>, String, String, String), Vec<usize>> = HashMap::new();
    for &idx in &candidates {
        let (set, _, txn) = &tagged_txns[idx];
        let asset_account = txn
            .postings
            .iter()
            .find(|p| p.account.starts_with("assets:"))
            .map(|p| p.account.clone())
            .unwrap_or_default();
        let narration = txn.narration.clone().unwrap_or_default();
        let key = (set.clone(), txn.datetime.clone(), asset_account, narration);
        groups.entry(key).or_default().push(idx);
    }

    // Within each group, partition into sells and buys and pair them across commodities.
    // (index, new_account, swap_meta, optional price annotation)
    let mut mutations: Vec<(
        usize,
        String,
        String,
        Option<crate::ledger_parser::PriceAnnotation>,
    )> = Vec::new();

    for indices in groups.values() {
        let mut sells: Vec<SwapLeg> = Vec::new();
        let mut buys: Vec<SwapLeg> = Vec::new();
        for &idx in indices {
            let Some(ap) = asset_posting(&tagged_txns[idx].2) else {
                continue;
            };
            let leg = SwapLeg {
                idx,
                commodity: ap.commodity.clone(),
                amount: ap.amount.abs(),
            };
            if ap.amount < 0.0 {
                sells.push(leg);
            } else if ap.amount > 0.0 {
                buys.push(leg);
            }
        }

        for (sell_idx, buy_idx) in pair_swap_legs(sells, buys) {
            let sell_txn = &tagged_txns[sell_idx].2;
            let buy_txn = &tagged_txns[buy_idx].2;
            let (Some(s_id), Some(b_id)) = (
                extract_txn_id_from_meta(&sell_txn.meta),
                extract_txn_id_from_meta(&buy_txn.meta),
            ) else {
                continue;
            };
            // Both legs were partitioned on their asset posting, so both have one.
            let (Some(sp), Some(bp)) = (asset_posting(sell_txn), asset_posting(buy_txn)) else {
                continue;
            };

            // Derive price annotation from the denominator side of the swap
            let mut sell_price: Option<crate::ledger_parser::PriceAnnotation> = None;
            let mut buy_price: Option<crate::ledger_parser::PriceAnnotation> = None;

            let base_currency = base_currency_of(tagged_txns[sell_idx].0.as_deref());
            if let (Some(pg), Some(base)) = (price_graph, base_currency) {
                let sp_has_price = sp.price.is_some() || sp.cost.is_some();
                let bp_has_price = bp.price.is_some() || bp.cost.is_some();

                let s_is_denom = is_base_like(&sp.commodity, base);
                let b_is_denom = is_base_like(&bp.commodity, base);

                if b_is_denom && !sp_has_price && bp.amount.abs() > 1e-9 {
                    // Buy side is denominator — price sell side from buy's value
                    if let Some(buy_val) =
                        swap_side_value(pg, &bp.commodity, bp.amount.abs(), &buy_txn.datetime, base)
                    {
                        let price_per_unit = buy_val / sp.amount.abs();
                        sell_price = Some(crate::ledger_parser::PriceAnnotation {
                            is_total: false,
                            amount: price_per_unit,
                            amount_text: format!("{price_per_unit:.6}"),
                            commodity: base.to_string(),
                        });
                    }
                } else if s_is_denom && !bp_has_price && sp.amount.abs() > 1e-9 {
                    // Sell side is denominator — price buy side from sell's value
                    if let Some(sell_val) = swap_side_value(
                        pg,
                        &sp.commodity,
                        sp.amount.abs(),
                        &sell_txn.datetime,
                        base,
                    ) {
                        let price_per_unit = sell_val / bp.amount.abs();
                        buy_price = Some(crate::ledger_parser::PriceAnnotation {
                            is_total: false,
                            amount: price_per_unit,
                            amount_text: format!("{price_per_unit:.6}"),
                            commodity: base.to_string(),
                        });
                    }
                } else if !sp_has_price && !bp_has_price {
                    // Neither is a denominator — try to price whichever side we can
                    let sell_val = swap_side_value(
                        pg,
                        &sp.commodity,
                        sp.amount.abs(),
                        &sell_txn.datetime,
                        base,
                    );
                    let buy_val = swap_side_value(
                        pg,
                        &bp.commodity,
                        bp.amount.abs(),
                        &buy_txn.datetime,
                        base,
                    );
                    if let (Some(sv), None) = (sell_val, buy_val) {
                        let price_per_unit = sv / bp.amount.abs();
                        buy_price = Some(crate::ledger_parser::PriceAnnotation {
                            is_total: false,
                            amount: price_per_unit,
                            amount_text: format!("{price_per_unit:.6}"),
                            commodity: base.to_string(),
                        });
                    } else if let (None, Some(bv)) = (sell_val, buy_val) {
                        let price_per_unit = bv / sp.amount.abs();
                        sell_price = Some(crate::ledger_parser::PriceAnnotation {
                            is_total: false,
                            amount: price_per_unit,
                            amount_text: format!("{price_per_unit:.6}"),
                            commodity: base.to_string(),
                        });
                    }
                }
            }

            // Build the swap meta segments for each side. `swap:txn:HASH`
            // names the partner's on-chain hash; `swap_partner_commodity`
            // disambiguates which leg is the partner when several ledger
            // txns share that hash (e.g. on-chain swap with a 0-amount
            // contract-interaction sibling). Frontends look up the partner
            // by (hash, commodity) without heuristics.
            let sell_swap_meta = format!("swap:{b_id}, swap_partner_commodity:{}", bp.commodity);
            let buy_swap_meta = format!("swap:{s_id}, swap_partner_commodity:{}", sp.commodity);
            mutations.push((
                sell_idx,
                "equity:trading:sell".to_string(),
                sell_swap_meta,
                sell_price,
            ));
            mutations.push((
                buy_idx,
                "equity:trading:buy".to_string(),
                buy_swap_meta,
                buy_price,
            ));
        }
    }

    // Apply mutations
    for (idx, new_account, swap_meta, price) in mutations {
        let txn = &mut tagged_txns[idx].2;
        for posting in &mut txn.postings {
            if posting.account == "equity:trading" {
                posting.account = new_account.clone();
                break;
            }
        }
        // Add price annotation to the asset posting if derived from swap counterpart
        if let Some(price_ann) = price {
            for posting in &mut txn.postings {
                if posting.account.starts_with("assets:") && posting.price.is_none() {
                    posting.price = Some(price_ann.clone());
                    break;
                }
            }
        }
        if let Some(ref mut meta) = txn.meta {
            meta.push_str(", ");
            meta.push_str(&swap_meta);
        } else {
            txn.meta = Some(swap_meta);
        }
    }

    // Build indices for fast partner lookup.
    // txn_id_index: every leg carrying each `txn:` id, in ledger order. Legs of
    // one on-chain swap share a hash, so an explicit `swap:txn:xxx` reference
    // is resolved among them by `resolve_swap_ref`.
    // tagged_partner_map: for rule-tagged legs without a swap ref, pre-compute
    // each leg's partner by grouping peers with the same (account set, datetime,
    // asset_account),
    // splitting into sell/buy sides, sorting both by abs amount, and rank-pairing.
    // This handles multi-fill correctly and scopes to one account so independent
    // trades in other accounts never cross-contaminate. Narration is not part of
    // the key because post-rule legs of one swap can carry different narrations
    // (e.g. "Swap SOL for SKBDI" on the sell leg, "Receive SKBDI" on the buy).
    let mut txn_id_index: HashMap<String, Vec<usize>> = HashMap::new();
    let mut tagged_groups: HashMap<(Option<String>, String, String), Vec<usize>> = HashMap::new();
    for (i, (set, _, txn)) in tagged_txns.iter().enumerate() {
        if let Some(id) = extract_txn_id_from_meta(&txn.meta) {
            txn_id_index.entry(id).or_default().push(i);
        }
        let has_eq = txn
            .postings
            .iter()
            .any(|p| p.account == "equity:trading:sell" || p.account == "equity:trading:buy");
        if !has_eq {
            continue;
        }
        let asset_account = txn
            .postings
            .iter()
            .find(|p| p.account.starts_with("assets:"))
            .map(|p| p.account.clone())
            .unwrap_or_default();
        tagged_groups
            .entry((set.clone(), txn.datetime.clone(), asset_account))
            .or_default()
            .push(i);
    }

    let mut tagged_partner_map: HashMap<usize, usize> = HashMap::new();
    for indices in tagged_groups.values() {
        // Partition by (side, commodity) so a group containing pairs in
        // multiple commodity directions (e.g. Kraken at one block-second
        // with BTC↔ETH and ETH↔BTC swaps) doesn't rank-pair across
        // commodities. 0-amount no-op legs (contract-interaction siblings)
        // never qualify as a swap participant — drop them.
        let mut sells_by_commodity: HashMap<String, Vec<(usize, f64)>> = HashMap::new();
        let mut buys_by_commodity: HashMap<String, Vec<(usize, f64)>> = HashMap::new();
        for &idx in indices {
            let txn = &tagged_txns[idx].2;
            let is_sell = txn
                .postings
                .iter()
                .any(|p| p.account == "equity:trading:sell");
            let is_buy = txn
                .postings
                .iter()
                .any(|p| p.account == "equity:trading:buy");
            let asset_posting = txn
                .postings
                .iter()
                .find(|p| p.account.starts_with("assets:"));
            let Some(ap) = asset_posting else { continue };
            let abs_amount = ap.amount.abs();
            if abs_amount < 1e-9 {
                continue;
            }
            if is_sell {
                sells_by_commodity
                    .entry(ap.commodity.clone())
                    .or_default()
                    .push((idx, abs_amount));
            } else if is_buy {
                buys_by_commodity
                    .entry(ap.commodity.clone())
                    .or_default()
                    .push((idx, abs_amount));
            }
        }

        // For each (sell commodity, buy commodity) subset where they
        // differ, rank-pair by sorted absolute amount. Same-commodity
        // sell/buy is a self-transfer shape, not a swap — skip it so the
        // tagged_partner_map never points across same commodities.
        //
        // DETERMINISM: iterate commodity keys in sorted order, not HashMap
        // iteration order. When a single trade has multiple plausible
        // partners (e.g. a USDC sell that could pair with an AUD buy OR a
        // USDT buy at the same datetime/account), `tagged_partner_map.insert`
        // overwrites — so the partner that gets chosen is whichever (sc, bc)
        // pair the loop hits LAST. HashMap iteration order is randomized
        // per-process, so without sorted keys two cold regens of the same
        // sources pick different swap partners and emit different
        // `swap:txn:<id>` / `swap_partner_commodity:<commodity>` metadata
        // (and different auto-link price annotations downstream). This was
        // the production root cause of "SEI moves" / "Kraken trade swap
        // partners change run-to-run". See invariant test
        // `pipeline_is_deterministic_across_two_cold_runs`.
        // AGGREGATOR MULTI-LEG SWAPS: a single user-perceived swap routed
        // through Jupiter v6 / 1inch / similar emits multiple atomic transfers
        // — N sell legs + M buy legs all sharing one on-chain `txn:` id. If
        // we rank-paired by individual posting amount, only one sell would
        // get the swap meta and the others would either be orphaned (path 5
        // / market-price proceeds) or — via the count-mismatch fallback in
        // `find_equity_swap_sibling` — wrongly attributed the FULL partner
        // value as proceeds. Both ways the disposal value in the CGT report
        // is wrong: the total proceeds across the legs disagrees with what
        // the user actually received (the partner-side market value).
        //
        // Fix: aggregate same-`txn_id` postings within each commodity bucket
        // into one synthetic leg (total amount + member indices). Rank-pair
        // the *aggregates*, then propagate the swap meta + a pro-rata price
        // annotation to every constituent posting. After this:
        //   - every member of an aggregate carries `swap:txn:<partner-id>`
        //     so `find_equity_swap_sibling` resolves deterministically;
        //   - every member carries a `@ X AUD` price annotation, so
        //     `resolve_sale_proceeds` path 1 (price-on-posting) fires
        //     uniformly and the per-leg proceeds pro-rate by quantity.
        //
        // See invariant test `cgt_proceeds_for_multi_leg_swap_equals_partner_value`.

        // Group by txn_id within each commodity. Posting with no txn_id meta
        // gets a unique synthetic key so it stays its own singleton aggregate.
        let aggregate = |list: &[(usize, f64)],
                         tagged_txns: &[(Option<String>, Option<String>, Transaction)]|
         -> Vec<(f64, Vec<usize>)> {
            let mut by_txn_id: std::collections::BTreeMap<String, (f64, Vec<usize>)> =
                std::collections::BTreeMap::new();
            for &(idx, amount) in list {
                let txn = &tagged_txns[idx].2;
                let key = extract_txn_id_from_meta(&txn.meta)
                    .unwrap_or_else(|| format!("__anon_{idx}"));
                let entry = by_txn_id.entry(key).or_insert((0.0, Vec::new()));
                entry.0 += amount;
                entry.1.push(idx);
            }
            // Each member list is index-ordered for determinism.
            for (_, idxs) in by_txn_id.values_mut() {
                idxs.sort();
            }
            by_txn_id.into_values().collect()
        };

        let sells_aggregated: HashMap<String, Vec<(f64, Vec<usize>)>> = sells_by_commodity
            .iter()
            .map(|(c, v)| (c.clone(), aggregate(v, tagged_txns)))
            .collect();
        let buys_aggregated: HashMap<String, Vec<(f64, Vec<usize>)>> = buys_by_commodity
            .iter()
            .map(|(c, v)| (c.clone(), aggregate(v, tagged_txns)))
            .collect();

        let mut sell_commodities: Vec<&String> = sells_aggregated.keys().collect();
        sell_commodities.sort();
        for sc in sell_commodities {
            let sells = &sells_aggregated[sc];
            let mut buy_commodities: Vec<&String> = buys_aggregated.keys().collect();
            buy_commodities.sort();
            for bc in buy_commodities {
                if sc == bc {
                    continue;
                }
                let buys = &buys_aggregated[bc];
                let mut sa = sells.clone();
                let mut ba = buys.clone();
                // Sort aggregates by total amount, tie-break on the first
                // member index for determinism across cold regens.
                sa.sort_by(|a, b| {
                    a.0.partial_cmp(&b.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.1.first().cmp(&b.1.first()))
                });
                ba.sort_by(|a, b| {
                    a.0.partial_cmp(&b.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.1.first().cmp(&b.1.first()))
                });

                for (sell_agg, buy_agg) in sa.iter().zip(ba.iter()) {
                    if sell_agg.1.is_empty() || buy_agg.1.is_empty() {
                        continue;
                    }
                    let sell_first = sell_agg.1[0];
                    let buy_first = buy_agg.1[0];
                    // Every sell-side posting points at the buy aggregate's
                    // canonical (first) index — same for the buy side.
                    // The downstream meta stamping pass keys on
                    // `tagged_partner_map`, so all members of the aggregate
                    // get the same `swap:txn:<partner-id>` annotation.
                    for &idx in &sell_agg.1 {
                        tagged_partner_map.insert(idx, buy_first);
                    }
                    for &idx in &buy_agg.1 {
                        tagged_partner_map.insert(idx, sell_first);
                    }

                    // Per-aggregate price derivation. ATO position 116-20:
                    // disposal proceeds = market value of what was received.
                    // So for a USDC → HNT swap we price the USDC disposal at
                    // (HNT market value / USDC qty), NOT at USDC's own market
                    // price (which would yield a different number if the
                    // trade had slippage). Symmetrically the HNT acquisition
                    // takes its own market price.
                    //
                    // When exactly one side is base-like (a known cash value),
                    // anchor on the RECEIVED (buy) side and fall back to the
                    // disposed (sell) side only if the buy side has no price
                    // feed. This is symmetric: a stable→real BUY values at the
                    // real token received (its market price), and a real→stable
                    // SELL values at the stablecoin received (the ground-truth
                    // proceeds) — NOT the disposed token's own market price,
                    // which can be stale/illiquid and would otherwise inflate
                    // proceeds (e.g. selling 1158.86 PERP for 20.46 USDC must
                    // book ~the USDC value, not 1158.86 × a stale PERP quote).
                    let base_currency = base_currency_of(tagged_txns[sell_first].0.as_deref());
                    if let (Some(pg), Some(base)) = (price_graph, base_currency) {
                        let datetime = &tagged_txns[sell_first].2.datetime;
                        let sc_base = is_base_like(sc, base);
                        let bc_base = is_base_like(bc, base);

                        // Choose which aggregate's market value to anchor on.
                        let aud_value: Option<f64> = match (sc_base, bc_base) {
                            (true, false) | (false, true) => {
                                // Exactly one base-like side: anchor on the
                                // received (buy) side; fall back to the sell side
                                // only if the buy side has no price feed.
                                swap_side_value(pg, bc, buy_agg.0, datetime, base)
                                    .or_else(|| swap_side_value(pg, sc, sell_agg.0, datetime, base))
                            }
                            (false, false) => {
                                // Both real. Average sell and buy market values
                                // (if either is missing the other wins).
                                match (
                                    swap_side_value(pg, sc, sell_agg.0, datetime, base),
                                    swap_side_value(pg, bc, buy_agg.0, datetime, base),
                                ) {
                                    (Some(s), Some(b)) => Some((s + b) / 2.0),
                                    (Some(s), None) => Some(s),
                                    (None, Some(b)) => Some(b),
                                    (None, None) => None,
                                }
                            }
                            (true, true) => {
                                // Both base-like (USDC ↔ USDT etc.) — values are
                                // ~1:1 in the base currency, no need to override.
                                None
                            }
                        };

                        if let Some(aud_value) = aud_value {
                            // Per-unit price for each side = same AUD value
                            // divided by that side's total quantity. Stamp on
                            // every constituent asset posting that doesn't
                            // already have an explicit price.
                            if sell_agg.0 > 1e-12 {
                                let sell_per_unit = aud_value / sell_agg.0;
                                let txt = format!("{sell_per_unit:.6}");
                                for &idx in &sell_agg.1 {
                                    let txn = &mut tagged_txns[idx].2;
                                    for p in &mut txn.postings {
                                        if p.account.starts_with("assets:")
                                            && p.price.is_none()
                                        {
                                            p.price = Some(
                                                crate::ledger_parser::PriceAnnotation {
                                                    is_total: false,
                                                    amount: sell_per_unit,
                                                    amount_text: txt.clone(),
                                                    commodity: base.to_string(),
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                            if buy_agg.0 > 1e-12 {
                                let buy_per_unit = aud_value / buy_agg.0;
                                let txt = format!("{buy_per_unit:.6}");
                                for &idx in &buy_agg.1 {
                                    let txn = &mut tagged_txns[idx].2;
                                    for p in &mut txn.postings {
                                        if p.account.starts_with("assets:")
                                            && p.price.is_none()
                                        {
                                            p.price = Some(
                                                crate::ledger_parser::PriceAnnotation {
                                                    is_total: false,
                                                    amount: buy_per_unit,
                                                    amount_text: txt.clone(),
                                                    commodity: base.to_string(),
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Meta stamping pass: ensure every paired txn carries `swap:txn:HASH`
    // (partner's on-chain hash) and `swap_partner_commodity:COMMODITY`
    // (partner's primary commodity). The pair is the single source of truth
    // frontends use to render the swap row — no heuristic partner search.
    // Idempotent: skips tags already present (so re-runs and pre-suffixed
    // saved-link pairs converge to the same state).
    {
        let partner_info: Vec<(usize, Option<String>, Option<String>)> = tagged_partner_map
            .iter()
            .map(|(&idx, &partner_idx)| {
                let partner_txn = &tagged_txns[partner_idx].2;
                let partner_id = extract_txn_id_from_meta(&partner_txn.meta);
                let partner_commodity = partner_txn
                    .postings
                    .iter()
                    .find(|p| p.account.starts_with("assets:"))
                    .map(|p| p.commodity.clone());
                (idx, partner_id, partner_commodity)
            })
            .collect();
        for (idx, partner_id, partner_commodity) in partner_info {
            let meta = &mut tagged_txns[idx].2.meta;
            if let Some(pid) = partner_id {
                append_unique_meta_tag(meta, &format!("swap:{pid}"));
            }
            if let Some(pc) = partner_commodity {
                append_unique_meta_tag(meta, &format!("swap_partner_commodity:{pc}"));
            }
        }
    }

    // Price annotation pass: for any transaction with a swap:txn:xxx reference,
    // look up the partner and derive a price if one side can be priced and the other can't.
    if let Some(pg) = price_graph {
        // Find transactions with swap refs that need pricing
        let mut price_mutations: Vec<(usize, crate::ledger_parser::PriceAnnotation)> = Vec::new();
        for (i, (set, _, txn)) in tagged_txns.iter().enumerate() {
            // Skip if asset posting already has a price/cost
            let asset = match asset_posting(txn) {
                Some(p) if p.price.is_none() && p.cost.is_none() => p,
                _ => continue,
            };
            let Some(base) = base_currency_of(set.as_deref()) else {
                continue;
            };

            // Partner lookup: the explicit `swap:txn:HASH` reference takes
            // priority, resolved among the legs sharing that hash; otherwise
            // use the pre-computed rank-paired map.
            let Some(partner) = resolve_swap_ref(i, tagged_txns, &txn_id_index)
                .or_else(|| tagged_partner_map.get(&i).copied())
                .map(|idx| &tagged_txns[idx].2)
            else {
                continue;
            };

            if let Some(pa) = asset_posting(partner) {
                if pa.commodity == asset.commodity {
                    continue;
                }

                // Determine which side is the denominator and derive price
                let a_denom = is_base_like(&asset.commodity, base);
                let p_denom = is_base_like(&pa.commodity, base);

                let partner_val = if p_denom || !a_denom {
                    swap_side_value(
                        pg,
                        &pa.commodity,
                        pa.amount.abs(),
                        &partner.datetime,
                        base,
                    )
                } else {
                    None
                };

                if let Some(pv) = partner_val {
                    if asset.amount.abs() > 1e-9 {
                        let ppu = pv / asset.amount.abs();
                        price_mutations.push((
                            i,
                            crate::ledger_parser::PriceAnnotation {
                                is_total: false,
                                amount: ppu,
                                amount_text: format!("{ppu:.6}"),
                                commodity: base.to_string(),
                            },
                        ));
                    }
                }
            }
        }

        for (idx, price_ann) in price_mutations {
            let txn = &mut tagged_txns[idx].2;
            for posting in &mut txn.postings {
                if posting.account.starts_with("assets:")
                    && posting.price.is_none()
                    && posting.cost.is_none()
                {
                    posting.price = Some(price_ann.clone());
                    break;
                }
            }
        }
    }
}

fn extract_txn_id_from_meta(meta: &Option<String>) -> Option<String> {
    meta.as_ref().and_then(|m| {
        m.split(',')
            .map(|p| p.trim())
            .find(|p| p.starts_with("txn:"))
            .map(String::from)
    })
}

/// Account roots that may back a source folder. A folder directly under the
/// entity named after one of these is that root's explicit-layout marker
/// (`<entity>/<root>/<rest>` ⇒ `<root>:<rest>`); every other folder derives
/// under `assets`. Deliberately ONLY these two: wider root sets collide with
/// legacy shorthand folders (e.g. alex/equity/broker/… is
/// assets:equity:broker:…), and income/expenses/equity accounts are never
/// folder-backed — they live in `_accounts`.
pub const FOLDER_ROOTS: [&str; 2] = ["assets", "liabilities"];

/// True for accounts under a folder-backed root (`assets:…`, `liabilities:…`).
pub fn under_folder_root(account: &str) -> bool {
    FOLDER_ROOTS
        .iter()
        .any(|root| account.strip_prefix(root).is_some_and(|rest| rest.starts_with(':')))
}

/// The first segment of an account name (`liabilities:credit-card:amex` ⇒
/// `liabilities`).
fn account_root(account: &str) -> &str {
    account.split(':').next().unwrap_or(account)
}

/// Derive an account name from a folder path relative to sources_dir.
/// E.g. "alex/cash/bank/cba/savings" → "assets:cash:bank:cba:savings"
/// E.g. "alex/liabilities/credit-card/amex" → "liabilities:credit-card:amex"
/// E.g. "alex-savings" → "assets:savings"
/// Only a second path segment equal to one of [`FOLDER_ROOTS`] switches the
/// root; the bare `<entity>/<root>` folder derives to the root marker itself,
/// which is never an account of its own.
pub fn folder_to_account_name(folder_relative: &str) -> String {
    let parts: Vec<&str> = folder_relative.split('/').collect();
    if parts.len() >= 2 && FOLDER_ROOTS.contains(&parts[1]) {
        // Explicit layout: <entity>/<root>/<rest> mirrors the account name.
        let root = parts[1];
        if parts.len() == 2 {
            root.to_string()
        } else {
            format!("{root}:{}", parts[2..].join(":"))
        }
    } else if parts.len() >= 2 {
        // Multi-level: skip first segment (owner), rest becomes account hierarchy
        let account_parts = &parts[1..];
        format!("assets:{}", account_parts.join(":"))
    } else {
        // Single-level: strip owner prefix (e.g. "alex-savings" → "savings")
        let segment = parts[0];
        if let Some(pos) = segment.find('-') {
            let rest = &segment[pos + 1..];
            format!("assets:{rest}")
        } else {
            format!("assets:{segment}")
        }
    }
}

/// Gather rules from `start` folder up to `sources_dir`, merging all `_rules.json` files.
/// Rules from more specific (closer) folders come first and have higher priority.
fn gather_rules_chain(start: &Path, sources_dir: &Path) -> RulesFile {
    let mut all_rules = Vec::new();
    let mut current = start;
    loop {
        let folder_rules = RulesFile::load(current);
        all_rules.extend(folder_rules.rules);
        if current == sources_dir {
            break;
        }
        match current.parent() {
            Some(p) if p.starts_with(sources_dir) || p == sources_dir => current = p,
            _ => break,
        }
    }
    RulesFile { rules: all_rules }
}

/// Gather `_labels.json` files from `start` up to `sources_dir`, closest
/// folder first so per-folder labels beat parent-folder ones (labels apply
/// first-match, mirroring the CSV transform's chain in `csv_transform.rs`).
fn gather_labels_chain(start: &Path, sources_dir: &Path) -> LabelsFile {
    let mut all_labels = Vec::new();
    let mut current = start;
    loop {
        let folder_labels = LabelsFile::load(current);
        all_labels.extend(folder_labels.labels);
        if current == sources_dir {
            break;
        }
        match current.parent() {
            Some(p) if p.starts_with(sources_dir) || p == sources_dir => current = p,
            _ => break,
        }
    }
    LabelsFile { labels: all_labels }
}

/// Parse all manual.transactions files found in source folders (recursive).
/// Returns tuples of (account_set, folder_rel, transaction).
/// Uses cache to skip re-parsing unchanged files.
fn parse_all_manual_transactions(
    sources_dir: &Path,
    cache: &mut BuildCache,
) -> Result<Vec<(Option<String>, String, Transaction)>, String> {
    let mut all_txns = Vec::new();
    let mut seen_folders = std::collections::HashSet::new();

    for entry in walkdir::WalkDir::new(sources_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) != Some(FILE_MANUAL) {
            continue;
        }
        // Derive account_set from the parent folder's relative path
        let folder_rel = path
            .parent()
            .and_then(|p| p.strip_prefix(sources_dir).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let account_set = if folder_rel.is_empty() {
            None
        } else {
            extract_account_set(&folder_rel)
        };
        let rel_path = path
            .strip_prefix(sources_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        seen_folders.insert(rel_path.clone());

        // Build combined hash: manual.transactions content + _rules.json and
        // _labels.json from this folder and all parents up to sources_dir.
        // This ensures rule/label changes at any level invalidate the manual
        // transaction cache.
        let folder = path.parent().unwrap_or(Path::new(""));
        let content_h = file_hash(path)?;
        let mut rules_hash_parts = Vec::new();
        {
            let mut walk = folder;
            loop {
                for chain_file in ["_rules.json", "_labels.json"] {
                    let rp = walk.join(chain_file);
                    if rp.exists() {
                        rules_hash_parts.push(file_hash(&rp)?);
                    }
                }
                if walk == sources_dir {
                    break;
                }
                match walk.parent() {
                    Some(p) if p.starts_with(sources_dir) || p == sources_dir => walk = p,
                    _ => break,
                }
            }
        }
        let combined_h = if rules_hash_parts.is_empty() {
            string_hash(&format!("{content_h}:m={}", crate::rules::MATCHER_VERSION))
        } else {
            string_hash(&format!(
                "{content_h}:{}:m={}",
                rules_hash_parts.join(":"),
                crate::rules::MATCHER_VERSION
            ))
        };
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let rules = gather_rules_chain(folder, sources_dir);
        let labels = gather_labels_chain(folder, sources_dir);

        let mut txns = parse_transactions(&contents).transactions;
        stamp_manual_txn_ids(&rel_path, &mut txns);
        for mut txn in txns {
            apply_labels_and_rules(&mut txn, &labels, &rules);
            all_txns.push((account_set.clone(), folder_rel.clone(), txn));
        }

        cache.manual_hashes.insert(rel_path.clone(), combined_h);
    }

    cache.manual_hashes.retain(|k, _| seen_folders.contains(k));

    Ok(all_txns)
}

/// Load the legacy `_ignored.txt` ids from the sources root, with cache.
fn load_ignored_ids(sources_dir: &Path, cache: &mut BuildCache) -> HashSet<String> {
    let ignored_path = sources_dir.join(FILE_IGNORED);
    if !ignored_path.exists() {
        cache.ignored_hash = None;
        cache.ignored_ids.clear();
        return HashSet::new();
    }
    // Check cache by content hash
    if let Ok(content_h) = file_hash(&ignored_path) {
        if cache.ignored_hash.as_deref() == Some(&content_h) && !cache.ignored_ids.is_empty() {
            return cache.ignored_ids.iter().cloned().collect();
        }
        // Parse fresh
        let mut ids = Vec::new();
        if let Ok(contents) = fs::read_to_string(&ignored_path) {
            for line in contents.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    ids.push(trimmed.to_string());
                }
            }
        }
        cache.ignored_hash = Some(content_h);
        cache.ignored_ids = ids.clone();
        ids.into_iter().collect()
    } else {
        HashSet::new()
    }
}

/// True when one of the legacy `_ignored.txt` ids is a whole `,`-separated
/// segment of `meta`. Whole-segment, like the `txn:` rules `append_hide_rule`
/// writes: a substring check would also drop the partner of an auto-linked
/// swap (its meta carries `swap:txn:<ignored id>`) and every id the ignored
/// one prefixes (`txn:<ignored id>1`). The ids are verbatim meta segments, so
/// equality is exact; one set lookup per segment keeps the per-transaction
/// cost independent of how many ids the file lists.
fn has_ignored_id(meta: Option<&str>, ignored_ids: &HashSet<String>) -> bool {
    meta.is_some_and(|m| m.split(',').any(|seg| ignored_ids.contains(seg.trim())))
}

/// Extract declared account names from accounts.transactions text.
/// Returns a list of account names (e.g., "assets:mybank:checking").
fn extract_declared_accounts(accounts_text: &str) -> Vec<String> {
    let mut accounts = Vec::new();
    for line in accounts_text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("account ") {
            // "account assets:mybank:checking USD" → "assets:mybank:checking"
            let name = rest.split_whitespace().next().unwrap_or("").to_string();
            if !name.is_empty() {
                accounts.push(name);
            }
        }
    }
    accounts
}

/// Look up an account name from declared accounts by matching against a folder path.
/// For folder "mybank/checking" and declared "assets:mybank:checking",
/// converts "assets:mybank:checking" → "mybank/checking" and checks for match.
/// Only declarations under the folder's own root are candidates — a legacy
/// folder derives under `assets`, `<entity>/liabilities/…` under
/// `liabilities` — so the folder layout decides the root and the declaration
/// only refines the leaf.
fn lookup_account_for_folder(
    folder_relative: &str,
    declared_accounts: &[String],
) -> Option<String> {
    let derived = folder_to_account_name(folder_relative);
    let prefix = format!("{}:", account_root(&derived));
    for acct in declared_accounts {
        if let Some(rest) = acct.strip_prefix(&prefix) {
            let folder_form = rest.replace(':', "/");
            if folder_relative == folder_form
                || folder_relative.ends_with(&format!("/{folder_form}"))
            {
                return Some(acct.clone());
            }
        }
    }
    None
}

fn parse_accounts_file(
    dir: &Path,
    cache_key: &str,
    cache: &mut BuildCache,
) -> Result<String, String> {
    let accounts_path = dir.join(FILE_ACCOUNTS);
    if !accounts_path.exists() {
        return Ok(String::new());
    }
    // Check cache by content hash
    let content_h = file_hash(&accounts_path)?;
    if let Some(cached_h) = cache.accounts_hashes.get(cache_key) {
        if *cached_h == content_h {
            if let Some(cached_text) = cache.accounts_cached.get(cache_key) {
                return Ok(cached_text.clone());
            }
        }
    }
    let contents = fs::read_to_string(&accounts_path)
        .map_err(|e| format!("failed to read {}: {e}", accounts_path.display()))?;
    let normalized = normalize_blank_lines(&contents);
    cache
        .accounts_hashes
        .insert(cache_key.to_string(), content_h);
    cache
        .accounts_cached
        .insert(cache_key.to_string(), normalized.clone());
    Ok(normalized)
}

/// Collect all per-folder accounts.transactions files from source folders.
/// Returns a map of folder_relative_path -> accounts text.
/// Derive the opening-balance account list straight from the sources
/// accounts.transactions files (declarations only — cheap, no txn parsing).
/// Fallback for `load_pipeline_metadata` when the stored metadata predates the
/// `accounts_with_opening` field: the pipeline's unchanged-sources early exit
/// never rewrites pipeline-metadata.json, so an upgrade alone doesn't backfill it.
pub fn accounts_with_opening_from_sources(sources_dir: &Path) -> Vec<String> {
    let mut text = String::new();
    for entry in walkdir::WalkDir::new(sources_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(FILE_ACCOUNTS) {
            if let Ok(t) = fs::read_to_string(path) {
                text.push_str(&t);
                text.push('\n');
            }
        }
    }
    parse_transactions(&text).accounts_with_opening
}

/// Skips the root-level accounts.transactions (handled separately for backward compat).
fn collect_folder_accounts(
    sources_dir: &Path,
    cache: &mut BuildCache,
) -> Result<HashMap<String, String>, String> {
    let mut result = HashMap::new();
    let mut seen_keys = std::collections::HashSet::new();
    for entry in walkdir::WalkDir::new(sources_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) != Some(FILE_ACCOUNTS) {
            continue;
        }
        // Skip root-level (handled separately as fallback)
        if path.parent() == Some(sources_dir) {
            continue;
        }
        if let Some(parent) = path.parent() {
            if let Ok(rel) = parent.strip_prefix(sources_dir) {
                let rel_str = rel.to_string_lossy().to_string();
                if !rel_str.is_empty() {
                    seen_keys.insert(rel_str.clone());
                    // Check cache by content hash
                    let content_h = file_hash(path)?;
                    if let Some(cached_h) = cache.accounts_hashes.get(&rel_str) {
                        if *cached_h == content_h {
                            if let Some(cached_text) = cache.accounts_cached.get(&rel_str) {
                                result.insert(rel_str, cached_text.clone());
                                continue;
                            }
                        }
                    }
                    let text = fs::read_to_string(path)
                        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
                    let normalized = normalize_blank_lines(&text);
                    cache.accounts_hashes.insert(rel_str.clone(), content_h);
                    cache
                        .accounts_cached
                        .insert(rel_str.clone(), normalized.clone());
                    result.insert(rel_str, normalized);
                }
            }
        }
    }
    // Prune stale accounts cache entries (but keep root "" key)
    cache
        .accounts_hashes
        .retain(|k, _| k.is_empty() || seen_keys.contains(k));
    cache
        .accounts_cached
        .retain(|k, _| k.is_empty() || seen_keys.contains(k));
    Ok(result)
}

/// Build combined accounts text for a specific account set from per-folder files.
/// Falls back to root_accounts_text if no per-folder files exist for that set.
///
/// Iterates folder keys in sorted order so the output is deterministic across runs
/// (HashMap iteration order is randomized per process and would otherwise churn the
/// generated `accounts.transactions` file even when sources are unchanged).
fn accounts_text_for_set(
    account_set: &str,
    folder_accounts: &HashMap<String, String>,
    root_accounts_text: &str,
) -> String {
    let mut keys: Vec<&String> = folder_accounts.keys().collect();
    keys.sort();
    let mut set_text = String::new();
    for folder_rel in keys {
        if extract_account_set(folder_rel).as_deref() == Some(account_set) {
            if !set_text.is_empty() && !set_text.ends_with('\n') {
                set_text.push('\n');
            }
            set_text.push_str(&folder_accounts[folder_rel]);
        }
    }
    // Fall back to root if no per-folder accounts found for this set
    if set_text.is_empty() {
        set_text = root_accounts_text.to_string();
    }
    set_text
}

/// Stable sort key for transactions sharing identical (datetime, meta).
/// Without this third tiebreak, two transactions in the same block-second with
/// the same (or empty) meta would retain their input order — which itself
/// depends on HashSet iteration of unchanged_folders, making the FIFO inventory
/// (and thus CGT reports + per-folder ledger output) non-deterministic.
///
/// Exposed `pub(crate)` so the post-write loader in `generated_store::load_active_ledger`
/// can apply the SAME tiebreak. Without that, the loader's 2-key sort
/// `(datetime, meta)` would leave ties resolved by file-input order — a different
/// answer to `sort_tagged_txns`'s 3-key sort, which is the latent bug we want closed.
pub(crate) fn txn_sort_key(txn: &Transaction) -> String {
    extract_txn_id_from_meta(&txn.meta).unwrap_or_else(|| transaction_to_text(txn))
}

/// Sort tagged transactions by (datetime, meta, txn_sort_key) for fully
/// deterministic ordering. See `txn_sort_key` for why the third key is needed.
fn sort_tagged_txns(txns: &mut [(Option<String>, Option<String>, Transaction)]) {
    txns.sort_by(|(_, _, a), (_, _, b)| {
        a.datetime
            .cmp(&b.datetime)
            .then_with(|| {
                a.meta
                    .as_deref()
                    .unwrap_or("")
                    .cmp(b.meta.as_deref().unwrap_or(""))
            })
            .then_with(|| txn_sort_key(a).cmp(&txn_sort_key(b)))
    });
}

struct PipelineState {
    /// (account_set, folder_rel, transaction). `folder_rel` is `None` for
    /// root-level inputs that are never written to a per-folder ledger.
    tagged_txns: Vec<(Option<String>, Option<String>, Transaction)>,
    csv_transformed: usize,
    csv_cached: usize,
    ofx_transformed: usize,
    ofx_cached: usize,
    warnings: Vec<String>,
    owner_accounts: HashMap<String, Vec<String>>,
    account_folders: AccountFolders,
    active_csv_paths: Vec<String>,
    /// Accounts to declare in each set's accounts.transactions because a
    /// source folder derives them: `set → accounts`, sorted so the written
    /// file is deterministic. Keyed by set for the same reason as
    /// [`AccountFolders`]: two sets may derive the same account name.
    auto_accounts: HashMap<String, BTreeSet<String>>,
}

/// Full pipeline: scan sources -> resolve transforms -> transform CSVs
/// -> parse OFX files -> parse manual.transactions -> parse accounts.transactions
/// -> merge all -> sort -> partition by month and account set -> write output files
pub fn run_pipeline(config: &PipelineConfig) -> Result<PipelineResult, String> {
    let _guard = pipeline_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let pipeline_start = std::time::Instant::now();
    let mut lap = pipeline_start;
    // Timing helper: prints elapsed since last lap and resets.
    macro_rules! timing {
        ($label:expr) => {{
            let now = std::time::Instant::now();
            eprintln!(
                "[pipeline] {:30} {:>6.1}ms",
                $label,
                now.duration_since(lap).as_secs_f64() * 1000.0
            );
            #[allow(unused_assignments)]
            {
                lap = now;
            }
        }};
    }

    fs::create_dir_all(&config.sources_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&config.generated_dir).map_err(|e| e.to_string())?;

    // Determine which folders we know changed (hint from caller, e.g. save_trade_link).
    let hint_changed: Option<std::collections::HashSet<String>> = config
        .changed_folder_hint
        .as_ref()
        .map(|v| v.iter().cloned().collect());
    let has_hint = hint_changed.is_some();

    // When we have a hint, determine affected account sets and only load those folders' caches.
    // This avoids the 1.3s cost of loading all 46K+ cached transactions.
    let affected_sets: Option<std::collections::HashSet<String>> =
        hint_changed.as_ref().map(|changed| {
            changed
                .iter()
                .filter_map(|f| extract_account_set(f))
                .collect()
        });

    let source_folders = collect_source_folders(&config.sources_dir)?;
    let reconcile_warnings = reconcile_generated_folders(&config.generated_dir, &source_folders);
    timing!("reconcile_folders");

    let mut cache = if has_hint {
        // Selective load: only folder caches for the affected account sets
        let folders_to_load: std::collections::HashSet<String> = source_folders
            .iter()
            .filter(|f| {
                if let Some(ref sets) = affected_sets {
                    extract_account_set(f).is_some_and(|s| sets.contains(&s))
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        eprintln!(
            "[pipeline] hint: loading {} of {} folder caches",
            folders_to_load.len(),
            source_folders.len()
        );
        load_cache_filtered(&config.generated_dir, Some(&folders_to_load))
    } else {
        load_cache(&config.generated_dir)
    };
    timing!("load_cache");

    // Layer 1: Global early-exit via inputs_hash (skip when we have a hint — we know something changed).
    // MATCHER_VERSION is folded in so a version bump gets past this early exit
    // too — otherwise unchanged sources short-circuit before the folder-level
    // checks ever see the new version. Relay sync state is left out: a sync
    // rewrites it without changing any pipeline input.
    let global_fp = if has_hint {
        String::new() // skip fingerprinting
    } else {
        string_hash(&format!(
            "{}:m={}",
            sources_fingerprint(&config.sources_dir)?,
            crate::rules::MATCHER_VERSION
        ))
    };
    timing!("global_fingerprint");
    if !has_hint
        && !config.force
        && !global_fp.is_empty()
        && cache.inputs_hash.as_deref() == Some(&global_fp)
    {
        // Validate that every known output file still matches its cached
        // hash. Without this check, an externally-modified generated file
        // (git checkout, manual edit, another tool) is invisible — the
        // sources fingerprint matches, so the pipeline returns immediately
        // and never writes the corrected file. With it, any divergence
        // triggers the full pipeline, which re-loads cached txns from
        // cache.entries and lets write_if_changed restore the on-disk file.
        let outputs_intact = cache.output_hashes.iter().all(|(rel_key, expected)| {
            let path = config.generated_dir.join(rel_key);
            match fs::read_to_string(&path) {
                Ok(text) => &string_hash(&text) == expected,
                Err(_) => false,
            }
        });
        if outputs_intact {
            if let Some(meta) = PipelineMetadata::load(&config.generated_dir) {
                let mut csv_cached_count = 0usize;
                let mut ofx_cached_count = 0usize;
                for key in cache.entries.keys() {
                    if key.ends_with(".csv") {
                        csv_cached_count += 1;
                    } else if key.ends_with(".ofx") {
                        ofx_cached_count += 1;
                    }
                }
                return Ok(PipelineResult {
                    csv_transformed: 0,
                    csv_cached: csv_cached_count,
                    ofx_transformed: 0,
                    ofx_cached: ofx_cached_count,
                    manual_count: 0,
                    total_written: 0,
                    warnings: reconcile_warnings,
                    owner_accounts: meta.owner_accounts,
                    account_folders: meta.account_folders,
                    account_properties: meta.account_properties,
                    early_exit: true,
                    output_files_written: 0,
                    output_files_skipped: 0,
                    changed_folders: Vec::new(),
                    in_memory: None,
                });
            }
        }
    }

    let csv_files = scan_csv_files(&config.sources_dir)?;
    let ofx_files = scan_ofx_files(&config.sources_dir)?;
    timing!("scan_files");

    // Layer 2: Per-folder fingerprints for incremental processing.
    let mut folder_fps: HashMap<String, String> = HashMap::new();
    for folder_rel in &source_folders {
        // When we have a hint, only fingerprint the changed folders (skip unchanged).
        if let Some(ref changed) = hint_changed {
            if !hint_matches(changed, folder_rel) {
                // Use cached fingerprint for unchanged folders
                if let Some(fp) = cache.folder_hashes.get(folder_rel) {
                    folder_fps.insert(folder_rel.clone(), fp.clone());
                }
                continue;
            }
        }
        let folder_path = config.sources_dir.join(folder_rel);
        // Fold MATCHER_VERSION in so a version bump invalidates EVERY folder,
        // not just folders whose content happens to change. Without this the
        // layer-2 unchanged-folder skip keeps serving transactions transformed
        // by old matcher logic (verified live: a v3->v4 bump re-transformed
        // only the one touched folder; 41k rows elsewhere stayed cached).
        let mut fp = string_hash(&format!(
            "{}:m={}",
            dir_fingerprint(&folder_path)?,
            crate::rules::MATCHER_VERSION
        ));
        let mut parent = folder_path.parent();
        while let Some(p) = parent {
            if !p.starts_with(&config.sources_dir) || p < config.sources_dir.as_path() {
                break;
            }
            for parent_file in &["_rules.json", "_labels.json"] {
                let path = p.join(parent_file);
                if path.exists() {
                    if let Ok(meta) = fs::metadata(&path) {
                        let mtime = meta
                            .modified()
                            .unwrap_or(std::time::UNIX_EPOCH)
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        fp = string_hash(&format!(
                            "{fp}:parent_{}:{}:{}",
                            parent_file,
                            mtime,
                            meta.len()
                        ));
                    }
                }
            }
            if p == config.sources_dir.as_path() {
                break;
            }
            parent = p.parent();
        }
        folder_fps.insert(folder_rel.clone(), fp);
    }
    timing!("folder_fingerprints");

    // Parse accounts.transactions: per-folder files + root-level fallback (with cache)
    let root_accounts_text = parse_accounts_file(&config.sources_dir, "", &mut cache)?;
    let folder_accounts = collect_folder_accounts(&config.sources_dir, &mut cache)?;
    timing!("parse_accounts");

    // Build combined declared accounts for lookups (all folders + root).
    // Iterate keys in sorted order so the resulting text — and any downstream
    // ordering of declared_accounts — is deterministic across runs.
    let mut all_accounts_text = root_accounts_text.clone();
    let mut folder_keys: Vec<&String> = folder_accounts.keys().collect();
    folder_keys.sort();
    for key in folder_keys {
        let text = &folder_accounts[key];
        if !all_accounts_text.is_empty() && !all_accounts_text.ends_with('\n') {
            all_accounts_text.push('\n');
        }
        all_accounts_text.push_str(text);
    }
    let mut declared_accounts = extract_declared_accounts(&all_accounts_text);

    // Also include folder-derived account names so that self-transfer detection
    // works for any wallet folder, even without an accounts.transactions file.
    for folder_rel in &source_folders {
        let folder_account = folder_to_account_name(folder_rel);
        if !declared_accounts.contains(&folder_account) {
            declared_accounts.push(folder_account);
        }
    }

    // Extract account properties (e.g. friendly names) and opening-balance
    // declarations from all accounts files
    let all_accounts_parsed = parse_transactions(&all_accounts_text);
    let account_properties = all_accounts_parsed.account_properties;
    let accounts_with_opening = all_accounts_parsed.accounts_with_opening;

    // Layer 2: Determine which folders are unchanged and can use cached transactions.
    let mut unchanged_folders: std::collections::HashSet<String> = std::collections::HashSet::new();
    if !config.force {
        if let Some(ref changed) = hint_changed {
            // With a hint, all folders NOT covered by the hint (exact or
            // as a descendant of a hinted parent) are unchanged.
            for folder_rel in &source_folders {
                if !hint_matches(changed, folder_rel) {
                    unchanged_folders.insert(folder_rel.clone());
                }
            }
        } else {
            // Without a hint, compare fingerprints to detect changes.
            for (folder_rel, fp) in &folder_fps {
                if let Some(cached_fp) = cache.folder_hashes.get(folder_rel) {
                    // Folder is unchanged if fingerprint matches and generated file exists
                    let generated_file = config.generated_dir.join(folder_rel).join(FILE_LEDGER);
                    if cached_fp == fp && generated_file.exists() {
                        unchanged_folders.insert(folder_rel.clone());
                    }
                }
            }
        }
    }

    timing!("layer2_unchanged_folders");

    // Initialize pipeline state
    let mut state = PipelineState {
        tagged_txns: Vec::new(),
        csv_transformed: 0,
        csv_cached: 0,
        ofx_transformed: 0,
        ofx_cached: 0,
        warnings: reconcile_warnings,
        owner_accounts: HashMap::new(),
        account_folders: HashMap::new(),
        active_csv_paths: Vec::new(),
        auto_accounts: HashMap::new(),
    };

    // Load ignored ids once, up front. The cache-load loop below uses them to
    // detect when an unchanged folder's on-disk ledger contains a now-ignored
    // txn (forcing that folder to be rewritten via `force_rewrite_folders` —
    // otherwise unchanged folders are skipped and their stale ledger files
    // keep showing the hidden txn).
    let ignored_ids = load_ignored_ids(&config.sources_dir, &mut cache);

    // Folders whose on-disk ledger was loaded via the on_disk_intact branch
    // below. These ledgers ALREADY contain the manual.transactions content
    // inlined from the previous run, so the manual loop further down must
    // not re-push it into tagged_txns (otherwise we'd double-count and the
    // rewritten ledger would drift).
    let mut intact_folders: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // Folders that need to be rewritten even though their fingerprint is
    // unchanged: on-disk ledger is corrupt/missing (recovery path) or carries
    // a now-ignored txn that must be stripped. tagged_txns is the source of
    // truth for content; this set is the rewrite-trigger signal that survives
    // the unchanged-folders fingerprint optimisation.
    let mut force_rewrite_folders: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // Load cached transactions for unchanged folders.
    // When we have a hint, only load folders in the affected set(s) — skip the rest entirely.
    for folder_rel in &unchanged_folders {
        // Skip folders outside the affected set when using hints
        if let Some(ref sets) = affected_sets {
            let folder_set = extract_account_set(folder_rel);
            if !folder_set.as_ref().is_some_and(|s| sets.contains(s)) {
                // Still track folder metadata for the result
                if let Some(ref owner) = folder_set {
                    let account_name = folder_to_account_name(folder_rel);
                    state
                        .owner_accounts
                        .entry(owner.clone())
                        .or_default()
                        .push(account_name.clone());
                    state
                        .account_folders
                        .entry(owner.clone())
                        .or_default()
                        .insert(account_name.clone(), folder_rel.clone());
                    state
                        .auto_accounts
                        .entry(owner.clone())
                        .or_default()
                        .insert(account_name);
                }
                continue;
            }
        }

        let account_set = extract_account_set(folder_rel);

        // Track owner_accounts/account_folders/auto_accounts from folder structure
        if let Some(ref owner) = account_set {
            let account_name = folder_to_account_name(folder_rel);
            state
                .owner_accounts
                .entry(owner.clone())
                .or_default()
                .push(account_name.clone());
            state
                .account_folders
                .entry(owner.clone())
                .or_default()
                .insert(account_name.clone(), folder_rel.clone());
            state
                .auto_accounts
                .entry(owner.clone())
                .or_default()
                .insert(account_name);
        }

        // Two states feed unchanged-folder loading:
        //
        //   intact   — on-disk ledger.transactions hash matches the cached
        //              output_hash. The file is what we last wrote, including
        //              auto_link price annotations and any other late-stage
        //              mutations. Load from disk and push into tagged_txns;
        //              skip rewrite unless a now-ignored txn forces it.
        //
        //   diverged — on-disk file has been clobbered externally (git
        //              checkout, manual edit, another tool) or is missing.
        //              Load from cache.entries (pre-auto_link) and flag for
        //              rewrite via `force_rewrite_folders`. The downstream
        //              auto_link path re-applies price annotations.
        let ledger_path = config.generated_dir.join(folder_rel).join(FILE_LEDGER);
        let on_disk_text = fs::read_to_string(&ledger_path).ok();
        let ledger_rel_key = format!("{folder_rel}/{FILE_LEDGER}");
        let on_disk_intact = match (&on_disk_text, cache.output_hashes.get(&ledger_rel_key)) {
            (Some(t), Some(expected)) => &string_hash(t) == expected,
            _ => false,
        };

        if on_disk_intact {
            let text = on_disk_text.expect("intact implies Some(text)");
            let result = parse_transactions(&text);
            let count = result.transactions.len();
            intact_folders.insert(folder_rel.clone());
            let folder_has_ignored = !ignored_ids.is_empty()
                && result
                    .transactions
                    .iter()
                    .any(|t| has_ignored_id(t.meta.as_deref(), &ignored_ids));
            // The folder is fingerprint-unchanged but the on-disk ledger
            // includes a now-ignored txn: force a rewrite. tagged_txns is
            // filtered with the same `has_ignored_id` below (`filter_ignored`),
            // so the rewrite omits them automatically.
            if folder_has_ignored {
                force_rewrite_folders.insert(folder_rel.clone());
            }
            for txn in result.transactions {
                state
                    .tagged_txns
                    .push((account_set.clone(), Some(folder_rel.clone()), txn));
            }
            state.csv_cached += count;
        } else {
            // Recovery path: on-disk file is corrupt or missing. Rebuild
            // from cache.entries and flag for rewrite. Manual transactions
            // are deliberately excluded here — they're re-parsed downstream
            // from sources/.../manual.transactions, so loading them from
            // cache too would double-count.
            let folder_prefix = if folder_rel.is_empty() {
                String::new()
            } else {
                format!("{folder_rel}/")
            };
            let mut entries_for_folder: Vec<(&String, &CacheEntry)> = cache
                .entries
                .iter()
                .filter(|(k, _)| !folder_prefix.is_empty() && k.starts_with(&folder_prefix))
                .collect();
            entries_for_folder.sort_by(|a, b| a.0.cmp(b.0));
            let mut count = 0usize;
            for (_path, entry) in &entries_for_folder {
                for ct in &entry.transactions {
                    let txn = match ct.txn.as_ref() {
                        Some(t) => t.clone(),
                        None => match parse_transactions(&ct.text).transactions.into_iter().next() {
                            Some(t) => t,
                            None => continue,
                        },
                    };
                    state
                        .tagged_txns
                        .push((account_set.clone(), Some(folder_rel.clone()), txn));
                    count += 1;
                }
            }
            if count > 0 {
                force_rewrite_folders.insert(folder_rel.clone());
            }
            state.csv_cached += count;
        }
    }

    timing!("load_cached_txns");

    // Process CSV and OFX files
    process_csv_files(
        &mut state,
        &csv_files,
        config,
        &mut cache,
        &unchanged_folders,
        &declared_accounts,
    )?;
    timing!("process_csv");
    process_ofx_files(
        &mut state,
        &ofx_files,
        config,
        &mut cache,
        &unchanged_folders,
        &declared_accounts,
    )?;
    timing!("process_ofx");

    // Prune cache entries for files that no longer exist
    let active_paths: Vec<String> = state.active_csv_paths.clone();
    let ofx_relatives: Vec<String> = ofx_files
        .iter()
        .filter_map(|p| {
            p.strip_prefix(&config.sources_dir)
                .ok()
                .map(|r| r.to_string_lossy().to_string())
        })
        .collect();
    cache
        .entries
        .retain(|k, _| active_paths.contains(k) || ofx_relatives.contains(k));

    // Parse manual transactions (root-level and per-folder, with cache)
    let manual_txns = parse_all_manual_transactions(&config.sources_dir, &mut cache)?;
    timing!("parse_manual");
    let manual_count = manual_txns.len();
    // Tag manual transactions: use folder-derived set, falling back to posting inference.
    //
    // For folders loaded via the on_disk_intact branch (intact_folders), the
    // on-disk ledger already contains the manual entries inlined from the
    // previous run, and the on_disk_intact branch already pushed them into
    // tagged_txns. Re-adding them here would double-count. Skip those folders.
    for (folder_set, folder_rel, txn) in manual_txns {
        let set = folder_set.or_else(|| infer_account_set_from_txn(&txn, &state.owner_accounts));
        // Root-level manual.transactions (folder_rel == "") have no per-folder
        // ledger; they live in tagged_txns for in-memory reads only.
        let folder_opt: Option<String> = if folder_rel.is_empty() {
            None
        } else {
            Some(folder_rel.clone())
        };
        // Register the folder in account_folders for manual-only folders
        // (CSV/OFX folders are registered by their own processing paths, but a
        // folder containing only a manual.transactions file would otherwise be
        // missing from the metadata map).
        if !folder_rel.is_empty() {
            let account_name = folder_to_account_name(&folder_rel);
            if let Some(ref owner) = set {
                state
                    .account_folders
                    .entry(owner.clone())
                    .or_default()
                    .entry(account_name.clone())
                    .or_insert_with(|| folder_rel.clone());
                state
                    .auto_accounts
                    .entry(owner.clone())
                    .or_default()
                    .insert(account_name);
            }
            if intact_folders.contains(&folder_rel) {
                continue;
            }
        }
        state.tagged_txns.push((set, folder_opt, txn));
    }

    // Filter out ignored transactions (ids loaded above)
    if !ignored_ids.is_empty() {
        state
            .tagged_txns
            .retain(|(_, _, txn)| !has_ignored_id(txn.meta.as_deref(), &ignored_ids));
    }

    timing!("filter_ignored");

    // Rebuild owner_accounts from transaction postings and source folders
    rebuild_owner_accounts(
        &mut state.owner_accounts,
        &state.tagged_txns,
        &config.sources_dir,
    );
    timing!("rebuild_owner_accounts");

    // Sort: by datetime, then meta, then a per-txn tiebreak — see `sort_tagged_txns`.
    sort_tagged_txns(&mut state.tagged_txns);

    // Determine account sets from folder structure
    let mut account_sets: Vec<String> = state.owner_accounts.keys().cloned().collect();
    account_sets.sort();
    account_sets.dedup();

    timing!("sort");

    // Auto-link equity:trading swap pairs and derive prices from counterparts,
    // each set's legs in that set's own base currency (set-then-vault lookup;
    // a transaction that belongs to no set prices in the vault default).
    let price_graph = crate::ledger_parser::PriceGraph::load(&config.sources_dir);
    let base_currencies = crate::display_config::resolve_base_currencies(
        &config.generated_dir,
        state
            .tagged_txns
            .iter()
            .map(|(set, _, _)| set.as_deref().unwrap_or("")),
    );

    // Snapshot per-folder serialised text BEFORE auto_link so we can detect
    // which folders auto_link mutated (price annotations, sell/buy suffixes,
    // swap-cross-reference meta). Folders whose post-link content differs
    // join `changed_folders` so their ledger files get rewritten.
    let pre_link_by_folder: HashMap<String, Vec<String>> = {
        let mut acc: HashMap<String, Vec<String>> = HashMap::new();
        for (_, folder_opt, txn) in &state.tagged_txns {
            if let Some(folder_rel) = folder_opt {
                acc.entry(folder_rel.clone())
                    .or_default()
                    .push(transaction_to_text(txn));
            }
        }
        for v in acc.values_mut() {
            v.sort();
        }
        acc
    };

    // auto_link is the dominant rebuild cost (~2.7s on a 54k-txn vault) — it
    // scans the whole affected set. But its output only changes when an
    // equity:trading leg is added/removed/modified, which can only happen in a
    // folder that changed this run. So run it only when a CHANGED folder
    // actually contains equity:trading; otherwise the swap links already cached
    // in tagged_txns are still correct and we skip the scan entirely.
    //
    // Two narrowing layers: the hint restricts tagged_txns to the affected
    // account-set (the linker groups legs by set, so swaps never pair across
    // sets — a future cross-set swap feature must force a full rebuild); this
    // skip then avoids the scan when no changed folder trades. `force` empties
    // unchanged_folders, so a forced full rebuild always re-runs auto_link.
    let changed_touches_equity_trading = state.tagged_txns.iter().any(|(_, folder_opt, txn)| {
        let in_changed_folder = match folder_opt {
            Some(f) => !unchanged_folders.contains(f),
            None => true, // unknown provenance — treat as changed (conservative)
        };
        in_changed_folder
            && txn
                .postings
                .iter()
                .any(|p| p.account.starts_with("equity:trading"))
    });
    if changed_touches_equity_trading {
        link_equity_swaps_per_set(&mut state.tagged_txns, Some(&price_graph), &|set| {
            base_currencies
                .get(set.unwrap_or(""))
                .and_then(|currency| currency.as_deref())
        });
    }

    // Detect folders whose tagged_txns content was mutated by auto_link.
    let mut auto_link_modified_folders: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    {
        let mut post_link_by_folder: HashMap<&String, Vec<String>> = HashMap::new();
        for (_, folder_opt, txn) in &state.tagged_txns {
            if let Some(folder_rel) = folder_opt {
                post_link_by_folder
                    .entry(folder_rel)
                    .or_default()
                    .push(transaction_to_text(txn));
            }
        }
        for (folder_rel, mut post_texts) in post_link_by_folder {
            post_texts.sort();
            let pre = pre_link_by_folder.get(folder_rel.as_str());
            if pre.map(|v| v.as_slice()) != Some(post_texts.as_slice()) {
                auto_link_modified_folders.insert(folder_rel.clone());
            }
        }
    }

    timing!("auto_link_equity_swaps");

    // Folders to write this run:
    //   - source_folders whose fingerprint changed (CSV/OFX/manual edits)
    //   - folders flagged by force_rewrite_folders (corrupt/missing on-disk
    //     ledger or now-ignored content)
    //   - folders whose tagged_txns content was mutated by auto_link
    let mut changed_folders: std::collections::HashSet<String> = source_folders
        .iter()
        .filter(|f| !unchanged_folders.contains(*f))
        .cloned()
        .collect();
    changed_folders.extend(force_rewrite_folders);
    changed_folders.extend(auto_link_modified_folders);
    let mut changed_folders_sorted: Vec<String> = changed_folders.iter().cloned().collect();
    changed_folders_sorted.sort();
    if !changed_folders_sorted.is_empty() {
        eprintln!(
            "[pipeline] changed_folders: {:?}",
            changed_folders_sorted
        );
    }

    // Write per-folder ledger.transactions files for changed folders.
    // tagged_txns is the single source of truth for content; per-folder
    // partitions are derived inside write_pipeline_output.
    let (total_written, output_files_written, output_files_skipped) = write_pipeline_output(
        config,
        &account_sets,
        &root_accounts_text,
        &folder_accounts,
        &state.auto_accounts,
        &mut cache.output_hashes,
        &changed_folders,
        &state.tagged_txns,
        &source_folders,
    )?;

    timing!("write_output");

    // Update cache with fingerprints for changed folders.
    cache.inputs_hash = Some(global_fp);
    for folder_rel in &changed_folders {
        if let Some(fp) = folder_fps.get(folder_rel) {
            cache.folder_hashes.insert(folder_rel.clone(), fp.clone());
        }
    }
    // Prune fingerprints for folders that no longer exist.
    let active_folders: std::collections::HashSet<&String> = folder_fps.keys().collect();
    cache
        .folder_hashes
        .retain(|k, _| active_folders.contains(k));

    save_cache(&config.generated_dir, &cache, Some(&changed_folders))?;
    timing!("save_cache");

    // Register declared accounts from every folder's accounts.transactions so
    // declaration-only accounts (added but not yet imported into) are
    // folder-backed in the sidebar. Data-derived mappings always win — this
    // only fills gaps. Only declarations under the folder's own root qualify
    // (`assets:*` in a legacy or `<entity>/assets/…` folder, `liabilities:*`
    // under `<entity>/liabilities/…`): income/expenses/equity, and a
    // liabilities declaration parked in `_accounts`, keep appearing in the
    // Categories pane, whose filter excludes folder-backed accounts.
    for (folder_rel, text) in &folder_accounts {
        let folder_account = folder_to_account_name(folder_rel);
        let folder_root = account_root(&folder_account);
        for name in extract_declared_accounts(text) {
            if !under_folder_root(&name) || account_root(&name) != folder_root {
                continue;
            }
            if let Some(owner) = extract_account_set(folder_rel) {
                state
                    .account_folders
                    .entry(owner.clone())
                    .or_default()
                    .entry(name.clone())
                    .or_insert_with(|| folder_rel.clone());
                let owned = state.owner_accounts.entry(owner).or_default();
                if !owned.contains(&name) {
                    owned.push(name);
                }
            }
        }
    }

    // Save pipeline metadata
    let metadata = PipelineMetadata {
        owner_accounts: state.owner_accounts.clone(),
        account_folders: state.account_folders.clone(),
        account_properties: account_properties.clone(),
        accounts_with_opening,
    };
    metadata.save(&config.generated_dir)?;
    if let Some(warning) = crate::generator::stamp(&config.generated_dir)? {
        state.warnings.push(warning);
    }

    timing!("save_metadata");

    eprintln!("[pipeline] TOTAL {:>38.1}ms  (csv:{}/{} ofx:{}/{} manual:{} output_written:{} output_skipped:{})",
    pipeline_start.elapsed().as_secs_f64() * 1000.0,
    state.csv_transformed, state.csv_cached,
    state.ofx_transformed, state.ofx_cached,
    manual_count, output_files_written, output_files_skipped);

    // Build per-set accounts text for in-memory ParseResult construction
    let mut accounts_text_by_set: HashMap<String, String> = HashMap::new();
    for set_name in &account_sets {
        let mut set_text = String::new();
        // Auto-generated account declarations for this set
        for acct in state.auto_accounts.get(set_name).into_iter().flatten() {
            set_text.push_str(&format!("account {acct}\n"));
        }
        let folder_text = accounts_text_for_set(set_name, &folder_accounts, &root_accounts_text);
        if !folder_text.is_empty() {
            if !set_text.is_empty() && !set_text.ends_with('\n') {
                set_text.push('\n');
            }
            set_text.push_str(&folder_text);
        }
        accounts_text_by_set.insert(set_name.clone(), set_text);
    }

    // Persist pipeline warnings to generated/warnings.json so the
    // `emuacc-issues` CLI and the Tauri `collect_issues_cmd` can read them.
    // Schema matches `crate::issues::PipelineWarningsFile`.
    let warnings_path = config.generated_dir.join("warnings.json");
    let warnings_json = serde_json::json!({ "warnings": state.warnings });
    let _ = fs::write(
        &warnings_path,
        serde_json::to_string_pretty(&warnings_json).unwrap_or_default(),
    );

    Ok(PipelineResult {
        csv_transformed: state.csv_transformed,
        csv_cached: state.csv_cached,
        ofx_transformed: state.ofx_transformed,
        ofx_cached: state.ofx_cached,
        manual_count,
        total_written,
        warnings: state.warnings,
        owner_accounts: state.owner_accounts,
        account_folders: state.account_folders,
        account_properties,
        early_exit: false,
        output_files_written,
        output_files_skipped,
        changed_folders: changed_folders_sorted,
        in_memory: Some(Box::new(PipelineInMemory {
            tagged_txns: state.tagged_txns,
            accounts_text_by_set,
        })),
    })
}

fn process_csv_files(
    state: &mut PipelineState,
    csv_files: &[PathBuf],
    config: &PipelineConfig,
    cache: &mut BuildCache,
    unchanged_folders: &std::collections::HashSet<String>,
    declared_accounts: &[String],
) -> Result<(), String> {
    // Per-folder transform cache: all CSVs in the same folder share the same
    // transform chain, so resolve once per folder instead of once per file.
    let mut folder_transform_cache: HashMap<String, crate::csv_transform::TransformChain> =
        HashMap::new();

    for csv_path in csv_files {
        let relative = csv_path
            .strip_prefix(&config.sources_dir)
            .map_err(|_| "CSV not under sources dir".to_string())?
            .to_string_lossy()
            .to_string();
        state.active_csv_paths.push(relative.clone());

        // Layer 2: Skip files in unchanged folders (already loaded from cache above).
        let folder_relative = csv_path
            .parent()
            .and_then(|p| p.strip_prefix(&config.sources_dir).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if !folder_relative.is_empty() && unchanged_folders.contains(&folder_relative) {
            continue;
        }

        let content_h = file_hash(csv_path)?;
        let chain = match folder_transform_cache.get(&folder_relative) {
            Some(c) => c.clone(),
            None => {
                let c = resolve_transform(csv_path, &config.sources_dir)?;
                folder_transform_cache.insert(folder_relative.clone(), c.clone());
                c
            }
        };

        // Determine account set from source path
        let account_set = extract_account_set(&relative);

        // Track owner → account mapping and auto-declarations from folder structure
        if let Some(ref owner) = account_set {
            if !folder_relative.is_empty() {
                let account_name = folder_to_account_name(&folder_relative);
                state
                    .owner_accounts
                    .entry(owner.clone())
                    .or_default()
                    .push(account_name.clone());
                state
                    .account_folders
                    .entry(owner.clone())
                    .or_default()
                    .insert(account_name.clone(), folder_relative.clone());
                state
                    .auto_accounts
                    .entry(owner.clone())
                    .or_default()
                    .insert(account_name);
            }
        }

        if let Some(cached) = lookup(cache, &relative, &content_h, &chain.combined_hash) {
            state.csv_cached += 1;
            let folder_acct = if !folder_relative.is_empty() {
                Some(folder_to_account_name(&folder_relative))
            } else {
                None
            };
            let folder_opt: Option<String> = if folder_relative.is_empty() {
                None
            } else {
                Some(folder_relative.clone())
            };
            for ct in cached {
                let mut txn = if let Some(t) = ct.txn.clone() {
                    t
                } else {
                    let result = parse_transactions(&ct.text);
                    match result.transactions.into_iter().next() {
                        Some(t) => t,
                        None => continue,
                    }
                };
                if let Some(ref acct) = folder_acct {
                    for posting in &mut txn.postings {
                        if posting.account == FALLBACK_ASSET_ACCOUNT {
                            posting.account = acct.clone();
                        }
                    }
                }
                state
                    .tagged_txns
                    .push((account_set.clone(), folder_opt.clone(), txn));
            }
            continue;
        }

        let mut txns = transform_csv_with_default(
            csv_path,
            &config.sources_dir,
            &chain,
            &config.default_expense_account,
        )?;
        state.csv_transformed += 1;

        // Auto self-transfer: for transactions not matched by rules, check if
        // the transaction text mentions a declared account identifier
        for txn in &mut txns {
            if txn.postings.len() >= 2 && txn.postings[1].account == config.default_expense_account
            {
                apply_rules_with_accounts(txn, &RulesFile::default(), declared_accounts);
            }
        }

        let cached_txns: Vec<CachedTransaction> = txns
            .iter()
            .map(|t| CachedTransaction {
                text: transaction_to_text(t),
                txn: Some(t.clone()),
            })
            .collect();

        cache.entries.insert(
            relative,
            CacheEntry {
                content_hash: content_h,
                transform_hash: chain.combined_hash,
                transactions: cached_txns,
            },
        );

        let folder_opt: Option<String> = if folder_relative.is_empty() {
            None
        } else {
            Some(folder_relative.clone())
        };
        for txn in txns {
            state
                .tagged_txns
                .push((account_set.clone(), folder_opt.clone(), txn));
        }
    }
    Ok(())
}

fn process_ofx_files(
    state: &mut PipelineState,
    ofx_files: &[PathBuf],
    config: &PipelineConfig,
    cache: &mut BuildCache,
    unchanged_folders: &std::collections::HashSet<String>,
    declared_accounts: &[String],
) -> Result<(), String> {
    for ofx_path in ofx_files {
        let relative = ofx_path
            .strip_prefix(&config.sources_dir)
            .map_err(|_| "OFX not under sources dir".to_string())?
            .to_string_lossy()
            .to_string();

        // Layer 2: Skip OFX files in unchanged folders.
        let ofx_folder_check = ofx_path
            .parent()
            .and_then(|p| p.strip_prefix(&config.sources_dir).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if !ofx_folder_check.is_empty() && unchanged_folders.contains(&ofx_folder_check) {
            continue;
        }

        let content_h = file_hash(ofx_path)?;
        let cache_key = relative.clone();

        let account_set = extract_account_set(&relative);

        // Track owner → account mapping from OFX folder structure
        let ofx_folder_rel = ofx_path
            .parent()
            .and_then(|p| p.strip_prefix(&config.sources_dir).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let ofx_account = if !ofx_folder_rel.is_empty() {
            let acct = lookup_account_for_folder(&ofx_folder_rel, declared_accounts)
                .unwrap_or_else(|| folder_to_account_name(&ofx_folder_rel));
            if let Some(ref owner) = account_set {
                state
                    .owner_accounts
                    .entry(owner.clone())
                    .or_default()
                    .push(acct.clone());
                state
                    .account_folders
                    .entry(owner.clone())
                    .or_default()
                    .insert(acct.clone(), ofx_folder_rel.clone());
                state
                    .auto_accounts
                    .entry(owner.clone())
                    .or_default()
                    .insert(acct.clone());
            }
            Some(acct)
        } else {
            None
        };

        // Load labels + rules for OFX cache key (both affect output, must be in cache key)
        let ofx_dir = ofx_path.parent().unwrap_or(Path::new(""));
        let rules = gather_rules_chain(ofx_dir, &config.sources_dir);
        let labels = gather_labels_chain(ofx_dir, &config.sources_dir);
        let ofx_transform_h =
            string_hash(&format!("{content_h}{}{}", labels.hash(), rules.hash()));

        // Use cache for OFX files too — transform_hash includes content + rules
        if let Some(cached) = lookup(cache, &cache_key, &content_h, &ofx_transform_h) {
            state.ofx_cached += 1;
            let folder_opt: Option<String> = if ofx_folder_rel.is_empty() {
                None
            } else {
                Some(ofx_folder_rel.clone())
            };
            for ct in cached {
                let mut txn = if let Some(t) = ct.txn.clone() {
                    t
                } else {
                    let result = parse_transactions(&ct.text);
                    match result.transactions.into_iter().next() {
                        Some(t) => t,
                        None => continue,
                    }
                };
                // Fix up stale account names from old cache entries
                if let Some(ref acct) = ofx_account {
                    for posting in &mut txn.postings {
                        if posting.account == FALLBACK_ASSET_ACCOUNT
                            || posting.account.starts_with("unknown:")
                        {
                            posting.account = acct.clone();
                        }
                    }
                }
                state
                    .tagged_txns
                    .push((account_set.clone(), folder_opt.clone(), txn));
            }
            continue;
        }

        let content = fs::read_to_string(ofx_path)
            .map_err(|e| format!("failed to read OFX {}: {e}", ofx_path.display()))?;
        let ofx = parse_ofx(&content)?;

        // Use the account name looked up earlier from declarations / folder structure
        let account_name = ofx_account
            .clone()
            .unwrap_or_else(|| FALLBACK_ASSET_ACCOUNT.to_string());

        // labels + rules already loaded above for cache key
        let txns = ofx_to_transactions_with_default(
            &ofx,
            &account_name,
            &relative,
            &labels,
            &rules,
            &config.default_expense_account,
        )?;
        state.ofx_transformed += 1;

        let cached_txns: Vec<CachedTransaction> = txns
            .iter()
            .map(|t| CachedTransaction {
                text: transaction_to_text(t),
                txn: Some(t.clone()),
            })
            .collect();

        cache.entries.insert(
            cache_key,
            CacheEntry {
                content_hash: content_h,
                transform_hash: ofx_transform_h,
                transactions: cached_txns,
            },
        );

        let folder_opt: Option<String> = if ofx_folder_rel.is_empty() {
            None
        } else {
            Some(ofx_folder_rel.clone())
        };
        for txn in txns {
            state
                .tagged_txns
                .push((account_set.clone(), folder_opt.clone(), txn));
        }
    }
    Ok(())
}

fn walk_source_folders(
    dir: &Path,
    owner: &str,
    prefix: &str,
    owner_accounts: &mut HashMap<String, Vec<String>>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name.starts_with('_') || name == "imports" {
            continue;
        }
        let relative = if prefix.is_empty() {
            format!("{}/{}", owner, name)
        } else {
            format!("{}/{}", prefix, name)
        };
        let account = folder_to_account_name(&relative);
        // The bare `<entity>/assets` / `<entity>/liabilities` directory is an
        // explicit-layout root marker, not an account of its own.
        if !FOLDER_ROOTS.contains(&account.as_str()) {
            owner_accounts
                .entry(owner.to_string())
                .or_default()
                .push(account);
        }
        walk_source_folders(&path, owner, &relative, owner_accounts);
    }
}

fn rebuild_owner_accounts(
    owner_accounts: &mut HashMap<String, Vec<String>>,
    tagged_txns: &[(Option<String>, Option<String>, Transaction)],
    sources_dir: &Path,
) {
    // Rebuild owner_accounts from actual transaction postings so that
    // transform-specified accounts (e.g. "assets:cash:bank:ubank:savings") are tracked
    // instead of only folder-derived names (e.g. "assets:cash:bank:ubank").
    // Both folder roots count: a liabilities folder's postings own their
    // account the same way an asset folder's do.
    for accounts in owner_accounts.values_mut() {
        accounts.clear();
    }
    for (set, _, txn) in tagged_txns {
        if let Some(ref owner) = set {
            for posting in &txn.postings {
                if under_folder_root(&posting.account) {
                    owner_accounts
                        .entry(owner.clone())
                        .or_default()
                        .push(posting.account.clone());
                }
            }
        }
    }
    // Include accounts from source folders that have no data files yet
    // (e.g. newly added accounts with empty imports/ directories).
    // Walks recursively to arbitrary depth.
    if let Ok(entries) = fs::read_dir(sources_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.starts_with('.') || dir_name.starts_with('_') {
                continue;
            }
            walk_source_folders(&path, &dir_name, "", owner_accounts);
        }
    }

    for accounts in owner_accounts.values_mut() {
        accounts.sort();
        accounts.dedup();
    }
}

/// Tracks output file writes with content-addressed hashing (Layer 4 cache).
struct OutputTracker<'a> {
    generated_dir: PathBuf,
    output_hashes: &'a mut HashMap<String, String>,
    /// Keys of output files written or skipped in this run.
    live_outputs: std::collections::HashSet<String>,
    files_written: usize,
    files_skipped: usize,
}

impl<'a> OutputTracker<'a> {
    fn new(generated_dir: &Path, output_hashes: &'a mut HashMap<String, String>) -> Self {
        Self {
            generated_dir: generated_dir.to_path_buf(),
            output_hashes,
            live_outputs: std::collections::HashSet::new(),
            files_written: 0,
            files_skipped: 0,
        }
    }

    /// Write file only if content hash differs from cached. Returns Ok(true) if written.
    fn write_if_changed(&mut self, path: &Path, content: &str) -> Result<bool, String> {
        let rel_key = path
            .strip_prefix(&self.generated_dir)
            .map_err(|e| format!("strip_prefix failed: {e}"))?
            .to_string_lossy()
            .to_string();
        let content_hash = string_hash(content);
        self.live_outputs.insert(rel_key.clone());

        // Skip the write only if our last-written hash matches the new content
        // hash AND the on-disk content still matches that hash. If anything
        // outside the pipeline has modified the file between runs (git
        // checkout, manual edit, another tool), the on-disk content diverges
        // from output_hashes and we must rewrite to recover. Without the
        // on-disk check the pipeline silently leaves the wrong content in
        // place and no amount of regen will fix it.
        if self.output_hashes.get(&rel_key).map(|h| h.as_str()) == Some(content_hash.as_str()) {
            let on_disk_matches = fs::read_to_string(path)
                .ok()
                .map(|s| string_hash(&s) == content_hash)
                .unwrap_or(false);
            if on_disk_matches {
                self.files_skipped += 1;
                return Ok(false);
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(path, content).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
        self.output_hashes.insert(rel_key, content_hash);
        self.files_written += 1;
        Ok(true)
    }
}

/// Mark all existing output files for an account set as live (prevents pruning)
/// without serializing or re-hashing them. Used for unchanged sets.
#[allow(clippy::too_many_arguments)]
fn write_pipeline_output(
    config: &PipelineConfig,
    account_sets: &[String],
    root_accounts_text: &str,
    folder_accounts: &HashMap<String, String>,
    auto_accounts: &HashMap<String, BTreeSet<String>>,
    output_hashes: &mut HashMap<String, String>,
    changed_folders: &std::collections::HashSet<String>,
    tagged_txns: &[(Option<String>, Option<String>, Transaction)],
    source_folders: &[String],
) -> Result<(usize, usize, usize), String> {
    let mut total_written = 0usize;
    let mut tracker = OutputTracker::new(&config.generated_dir, output_hashes);

    let mut per_folder = partition_txns_by_folder(tagged_txns, changed_folders);

    for folder_rel in changed_folders {
        let txns = per_folder.remove(folder_rel.as_str()).unwrap_or_default();
        total_written += write_folder_ledger(config, folder_rel, &txns, &mut tracker)?;
    }

    mark_live_outputs(config, source_folders, changed_folders, &mut tracker);

    write_account_sets(
        config,
        account_sets,
        root_accounts_text,
        folder_accounts,
        auto_accounts,
        &mut tracker,
    )?;

    prune_stale_outputs(config, &mut tracker);

    Ok((total_written, tracker.files_written, tracker.files_skipped))
}

/// Partition tagged transactions into per-folder buckets, keeping only folders
/// in `changed_folders`. Tagged-txn order is preserved (already sorted globally).
fn partition_txns_by_folder<'a>(
    tagged_txns: &'a [(Option<String>, Option<String>, Transaction)],
    changed_folders: &std::collections::HashSet<String>,
) -> HashMap<&'a str, Vec<&'a Transaction>> {
    let mut per_folder: HashMap<&str, Vec<&Transaction>> = HashMap::new();
    for (_, folder_opt, txn) in tagged_txns {
        if let Some(folder_rel) = folder_opt {
            if changed_folders.contains(folder_rel) {
                per_folder
                    .entry(folder_rel.as_str())
                    .or_default()
                    .push(txn);
            }
        }
    }
    per_folder
}

/// Write the ledger and summary.json for one folder. Returns the number of
/// transactions written.
fn write_folder_ledger(
    config: &PipelineConfig,
    folder_rel: &str,
    txns: &[&Transaction],
    tracker: &mut OutputTracker,
) -> Result<usize, String> {
    let folder_dir = config.generated_dir.join(folder_rel);
    fs::create_dir_all(&folder_dir).map_err(|e| e.to_string())?;

    let mut text = String::new();
    let mut balances_map: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    for txn in txns {
        let serialised = transaction_to_text(txn);
        text.push_str(&serialised);
        if !serialised.ends_with('\n') {
            text.push('\n');
        }
        text.push('\n');
        for posting in &txn.postings {
            *balances_map
                .entry(posting.account.clone())
                .or_default()
                .entry(posting.commodity.clone())
                .or_insert(0.0) += posting.amount;
        }
    }

    tracker.write_if_changed(&folder_dir.join(FILE_LEDGER), &normalize_blank_lines(&text))?;

    let summary = FolderSummary {
        transaction_count: txns.len(),
        balances: balances_map
            .into_iter()
            .map(|(account, by_commodity)| AccountBalance {
                account,
                totals: by_commodity
                    .into_iter()
                    .map(|(commodity, amount)| CommodityAmount { commodity, amount })
                    .collect(),
            })
            .collect(),
    };
    let summary_json = serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?;
    tracker.write_if_changed(&folder_dir.join(FILE_SUMMARY), &summary_json)?;

    Ok(txns.len())
}

/// Mark ledger + summary files as live for all source and changed folders so
/// they are not pruned even if their content did not change this run.
fn mark_live_outputs(
    config: &PipelineConfig,
    source_folders: &[String],
    changed_folders: &std::collections::HashSet<String>,
    tracker: &mut OutputTracker,
) {
    let mark = |folder_rel: &str, tracker: &mut OutputTracker| {
        for file in &[FILE_LEDGER, FILE_SUMMARY] {
            let path = config.generated_dir.join(folder_rel).join(file);
            if let Ok(rel) = path.strip_prefix(&config.generated_dir) {
                tracker.live_outputs.insert(rel.to_string_lossy().to_string());
            }
        }
    };
    for folder_rel in source_folders {
        mark(folder_rel, tracker);
    }
    for folder_rel in changed_folders {
        mark(folder_rel, tracker);
    }
}

/// Write account declaration files for each account set (or the generated root
/// when no sets exist).
fn write_account_sets(
    config: &PipelineConfig,
    account_sets: &[String],
    root_accounts_text: &str,
    folder_accounts: &HashMap<String, String>,
    auto_accounts: &HashMap<String, BTreeSet<String>>,
    tracker: &mut OutputTracker,
) -> Result<(), String> {
    if account_sets.is_empty() && !root_accounts_text.is_empty() {
        tracker.write_if_changed(&config.generated_dir.join(FILE_ACCOUNTS), root_accounts_text)?;
        return Ok(());
    }
    for set_name in account_sets {
        let set_dir = config.generated_dir.join(set_name);
        fs::create_dir_all(&set_dir).map_err(|e| e.to_string())?;

        let mut accounts_text: String = auto_accounts
            .get(set_name)
            .into_iter()
            .flatten()
            .map(|a| format!("account {a}\n"))
            .collect();

        let folder_text = accounts_text_for_set(set_name, folder_accounts, root_accounts_text);
        if !folder_text.is_empty() {
            if !accounts_text.is_empty() {
                accounts_text.push('\n');
            }
            accounts_text.push_str(&folder_text);
        }
        if !accounts_text.is_empty() {
            tracker.write_if_changed(&set_dir.join(FILE_ACCOUNTS), &accounts_text)?;
        }
    }
    Ok(())
}

/// Make the folders under `generated/` agree with the folders this run writes,
/// before the early-exit check and before anything reads them. Two things the
/// build cache cannot see (TASK-093):
///
/// * A folder whose name differs from its source folder only by case. An older
///   pipeline wrote mixed-case wallet folders; on a case-insensitive filesystem
///   the lowercase write lands inside the old folder and git never sees a
///   rename, on a case-sensitive one it creates a sibling and both get read.
///   Each path component is renamed to the source folder's case, so every
///   filesystem ends up with one folder under the name the metadata records.
/// * A ledger folder no source folder owns: the stale sibling from the case
///   above, a folder a git checkout brought back, a source folder deleted on
///   another machine. Its ledger and summary are removed so reports cannot
///   count it, and the directory goes too once empty. A set-root ledger (the
///   legacy layout) is left alone.
///
/// Returns one warning per rename and per removal.
fn reconcile_generated_folders(generated_dir: &Path, source_folders: &[String]) -> Vec<String> {
    let mut warnings = Vec::new();
    for folder_rel in source_folders {
        warnings.extend(rename_to_source_case(generated_dir, folder_rel));
    }
    let owned: std::collections::HashSet<&str> =
        source_folders.iter().map(String::as_str).collect();
    let set_roots: std::collections::HashSet<String> = source_folders
        .iter()
        .filter_map(|f| extract_account_set(f))
        .collect();
    for entry in walkdir::WalkDir::new(generated_dir)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !e.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() || entry.file_name() != FILE_LEDGER {
            continue;
        }
        let Some(folder) = entry.path().parent() else {
            continue;
        };
        let Ok(rel) = folder.strip_prefix(generated_dir) else {
            continue;
        };
        let rel = rel.to_string_lossy();
        if rel.is_empty() || owned.contains(rel.as_ref()) || set_roots.contains(rel.as_ref()) {
            continue;
        }
        for file in [FILE_LEDGER, FILE_SUMMARY] {
            let _ = fs::remove_file(folder.join(file));
        }
        let _ = fs::remove_dir(folder);
        warnings.push(format!(
            "Removed stale ledger folder generated/{rel} (no source folder writes it)"
        ));
    }
    warnings
}

/// Rename each component of `generated/<folder_rel>` whose on-disk case differs
/// from the source folder's. `Path::exists` cannot tell (it matches
/// case-insensitively on APFS), so each parent is listed and compared exactly.
fn rename_to_source_case(generated_dir: &Path, folder_rel: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut dir = generated_dir.to_path_buf();
    for component in Path::new(folder_rel).components() {
        let wanted = component.as_os_str();
        let Ok(entries) = fs::read_dir(&dir) else {
            break;
        };
        let mut names: Vec<std::ffi::OsString> = entries.flatten().map(|e| e.file_name()).collect();
        names.sort();
        if names.iter().any(|n| n.as_os_str() == wanted) {
            dir.push(wanted);
            continue;
        }
        let wanted_lower = wanted.to_string_lossy().to_lowercase();
        let Some(variant) = names
            .iter()
            .find(|n| n.to_string_lossy().to_lowercase() == wanted_lower && dir.join(n).is_dir())
        else {
            break;
        };
        let from = dir.join(variant);
        let to = dir.join(wanted);
        if let Err(e) = fs::rename(&from, &to) {
            eprintln!(
                "[pipeline] could not rename {} to {}: {e}",
                from.display(),
                to.display()
            );
            break;
        }
        let rel = |p: &Path| {
            p.strip_prefix(generated_dir)
                .unwrap_or(p)
                .to_string_lossy()
                .to_string()
        };
        warnings.push(format!(
            "Renamed generated/{} to generated/{} (the source folder's case)",
            rel(&from),
            rel(&to)
        ));
        dir.push(wanted);
    }
    warnings
}

/// Remove files from `generated_dir` that were not marked live this run, then
/// clean up any now-empty parent directories.
fn prune_stale_outputs(config: &PipelineConfig, tracker: &mut OutputTracker) {
    let stale_keys: Vec<String> = tracker
        .output_hashes
        .keys()
        .filter(|k| !tracker.live_outputs.contains(k.as_str()))
        .cloned()
        .collect();
    let mut dirs_to_check = std::collections::BTreeSet::new();
    for key in &stale_keys {
        // On a case-insensitive filesystem a key that differs from a live one
        // only by case names the live file; deleting it would delete this
        // run's output. Drop the key, keep the file.
        let shadows_live = tracker
            .live_outputs
            .iter()
            .any(|live| live.eq_ignore_ascii_case(key));
        let stale_path = config.generated_dir.join(key);
        if !shadows_live && stale_path.exists() {
            if let Some(parent) = stale_path.parent() {
                dirs_to_check.insert(parent.to_path_buf());
            }
            let _ = fs::remove_file(&stale_path);
        }
        tracker.output_hashes.remove(key.as_str());
    }
    for dir in dirs_to_check.iter().rev() {
        if dir.exists() && dir.as_path() != config.generated_dir.as_path() {
            let _ = fs::remove_dir(dir);
        }
    }
}

/// Infer account_set from a manual transaction by matching its posting accounts
/// against known owner_accounts.
fn infer_account_set_from_txn(
    txn: &Transaction,
    owner_accounts: &HashMap<String, Vec<String>>,
) -> Option<String> {
    for posting in &txn.postings {
        for (owner, accounts) in owner_accounts {
            for known in accounts {
                if posting.account == *known || posting.account.starts_with(&format!("{known}:")) {
                    return Some(owner.clone());
                }
            }
        }
    }
    None
}

/// Account name used by `append_hide_rule` for the hidden-by-user rule's
/// contra account. The user's `config.json` lists `"ignore"` (or whatever
/// prefix matches) under `hidden_accounts`, so any account starting with
/// `ignore:` is filtered when "Show Ignored" is off — same UX as the
/// legacy `_ignored.txt` filter, no separate subsystem needed.
pub const HIDE_AMOUNT_ACCOUNT: &str = "ignore:hidden";

/// Append a meta-pattern rule into a folder's `_rules.json` so the
/// transaction with the given id is re-categorised to `ignore:hidden` on
/// the next pipeline run. No-op if a rule with the same pattern + match
/// field already exists.
///
/// Replaces the legacy `append_to_ignored` global file. By living in a
/// per-folder rules file the hide is naturally scoped — only the affected
/// folder's cache invalidates.
pub fn append_hide_rule(
    sources_dir: &Path,
    account_folder: &str,
    txn_id: &str,
) -> Result<bool, String> {
    let folder = sources_dir.join(account_folder);
    let mut rules = RulesFile::try_load(&folder)?;
    // Bare `txn:HASH` pattern with `match_field: "meta"`: matched
    // segment-exact against the comma-separated meta. See
    // `features/architecture/txn_id_rule_priority.feature`.
    let pattern = txn_id.to_string();
    let match_field = Some("meta".to_string());
    let amount_account = Some(HIDE_AMOUNT_ACCOUNT.to_string());

    let already = rules
        .rules
        .iter()
        .any(|r| r.pattern == pattern && r.match_field == match_field);
    if already {
        return Ok(false);
    }
    rules.insert_rule(Rule {
        id: generate_rule_id(&pattern),
        pattern,
        match_field,
        payee: None,
        commodity: None,
        comment: Some("Hidden by user".to_string()),
        amount_condition: None,
        fee_condition: None,
        payee_condition: None,
        narration_condition: None,
        commodity_condition: None,
        meta_condition: None,        amount_account,
        fee_account: None,
        postings: vec![],
    });
    rules.save(&folder)?;
    Ok(true)
}

/// Append a txn ID to _ignored.txt in the sources directory. Each write
/// rewrites the file in canonical form (deduped, original order, one id per
/// line) so any pre-existing duplicates get cleaned up. The write is skipped
/// entirely when the file is already canonical and contains the id, so the
/// pipeline cache stays valid for true no-ops.
pub fn append_to_ignored(sources_dir: &Path, txn_id: &str) -> Result<(), String> {
    let ignored_path = sources_dir.join(FILE_IGNORED);
    let existing = if ignored_path.exists() {
        fs::read_to_string(&ignored_path)
            .map_err(|e| format!("failed to read _ignored.txt: {e}"))?
    } else {
        String::new()
    };
    let mut seen: HashSet<String> = HashSet::new();
    let mut entries: Vec<String> = Vec::new();
    for line in existing.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
            entries.push(trimmed.to_string());
        }
    }
    if seen.insert(txn_id.to_string()) {
        entries.push(txn_id.to_string());
    }
    let canonical = if entries.is_empty() {
        String::new()
    } else {
        entries.join("\n") + "\n"
    };
    if canonical == existing {
        return Ok(());
    }
    fs::write(&ignored_path, canonical)
        .map_err(|e| format!("failed to write _ignored.txt: {e}"))
}

/// Delete a manual transaction from a folder's manual.transactions and rebuild.
pub fn delete_manual_transaction_and_rebuild(
    config: &PipelineConfig,
    datetime: &str,
    payee: &str,
    narration: &str,
    account_folder: &str,
) -> Result<PipelineResult, String> {
    validate_folder_name(account_folder)?;
    let manual_path = config.sources_dir.join(account_folder).join(FILE_MANUAL);
    if !manual_path.exists() {
        // Try root-level manual.transactions
        let root_manual = config.sources_dir.join(FILE_MANUAL);
        if root_manual.exists() {
            remove_manual_transaction(&root_manual, datetime, payee, narration)?;
        }
    } else {
        remove_manual_transaction(&manual_path, datetime, payee, narration)?;
    }
    run_pipeline(config)
}

/// Remove a specific manual transaction from a file by matching datetime/payee/narration.
fn remove_manual_transaction(
    path: &Path,
    datetime: &str,
    payee: &str,
    narration: &str,
) -> Result<(), String> {
    let contents =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;

    let result = parse_transactions(&contents);
    let mut new_text = String::new();
    let mut found = false;

    for txn in &result.transactions {
        if !found
            && txn.datetime == datetime
            && txn.payee.as_deref() == Some(payee)
            && txn.narration.as_deref() == Some(narration)
        {
            found = true;
            continue; // Skip this transaction
        }
        new_text.push_str(&transaction_to_text(txn));
        new_text.push('\n');
    }

    let text = normalize_blank_lines(&new_text);
    fs::write(path, text).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

/// Convenience: append a manual transaction to a folder's manual.transactions
/// then run the pipeline.
pub fn append_manual_and_rebuild(
    config: &PipelineConfig,
    input: &ManualTransactionInput,
    account_folder: &str,
) -> Result<PipelineResult, String> {
    validate_folder_name(account_folder)?;
    let folder_dir = config.sources_dir.join(account_folder);
    fs::create_dir_all(&folder_dir).map_err(|e| e.to_string())?;

    let manual_path = folder_dir.join(FILE_MANUAL);
    let text = manual_input_to_text(input);
    append_text(&manual_path, &text).map_err(|e| e.to_string())?;

    run_pipeline(config)
}

/// Convenience: append an account declaration to the account folder's
/// accounts.transactions then run the pipeline.
///
/// When `account_folder` is provided (e.g. from the frontend's resolved map),
/// it is used directly. Otherwise the folder is computed from the account name
/// and account set.
pub fn append_account_and_rebuild(
    config: &PipelineConfig,
    account_name: &str,
    currency: Option<&str>,
    opening_balance: Option<&str>,
    account_set: &str,
    account_folder: Option<&str>,
) -> Result<PipelineResult, String> {
    fs::create_dir_all(&config.sources_dir).map_err(|e| e.to_string())?;

    let folder_backed = under_folder_root(account_name);
    let folder = if let Some(f) = account_folder.filter(|s| !s.is_empty()) {
        f.to_string()
    } else if !folder_backed {
        // Accounts outside the folder roots (expenses/equity/income) are
        // declaration-only: no data folder. The declaration lives under the
        // set's `_accounts` folder, which the source-folder walk skips
        // (leading underscore) so it never derives an owner account or
        // becomes folder-backed — the account surfaces in the Categories pane.
        if account_set.is_empty() {
            "_accounts".to_string()
        } else {
            format!("{account_set}/_accounts")
        }
    } else {
        // Explicit layout: the folder mirrors the account name under the
        // entity — account_set="acme", account_name="assets:crypto:wallet:ethereum"
        // → "acme/assets/crypto/wallet/ethereum", and
        // "liabilities:credit-card:amex" → "acme/liabilities/credit-card/amex".
        // Round-trips through folder_to_account_name's FOLDER_ROOTS branch.
        let path = account_name.replace(':', "/");
        if account_set.is_empty() {
            path
        } else {
            format!("{account_set}/{path}")
        }
    };
    let folder_dir = config.sources_dir.join(&folder);
    fs::create_dir_all(&folder_dir)
        .map_err(|e| format!("failed to create folder dir {}: {e}", folder_dir.display()))?;

    // Write to the folder's accounts.transactions (not root-level)
    let accounts_path = folder_dir.join(FILE_ACCOUNTS);

    let mut text = format!("account {}", account_name);
    if let Some(curr) = currency {
        text.push(' ');
        text.push_str(curr);
    }
    text.push('\n');

    if let Some(balance) = opening_balance {
        if crate::ledger_parser::parse_amount(balance).is_none() {
            return Err(format!("invalid opening balance amount: {balance}"));
        }
        let curr = currency.unwrap_or("USD");
        text.push_str(&format!("    opening {} {}\n", balance, curr));
    }

    append_text(&accounts_path, &text).map_err(|e| e.to_string())?;

    if folder_backed {
        let imports_dir = folder_dir.join(DIR_IMPORTS);
        fs::create_dir_all(&imports_dir).map_err(|e| {
            format!(
                "failed to create imports dir {}: {e}",
                imports_dir.display()
            )
        })?;
    }

    run_pipeline(config)
}

/// Remove empty ancestor directories between `leaf` and `stop_at` (exclusive).
fn cleanup_empty_ancestors(leaf: &Path, stop_at: &Path) {
    let mut cur = leaf.to_path_buf();
    while cur.starts_with(stop_at) && cur != stop_at {
        if cur.exists() {
            match fs::read_dir(&cur) {
                Ok(mut entries) => {
                    if entries.next().is_some() {
                        break; // not empty
                    }
                    let _ = fs::remove_dir(&cur);
                }
                Err(_) => break,
            }
        }
        if !cur.pop() {
            break;
        }
    }
}

/// Rename an account's source folder and rebuild the pipeline.
/// Both `old_folder` and `new_folder` are relative to `sources_dir`
/// (e.g. "alex/binance" → "alex/binance/personal").
pub fn rename_account_folder_and_rebuild(
    config: &PipelineConfig,
    old_folder: &str,
    new_folder: &str,
) -> Result<PipelineResult, String> {
    validate_folder_name(old_folder)?;
    validate_folder_name(new_folder)?;

    let old_path = config.sources_dir.join(old_folder);
    let new_path = config.sources_dir.join(new_folder);

    if !old_path.exists() {
        return Err(format!("source folder '{}' does not exist", old_folder));
    }
    if new_path.exists() {
        return Err(format!("target folder '{}' already exists", new_folder));
    }

    // Handle rename-into-subdirectory (e.g. binance/ → binance/personal/)
    // by moving to a temp dir first, then to the final location.
    if new_path.starts_with(&old_path) {
        let temp_name = format!("__rename_tmp_{}", std::process::id());
        let temp_path = config.sources_dir.join(&temp_name);
        fs::rename(&old_path, &temp_path)
            .map_err(|e| format!("failed to move '{}' to temp: {e}", old_folder))?;
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create parent dirs for '{}': {e}", new_folder))?;
        }
        fs::rename(&temp_path, &new_path)
            .map_err(|e| format!("failed to move temp to '{}': {e}", new_folder))?;
    } else {
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create parent dirs for '{}': {e}", new_folder))?;
        }
        fs::rename(&old_path, &new_path)
            .map_err(|e| format!("failed to rename '{}' to '{}': {e}", old_folder, new_folder))?;
    }

    // Clean up empty ancestor directories left behind
    cleanup_empty_ancestors(
        old_path.parent().unwrap_or(&config.sources_dir),
        &config.sources_dir,
    );

    run_pipeline(config)
}

/// Delete an account's source folder and rebuild the pipeline.
/// `folder` is relative to `sources_dir` (e.g. "alex/binance").
pub fn delete_account_folder_and_rebuild(
    config: &PipelineConfig,
    folder: &str,
) -> Result<PipelineResult, String> {
    validate_folder_name(folder)?;

    let folder_path = config.sources_dir.join(folder);
    if !folder_path.exists() {
        return Err(format!("source folder '{}' does not exist", folder));
    }

    fs::remove_dir_all(&folder_path).map_err(|e| format!("failed to delete '{}': {e}", folder))?;

    // Clean up empty ancestor directories
    cleanup_empty_ancestors(
        folder_path.parent().unwrap_or(&config.sources_dir),
        &config.sources_dir,
    );

    run_pipeline(config)
}

/// Find the accounts.transactions file that declares the given account name.
/// Starts from the derived folder path and walks up ancestor directories
/// until it finds an accounts.transactions containing the account declaration,
/// stopping at the sources_dir boundary.
fn find_accounts_file(
    sources_dir: &Path,
    account_name: &str,
    account_set: &str,
) -> Result<PathBuf, String> {
    let parts: Vec<&str> = account_name.split(':').collect();
    let short_name = if parts.len() > 1 {
        parts[1..].join("/")
    } else {
        account_name.to_string()
    };
    let folder = if account_set.is_empty() {
        short_name
    } else {
        format!("{}/{}", account_set, short_name)
    };

    // Walk from the deepest derived path upward toward sources_dir
    let mut candidate = sources_dir.join(&folder);
    loop {
        let accounts_path = candidate.join(FILE_ACCOUNTS);
        if accounts_path.exists() {
            // Check if this file actually declares the target account
            let contents = fs::read_to_string(&accounts_path)
                .map_err(|e| format!("failed to read {}: {e}", accounts_path.display()))?;
            let decl_prefix = format!("account {}", account_name);
            if contents.lines().any(|line| {
                let trimmed = line.trim();
                trimmed == decl_prefix || trimmed.starts_with(&format!("{} ", decl_prefix))
            }) {
                return Ok(accounts_path);
            }
        }
        // Don't walk above the sources directory
        if candidate == sources_dir {
            break;
        }
        candidate = match candidate.parent() {
            Some(p) if p.starts_with(sources_dir) => p.to_path_buf(),
            _ => break,
        };
    }

    // No existing file found — create the deepest folder and accounts file
    let deepest = sources_dir.join(&folder);
    fs::create_dir_all(&deepest)
        .map_err(|e| format!("failed to create {}: {e}", deepest.display()))?;
    let accounts_path = deepest.join(FILE_ACCOUNTS);
    let content = format!("account {}\n", account_name);
    fs::write(&accounts_path, &content)
        .map_err(|e| format!("failed to create {}: {e}", accounts_path.display()))?;
    Ok(accounts_path)
}

/// Update the friendly name for an account by rewriting the folder's accounts.transactions.
/// Inserts or replaces the `    name <value>` line under the matching account declaration.
pub fn update_account_name(
    config: &PipelineConfig,
    account_name: &str,
    friendly_name: &str,
    account_set: &str,
) -> Result<PipelineResult, String> {
    update_account_property(config, account_name, "name", friendly_name, account_set)
}

pub fn update_opening_balance(
    config: &PipelineConfig,
    account_name: &str,
    amount: &str,
    commodity: &str,
    account_set: &str,
) -> Result<PipelineResult, String> {
    if crate::ledger_parser::parse_amount(amount).is_none() {
        return Err(format!("invalid opening balance amount: {amount}"));
    }
    let value = format!("{} {}", amount, commodity);
    update_account_property(config, account_name, "opening", &value, account_set)
}

fn update_account_property(
    config: &PipelineConfig,
    account_name: &str,
    property_kind: &str,
    property_value: &str,
    account_set: &str,
) -> Result<PipelineResult, String> {
    let accounts_path = find_accounts_file(&config.sources_dir, account_name, account_set)?;

    let contents = fs::read_to_string(&accounts_path)
        .map_err(|e| format!("failed to read {}: {e}", accounts_path.display()))?;

    let new_line = format!("    {} {}", property_kind, property_value);
    let mut output = String::new();
    let mut in_target_account = false;
    let mut property_written = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("account ") || trimmed == "account" {
            if in_target_account && !property_written {
                output.push_str(&new_line);
                output.push('\n');
                property_written = true;
            }
            let decl_name = trimmed.split_whitespace().nth(1).unwrap_or("");
            in_target_account = decl_name == account_name;
            output.push_str(line);
            output.push('\n');
            continue;
        }
        if line.starts_with("    ") && in_target_account {
            let child_kind = line.split_whitespace().next().unwrap_or("");
            if child_kind == property_kind {
                output.push_str(&new_line);
                output.push('\n');
                property_written = true;
                continue;
            }
        }
        if trimmed.is_empty() && in_target_account && !property_written {
            output.push_str(&new_line);
            output.push('\n');
            property_written = true;
        }
        output.push_str(line);
        output.push('\n');
    }

    if in_target_account && !property_written {
        output.push_str(&new_line);
        output.push('\n');
    }

    fs::write(&accounts_path, &output)
        .map_err(|e| format!("failed to write {}: {e}", accounts_path.display()))?;

    run_pipeline(config)
}

/// Import a CSV file into the sources directory under the given account folder,
/// then run the pipeline.
pub fn import_csv_to_sources(
    config: &PipelineConfig,
    source_path: &Path,
    account_folder: &str,
) -> Result<PipelineResult, String> {
    let filename = source_path
        .file_name()
        .ok_or_else(|| "source path has no filename".to_string())?;
    let dest_dir = config.sources_dir.join(account_folder);
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("failed to create dest dir {}: {e}", dest_dir.display()))?;
    let dest = dest_dir.join(filename);
    fs::copy(source_path, &dest)
        .map_err(|e| format!("failed to copy CSV to {}: {e}", dest.display()))?;
    run_pipeline(config)
}

/// Process pending imports in the imports/ subdirectory for a single account folder.
/// Moves files from imports/ into the parent folder, then runs the pipeline.
pub fn process_imports(
    config: &PipelineConfig,
    account_folder: &str,
) -> Result<(ImportResult, PipelineResult), String> {
    validate_folder_name(account_folder)?;
    let folder_dir = config.sources_dir.join(account_folder);
    let imports_dir = folder_dir.join(DIR_IMPORTS);

    let mut import_result = ImportResult {
        files_processed: Vec::new(),
        files_skipped: Vec::new(),
        warnings: Vec::new(),
    };

    if imports_dir.exists() {
        let entries =
            fs::read_dir(&imports_dir).map_err(|e| format!("failed to read imports dir: {e}"))?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                // Skip hidden files
                if filename.starts_with('.') {
                    continue;
                }
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "csv" || ext == "ofx" {
                    let dest = folder_dir.join(&filename);
                    fs::rename(&path, &dest).map_err(|e| {
                        format!(
                            "failed to move {} to {}: {e}",
                            path.display(),
                            dest.display()
                        )
                    })?;
                    import_result.files_processed.push(filename);
                } else {
                    import_result.files_skipped.push(filename);
                }
            }
        }
    }

    let pipeline_result = run_pipeline(config)?;
    Ok((import_result, pipeline_result))
}

/// Process pending imports across ALL account folders.
pub fn process_all_imports(
    config: &PipelineConfig,
) -> Result<(ImportResult, PipelineResult), String> {
    let mut combined_import = ImportResult {
        files_processed: Vec::new(),
        files_skipped: Vec::new(),
        warnings: Vec::new(),
    };

    if config.sources_dir.exists() {
        let entries = fs::read_dir(&config.sources_dir)
            .map_err(|e| format!("failed to read sources dir: {e}"))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let folder_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if folder_name.starts_with('_') || folder_name.starts_with('.') {
                    continue;
                }
                let imports_dir = path.join(DIR_IMPORTS);
                if imports_dir.exists() {
                    let sub_entries = fs::read_dir(&imports_dir)
                        .map_err(|e| format!("failed to read imports dir: {e}"))?;
                    for sub in sub_entries {
                        let sub = sub.map_err(|e| e.to_string())?;
                        let sub_path = sub.path();
                        if sub_path.is_file() {
                            let filename = sub_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            if filename.starts_with('.') {
                                continue;
                            }
                            let ext = sub_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if ext == "csv" || ext == "ofx" {
                                let dest = path.join(&filename);
                                fs::rename(&sub_path, &dest).map_err(|e| {
                                    format!("failed to move {}: {e}", sub_path.display())
                                })?;
                                combined_import
                                    .files_processed
                                    .push(format!("{}/{}", folder_name, filename));
                            } else {
                                combined_import
                                    .files_skipped
                                    .push(format!("{}/{}", folder_name, filename));
                            }
                        }
                    }
                }
            }
        }
    }

    let pipeline_result = run_pipeline(config)?;
    Ok((combined_import, pipeline_result))
}

/// Detect gaps in monthly data per account by scanning archive ledger files.
pub fn detect_account_gaps(generated_dir: &Path) -> Result<Vec<AccountGap>, String> {
    let archive_dir = generated_archive_dir(generated_dir);
    if !archive_dir.exists() {
        return Ok(Vec::new());
    }

    // account → set of YYYYMM months
    let mut account_months: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let entries =
        fs::read_dir(&archive_dir).map_err(|e| format!("failed to read archive dir: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !filename.ends_with(".transactions") || !filename.starts_with("ledger-") {
            continue;
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let result = parse_transactions(&contents);

        for txn in &result.transactions {
            for posting in &txn.postings {
                if under_folder_root(&posting.account) {
                    if let Some(yyyymm) = yyyymm_from_date(&txn.date) {
                        account_months
                            .entry(posting.account.clone())
                            .or_default()
                            .push(yyyymm);
                    }
                }
            }
        }
    }

    let mut gaps = Vec::new();
    for (account, mut months) in account_months {
        months.sort();
        months.dedup();

        if months.is_empty() {
            continue;
        }

        let first = months.first().unwrap().clone();
        let last = months.last().unwrap().clone();

        // Generate all expected months between first and last
        let expected = generate_month_range(&first, &last);
        let missing: Vec<String> = expected
            .into_iter()
            .filter(|m| !months.contains(m))
            .collect();

        gaps.push(AccountGap {
            account,
            first_month: first,
            last_month: last,
            missing_months: missing,
        });
    }

    Ok(gaps)
}

/// Generate all YYYYMM values between start and end (inclusive).
fn generate_month_range(start: &str, end: &str) -> Vec<String> {
    let mut result = Vec::new();
    if start.len() != 6 || end.len() != 6 {
        return result;
    }

    let start_year: i32 = match start[0..4].parse() {
        Ok(y) => y,
        Err(_) => return result,
    };
    let start_month: i32 = match start[4..6].parse() {
        Ok(m) => m,
        Err(_) => return result,
    };
    let end_year: i32 = match end[0..4].parse() {
        Ok(y) => y,
        Err(_) => return result,
    };
    let end_month: i32 = match end[4..6].parse() {
        Ok(m) => m,
        Err(_) => return result,
    };

    let mut y = start_year;
    let mut m = start_month;

    loop {
        result.push(format!("{y:04}{m:02}"));
        if y == end_year && m == end_month {
            break;
        }
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
        if y > end_year + 1 {
            break; // Safety limit
        }
    }

    result
}

/// Existing `_transform.rhai` scripts under `sources_dir`, ranked best-first for
/// `account_folder` by longest shared leading path-component prefix (ties broken
/// toward the shallower folder). Only includes candidates with a meaningful
/// overlap (>= 2 components, e.g. same owner + category).
fn ranked_existing_transforms(sources_dir: &Path, account_folder: &str) -> Vec<String> {
    let target: Vec<&str> = account_folder
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();
    if target.len() < 2 {
        return Vec::new();
    }
    let mut cands: Vec<(usize, usize, String)> = Vec::new(); // (shared, depth, content)
    let mut walker = vec![sources_dir.to_path_buf()];
    while let Some(dir) = walker.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walker.push(path);
                continue;
            }
            if !path.file_name().map(|n| n == FILE_TRANSFORM).unwrap_or(false) {
                continue;
            }
            let Some(folder_rel) = path
                .parent()
                .and_then(|p| p.strip_prefix(sources_dir).ok())
                .map(|p| p.to_string_lossy().to_string())
            else {
                continue;
            };
            let comps: Vec<&str> = folder_rel
                .split(['/', '\\'])
                .filter(|s| !s.is_empty())
                .collect();
            let shared = target
                .iter()
                .zip(comps.iter())
                .take_while(|(a, b)| a == b)
                .count();
            if shared < 2 {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                cands.push((shared, comps.len(), content));
            }
        }
    }
    // Highest shared prefix first; among equals, the shallower folder first.
    cands.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    cands.into_iter().map(|(_, _, c)| c).collect()
}

/// Returns true if `script` actually transforms `csv_path` — it compiles and,
/// for a sample of rows, evaluates without error and yields transaction-shaped
/// output (a date plus postings/amount, or an explicit `skip`). A column-count
/// mismatch or any eval error fails the check. Used so the import dialog only
/// defaults to an existing transform when that transform fits the CSV at hand.
fn transform_parses_csv(script: &str, csv_path: &Path, account_folder: &str) -> bool {
    let mut engine = rhai::Engine::new();
    engine.register_fn("is_alphabetic", |c: char| c.is_alphabetic());
    let Ok(ast) = engine.compile(script) else {
        return false;
    };
    let Ok(mut reader) = csv::Reader::from_path(csv_path) else {
        return false;
    };
    let headers: Vec<String> = match reader.headers() {
        Ok(h) => h.iter().map(|s| s.to_string()).collect(),
        Err(_) => return false,
    };
    let account_name = folder_to_account_name(account_folder);
    let rel = csv_path.to_string_lossy().to_string();
    let mut ok_rows = 0usize;
    for (i, rec) in reader.records().enumerate() {
        if i >= 50 {
            break;
        }
        let Ok(record) = rec else {
            return false; // malformed / inconsistent column count vs header
        };
        let mut row_map = rhai::Map::new();
        for (j, header) in headers.iter().enumerate() {
            let value = record.get(j).unwrap_or("");
            row_map.insert(header.clone().into(), value.to_string().into());
        }
        row_map.insert("_row_index".into(), (i as rhai::INT).into());
        row_map.insert("_source_path".into(), rel.clone().into());
        row_map.insert("_account".into(), account_name.clone().into());

        let mut scope = rhai::Scope::new();
        scope.push("row", row_map);
        let result: rhai::Map = match engine.eval_ast_with_scope(&mut scope, &ast) {
            Ok(m) => m,
            Err(_) => return false,
        };
        // Rows the transform deliberately drops are fine.
        if result
            .get("skip")
            .and_then(|v| v.clone().as_bool().ok())
            .unwrap_or(false)
        {
            ok_rows += 1;
            continue;
        }
        let has_date = result
            .get("date")
            .and_then(|v| v.clone().into_string().ok())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        let has_amount_or_postings =
            result.contains_key("postings") || result.contains_key("amount");
        if !has_date || !has_amount_or_postings {
            return false;
        }
        ok_rows += 1;
    }
    ok_rows > 0
}

/// Check if a transform exists for an account folder. If not, default to the
/// nearest existing transform that actually parses this CSV (so the import dialog
/// shows the one it finds); only generate a FIXME scaffold when none exists or
/// none fit. Returns Ok(None) if a transform already applies to the folder,
/// Ok(Some(script)) with the script the dialog should display otherwise.
pub fn suggest_transform_for_import(
    config: &PipelineConfig,
    source_path: &Path,
    account_folder: &str,
    currency: Option<&str>,
) -> Result<Option<String>, String> {
    validate_folder_name(account_folder)?;

    // OFX files are parsed natively (ofx_parser + rules), never via a Rhai
    // transform — importing one needs no suggestion regardless of folder.
    if source_path.extension().and_then(|e| e.to_str()) == Some("ofx") {
        return Ok(None);
    }

    let dest_dir = config.sources_dir.join(account_folder);
    let transform_path = dest_dir.join(FILE_TRANSFORM);

    if transform_path.exists() {
        return Ok(None);
    }

    // Also check parent directories up to sources_dir (same logic as resolve_transform).
    // Build the canonical dest path from the canonical sources_dir so this works even
    // when the destination folder doesn't exist yet (new import target).
    let sources_canonical = config
        .sources_dir
        .canonicalize()
        .map_err(|e| format!("cannot canonicalize sources dir: {e}"))?;
    let dest_canonical = sources_canonical.join(account_folder);

    {
        let mut current = dest_canonical.as_path();
        loop {
            let candidate = current.join(FILE_TRANSFORM);
            if candidate.exists() {
                return Ok(None);
            }
            if current == sources_canonical {
                break;
            }
            current = match current.parent() {
                Some(p) => p,
                None => break,
            };
        }
    }

    // No transform applies to this folder or its parents. Prefer reusing the
    // nearest existing transform (e.g. a sibling account for the same broker),
    // but only one that actually parses this CSV — so the dialog defaults to a
    // real, working script rather than a blank FIXME scaffold. Fall back to a
    // generated scaffold when nothing related fits.
    for candidate in ranked_existing_transforms(&config.sources_dir, account_folder) {
        if transform_parses_csv(&candidate, source_path, account_folder) {
            return Ok(Some(candidate));
        }
    }

    let script = crate::transform_suggest::generate_suggestion(source_path, currency)?;
    Ok(Some(script))
}

/// Save a transform script, copy the CSV, and run the pipeline.
pub fn save_transform_and_rebuild(
    config: &PipelineConfig,
    source_path: &Path,
    account_folder: &str,
    script: &str,
) -> Result<PipelineResult, String> {
    // Validate Rhai syntax before saving
    let mut engine = rhai::Engine::new();
    engine.register_fn("is_alphabetic", |c: char| c.is_alphabetic());
    let ast = engine
        .compile(script)
        .map_err(|e| format!("Rhai syntax error: {e}"))?;

    // Dry-run: evaluate the script against the first CSV row to catch runtime errors
    if source_path.extension().map(|e| e == "csv").unwrap_or(false) {
        if let Ok((headers, sample_rows)) =
            crate::transform_suggest::read_csv_sample(source_path, 1)
        {
            if let Some(first_row) = sample_rows.first() {
                let mut row_map = rhai::Map::new();
                for (i, header) in headers.iter().enumerate() {
                    let value = first_row.get(i).map(|s| s.as_str()).unwrap_or("");
                    row_map.insert(header.clone().into(), value.to_string().into());
                }
                row_map.insert("_row_index".into(), (0 as rhai::INT).into());
                row_map.insert(
                    "_source_path".into(),
                    source_path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default()
                        .into(),
                );
                row_map.insert("_account".into(), "test".to_string().into());

                let mut scope = rhai::Scope::new();
                scope.push("row", row_map);
                if let Err(e) = engine.eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast) {
                    return Err(format!("Rhai transform error at row 1: {e}"));
                }
            }
        }
    }

    validate_folder_name(account_folder)?;
    let dest_dir = config.sources_dir.join(account_folder);
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("failed to create dest dir {}: {e}", dest_dir.display()))?;

    let transform_path = dest_dir.join(FILE_TRANSFORM);
    fs::write(&transform_path, script).map_err(|e| format!("failed to write transform: {e}"))?;

    let filename = source_path
        .file_name()
        .ok_or_else(|| "source path has no filename".to_string())?;
    let dest = dest_dir.join(filename);
    fs::copy(source_path, &dest)
        .map_err(|e| format!("failed to copy CSV to {}: {e}", dest.display()))?;

    run_pipeline(config)
}

/// Stamp a `txn:man-…` id on every transaction in one `manual.transactions`
/// file that carries no `txn:` tag of its own.
///
/// The id hashes the file path and the transaction's content (datetime,
/// payee, narration, postings) but never its position in the file, so
/// inserting or deleting other transactions leaves it unchanged and the
/// `txn:` rules that point at it (hide rules, trade links) keep matching.
/// Identical transactions are told apart by their occurrence count in file
/// order; they are the one case where removing an earlier entry renumbers a
/// later one.
fn stamp_manual_txn_ids(rel_path: &str, txns: &mut [Transaction]) {
    let mut occurrences: HashMap<String, usize> = HashMap::new();
    for txn in txns.iter_mut() {
        if txn.meta.as_ref().is_some_and(|m| m.contains("txn:")) {
            continue;
        }
        let mut content = format!(
            "{rel_path}:{}:{}:{}",
            txn.datetime,
            txn.payee.as_deref().unwrap_or(""),
            txn.narration.as_deref().unwrap_or("")
        );
        for p in &txn.postings {
            content.push_str(&format!(":{}:{}:{}", p.account, p.amount_text, p.commodity));
        }
        let occurrence = occurrences.entry(content.clone()).or_insert(0);
        let digest = hex::encode(Sha256::digest(format!("{content}#{occurrence}")));
        *occurrence += 1;
        let tag = format!("txn:man-{}", &digest[..12]);
        txn.meta = Some(match txn.meta.take() {
            Some(existing) if !existing.trim().is_empty() => format!("{existing}, {tag}"),
            _ => tag,
        });
    }
}

#[cfg(test)]
mod manual_txn_id_tests {
    use super::*;

    const FIRST: &str = "2025-01-10 * \"First\" \"one\" ;\n    assets:bank 1.00 AUD\n    expenses:manual -1.00 AUD\n\n";
    const SECOND: &str = "2025-01-11 * \"Second\" \"two\" ;\n    assets:bank 1.00 AUD\n    expenses:manual -1.00 AUD\n\n";
    const THIRD: &str = "2025-01-12 * \"Third\" \"three\" ;\n    assets:bank 1.00 AUD\n    expenses:manual -1.00 AUD\n\n";

    fn stamped_metas(text: &str) -> Vec<String> {
        let mut txns = parse_transactions(text).transactions;
        stamp_manual_txn_ids("bank/manual.transactions", &mut txns);
        txns.into_iter()
            .map(|t| t.meta.expect("every manual transaction is stamped"))
            .collect()
    }

    #[test]
    fn deleting_an_earlier_transaction_keeps_later_ids() {
        let before = stamped_metas(&format!("{FIRST}{SECOND}{THIRD}"));
        let after = stamped_metas(&format!("{SECOND}{THIRD}"));
        assert_eq!(&after[..], &before[1..]);
    }

    #[test]
    fn inserting_an_earlier_transaction_keeps_later_ids() {
        let before = stamped_metas(&format!("{SECOND}{THIRD}"));
        let after = stamped_metas(&format!("{FIRST}{SECOND}{THIRD}"));
        assert_eq!(&after[1..], &before[..]);
    }

    #[test]
    fn identical_transactions_get_distinct_stable_ids() {
        let ids = stamped_metas(&format!("{FIRST}{FIRST}"));
        assert_ne!(ids[0], ids[1]);
        assert_eq!(ids, stamped_metas(&format!("{FIRST}{FIRST}")));
    }

    #[test]
    fn stamped_tag_is_bare_when_the_meta_was_empty() {
        let ids = stamped_metas(FIRST);
        assert!(ids[0].starts_with("txn:man-"), "got {:?}", ids[0]);
        assert_eq!(ids[0].len(), "txn:man-".len() + 12);
    }

    #[test]
    fn an_explicit_txn_tag_is_kept() {
        let text = "2025-01-10 * \"First\" \"one\" ; txn:keep-me\n    assets:bank 1.00 AUD\n    expenses:manual -1.00 AUD\n";
        assert_eq!(stamped_metas(text), vec!["txn:keep-me".to_string()]);
    }
}

/// Render a manual-transaction form input as ledger text: one header line,
/// then one line per posting.
fn manual_input_to_text(input: &ManualTransactionInput) -> String {
    let mut header = input.datetime.clone();
    header.push(' ');

    if let Some(status) = input.status {
        header.push(status);
        header.push(' ');
    }

    header.push_str(&quote(&input.payee));
    header.push(' ');
    header.push_str(&quote(&input.narration));

    let mut lines = Vec::with_capacity(1 + input.postings.len() + 1);
    lines.push(header);
    for p in &input.postings {
        let mut posting_line = format!("    {} {} {}", p.account, p.amount, p.commodity);
        if let Some(rem) = p.remainder.as_ref().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }) {
            posting_line.push(' ');
            posting_line.push_str(rem);
        }
        lines.push(posting_line);
    }
    lines.push(String::new());
    lines.join("\n")
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceImportResult {
    pub commodities: Vec<String>,
    pub total_count: usize,
}

/// Import a prices file (P-directive or CSV format), auto-detect format,
/// group by commodity, and write per-commodity files under `sources/_prices/`.
/// When `merge` is true, existing entries survive unless the import quotes the
/// same datetime in the same quote currency (see `write_price_file`).
/// When false, existing per-commodity files are replaced.
pub fn import_prices_file(
    sources_dir: &Path,
    file_path: &Path,
    merge: bool,
) -> Result<PriceImportResult, String> {
    let contents =
        fs::read_to_string(file_path).map_err(|e| format!("failed to read prices file: {e}"))?;

    // Auto-detect format: if first non-blank/non-comment line starts with 'P ' → P-directive
    let is_p_directive = contents
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with(';'))
        .map(|l| l.starts_with("P "))
        .unwrap_or(false);

    let prices = if is_p_directive {
        parse_prices(&contents).prices
    } else {
        parse_prices_csv(&contents).prices
    };
    if prices.is_empty() {
        return Err("no valid price directives".to_string());
    }
    let total_count = prices.len();

    // Group by commodity; a BTreeMap keeps the files written, and reported, in name order.
    let mut grouped: BTreeMap<String, Vec<PriceDirective>> = BTreeMap::new();
    for p in prices {
        grouped.entry(p.commodity.clone()).or_default().push(p);
    }

    let commodities: Vec<String> = grouped.keys().cloned().collect();
    for (commodity, directives) in grouped {
        write_price_file(sources_dir, &commodity, directives, merge)?;
    }

    Ok(PriceImportResult {
        total_count,
        commodities,
    })
}

/// Set (or replace) a single price directive for a commodity at a given datetime.
/// Writes to `sources/_prices/{commodity}.txt`, replacing only the entry with the
/// same date in the same quote currency (see `write_price_file`).
pub fn set_price_directive(
    sources_dir: &Path,
    commodity: &str,
    datetime: &str,
    price_amount_text: &str,
    quote_commodity: &str,
) -> Result<(), String> {
    // Use only the date portion (YYYY-MM-DD) — a space-separated time component
    // would break the P directive format which is whitespace-tokenized.
    let date_only = &datetime[..datetime.len().min(10)];

    let price_amount: f64 = price_amount_text
        .parse()
        .map_err(|e| format!("invalid price: {e}"))?;

    let directive = PriceDirective {
        datetime: date_only.to_string(),
        commodity: commodity.to_string(),
        price_amount,
        price_amount_text: price_amount_text.to_string(),
        quote_commodity: quote_commodity.to_string(),
    };
    write_price_file(sources_dir, commodity, vec![directive], true)
}

/// The identity of a quote within one commodity's price file. A file can hold
/// `P 2024-06-15 HNT 4.00 AUD` and `P 2024-06-15 HNT 2.50 USD` side by side, so
/// the datetime alone does not identify an entry — the quote currency is part of it.
fn price_key(d: &PriceDirective) -> (&str, &str) {
    (d.datetime.as_str(), d.quote_commodity.as_str())
}

/// Write `directives` to `sources/_prices/{commodity}.txt` in P-directive format,
/// sorted by `price_key`. With `merge`, entries already in the file survive unless a
/// new directive has the same `price_key`; a file that exists but cannot be read is
/// an error, never treated as empty, so a merge can't silently drop price history.
fn write_price_file(
    sources_dir: &Path,
    commodity: &str,
    mut directives: Vec<PriceDirective>,
    merge: bool,
) -> Result<(), String> {
    let prices_dir = sources_dir.join(DIR_PRICES);
    fs::create_dir_all(&prices_dir).map_err(|e| format!("failed to create prices dir: {e}"))?;
    let dest_path = prices_dir.join(format!("{}.txt", price_file_stem(commodity)));

    if merge && dest_path.exists() {
        let existing = fs::read_to_string(&dest_path)
            .map_err(|e| format!("failed to read {}: {e}", dest_path.display()))?;
        let new_keys: HashSet<(&str, &str)> = directives.iter().map(price_key).collect();
        let kept: Vec<PriceDirective> = parse_prices(&existing)
            .prices
            .into_iter()
            .filter(|p| !new_keys.contains(&price_key(p)))
            .collect();
        directives.extend(kept);
    }

    directives.sort_by(|a, b| price_key(a).cmp(&price_key(b)));

    let output: String = directives
        .iter()
        .map(|d| {
            format!(
                "P {} {} {} {}\n",
                d.datetime, d.commodity, d.price_amount_text, d.quote_commodity
            )
        })
        .collect();

    fs::write(&dest_path, output)
        .map_err(|e| format!("failed to write {}: {e}", dest_path.display()))
}

#[cfg(test)]
mod reconcile_generated_folders_tests {
    use super::*;

    fn write_ledger(generated: &Path, folder_rel: &str) {
        let dir = generated.join(folder_rel);
        fs::create_dir_all(&dir).expect("create folder");
        fs::write(
            dir.join(FILE_LEDGER),
            "2025-01-01 * \"x\" \"y\" ; txn:a\n    assets:x 1 AUD\n    expenses:unknown -1 AUD\n",
        )
        .expect("write ledger");
        fs::write(dir.join(FILE_SUMMARY), "{}").expect("write summary");
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .expect("read dir")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn wrong_case_folder_is_renamed_to_the_source_folder_case() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let generated = tmp.path();
        write_ledger(generated, "alex/x/Foo");

        let warnings = reconcile_generated_folders(generated, &["alex/x/foo".to_string()]);

        assert_eq!(names(&generated.join("alex/x")), vec!["foo"]);
        assert!(generated.join("alex/x/foo").join(FILE_LEDGER).is_file());
        assert_eq!(
            warnings,
            vec!["Renamed generated/alex/x/Foo to generated/alex/x/foo (the source folder's case)".to_string()]
        );
    }

    #[test]
    fn case_variant_sibling_is_removed_on_a_case_sensitive_filesystem() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let generated = tmp.path();
        write_ledger(generated, "alex/x/foo");
        write_ledger(generated, "alex/x/Foo");
        if names(&generated.join("alex/x")).len() == 1 {
            // Case-insensitive filesystem: the second write landed in the
            // first folder, so the sibling case cannot be set up here.
            eprintln!("case-insensitive filesystem; the sibling case is covered on Linux");
            return;
        }

        let warnings = reconcile_generated_folders(generated, &["alex/x/foo".to_string()]);

        assert_eq!(names(&generated.join("alex/x")), vec!["foo"]);
        assert_eq!(
            warnings,
            vec!["Removed stale ledger folder generated/alex/x/Foo (no source folder writes it)".to_string()]
        );
    }

    #[test]
    fn only_folders_no_source_folder_owns_are_removed() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let generated = tmp.path();
        write_ledger(generated, "alex/x/foo");
        write_ledger(generated, "alex/x/old");
        // Legacy set-root ledger and the build cache are not the pipeline's folders.
        write_ledger(generated, "alex");
        write_ledger(generated, ".cache/alex/x/stray");

        let warnings = reconcile_generated_folders(generated, &["alex/x/foo".to_string()]);

        assert!(generated.join("alex/x/foo").join(FILE_LEDGER).is_file());
        assert!(generated.join("alex").join(FILE_LEDGER).is_file());
        assert!(generated.join(".cache/alex/x/stray").join(FILE_LEDGER).is_file());
        assert!(!generated.join("alex/x/old").exists(), "stale folder should be gone");
        assert_eq!(
            warnings,
            vec!["Removed stale ledger folder generated/alex/x/old (no source folder writes it)".to_string()]
        );
    }
}

#[cfg(test)]
mod folder_account_tests {
    use super::*;

    #[test]
    fn legacy_shorthand_derives_under_assets() {
        assert_eq!(
            folder_to_account_name("alex/cash/bank/cba/savings"),
            "assets:cash:bank:cba:savings"
        );
        assert_eq!(folder_to_account_name("alex-savings"), "assets:savings");
        assert_eq!(folder_to_account_name("savings"), "assets:savings");
        // `equity` is an asset-class segment here, never a root.
        assert_eq!(
            folder_to_account_name("alex/equity/broker/commsec"),
            "assets:equity:broker:commsec"
        );
    }

    #[test]
    fn explicit_assets_layout_mirrors_the_account() {
        assert_eq!(
            folder_to_account_name("acme/assets/crypto/wallet/0xAbC"),
            "assets:crypto:wallet:0xAbC"
        );
        assert_eq!(folder_to_account_name("acme/assets"), "assets");
    }

    #[test]
    fn explicit_liabilities_layout_switches_root() {
        assert_eq!(
            folder_to_account_name("alex/liabilities/credit-card/amex"),
            "liabilities:credit-card:amex"
        );
        assert_eq!(folder_to_account_name("alex/liabilities"), "liabilities");
        // Only the second path segment is a root marker.
        assert_eq!(
            folder_to_account_name("alex/cash/liabilities"),
            "assets:cash:liabilities"
        );
        assert_eq!(folder_to_account_name("liabilities"), "assets:liabilities");
    }

    #[test]
    fn under_folder_root_covers_both_roots_only() {
        assert!(under_folder_root("assets:cash"));
        assert!(under_folder_root("liabilities:credit-card:amex"));
        assert!(!under_folder_root("assets"));
        assert!(!under_folder_root("liabilities"));
        assert!(!under_folder_root("expenses:food"));
        assert!(!under_folder_root("income:salary"));
        assert!(!under_folder_root("equity:opening"));
        assert!(!under_folder_root("assetsx:y"));
    }

    #[test]
    fn lookup_matches_a_declaration_under_the_folder_root() {
        let declared = vec![
            "assets:mybank:checking".to_string(),
            "liabilities:credit-card:amex".to_string(),
        ];
        assert_eq!(
            lookup_account_for_folder("alex/mybank/checking", &declared).as_deref(),
            Some("assets:mybank:checking")
        );
        assert_eq!(
            lookup_account_for_folder("mybank/checking", &declared).as_deref(),
            Some("assets:mybank:checking")
        );
        assert_eq!(
            lookup_account_for_folder("alex/liabilities/credit-card/amex", &declared).as_deref(),
            Some("liabilities:credit-card:amex")
        );
    }

    #[test]
    fn lookup_ignores_declarations_under_another_root() {
        // A legacy folder derives under assets, so a liabilities declaration
        // with the same suffix is not a candidate: the layout decides the root.
        let liabilities = vec!["liabilities:credit-card:amex".to_string()];
        assert_eq!(
            lookup_account_for_folder("alex/credit-card/amex", &liabilities),
            None
        );
        let assets = vec!["assets:credit-card:amex".to_string()];
        assert_eq!(
            lookup_account_for_folder("alex/liabilities/credit-card/amex", &assets),
            None
        );
        assert_eq!(lookup_account_for_folder("alex/other", &assets), None);
    }
}

#[cfg(test)]
mod ignored_ids_tests {
    use super::*;

    fn ignored(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn matches_a_whole_segment_anywhere_in_meta() {
        let set = ignored(&["txn:abc"]);
        assert!(has_ignored_id(Some("txn:abc"), &set));
        assert!(has_ignored_id(Some("txn:abc, rule:r1"), &set));
        assert!(has_ignored_id(Some("rule:r1 ,  txn:abc"), &set));
    }

    #[test]
    fn keeps_the_swap_partner_that_references_the_ignored_id() {
        let set = ignored(&["txn:abc"]);
        assert!(!has_ignored_id(
            Some("txn:def, swap:txn:abc, swap_partner_commodity:SOL"),
            &set
        ));
    }

    #[test]
    fn keeps_ids_the_ignored_id_prefixes() {
        let set = ignored(&["txn:abc"]);
        assert!(!has_ignored_id(Some("txn:abc1"), &set));
        assert!(!has_ignored_id(Some("txn:abcd, rule:r1"), &set));
    }

    #[test]
    fn keeps_segments_that_merely_embed_the_id() {
        let set = ignored(&["txn:abc"]);
        assert!(!has_ignored_id(Some("source_payee:txn:abc"), &set));
    }

    #[test]
    fn no_meta_is_never_ignored() {
        let set = ignored(&["txn:abc"]);
        assert!(!has_ignored_id(None, &set));
        assert!(!has_ignored_id(Some(""), &set));
    }
}

#[cfg(test)]
mod append_hide_rule_tests {
    use super::*;
    use std::time::SystemTime;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "emuacc-hide-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn writes_a_meta_rule_into_the_folders_rules_json() {
        let sources = tmp();
        let folder_rel = "alex/solana/abc";
        fs::create_dir_all(sources.join(folder_rel)).unwrap();

        let added = append_hide_rule(&sources, folder_rel, "txn:abc123").unwrap();
        assert!(added, "first call should write a new rule");

        let rules = RulesFile::load(&sources.join(folder_rel));
        assert_eq!(rules.rules.len(), 1);
        let r = &rules.rules[0];
        assert_eq!(r.pattern, "txn:abc123");
        assert_eq!(r.match_field.as_deref(), Some("meta"));
        assert_eq!(r.amount_account.as_deref(), Some(HIDE_AMOUNT_ACCOUNT));
        assert_eq!(r.payee, None);
        assert_eq!(r.commodity, None);
    }

    #[test]
    fn second_call_with_same_id_is_a_noop() {
        let sources = tmp();
        let folder_rel = "alex/solana/abc";
        fs::create_dir_all(sources.join(folder_rel)).unwrap();

        let first = append_hide_rule(&sources, folder_rel, "txn:abc123").unwrap();
        let second = append_hide_rule(&sources, folder_rel, "txn:abc123").unwrap();
        assert!(first, "first should add");
        assert!(!second, "second should skip — rule already present");

        let rules = RulesFile::load(&sources.join(folder_rel));
        assert_eq!(rules.rules.len(), 1, "no duplicate rule");
    }

    #[test]
    fn multiple_distinct_ids_each_get_their_own_rule() {
        let sources = tmp();
        let folder_rel = "bank";
        fs::create_dir_all(sources.join(folder_rel)).unwrap();

        append_hide_rule(&sources, folder_rel, "txn:a").unwrap();
        append_hide_rule(&sources, folder_rel, "txn:b").unwrap();

        let rules = RulesFile::load(&sources.join(folder_rel));
        assert_eq!(rules.rules.len(), 2);
        assert!(rules.rules.iter().any(|r| r.pattern == "txn:a"));
        assert!(rules.rules.iter().any(|r| r.pattern == "txn:b"));
    }

    /// New invariant: a freshly-written hide rule lands at index 0,
    /// even when the folder already has unrelated rules in front. See
    /// `features/architecture/txn_id_rule_priority.feature`.
    #[test]
    fn hide_rule_lands_at_index_zero_above_pre_existing_rules() {
        let sources = tmp();
        let folder_rel = "alex/wallet";
        let folder = sources.join(folder_rel);
        fs::create_dir_all(&folder).unwrap();

        let mut existing = RulesFile::default();
        existing.rules.push(Rule {
            id: "broad".into(),
            pattern: "*token_transfer*".into(),
            match_field: Some("meta".into()),
            payee: None,
            commodity: None,
            comment: Some("broad rule".into()),
            amount_condition: None,
            fee_condition: None,
            payee_condition: None,
            narration_condition: None,
            commodity_condition: None,
            meta_condition: None,            amount_account: Some("expenses:tokens".into()),
            fee_account: None,
            postings: vec![],
        });
        existing.save(&folder).unwrap();

        append_hide_rule(&sources, folder_rel, "txn:specific123").unwrap();

        let rules = RulesFile::load(&folder);
        assert_eq!(rules.rules[0].pattern, "txn:specific123");
        assert_eq!(rules.rules[1].pattern, "*token_transfer*");
    }

    /// A pre-existing broad meta rule must NOT shadow a freshly-added
    /// hide rule for the same txn id. The hide rule sits at the top
    /// and segment-matches the meta string.
    #[test]
    fn hide_rule_wins_against_pre_existing_broader_meta_rule() {
        let sources = tmp();
        let folder_rel = "alex/wallet";
        let folder = sources.join(folder_rel);
        fs::create_dir_all(&folder).unwrap();

        let mut existing = RulesFile::default();
        existing.rules.push(Rule {
            id: "broad".into(),
            pattern: "*token_transfer*".into(),
            match_field: Some("meta".into()),
            payee: None,
            commodity: None,
            comment: None,
            amount_condition: None,
            fee_condition: None,
            payee_condition: None,
            narration_condition: None,
            commodity_condition: None,
            meta_condition: None,            amount_account: Some("expenses:tokens".into()),
            fee_account: None,
            postings: vec![],
        });
        existing.save(&folder).unwrap();

        append_hide_rule(&sources, folder_rel, "txn:specific123").unwrap();

        let rules = RulesFile::load(&folder);
        let m = rules
            .find_match_prioritized(&crate::rules::MatchFields {
                payee: None,
                display_payee: None,
                narration: Some("token_transfer:receive"),
                meta: Some("token_transfer, txn:specific123"),
                commodity: None,
                display_commodity: None,
                amount: Some(-1.0),
                fee: None,
            })
            .expect("a hide rule should match");
        assert_eq!(m.amount_account.as_deref(), Some(HIDE_AMOUNT_ACCOUNT));
    }

    #[test]
    fn rules_are_only_written_to_the_named_folder() {
        let sources = tmp();
        fs::create_dir_all(sources.join("folder_a")).unwrap();
        fs::create_dir_all(sources.join("folder_b")).unwrap();

        append_hide_rule(&sources, "folder_a", "txn:x").unwrap();

        let a = RulesFile::load(&sources.join("folder_a"));
        let b = RulesFile::load(&sources.join("folder_b"));
        assert_eq!(a.rules.len(), 1);
        assert_eq!(b.rules.len(), 0);
    }

    /// Regression (TASK-100): a folder whose `_rules.json` exists but does
    /// not parse must not be overwritten with just the hide rule — that
    /// silently deleted every rule in the folder.
    #[test]
    fn refuses_to_overwrite_an_unparseable_rules_file() {
        let sources = tmp();
        let folder_rel = "alex/wallet";
        let folder = sources.join(folder_rel);
        fs::create_dir_all(&folder).unwrap();
        let broken = r#"{"rules": [{"id": "keep", "pattern": "*coffee*"},]}"#;
        let path = folder.join("_rules.json");
        fs::write(&path, broken).unwrap();

        let err = append_hide_rule(&sources, folder_rel, "txn:abc123")
            .expect_err("hiding into an unparseable rules file must fail");
        assert!(err.contains("_rules.json"), "error should name the file: {err}");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            broken,
            "the unparseable file must be left untouched"
        );
    }

    /// End-to-end: write a CSV row, find its txn id, write a hide rule, run
    /// the pipeline, and confirm the resulting transaction has its contra
    /// account replaced with `ignore:hidden`. This is the contract the
    /// frontend relies on — clicking delete must produce a txn the existing
    /// hidden_accounts filter elides.
    #[test]
    fn hide_rule_re_categorises_a_csv_transaction_to_ignore_hidden() {
        let root = tmp();
        let sources = root.join("sources");
        let generated = root.join("generated");
        let folder_rel = "set/bank";
        fs::create_dir_all(sources.join(folder_rel)).unwrap();

        // CSV with one row + a transform that produces a txn id.
        fs::write(
            sources.join(folder_rel).join("2026-01.csv"),
            "Date,Payee,Amount\n2026-01-15,Coffee Shop,-4.50\n",
        )
        .unwrap();
        fs::write(
            sources.join(folder_rel).join("_transform.rhai"),
            r#"
            #{
                date: row["Date"],
                payee: row["Payee"],
                narration: row["Payee"],
                amount: row["Amount"],
                commodity: "AUD",
            }
            "#,
        )
        .unwrap();

        let config = PipelineConfig {
            sources_dir: sources.clone(),
            generated_dir: generated.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };

        // First run: pipeline produces one txn with its own contra account.
        let _ = run_pipeline(&config).expect("first pipeline run");
        let folder_ledger = generated.join(folder_rel).join("ledger.transactions");
        let pre_text = fs::read_to_string(&folder_ledger).expect("read folder ledger");
        // Extract the txn id we'll then hide.
        let txn_id = pre_text
            .lines()
            .find(|l| l.contains("txn:"))
            .and_then(|l| l.split_whitespace().find(|t| t.starts_with("txn:")))
            .map(|s| s.trim_end_matches(',').to_string())
            .expect("txn id in folder ledger");

        // Append the hide rule and re-run.
        let added = append_hide_rule(&sources, folder_rel, &txn_id).expect("append hide rule");
        assert!(added);

        let _ = run_pipeline(&config).expect("second pipeline run");
        let post_text = fs::read_to_string(&folder_ledger).expect("read folder ledger after hide");

        assert!(
            post_text.contains("ignore:hidden"),
            "expected the folder ledger to mention 'ignore:hidden' after hide; got:\n{post_text}"
        );
    }

    /// Regression: an unchanged-intact folder with manual.transactions
    /// must not have its on-disk ledger doubled (or its tagged_txns
    /// duplicated) when other folders trigger a full pipeline rerun.
    ///
    /// The on_disk_intact branch loads the existing ledger (which already
    /// contains inlined manual entries from the previous run) into
    /// tagged_txns. The manual loop downstream must skip such folders via
    /// `intact_folders` to avoid pushing the same manuals a second time.
    ///
    /// Invariant: across regens, the on-disk ledger length and the
    /// in-memory tagged_txns count for an unchanged-intact folder must
    /// be stable. cache.entries is a transform-output cache, not a
    /// ledger-content source.
    #[test]
    fn unchanged_intact_folder_with_manual_does_not_grow_across_regens() {
        let root = tmp();
        let sources = root.join("sources");
        let generated = root.join("generated");
        let folder_a = "set/bank_a";
        let folder_b = "set/bank_b";
        fs::create_dir_all(sources.join(folder_a)).unwrap();
        fs::create_dir_all(sources.join(folder_b)).unwrap();

        // Folder A: CSV + transform + manual.transactions
        fs::write(
            sources.join(folder_a).join("2026-01.csv"),
            "Date,Payee,Amount\n2026-01-15,Coffee Shop,-4.50\n",
        )
        .unwrap();
        fs::write(
            sources.join(folder_a).join("_transform.rhai"),
            r#"
            #{
                date: row["Date"],
                payee: row["Payee"],
                narration: row["Payee"],
                amount: row["Amount"],
                commodity: "AUD",
            }
            "#,
        )
        .unwrap();
        fs::write(
            sources.join(folder_a).join("manual.transactions"),
            "2026-01-20 * \"Manual A\" \"manual entry a\"\n    assets:bank:test  100 AUD\n    income:other  -100 AUD\n",
        )
        .unwrap();

        // Folder B: CSV + transform (no manual yet — will change between runs to defeat global early-exit).
        fs::write(
            sources.join(folder_b).join("2026-01.csv"),
            "Date,Payee,Amount\n2026-01-16,Bakery,-2.00\n",
        )
        .unwrap();
        fs::write(
            sources.join(folder_b).join("_transform.rhai"),
            r#"
            #{
                date: row["Date"],
                payee: row["Payee"],
                narration: row["Payee"],
                amount: row["Amount"],
                commodity: "AUD",
            }
            "#,
        )
        .unwrap();

        let config = PipelineConfig {
            sources_dir: sources.clone(),
            generated_dir: generated.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };

        // Run 1: both folders fresh. Writes folder_a = [csv, manual], folder_b = [csv].
        let result1 = run_pipeline(&config).expect("run 1");
        let folder_a_ledger = generated.join(folder_a).join("ledger.transactions");
        let text1 = fs::read_to_string(&folder_a_ledger).expect("read after run 1");
        let parsed1 = crate::ledger_parser::parse_transactions(&text1);
        assert_eq!(
            parsed1.transactions.len(),
            2,
            "after run 1 expected folder_a to have CSV+manual = 2 txns; got {}\n{}",
            parsed1.transactions.len(),
            text1
        );
        let tagged1 = result1
            .in_memory
            .as_ref()
            .expect("in_memory result1")
            .tagged_txns
            .len();
        assert_eq!(
            tagged1, 3,
            "after run 1 expected 3 tagged_txns (folder_a CSV + folder_a manual + folder_b CSV); got {tagged1}"
        );

        // Run 2: change folder B (a manual is added there) to defeat the
        // global early-exit. Folder A stays unchanged-intact.
        fs::write(
            sources.join(folder_b).join("manual.transactions"),
            "2026-01-21 * \"Manual B\" \"manual entry b\"\n    assets:bank:test  50 AUD\n    income:other  -50 AUD\n",
        )
        .unwrap();
        let result2 = run_pipeline(&config).expect("run 2");
        let text2 = fs::read_to_string(&folder_a_ledger).expect("read after run 2");
        let parsed2 = crate::ledger_parser::parse_transactions(&text2);
        assert_eq!(
            parsed2.transactions.len(),
            2,
            "after run 2 folder_a should still have 2 txns (no growth); got {}\n{}",
            parsed2.transactions.len(),
            text2
        );
        let tagged2 = result2
            .in_memory
            .as_ref()
            .expect("in_memory result2")
            .tagged_txns
            .len();
        assert_eq!(
            tagged2, 4,
            "after run 2 expected 4 tagged_txns (no double-count of folder_a manual); got {tagged2}"
        );

        // Run 3: change folder B again to defeat early-exit. Folder A stays unchanged-intact.
        fs::write(
            sources.join(folder_b).join("manual.transactions"),
            "2026-01-22 * \"Manual B v2\" \"manual entry b v2\"\n    assets:bank:test  60 AUD\n    income:other  -60 AUD\n",
        )
        .unwrap();
        let result3 = run_pipeline(&config).expect("run 3");
        let text3 = fs::read_to_string(&folder_a_ledger).expect("read after run 3");
        let parsed3 = crate::ledger_parser::parse_transactions(&text3);
        assert_eq!(
            parsed3.transactions.len(),
            2,
            "after run 3 folder_a should still have 2 txns (no growth); got {}\n{}",
            parsed3.transactions.len(),
            text3
        );
        let tagged3 = result3
            .in_memory
            .as_ref()
            .expect("in_memory result3")
            .tagged_txns
            .len();
        assert_eq!(
            tagged3, 4,
            "after run 3 expected 4 tagged_txns (no double-count of folder_a manual); got {tagged3}"
        );
    }
}

#[cfg(test)]
mod determinism_tests {
    use super::*;
    use crate::ledger_parser::Posting;

    fn make_txn(
        datetime: &str,
        meta: Option<&str>,
        account: &str,
        amount: f64,
    ) -> Transaction {
        Transaction {
            date: datetime.split(' ').next().unwrap_or(datetime).to_string(),
            datetime: datetime.to_string(),
            status: Some('*'),
            payee: Some("payee".into()),
            narration: None,
            meta: meta.map(String::from),
            postings: vec![Posting {
                account: account.into(),
                amount,
                amount_text: format!("{amount}"),
                commodity: "USD".into(),
                remainder: None,
                cost: None,
                price: None,
            }],
            display_payee: None,
            amount,
            amount_commodity: "USD".into(),
            display_amount_commodity: None,
            fee: None,
            fee_commodity: None,
        }
    }

    #[test]
    fn accounts_text_for_set_iterates_folders_in_sorted_order() {
        let mut folder_accounts: HashMap<String, String> = HashMap::new();
        folder_accounts.insert("alex/zfolder".into(), "account assets:z\n".into());
        folder_accounts.insert("alex/afolder".into(), "account assets:a\n".into());
        folder_accounts.insert("alex/mfolder".into(), "account assets:m\n".into());

        let out = accounts_text_for_set("alex", &folder_accounts, "");

        let a_pos = out.find("assets:a").expect("assets:a present");
        let m_pos = out.find("assets:m").expect("assets:m present");
        let z_pos = out.find("assets:z").expect("assets:z present");
        assert!(a_pos < m_pos && m_pos < z_pos, "expected sorted order, got:\n{out}");
    }

    #[test]
    fn sort_tagged_txns_breaks_ties_when_meta_is_none() {
        // Same datetime, both meta=None. Without a third tiebreak the sort would
        // preserve input order, leaking upstream HashSet randomization into the FIFO
        // inventory and the cgt-*.md warnings list.
        let t1 = make_txn("2024-01-01 12:00:00", None, "assets:wallet:1", 10.0);
        let t2 = make_txn("2024-01-01 12:00:00", None, "assets:wallet:2", -10.0);

        let mut shuffled_a: Vec<(Option<String>, Option<String>, Transaction)> =
            vec![(None, None, t2.clone()), (None, None, t1.clone())];
        let mut shuffled_b: Vec<(Option<String>, Option<String>, Transaction)> =
            vec![(None, None, t1.clone()), (None, None, t2.clone())];

        sort_tagged_txns(&mut shuffled_a);
        sort_tagged_txns(&mut shuffled_b);

        let accounts = |v: &Vec<(Option<String>, Option<String>, Transaction)>| -> Vec<String> {
            v.iter()
                .map(|(_, _, t)| t.postings[0].account.clone())
                .collect()
        };
        assert_eq!(accounts(&shuffled_a), accounts(&shuffled_b),
            "sort must produce identical order regardless of input shuffle");
    }

    #[test]
    fn sort_tagged_txns_breaks_ties_by_txn_id_when_metas_match() {
        // Real-world Solana case: two postings in the same block-second carry only
        // a `txn:` ID in meta. The meta strings differ but pre-fix the sort only
        // looked at meta — leaving subsequent posting position dependent on input.
        let t1 = make_txn("2024-04-30 01:29:37", Some("txn:aaa"), "a", 1.0);
        let t2 = make_txn("2024-04-30 01:29:37", Some("txn:bbb"), "b", 2.0);
        let t3 = make_txn("2024-04-30 01:29:37", Some("txn:ccc"), "c", 3.0);

        let mut shuffled: Vec<(Option<String>, Option<String>, Transaction)> = vec![
            (None, None, t3.clone()),
            (None, None, t1.clone()),
            (None, None, t2.clone()),
        ];
        sort_tagged_txns(&mut shuffled);

        let ids: Vec<String> = shuffled
            .iter()
            .map(|(_, _, t)| t.meta.clone().unwrap_or_default())
            .collect();
        assert_eq!(ids, vec!["txn:aaa".to_string(), "txn:bbb".to_string(), "txn:ccc".to_string()]);
    }
}

#[cfg(test)]
mod auto_link_equity_swaps_tests {
    use super::*;
    use crate::ledger_parser::Posting;

    /// Build a single-asset trading leg: one `assets:` posting with the
    /// given commodity/amount and a contra `equity:trading:{side}` posting
    /// (already suffixed — mimics the saved-trade-link rule path).
    fn mk_pre_suffixed_leg(
        datetime: &str,
        asset_account: &str,
        amount: f64,
        commodity: &str,
        side: &str,
        txn_id: &str,
    ) -> Transaction {
        let contra = format!("equity:trading:{side}");
        Transaction {
            date: datetime.split(' ').next().unwrap_or(datetime).to_string(),
            datetime: datetime.to_string(),
            status: Some('*'),
            payee: Some("Kraken".into()),
            narration: Some("trade".into()),
            meta: Some(format!("txn:{txn_id}")),
            postings: vec![
                Posting {
                    account: asset_account.into(),
                    amount,
                    amount_text: format!("{amount}"),
                    commodity: commodity.into(),
                    remainder: None,
                    cost: None,
                    price: None,
                },
                Posting {
                    account: contra,
                    amount: -amount,
                    amount_text: format!("{}", -amount),
                    commodity: commodity.into(),
                    remainder: None,
                    cost: None,
                    price: None,
                },
            ],
            display_payee: None,
            amount,
            amount_commodity: commodity.into(),
            display_amount_commodity: None,
            fee: None,
            fee_commodity: None,
        }
    }

    fn meta_value(meta: &Option<String>, prefix: &str) -> Option<String> {
        meta_tag_value(meta, prefix).map(String::from)
    }

    fn leg(idx: usize, commodity: &str, amount: f64) -> SwapLeg {
        SwapLeg {
            idx,
            commodity: commodity.into(),
            amount,
        }
    }

    /// A two-hop convert (sell BTC, buy USDT, sell USDT, buy ETH) links each
    /// hop: BTC gives up the ETH buy it ranked first so the USDT sell, which
    /// only the ETH buy suits, gets a partner.
    #[test]
    fn pair_swap_legs_links_each_hop_of_a_two_hop_convert() {
        let sells = vec![leg(0, "BTC", 0.01), leg(2, "USDT", 600.0)];
        let buys = vec![leg(1, "USDT", 600.0), leg(3, "ETH", 0.2)];
        assert_eq!(pair_swap_legs(sells, buys), vec![(0, 1), (2, 3)]);
    }

    /// Two legs of one commodity are never paired: alone they stay
    /// unlinked, and fills in both directions at one second link within
    /// each direction.
    #[test]
    fn pair_swap_legs_never_pairs_one_commodity() {
        assert_eq!(
            pair_swap_legs(vec![leg(0, "BTC", 1.0)], vec![leg(1, "BTC", 1.0)]),
            vec![]
        );
        let sells = vec![leg(0, "BTC", 0.5), leg(2, "ETH", 2.0)];
        let buys = vec![leg(1, "ETH", 10.0), leg(3, "BTC", 0.1)];
        assert_eq!(pair_swap_legs(sells, buys), vec![(0, 1), (2, 3)]);
    }

    /// A multi-fill of one pair ranks by amount within the pair, smallest
    /// with smallest, whatever order the legs arrive in.
    #[test]
    fn pair_swap_legs_rank_pairs_a_multi_fill() {
        let sells = vec![leg(4, "BTC", 3.0), leg(0, "BTC", 1.0), leg(2, "BTC", 2.0)];
        let buys = vec![leg(3, "ETH", 20.0), leg(5, "ETH", 30.0), leg(1, "ETH", 10.0)];
        assert_eq!(pair_swap_legs(sells, buys), vec![(0, 1), (2, 3), (4, 5)]);
    }

    /// Regression: when a (datetime, asset_account) group contains pairs
    /// in BOTH directions (BTC↔ETH and ETH↔BTC), the partner-pairing must
    /// match within each (sell commodity, buy commodity) subgroup —
    /// otherwise the rank-pairing places same-commodity legs as partners
    /// and meta is stamped with `swap_partner_commodity` equal to the
    /// asset's own commodity, surfacing as wrong "links" in the UI.
    /// Mirrors the Kraken multi-fill case from issue #N (2016-01-20 spot
    /// trades: 740 ETH/360 ETH buys, 1.32/2.73 BTC sells, plus a reverse
    /// 77 ETH sell + 0.28 BTC buy).
    #[test]
    fn pairs_within_commodity_pair_when_group_has_mixed_directions() {
        let dt = "2016-01-20 02:34:27";
        let acc = "assets:exchange:kraken:personal";
        let mut tagged: Vec<(Option<String>, Option<String>, Transaction)> = vec![
            // BTC sell ↔ ETH buy, two-fill multi
            (None, None, mk_pre_suffixed_leg(dt, acc, -1.32544300, "BTC", "sell", "btc-sell-1")),
            (None, None, mk_pre_suffixed_leg(dt, acc, 360.175000, "ETH", "buy", "eth-buy-1")),
            (None, None, mk_pre_suffixed_leg(dt, acc, -2.73354100, "BTC", "sell", "btc-sell-2")),
            (None, None, mk_pre_suffixed_leg(dt, acc, 740.797019, "ETH", "buy", "eth-buy-2")),
            // ETH sell ↔ BTC buy, reverse direction
            (None, None, mk_pre_suffixed_leg(dt, acc, -77.661000, "ETH", "sell", "eth-sell-1")),
            (None, None, mk_pre_suffixed_leg(dt, acc, 0.28501600, "BTC", "buy", "btc-buy-1")),
        ];

        auto_link_equity_swaps(&mut tagged, None, None);

        // Every leg must have a partner reference, and the partner's
        // commodity must DIFFER from the leg's own commodity.
        for (_, _, txn) in &tagged {
            let own_commodity = &txn.amount_commodity;
            let partner_commodity = meta_value(&txn.meta, "swap_partner_commodity:")
                .unwrap_or_else(|| panic!(
                    "no swap_partner_commodity meta on {} leg ({}, {}); meta={:?}",
                    own_commodity, txn.amount, own_commodity, txn.meta
                ));
            assert_ne!(
                &partner_commodity, own_commodity,
                "{} leg of {} got partner commodity {} (same as own); meta={:?}",
                txn.amount, own_commodity, partner_commodity, txn.meta
            );
        }
    }

    /// Build a trading leg with an explicit contra account. Mirrors the real
    /// pipeline, where the eth `_transform.rhai` default contra is
    /// `expenses:unknown` until a `_rules.json` match promotes it to
    /// `equity:trading:{side}`.
    fn mk_leg_with_contra(
        datetime: &str,
        asset_account: &str,
        amount: f64,
        commodity: &str,
        contra: &str,
        txn_id: &str,
    ) -> Transaction {
        Transaction {
            date: datetime.split(' ').next().unwrap_or(datetime).to_string(),
            datetime: datetime.to_string(),
            status: Some('*'),
            payee: Some("Uniswap".into()),
            narration: Some(format!("token_transfer:trade {commodity}")),
            meta: Some(format!("txn:{txn_id}")),
            postings: vec![
                Posting {
                    account: asset_account.into(),
                    amount,
                    amount_text: format!("{amount}"),
                    commodity: commodity.into(),
                    remainder: None,
                    cost: None,
                    price: None,
                },
                Posting {
                    account: contra.into(),
                    amount: -amount,
                    amount_text: format!("{}", -amount),
                    commodity: commodity.into(),
                    remainder: None,
                    cost: None,
                    price: None,
                },
            ],
            display_payee: None,
            amount,
            amount_commodity: commodity.into(),
            display_amount_commodity: None,
            fee: None,
            fee_commodity: None,
        }
    }

    /// HYPOTHESIS CHECK 1 — the multi-leg aggregator DOES link an on-chain
    /// swap whose 2 sell legs + 2 buy legs all share ONE txn id (the real
    /// PERP→USDC Universal Router swap, tx 0x34ed…). Precondition: both sides
    /// are already routed to `equity:trading` (the thing the real swap is
    /// missing on the buy side). If this fails, multi-leg swap linking is
    /// broken or disabled — and the "implemented then disabled" memory is
    /// correct.
    #[test]
    fn links_onchain_multileg_swap_sharing_one_txn_id() {
        let dt = "2026-06-26 00:23:23";
        let acc = "assets:crypto:wallet:ethereum:0x1111111111111111111111111111111111111111";
        let tx = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let mut tagged: Vec<(Option<String>, Option<String>, Transaction)> = vec![
            (None, None, mk_pre_suffixed_leg(dt, acc, -985.0348891476083, "PERP", "sell", tx)),
            (None, None, mk_pre_suffixed_leg(dt, acc, -173.82968632016617, "PERP", "sell", tx)),
            (None, None, mk_pre_suffixed_leg(dt, acc, 3.047271, "USDC", "buy", tx)),
            (None, None, mk_pre_suffixed_leg(dt, acc, 17.416664, "USDC", "buy", tx)),
        ];

        auto_link_equity_swaps(&mut tagged, None, None);

        for (_, _, txn) in &tagged {
            let own = txn.amount_commodity.clone();
            let partner = meta_value(&txn.meta, "swap_partner_commodity:").unwrap_or_else(|| {
                panic!(
                    "leg {own} {} was NOT linked as a swap; meta={:?}",
                    txn.amount, txn.meta
                )
            });
            let expected = if own == "PERP" { "USDC" } else { "PERP" };
            assert_eq!(
                partner, expected,
                "leg {own} {} got partner {partner}, expected {expected}; meta={:?}",
                txn.amount, txn.meta
            );
        }
    }

    /// HYPOTHESIS CHECK 2 — the actual bug. Same swap, but the USDC buy legs
    /// are still on the rhai default contra `expenses:unknown` (no rule
    /// promoted them, because Universal-Router / v4 addresses aren't in
    /// `_rules.json`). With no buy side in `equity:trading`, the linker has
    /// nothing to pair the PERP sells against, so NOTHING is recognised as a
    /// swap — exactly what the screenshot shows. This isolates the cause to
    /// buy-side routing, not the leg count.
    #[test]
    fn unrouted_buy_side_prevents_swap_link() {
        let dt = "2026-06-26 00:23:23";
        let acc = "assets:crypto:wallet:ethereum:0x1111111111111111111111111111111111111111";
        let tx = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let mut tagged: Vec<(Option<String>, Option<String>, Transaction)> = vec![
            (None, None, mk_pre_suffixed_leg(dt, acc, -985.0348891476083, "PERP", "sell", tx)),
            (None, None, mk_pre_suffixed_leg(dt, acc, -173.82968632016617, "PERP", "sell", tx)),
            (None, None, mk_leg_with_contra(dt, acc, 3.047271, "USDC", "expenses:unknown", tx)),
            (None, None, mk_leg_with_contra(dt, acc, 17.416664, "USDC", "expenses:unknown", tx)),
        ];

        auto_link_equity_swaps(&mut tagged, None, None);

        for (_, _, txn) in &tagged {
            assert!(
                meta_value(&txn.meta, "swap_partner_commodity:").is_none(),
                "leg {} {} was unexpectedly linked though the buy side sits in \
                 expenses:unknown; meta={:?}",
                txn.amount_commodity, txn.amount, txn.meta
            );
        }
    }

    /// TASK-103: a swap side worth nothing prices nothing. A zero price feed
    /// on the partner (a dead token, a plugin that wrote 0) must leave the
    /// legs unpriced rather than stamp `@ 0`, which would give the CGT report
    /// a zero cost base or zero proceeds. Covers the rank-paired pass
    /// (ABC→XYZ, XYZ quoted at 0) and the final swap price pass (USDC→USDT,
    /// which the rank-paired pass leaves alone, USDT quoted at 0).
    #[test]
    fn zero_valued_swap_partner_never_stamps_a_zero_price() {
        let acc = "assets:wallet:eth";
        let token_dt = "2025-03-01 10:00:00";
        let stable_dt = "2025-03-01 11:00:00";
        let mut tagged: Vec<(Option<String>, Option<String>, Transaction)> = vec![
            (None, None, mk_pre_suffixed_leg(token_dt, acc, -100.0, "ABC", "sell", "0xtoken")),
            (None, None, mk_pre_suffixed_leg(token_dt, acc, 250.0, "XYZ", "buy", "0xtoken")),
            (None, None, mk_pre_suffixed_leg(stable_dt, acc, -100.5, "USDC", "sell", "0xstable")),
            (None, None, mk_pre_suffixed_leg(stable_dt, acc, 100.0, "USDT", "buy", "0xstable")),
        ];
        let pg = crate::ledger_parser::PriceGraph::from_entries(vec![
            pd("2025-03-01", "XYZ", 0.0, "AUD"),
            pd("2025-03-01", "USDT", 0.0, "AUD"),
        ]);

        auto_link_equity_swaps(&mut tagged, Some(&pg), Some("AUD"));

        for (_, _, txn) in &tagged {
            let ap = asset_posting(txn).expect("asset posting");
            assert!(
                ap.price.is_none(),
                "{} {} leg was priced from a zero-valued partner: {:?}",
                ap.amount, ap.commodity, ap.price
            );
        }
    }

    /// TASK-104: a vault of several account sets prices each set's swap
    /// legs in that set's own base currency and never pairs one set's legs
    /// with another's. alice (AUD) and bob (USD) each swap 100 USDC for
    /// 0.5 SOL at the same second from a same-named wallet account, the legs
    /// interleaved so that a set-blind rank-pairing crosses the sets. The
    /// pipeline used to resolve one base currency, the alphabetically first
    /// set's, and link every set's legs with it.
    #[test]
    fn each_set_prices_its_swap_legs_in_its_own_base_currency() {
        let dt = "2025-03-01 10:00:00";
        let acc = "assets:crypto:wallet:main";
        let alice = || Some("alice".to_string());
        let bob = || Some("bob".to_string());
        let mut tagged: Vec<(Option<String>, Option<String>, Transaction)> = vec![
            (
                alice(),
                None,
                mk_pre_suffixed_leg(dt, acc, -100.0, "USDC", "sell", "0xalice"),
            ),
            (
                bob(),
                None,
                mk_pre_suffixed_leg(dt, acc, -100.0, "USDC", "sell", "0xbob"),
            ),
            (
                bob(),
                None,
                mk_pre_suffixed_leg(dt, acc, 0.5, "SOL", "buy", "0xbob"),
            ),
            (
                alice(),
                None,
                mk_pre_suffixed_leg(dt, acc, 0.5, "SOL", "buy", "0xalice"),
            ),
        ];
        let pg = crate::ledger_parser::PriceGraph::from_entries(vec![
            pd("2025-03-01", "USDC", 1.0, "USD"),
            pd("2025-03-01", "SOL", 200.0, "USD"),
            pd("2025-03-01", "USD", 1.5, "AUD"),
        ]);
        let base_currency_of = |set: Option<&str>| match set {
            Some("alice") => Some("AUD"),
            Some("bob") => Some("USD"),
            _ => None,
        };

        link_equity_swaps_per_set(&mut tagged, Some(&pg), &base_currency_of);

        // 0.5 SOL is worth 100 USD = 150 AUD: per unit 200 USD / 300 AUD on
        // the SOL leg and 1 USD / 1.5 AUD on the 100 USDC leg.
        for (set, _, txn) in &tagged {
            let set = set.as_deref().expect("set");
            let ap = asset_posting(txn).expect("asset posting");
            let price = ap.price.as_ref().expect("every leg is priced");
            let base = base_currency_of(Some(set)).expect("base");
            assert_eq!(
                price.commodity, base,
                "{set} {} leg priced in {}",
                ap.commodity, price.commodity
            );
            let usd_per_unit = if ap.commodity == "SOL" { 200.0 } else { 1.0 };
            let expected = if base == "AUD" {
                usd_per_unit * 1.5
            } else {
                usd_per_unit
            };
            assert!(
                (price.amount - expected).abs() < 1e-9,
                "{set} {} leg priced at {} {base}, expected {expected}",
                ap.commodity,
                price.amount
            );
            assert_eq!(
                meta_value(&txn.meta, "swap:").as_deref(),
                Some(format!("txn:0x{set}").as_str()),
                "{set} {} leg paired outside its set: {:?}",
                ap.commodity,
                txn.meta
            );
        }
    }

    /// Build a price directive for an in-test PriceGraph.
    fn pd(
        datetime: &str,
        commodity: &str,
        price: f64,
        quote: &str,
    ) -> crate::ledger_parser::PriceDirective {
        crate::ledger_parser::PriceDirective {
            datetime: datetime.into(),
            commodity: commodity.into(),
            price_amount: price,
            price_amount_text: format!("{price}"),
            quote_commodity: quote.into(),
        }
    }

    /// Regression: a real→stable disposal (sell 1158.86 PERP, receive 20.46
    /// USDC in one on-chain swap) must book proceeds equal to the value of
    /// what was RECEIVED — the USDC — per ATO 116-20, NOT the disposed token's
    /// own (here stale) market price. With USDC = 1.00 USD and USD = 1.4042
    /// AUD that is ~28.74 AUD; anchoring on PERP's stale 0.1755 AUD quote
    /// would wrongly book ~203 AUD (×7.1). Guards the `auto_link_equity_swaps`
    /// price anchor (the single-base-like arm anchors on the buy side).
    #[test]
    fn real_to_stable_swap_proceeds_equal_usdc_received() {
        let dt = "2026-06-26 00:23:23";
        let acc = "assets:crypto:wallet:ethereum:0x1111111111111111111111111111111111111111";
        let tx = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let mut tagged: Vec<(Option<String>, Option<String>, Transaction)> = vec![
            (None, None, mk_pre_suffixed_leg(dt, acc, -985.0348891476083, "PERP", "sell", tx)),
            (None, None, mk_pre_suffixed_leg(dt, acc, -173.82968632016617, "PERP", "sell", tx)),
            (None, None, mk_pre_suffixed_leg(dt, acc, 3.047271, "USDC", "buy", tx)),
            (None, None, mk_pre_suffixed_leg(dt, acc, 17.416664, "USDC", "buy", tx)),
        ];

        // PERP market price is STALE (0.1755 AUD). USDC is the ground truth:
        // 1.00 USD, and USD → AUD = 1.4042 (one hop).
        let pg = crate::ledger_parser::PriceGraph::from_entries(vec![
            pd("2026-06-26", "PERP", 0.1755, "AUD"),
            pd("2026-06-26", "USDC", 1.00, "USD"),
            pd("2026-06-26", "USD", 1.4042, "AUD"),
        ]);

        auto_link_equity_swaps(&mut tagged, Some(&pg), Some("AUD"));

        // Proceeds = Σ over PERP sell legs of |qty| × stamped per-unit price.
        let perp_proceeds: f64 = tagged
            .iter()
            .filter(|(_, _, t)| t.amount_commodity == "PERP")
            .map(|(_, _, t)| {
                let ap = t
                    .postings
                    .iter()
                    .find(|p| p.account.starts_with("assets:"))
                    .expect("asset posting");
                let unit = ap.price.as_ref().map(|pr| pr.amount).unwrap_or(0.0);
                ap.amount.abs() * unit
            })
            .sum();

        // USDC received in AUD: 20.463935 × 1.4042 = 28.7355.
        let expected = 28.7355_f64;
        assert!(
            (perp_proceeds - expected).abs() < 0.05,
            "real→stable disposal proceeds should equal USDC received (~{expected:.2} AUD), \
             not the stale PERP market price (×0.1755 ⇒ ~203 AUD). Got {perp_proceeds:.2} AUD."
        );
    }
}

#[cfg(test)]
mod invariant_tests {
    //! End-to-end invariants the generated tree MUST satisfy. Each test here
    //! catches a real-world bug we hit (and have now fixed). If any of these
    //! fail in CI, the generated ledger is no longer reproducible — investigate
    //! before shipping a regen.
    use super::*;
    use std::time::SystemTime;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "emuacc-invariant-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Minimal OFX v1 SGML fixture with one STMTTRN. Date is parseable as
    /// `2026-01-15`. Amount is positive ($100) so the transform's default
    /// posting produces a real ledger entry.
    fn minimal_ofx() -> String {
        r#"OFXHEADER:100
DATA:OFXSGML

<OFX>
<BANKMSGSRSV1>
<STMTTRNRS>
<STMTRS>
<CURDEF>AUD
<BANKTRANLIST>
<STMTTRN>
<TRNTYPE>CREDIT
<DTPOSTED>20260115
<TRNAMT>100.00
<FITID>OFX-TXN-001
<MEMO>OFX-only test deposit
</STMTTRN>
</BANKTRANLIST>
</STMTRS>
</STMTTRNRS>
</BANKMSGSRSV1>
</OFX>"#
            .to_string()
    }

    /// Walk `generated_dir` and return `(relative_path, bytes)` for every
    /// regular file, sorted by path. Used to byte-compare two pipeline runs.
    fn snapshot_generated(generated_dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut out: Vec<(String, Vec<u8>)> = Vec::new();
        for entry in walkdir::WalkDir::new(generated_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let rel = p
                .strip_prefix(generated_dir)
                .unwrap()
                .to_string_lossy()
                .to_string();
            // Skip the build cache — it carries write timestamps and is by-design
            // run-to-run different. The invariant is about *generated output*,
            // not internal cache state.
            if rel.starts_with(".cache/") {
                continue;
            }
            let bytes = fs::read(p).expect("read generated file");
            out.push((rel, bytes));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn diff_snapshots(
        a: &[(String, Vec<u8>)],
        b: &[(String, Vec<u8>)],
    ) -> Option<String> {
        if a.len() != b.len() {
            return Some(format!(
                "file count differs: run1={} run2={}\nrun1 paths: {:?}\nrun2 paths: {:?}",
                a.len(),
                b.len(),
                a.iter().map(|(p, _)| p).collect::<Vec<_>>(),
                b.iter().map(|(p, _)| p).collect::<Vec<_>>(),
            ));
        }
        for ((pa, ba), (pb, bb)) in a.iter().zip(b.iter()) {
            if pa != pb {
                return Some(format!("path mismatch: {pa} vs {pb}"));
            }
            if ba != bb {
                let sa = String::from_utf8_lossy(ba);
                let sb = String::from_utf8_lossy(bb);
                return Some(format!(
                    "content differs at {pa}\n---run1---\n{sa}\n---run2---\n{sb}"
                ));
            }
        }
        None
    }

    /// Bug 1 regression. An OFX-only folder (no CSV, no _transform.rhai, no
    /// per-folder accounts.transactions) must still get its
    /// `account assets:<…>` line emitted into the per-set
    /// accounts.transactions. Prior to the fix in `process_ofx_files`, this
    /// folder was walked, fingerprinted, and cached — but silently absent
    /// from the writer's `auto_accounts` list, so `cdia`/`smartaccess`-style
    /// folders disappeared from the generated declaration list on every
    /// cold-cache run.
    #[test]
    fn ofx_only_folder_emits_account_declaration_on_cold_cache() {
        let root = tmp();
        let sources = root.join("sources");
        let generated = root.join("generated");
        let folder_rel = "alex/cash/bank/cba/cdia";
        fs::create_dir_all(sources.join(folder_rel)).unwrap();
        fs::write(
            sources.join(folder_rel).join("statement.ofx"),
            minimal_ofx(),
        )
        .unwrap();

        let config = PipelineConfig {
            sources_dir: sources.clone(),
            generated_dir: generated.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };

        let _ = run_pipeline(&config).expect("pipeline run");

        let declared = fs::read_to_string(
            generated.join("alex").join(FILE_ACCOUNTS),
        )
        .expect("alex/accounts.transactions must exist");

        assert!(
            declared.contains("account assets:cash:bank:cba:cdia"),
            "OFX-only folder must emit its account declaration; got:\n{declared}"
        );
    }

    /// Bug 2 regression: cold-cache determinism. Two independent fresh runs
    /// of the same source tree must produce byte-identical generated output.
    /// If this fails, something in the pipeline is consuming a HashMap (or
    /// other unordered container) in iteration order — find it and add a
    /// canonical sort.
    ///
    /// The synthetic vault here exercises three bug classes we hit in
    /// production:
    /// 1. **CSV+OFX folder mix** — bug-1-adjacent path coverage.
    /// 2. **Multi-commodity balances** — `generate_balances_report_range`
    ///    iterates `by_commodity` (HashMap) and accumulates `total_value` in
    ///    f64. Different iteration order = different ULP-level rounding =
    ///    different `portfolio_weight` for every row.
    /// 3. **Multi-partner equity-swap shape** — same asset account at the
    ///    same datetime has a single sell that could pair with either of two
    ///    buys (different commodities). `auto_link_equity_swaps` uses
    ///    `tagged_partner_map.insert()` which is last-write-wins; HashMap
    ///    iteration order picks who "wins" non-deterministically. Result:
    ///    the same trade gets different `swap:txn:<id>` /
    ///    `swap_partner_commodity:<C>` metadata across runs, and downstream
    ///    auto-link price annotations shift too.
    #[test]
    fn pipeline_is_deterministic_across_two_cold_runs() {
        let make_vault = || {
            let root = tmp();
            let sources = root.join("sources");
            let generated = root.join("generated");

            // CSV folder A — emits AUD postings
            let a = "alex/bank/a";
            fs::create_dir_all(sources.join(a)).unwrap();
            fs::write(
                sources.join(a).join("2026-01.csv"),
                "Date,Payee,Amount\n2026-01-15,Coffee,-4.50\n2026-01-15,Bagel,-5.00\n",
            )
            .unwrap();
            fs::write(
                sources.join(a).join("_transform.rhai"),
                r#"#{ date: row["Date"], payee: row["Payee"], narration: row["Payee"], amount: row["Amount"], commodity: "AUD" }"#,
            )
            .unwrap();

            // CSV folder B — same-second AUD timestamps for ordering ties
            let b = "alex/bank/b";
            fs::create_dir_all(sources.join(b)).unwrap();
            fs::write(
                sources.join(b).join("2026-01.csv"),
                "Date,Payee,Amount\n2026-01-15,Salary,3500.00\n2026-01-15,Rent,-1800.00\n",
            )
            .unwrap();
            fs::write(
                sources.join(b).join("_transform.rhai"),
                r#"#{ date: row["Date"], payee: row["Payee"], narration: row["Payee"], amount: row["Amount"], commodity: "AUD" }"#,
            )
            .unwrap();

            // OFX-only folder (Bug 1 path coverage)
            let c = "alex/bank/c";
            fs::create_dir_all(sources.join(c)).unwrap();
            fs::write(sources.join(c).join("statement.ofx"), minimal_ofx()).unwrap();

            // Multi-commodity holdings folder so balances reports must iterate
            // a multi-key HashMap. Without sorted iteration, `total_value`
            // accumulates in different f64 orders, shifting every
            // `portfolio_weight` by ULPs.
            let d = "alex/wallet/multi-commodity";
            fs::create_dir_all(sources.join(d)).unwrap();
            fs::write(
                sources.join(d).join("manual.transactions"),
                "2026-01-10 * \"Open BTC\" \"open btc\"\n    assets:wallet:multi-commodity   1 BTC\n    equity:deposits                -1 BTC\n\
                 2026-01-10 * \"Open ETH\" \"open eth\"\n    assets:wallet:multi-commodity  10 ETH\n    equity:deposits               -10 ETH\n\
                 2026-01-10 * \"Open USDC\" \"open usdc\"\n    assets:wallet:multi-commodity 100 USDC\n    equity:deposits              -100 USDC\n\
                 2026-01-10 price BTC 1000 AUD\n\
                 2026-01-10 price ETH 100 AUD\n\
                 2026-01-10 price USDC 1 AUD\n",
            )
            .unwrap();

            // Multi-partner equity-swap shape: same asset account at the same
            // second has one USDC sell and TWO potential buy partners (AUD
            // and USDT). The buggy code path would non-deterministically
            // pair the USDC sell with whichever buy commodity HashMap
            // iteration hit last.
            let e = "alex/exchange/swap-test";
            fs::create_dir_all(sources.join(e)).unwrap();
            fs::write(
                sources.join(e).join("manual.transactions"),
                "2026-01-20 12:00:00 * \"Exchange\" \"trade USDC sell\" ; txn:SELL-USDC\n    assets:exchange:swap-test  -100 USDC\n    equity:trading:sell        100 USDC\n\
                 2026-01-20 12:00:00 * \"Exchange\" \"trade AUD buy\" ; txn:BUY-AUD\n    assets:exchange:swap-test  150 AUD\n    equity:trading:buy        -150 AUD\n\
                 2026-01-20 12:00:00 * \"Exchange\" \"trade USDT buy\" ; txn:BUY-USDT\n    assets:exchange:swap-test  100 USDT\n    equity:trading:buy        -100 USDT\n",
            )
            .unwrap();

            (root, sources, generated)
        };

        let (root1, sources1, generated1) = make_vault();
        let (root2, sources2, generated2) = make_vault();

        let cfg1 = PipelineConfig {
            sources_dir: sources1,
            generated_dir: generated1.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };
        let cfg2 = PipelineConfig {
            sources_dir: sources2,
            generated_dir: generated2.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };

        let _ = run_pipeline(&cfg1).expect("run 1");
        let _ = run_pipeline(&cfg2).expect("run 2");

        let snap1 = snapshot_generated(&generated1);
        let snap2 = snapshot_generated(&generated2);

        if let Some(diff) = diff_snapshots(&snap1, &snap2) {
            // Clean up tmpdirs only on failure path — Rust drops `root1`/`root2`
            // PathBufs naturally; we leak the dirs but the test framework runs in
            // a tmp prefix. Surfacing the diff is the priority.
            let _ = root1;
            let _ = root2;
            panic!("pipeline output is non-deterministic across cold runs:\n{diff}");
        }
    }

    /// Invariant: warm-cache regen produces byte-identical output to a
    /// cold-cache regen on the same sources. If warm output diverges from
    /// cold, the cache is silently editorialising — that breaks reviewability
    /// (a `git diff` after `regenerate` should reflect source/code changes,
    /// nothing else).
    #[test]
    fn pipeline_warm_cache_matches_cold_cache() {
        let root = tmp();
        let sources = root.join("sources");
        let generated_cold = root.join("generated_cold");
        let generated_warm = root.join("generated_warm");

        // Single CSV folder + OFX folder.
        let a = "alex/bank/a";
        fs::create_dir_all(sources.join(a)).unwrap();
        fs::write(
            sources.join(a).join("2026-01.csv"),
            "Date,Payee,Amount\n2026-01-15,Coffee,-4.50\n",
        )
        .unwrap();
        fs::write(
            sources.join(a).join("_transform.rhai"),
            r#"#{ date: row["Date"], payee: row["Payee"], narration: row["Payee"], amount: row["Amount"], commodity: "AUD" }"#,
        )
        .unwrap();
        let c = "alex/bank/c";
        fs::create_dir_all(sources.join(c)).unwrap();
        fs::write(sources.join(c).join("statement.ofx"), minimal_ofx()).unwrap();

        // Cold run into generated_cold.
        let cfg_cold = PipelineConfig {
            sources_dir: sources.clone(),
            generated_dir: generated_cold.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };
        let _ = run_pipeline(&cfg_cold).expect("cold run");
        let cold_snap = snapshot_generated(&generated_cold);

        // Warm run into generated_warm: run twice; the second hits cache.
        let cfg_warm = PipelineConfig {
            sources_dir: sources.clone(),
            generated_dir: generated_warm.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };
        let _ = run_pipeline(&cfg_warm).expect("warm run 1 (populates cache)");
        let _ = run_pipeline(&cfg_warm).expect("warm run 2 (uses cache)");
        let warm_snap = snapshot_generated(&generated_warm);

        if let Some(diff) = diff_snapshots(&cold_snap, &warm_snap) {
            panic!("warm-cache output differs from cold-cache output:\n{diff}");
        }
    }

    /// Aggregator multi-leg swap regression. A single user-perceived swap
    /// (Jupiter v6, 1inch, etc.) gets recorded on-chain as multiple atomic
    /// transfers — each appearing as a separate `equity:trading:sell`
    /// posting, but all sharing the same `txn:` id. The buy side is one
    /// (or more) postings with the same id.
    ///
    /// Pre-fix, `auto_link_equity_swaps` rank-paired by amount only,
    /// leaving the smallest fragment paired with the entire buy. Then
    /// `resolve_sale_proceeds` (in reports.rs) attributed the FULL buy
    /// value to that fragment as its sale proceeds, producing a per-unit
    /// figure inflated by ~1000× (production bug: a 17.76 USDC sell leg
    /// of a 35,522 USDC → 8,278 HNT Jupiter swap was credited with the
    /// entire ~$33k HNT value as proceeds for 17.76 units).
    ///
    /// **Invariant:** in any same-`(datetime, asset_account, txn_id)`
    /// multi-leg swap, the total CGT proceeds across all sells of a given
    /// commodity must equal the partner-side market value. Per-leg
    /// proceeds must pro-rate by quantity.
    #[test]
    fn cgt_proceeds_for_multi_leg_swap_equals_partner_value() {
        let root = tmp();
        let sources = root.join("sources");
        let generated = root.join("generated");
        let folder = "alex/exchange/swap-test";
        fs::create_dir_all(sources.join(folder)).unwrap();

        // Price feeds in AUD directly (skip USD→AUD conversion path).
        let prices = sources.join("_prices");
        fs::create_dir_all(&prices).unwrap();
        fs::write(prices.join("HNT.txt"), "P 2024-06-15 HNT 4.00 AUD\n").unwrap();
        fs::write(prices.join("USDC.txt"), "P 2024-06-15 USDC 1.00 AUD\n").unwrap();

        // 50k USDC buy lot for FIFO + a Jupiter-v6-shaped multi-leg swap.
        // The real-world economics: user gave 35,522.958882 USDC and received
        // 8,278.878 HNT @ $4.00 AUD = $33,115.512 of HNT. That IS the disposal
        // proceeds for the aggregated USDC sale.
        fs::write(
            sources.join(folder).join("manual.transactions"),
            "\
2024-01-01 * \"Init\" \"buy 50000 USDC\" ; txn:BUY-INIT
    assets:exchange:swap-test  50000.00 USDC @ 1.00 AUD
    equity:deposits  -50000.00 AUD

2024-06-15 12:00:00 * \"Jupiter v6\" \"trade USDC leg 1\" ; txn:JUP1
    assets:exchange:swap-test  -17761.479441 USDC
    equity:trading:sell  17761.479441 USDC

2024-06-15 12:00:00 * \"Jupiter v6\" \"trade USDC leg 2\" ; txn:JUP1
    assets:exchange:swap-test  -17743.717962 USDC
    equity:trading:sell  17743.717962 USDC

2024-06-15 12:00:00 * \"Jupiter v6\" \"trade USDC leg 3\" ; txn:JUP1
    assets:exchange:swap-test  -17.761479 USDC
    equity:trading:sell  17.761479 USDC

2024-06-15 12:00:00 * \"Jupiter v6\" \"HNT buy\" ; txn:JUP1
    assets:exchange:swap-test  8278.878 HNT
    equity:trading:buy  -8278.878 HNT
",
        )
        .unwrap();

        let cfg = PipelineConfig {
            sources_dir: sources.clone(),
            generated_dir: generated.clone(),
            now_yyyymm: "2024-06".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };
        // Reports refuse to run without a base currency; declare the vault
        // default the way a set-up data folder has it (prices above are AUD).
        fs::create_dir_all(&generated).unwrap();
        fs::write(generated.join("config.json"), r#"{"base_currency":"AUD"}"#).unwrap();
        let _ = run_pipeline(&cfg).expect("pipeline run");

        // Reports aren't written by run_pipeline itself — emuacc-regenerate
        // calls generate_all_reports separately. Mirror that here.
        crate::report_templates::generate_all_reports(
            &sources,
            &generated,
            &["alex".to_string()],
            crate::report_templates::ALL_FORMATS,
            &[],
        )
        .expect("generate reports");

        // FY24 in AU (2023-07-01 → 2024-06-30) contains the swap.
        let cgt_path = generated.join("alex/reports/cgt-2024.json");
        let cgt_text = fs::read_to_string(&cgt_path)
            .unwrap_or_else(|e| panic!("read cgt-2024.json at {}: {e}", cgt_path.display()));
        let cgt: serde_json::Value =
            serde_json::from_str(&cgt_text).expect("parse cgt JSON");

        let usdc_events: Vec<&serde_json::Value> = cgt["events"]
            .as_array()
            .expect("events array")
            .iter()
            .filter(|e| {
                e["sell_date"] == "2024-06-15" && e["commodity"] == "USDC"
            })
            .collect();

        assert!(
            !usdc_events.is_empty(),
            "Expected USDC disposal events on 2024-06-15. cgt-2024.json:\n{cgt_text}"
        );

        let total_proceeds: f64 = usdc_events
            .iter()
            .map(|e| e["sale_proceeds"].as_f64().unwrap_or(0.0))
            .sum();
        let total_qty: f64 = usdc_events
            .iter()
            .map(|e| e["quantity"].as_f64().unwrap_or(0.0))
            .sum();

        let hnt_qty = 8278.878_f64;
        let hnt_price_aud = 4.00_f64;
        let expected_total_proceeds = hnt_qty * hnt_price_aud; // 33,115.512

        assert!(
            (total_qty - 35522.958882).abs() < 0.01,
            "Total disposed USDC qty should equal sum of sell legs (35,522.958882); got {total_qty}"
        );

        assert!(
            (total_proceeds - expected_total_proceeds).abs() < 1.0,
            "Total CGT proceeds for the Jupiter multi-leg swap must equal the partner HNT \
             value (${expected_total_proceeds:.2}); got ${total_proceeds:.2} across {} events. \
             Pre-fix this would be ${pre_fix_total:.2} because each USDC sell leg is \
             credited with its OWN proceeds (USDC market price or full partner value) \
             rather than the swap's aggregate consideration being divided pro-rata. \
             Events:\n{}",
            usdc_events.len(),
            serde_json::to_string_pretty(&usdc_events).unwrap_or_default(),
            pre_fix_total = 17761.479441_f64 * 1.0 + 17743.717962_f64 * 1.0 + hnt_qty * hnt_price_aud,
        );

        // Per-unit proceeds across the three legs should be identical
        // (pro-rata). Tolerance allows for f64 division rounding.
        let per_units: Vec<f64> = usdc_events
            .iter()
            .map(|e| {
                let p = e["sale_proceeds"].as_f64().unwrap_or(0.0);
                let q = e["quantity"].as_f64().unwrap_or(1.0);
                p / q
            })
            .collect();
        if per_units.len() > 1 {
            let first = per_units[0];
            for &pu in &per_units {
                assert!(
                    (pu - first).abs() < 0.001,
                    "Per-unit proceeds must be identical across multi-leg swap rows; \
                     got {per_units:?}"
                );
            }
        }
    }

    /// Invariant: every source folder that produced ledger output must be
    /// declared in `<owner>/accounts.transactions`. The set of declared
    /// accounts and the set of folders actually written to `generated/<owner>/`
    /// should agree.
    ///
    /// This was the load-bearing assumption Bug 1 broke: OFX-only folders
    /// wrote a ledger.transactions but didn't declare their account, which
    /// then caused downstream consumers (queries scoped to declared accounts)
    /// to silently exclude them.
    #[test]
    fn accounts_transactions_lists_every_source_folder_with_output() {
        let root = tmp();
        let sources = root.join("sources");
        let generated = root.join("generated");

        // CSV folder
        let a = "alex/csv-folder";
        fs::create_dir_all(sources.join(a)).unwrap();
        fs::write(
            sources.join(a).join("2026-01.csv"),
            "Date,Payee,Amount\n2026-01-15,Coffee,-4.50\n",
        )
        .unwrap();
        fs::write(
            sources.join(a).join("_transform.rhai"),
            r#"#{ date: row["Date"], payee: row["Payee"], narration: row["Payee"], amount: row["Amount"], commodity: "AUD" }"#,
        )
        .unwrap();

        // OFX-only folder
        let b = "alex/ofx-only-folder";
        fs::create_dir_all(sources.join(b)).unwrap();
        fs::write(sources.join(b).join("statement.ofx"), minimal_ofx()).unwrap();

        // Manual-only folder
        let c = "alex/manual-only-folder";
        fs::create_dir_all(sources.join(c)).unwrap();
        fs::write(
            sources.join(c).join("manual.transactions"),
            "2026-01-20 * \"Manual entry\" \"manual\"\n    assets:bank:test  10 AUD\n    income:other  -10 AUD\n",
        )
        .unwrap();

        let cfg = PipelineConfig {
            sources_dir: sources.clone(),
            generated_dir: generated.clone(),
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        };
        let _ = run_pipeline(&cfg).expect("pipeline run");

        let declared = fs::read_to_string(
            generated.join("alex").join(FILE_ACCOUNTS),
        )
        .expect("alex/accounts.transactions must exist");

        // Each folder must have its derived account in the declarations.
        for (folder_rel, want) in [
            (a, "account assets:csv-folder"),
            (b, "account assets:ofx-only-folder"),
            (c, "account assets:manual-only-folder"),
        ] {
            assert!(
                declared.contains(want),
                "expected declaration for {folder_rel} ({want}); got:\n{declared}"
            );
        }
    }
}

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::SystemTime;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "emuacc-concurrency-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Verify that concurrent `run_pipeline` calls complete without panics,
    /// and that the global Mutex exists (compilation proof of the gate).
    /// The real safety guarantee is the Mutex serialisation itself; this test
    /// catches any regression that removes the lock or replaces it with a
    /// non-blocking primitive.
    #[test]
    fn run_pipeline_lock_is_present_and_reentrant_across_threads() {
        // Acquire the lock directly to confirm it compiles and works.
        {
            let _g = pipeline_lock().lock().unwrap();
            // lock held; a second try_lock must fail
            assert!(
                pipeline_lock().try_lock().is_err(),
                "PIPELINE_LOCK must be held exclusively"
            );
        }
        // After drop, a second acquisition must succeed
        assert!(
            pipeline_lock().try_lock().is_ok(),
            "PIPELINE_LOCK must be released after guard drops"
        );

        // Smoke-test: multiple threads calling run_pipeline on empty dirs all succeed.
        let failed = Arc::new(AtomicBool::new(false));
        let handles: Vec<_> = (0..3)
            .map(|_| {
                let failed = failed.clone();
                let root = tmp();
                let sources = root.join("sources");
                let generated = root.join("generated");
                fs::create_dir_all(&sources).unwrap();
                fs::create_dir_all(&generated).unwrap();
                std::thread::spawn(move || {
                    let cfg = PipelineConfig {
                        sources_dir: sources,
                        generated_dir: generated,
                        now_yyyymm: "2026-01".to_string(),
                        force: false,
                        default_expense_account: "expenses:unknown".to_string(),
                        changed_folder_hint: None,
                    };
                    if run_pipeline(&cfg).is_err() {
                        failed.store(true, Ordering::SeqCst);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert!(!failed.load(Ordering::SeqCst), "a pipeline run panicked or errored unexpectedly");
    }
}

#[cfg(test)]
mod suggest_transform_tests {
    use super::*;
    use std::time::SystemTime;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "emuacc-suggest-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_config(sources: PathBuf) -> PipelineConfig {
        let generated = sources.parent().unwrap().join("generated");
        fs::create_dir_all(&generated).unwrap();
        PipelineConfig {
            sources_dir: sources,
            generated_dir: generated,
            now_yyyymm: "2026-01".to_string(),
            force: false,
            default_expense_account: "expenses:unknown".to_string(),
            changed_folder_hint: None,
        }
    }

    fn minimal_csv(path: &Path) {
        fs::write(path, "Date,Payee,Amount\n2026-01-01,Test,10\n").unwrap();
    }

    /// When importing to a new subfolder that doesn't exist yet, if a parent
    /// folder already has a _transform.rhai the function must return Ok(None)
    /// (no suggestion needed), not Ok(Some(...)).
    #[test]
    fn no_suggestion_when_parent_has_transform_and_dest_does_not_exist() {
        let root = tmp();
        let sources = root.join("sources");
        fs::create_dir_all(&sources).unwrap();

        // Parent folder has a transform but the child folder does not exist yet
        let parent_folder = sources.join("alex");
        fs::create_dir_all(&parent_folder).unwrap();
        fs::write(
            parent_folder.join("_transform.rhai"),
            r#"#{ date: row["Date"], payee: row["Payee"], amount: row["Amount"], commodity: "AUD" }"#,
        )
        .unwrap();

        // The CSV file to import (lives outside sources; the dest folder doesn't exist)
        let csv = root.join("import.csv");
        minimal_csv(&csv);

        let config = make_config(sources);
        let result =
            suggest_transform_for_import(&config, &csv, "alex/new-wallet", None).unwrap();
        assert!(
            result.is_none(),
            "expected no suggestion (parent transform covers the new subfolder), got: {result:?}"
        );
    }

    /// OFX files are parsed natively by the pipeline (ofx_parser + rules) and
    /// never go through a Rhai transform, so importing one must not ask for a
    /// transform — even into a folder with no transform anywhere.
    #[test]
    fn no_suggestion_for_ofx_import() {
        let root = tmp();
        let sources = root.join("sources");
        fs::create_dir_all(&sources).unwrap();

        let ofx = root.join("import.ofx");
        fs::write(
            &ofx,
            "OFXHEADER:100\n\n<OFX><BANKMSGSRSV1></BANKMSGSRSV1></OFX>\n",
        )
        .unwrap();

        let config = make_config(sources);
        let result =
            suggest_transform_for_import(&config, &ofx, "alex/new-bank", None).unwrap();
        assert!(
            result.is_none(),
            "expected no suggestion for OFX (parsed natively, no transform), got: {result:?}"
        );
    }
}
