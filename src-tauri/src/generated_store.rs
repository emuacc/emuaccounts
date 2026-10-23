use crate::ledger_parser::{
    parse_transactions, AccountBalance, CommodityAmount, Diagnostic, ParseResult, Posting,
};
use crate::processing_pipeline::{folder_to_account_name, txn_sort_key, PipelineMetadata};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use walkdir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualPostingInput {
    pub account: String,
    pub amount: String,
    pub commodity: String,
    pub remainder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTransactionInput {
    /// Either `YYYY-MM-DD` or a full datetime supported by the ledger grammar.
    pub datetime: String,
    pub status: Option<char>,
    pub payee: String,
    pub narration: String,
    pub postings: Vec<ManualPostingInput>,
}

pub fn quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub fn posting_to_text(posting: &Posting) -> String {
    let mut out = format!(
        "    {} {} {}",
        posting.account, posting.amount_text, posting.commodity
    );
    // Price annotation (e.g. "@ 0.042 AUD")
    if let Some(ref price) = posting.price {
        if posting
            .remainder
            .as_ref()
            .is_none_or(|r| !r.contains('@'))
        {
            if price.is_total {
                out.push_str(&format!(" @@ {} {}", price.amount_text, price.commodity));
            } else {
                out.push_str(&format!(" @ {} {}", price.amount_text, price.commodity));
            }
        }
    }
    if let Some(rem) = posting.remainder.as_ref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }) {
        out.push(' ');
        out.push_str(rem);
    }
    out
}

fn looks_like_header(line: &str) -> bool {
    let b = line.as_bytes();
    if b.len() < 10 {
        return false;
    }
    b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2].is_ascii_digit()
        && b[3].is_ascii_digit()
        && b[4] == b'-'
        && b[5].is_ascii_digit()
        && b[6].is_ascii_digit()
        && b[7] == b'-'
        && b[8].is_ascii_digit()
        && b[9].is_ascii_digit()
}

pub fn normalize_blank_lines(contents: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut prev_nonblank_was_posting = false;

    for raw_line in contents.split('\n') {
        let is_blank = raw_line.trim().is_empty();
        let is_posting = raw_line.starts_with("    ");
        let is_header = looks_like_header(raw_line);

        if is_header && prev_nonblank_was_posting
            && out.last().is_some_and(|l| !l.trim().is_empty()) {
                out.push(String::new());
            }

        out.push(raw_line.to_string());

        if is_blank {
            prev_nonblank_was_posting = false;
        } else {
            prev_nonblank_was_posting = is_posting;
        }
    }

    while out.last().is_some_and(|l| l.is_empty()) && out.len() > 1 {
        out.pop();
    }

    out.join("\n")
}

pub fn append_text(path: &Path, text: &str) -> io::Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text)?;
        return Ok(());
    }

    let mut existing = fs::read_to_string(path)?;
    existing = normalize_blank_lines(&existing);
    if !existing.is_empty() {
        if existing.ends_with("\n\n") {
            // ok
        } else if existing.ends_with('\n') {
            existing.push('\n');
        } else {
            existing.push_str("\n\n");
        }
    }
    existing.push_str(text);
    fs::write(path, existing)
}

/// Remove the transaction block(s) containing `txn_id` from ledger text.
/// Blocks are separated by blank lines; any block whose header contains `txn_id` is dropped.
fn remove_txn_block_from_text(text: &str, txn_id: &str) -> String {
    let mut result = String::new();
    let mut current_block = String::new();
    let mut block_has_txn = false;

    for line in text.lines() {
        if line.trim().is_empty() {
            if !block_has_txn && !current_block.trim().is_empty() {
                result.push_str(&current_block);
                result.push('\n');
            }
            current_block.clear();
            block_has_txn = false;
        } else {
            if line.contains(txn_id) {
                block_has_txn = true;
            }
            current_block.push_str(line);
            current_block.push('\n');
        }
    }
    // Flush last block
    if !block_has_txn && !current_block.trim().is_empty() {
        result.push_str(&current_block);
    }
    result
}

/// Like `load_active_ledger` but removes the transaction identified by `txn_id` from
/// the in-memory text before parsing. Does not touch any files on disk.
pub fn load_active_ledger_excluding(base_dir: &Path, txn_id: &str) -> Result<ParseResult, String> {
    let mut ledger = load_ledger_text(base_dir)?;
    ledger.text = remove_txn_block_from_text(&ledger.text, txn_id);
    Ok(parse_ledger_text(&ledger))
}

/// A range of lines in the merged text that came from a specific file.
struct FileLineRange {
    file_path: String,
    start_line: usize, // 1-based, inclusive
    end_line: usize,   // 1-based, inclusive
}

/// The merged ledger text of a set, plus what the merge itself found wrong.
struct LedgerText {
    text: String,
    file_ranges: Vec<FileLineRange>,
    /// Orphan ledgers that were left out and transaction copies that were
    /// blanked, reported alongside the parser's own diagnostics.
    diagnostics: Vec<Diagnostic>,
}

/// The per-folder `ledger.transactions` files under a set directory, split into
/// the folders the pipeline wrote and the ones it did not.
///
/// `pipeline-metadata.json` (in the set directory, or one level up when the set
/// is a sub-directory of `generated/`) names every folder the pipeline writes in
/// `account_folders`. Any other `ledger.transactions` under the set is an
/// orphan: a folder an older pipeline wrote under a different case, one a git
/// checkout from another machine brought along, a source folder deleted
/// elsewhere. Reading it counts that account twice, and which copies exist
/// depends on how the filesystem treats case, so one vault gave different tax
/// figures on macOS and Linux (TASK-093). Without metadata (a hand-built set,
/// the legacy layout) every file is kept.
pub struct SetLedgers {
    /// Sorted; the ledgers reports read.
    pub files: Vec<PathBuf>,
    /// Sorted; ledgers under the set that no pipeline output folder owns.
    pub orphans: Vec<PathBuf>,
}

pub fn set_ledgers(set_dir: &Path) -> SetLedgers {
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(set_dir)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !is_hidden_name(e.file_name()))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name() == "ledger.transactions"
                // The set-root ledger is the legacy layout; the caller handles it.
                && e.path().parent() != Some(set_dir)
        })
        .map(walkdir::DirEntry::into_path)
        .collect();
    files.sort();
    let Some((generated_dir, metadata)) = pipeline_metadata_for_set(set_dir) else {
        return SetLedgers {
            files,
            orphans: Vec::new(),
        };
    };
    let written: HashSet<PathBuf> = metadata
        .written_folders()
        .map(|folder_rel| generated_dir.join(folder_rel))
        .collect();
    let (files, orphans) = files
        .into_iter()
        .partition(|path| path.parent().is_some_and(|folder| written.contains(folder)));
    SetLedgers { files, orphans }
}

fn is_hidden_name(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|n| n.starts_with('.'))
}

/// `pipeline-metadata.json` and the directory its folders are relative to: the
/// set directory itself when that is `generated/`, otherwise its parent.
fn pipeline_metadata_for_set(set_dir: &Path) -> Option<(PathBuf, PipelineMetadata)> {
    std::iter::once(set_dir)
        .chain(set_dir.parent())
        .find_map(|dir| PipelineMetadata::load(dir).map(|meta| (dir.to_path_buf(), meta)))
}

fn load_ledger_text(base_dir: &Path) -> Result<LedgerText, String> {
    let mut builder = LedgerTextBuilder::new();

    // Account declarations live at the set root.
    let accounts_path = base_dir.join("accounts.transactions");
    if accounts_path.exists() {
        builder.append_file(&accounts_path)?;
    }

    let SetLedgers { files, orphans } = set_ledgers(base_dir);
    if !files.is_empty() {
        for path in &files {
            builder.append_file(path)?;
        }
    } else {
        // Legacy layout: archive/*.transactions + root ledger.transactions
        let archive_dir = base_dir.join("archive");
        if archive_dir.exists() {
            let mut archive_files: Vec<PathBuf> = fs::read_dir(&archive_dir)
                .map_err(|e| format!("failed to read archive dir: {e}"))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("transactions"))
                .collect();
            archive_files.sort();
            for path in &archive_files {
                builder.append_file(path)?;
            }
        }
        let ledger = base_dir.join("ledger.transactions");
        if ledger.exists() {
            builder.append_file(&ledger)?;
        }
    }

    Ok(builder.finish(&orphans))
}

/// Builds the merged ledger text of a set one file at a time, recording which
/// lines came from which file and blanking any transaction block already read.
///
/// A block is blanked, not skipped, so every other line keeps its number and
/// parser diagnostics still resolve to the right file and line. Identity is
/// the block's text, gated on a `txn:` id: a self-transfer between two wallets
/// carries one `txn:` hash in both wallet ledgers with different postings (see
/// cross_folder_txn_propagation.feature) and must count twice, whereas a
/// byte-identical copy of an id is one event read twice.
struct LedgerTextBuilder {
    text: String,
    file_ranges: Vec<FileLineRange>,
    /// Paths appended so far, in order; the first occurrence of a block is
    /// reported by index because its line range is only known once the file
    /// is complete.
    files: Vec<String>,
    /// Next global line number, 1-based.
    current_line: usize,
    /// Blocks appended so far, keyed by the hash of their trimmed text: the
    /// byte range in `text` and the index into `files`.
    seen_blocks: HashMap<u64, Vec<(Range<usize>, usize)>>,
    diagnostics: Vec<Diagnostic>,
}

impl LedgerTextBuilder {
    fn new() -> Self {
        Self {
            text: String::new(),
            file_ranges: Vec::new(),
            files: Vec::new(),
            current_line: 1,
            seen_blocks: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    fn append_file(&mut self, path: &Path) -> Result<(), String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        if contents.trim().is_empty() {
            return Ok(());
        }
        let file_path = path.to_string_lossy().to_string();
        let file_index = self.files.len();
        self.files.push(file_path.clone());
        let start_line = self.current_line;
        for block in transaction_blocks(&contents) {
            self.append_block(block, file_index);
        }
        if !self.text.ends_with('\n') {
            self.text.push('\n');
        }
        let end_line = self.current_line - 1;
        // Blank line between files.
        self.text.push('\n');
        self.current_line += 1;
        self.file_ranges.push(FileLineRange {
            file_path,
            start_line,
            end_line,
        });
        Ok(())
    }

    fn append_block(&mut self, block: &str, file_index: usize) {
        let line_count = block.matches('\n').count() + usize::from(!block.ends_with('\n'));
        if let Some(txn_id) = block_txn_id(block) {
            let trimmed = block.trim_end();
            let key = hash_str(trimmed);
            let first_file = self.seen_blocks.get(&key).and_then(|seen| {
                seen.iter()
                    .find(|(range, _)| &self.text[range.clone()] == trimmed)
                    .map(|(_, file_index)| *file_index)
            });
            if let Some(first_file) = first_file {
                self.diagnostics.push(Diagnostic {
                    line: self.current_line,
                    column: 1,
                    message: format!(
                        "{txn_id} is already in {}; this copy is not counted",
                        self.files[first_file]
                    ),
                    file: Some(self.files[file_index].clone()),
                });
                self.text.extend(std::iter::repeat_n('\n', line_count));
                self.current_line += line_count;
                return;
            }
            let start = self.text.len();
            self.seen_blocks
                .entry(key)
                .or_default()
                .push((start..start + trimmed.len(), file_index));
        }
        self.text.push_str(block);
        self.current_line += line_count;
    }

    fn finish(mut self, orphans: &[PathBuf]) -> LedgerText {
        for orphan in orphans {
            let folder = orphan.parent().unwrap_or(orphan);
            self.diagnostics.push(Diagnostic {
                line: 0,
                column: 0,
                message: format!(
                    "{} is not a folder the pipeline writes (not in pipeline-metadata.json \
                     account_folders); its ledger is not counted. Rebuild, or delete the folder",
                    folder.display()
                ),
                file: Some(orphan.to_string_lossy().to_string()),
            });
        }
        LedgerText {
            text: self.text,
            file_ranges: self.file_ranges,
            diagnostics: self.diagnostics,
        }
    }
}

/// Split ledger text into contiguous slices: a preamble (anything before the
/// first transaction header), then one slice per header line with the lines up
/// to the next header. The slices concatenate back to `contents` exactly.
fn transaction_blocks(contents: &str) -> Vec<&str> {
    let mut starts: Vec<usize> = Vec::new();
    let mut offset = 0;
    for line in contents.split_inclusive('\n') {
        if looks_like_header(line) {
            starts.push(offset);
        }
        offset += line.len();
    }
    let mut blocks = Vec::with_capacity(starts.len() + 1);
    let mut prev = 0;
    for &start in &starts {
        if start > prev {
            blocks.push(&contents[prev..start]);
        }
        prev = start;
    }
    blocks.push(&contents[prev..]);
    blocks
}

/// The `txn:` anchor on a block's header line, when it has one.
fn block_txn_id(block: &str) -> Option<&str> {
    let header = block.lines().next()?;
    if !looks_like_header(header) {
        return None;
    }
    header
        .split([' ', ','])
        .map(str::trim)
        .find(|token| token.len() > "txn:".len() && token.starts_with("txn:"))
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Given a global line number in the merged text, find which file it came from.
fn resolve_file_for_line(file_ranges: &[FileLineRange], line: usize) -> Option<&str> {
    for range in file_ranges {
        if line >= range.start_line && line <= range.end_line {
            return Some(&range.file_path);
        }
    }
    None
}

/// Parse merged set text and attach what the merge itself found: the file each
/// parser diagnostic belongs to, the ledgers left out as orphans, and the
/// transaction copies that were not counted.
fn parse_ledger_text(ledger: &LedgerText) -> ParseResult {
    let mut result = if ledger.text.trim().is_empty() {
        empty_parse_result()
    } else {
        parse_transactions(&ledger.text)
    };
    for diag in &mut result.diagnostics {
        if diag.file.is_none() {
            diag.file =
                resolve_file_for_line(&ledger.file_ranges, diag.line).map(|s| s.to_string());
        }
    }
    result.diagnostics.extend(ledger.diagnostics.iter().cloned());
    result
}

/// A successful parse of nothing: what an empty vault or set yields.
pub fn empty_parse_result() -> ParseResult {
    ParseResult {
        ok: true,
        diagnostics: Vec::new(),
        transactions: Vec::new(),
        balances: Vec::new(),
        opening_balances: Vec::new(),
        accounts_with_opening: Vec::new(),
        account_properties: std::collections::HashMap::new(),
    }
}

pub fn load_active_ledger(base_dir: &Path) -> Result<ParseResult, String> {
    let ledger = load_ledger_text(base_dir)?;
    let mut result = parse_ledger_text(&ledger);
    // Sort transactions by datetime (per-folder files are individually sorted but need global sort).
    // The 3-key sort `(datetime, meta, txn_sort_key)` matches `sort_tagged_txns` in the
    // pipeline — without the third key, two transactions sharing (datetime, meta) would
    // retain file-input order, which is a different answer to what the pipeline wrote,
    // making downstream report output non-reproducible across (pipeline write → loader read).
    result.transactions.sort_by(|a, b| {
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
    Ok(result)
}

/// Load account tree from per-folder summary.json files.
/// Returns aggregated balances without parsing any transactions.
pub fn load_account_tree(set_dir: &Path) -> Result<Vec<AccountBalance>, String> {
    let mut all_balances: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, f64>,
    > = std::collections::BTreeMap::new();

    // Also load account declarations for accounts_with_opening info
    let accounts_path = set_dir.join("accounts.transactions");
    if accounts_path.exists() {
        if let Ok(text) = fs::read_to_string(&accounts_path) {
            let parsed = parse_transactions(&text);
            for b in &parsed.balances {
                // Ensure declared accounts with no transactions still appear
                all_balances.entry(b.account.clone()).or_default();
                for t in &b.totals {
                    *all_balances
                        .entry(b.account.clone())
                        .or_default()
                        .entry(t.commodity.clone())
                        .or_insert(0.0) += t.amount;
                }
            }
        }
    }

    // Per-folder summaries, for the folders the pipeline wrote only: an orphan
    // folder's summary would count its account a second time.
    for ledger in set_ledgers(set_dir).files {
        let Some(summary_path) = ledger.parent().map(|folder| folder.join("summary.json")) else {
            continue;
        };
        if let Ok(text) = fs::read_to_string(&summary_path) {
            if let Ok(summary) =
                serde_json::from_str::<crate::processing_pipeline::FolderSummary>(&text)
            {
                for b in &summary.balances {
                    for t in &b.totals {
                        *all_balances
                            .entry(b.account.clone())
                            .or_default()
                            .entry(t.commodity.clone())
                            .or_insert(0.0) += t.amount;
                    }
                }
            }
        }
    }

    Ok(all_balances
        .into_iter()
        .map(|(account, by_commodity)| AccountBalance {
            account,
            totals: by_commodity
                .into_iter()
                .map(|(commodity, amount)| CommodityAmount { commodity, amount })
                .collect(),
        })
        .collect())
}

/// Load transactions from per-folder files scoped to an account prefix.
/// Only reads folders whose derived account name matches the prefix.
/// Returns parsed, sorted transactions.
pub fn load_scoped_transactions(
    set_dir: &Path,
    account_prefix: &str,
) -> Result<Vec<crate::ledger_parser::Transaction>, String> {
    // Only the folders the pipeline wrote, in path order, so transactions that
    // tie on datetime/meta come out in the same order on every filesystem
    // (ext4 readdir order is arbitrary; APFS happens to be sorted). Merging
    // through the builder also drops a transaction a second file repeats.
    let mut builder = LedgerTextBuilder::new();
    for path in set_ledgers(set_dir)
        .files
        .iter()
        .filter(|path| folder_matches_account_prefix(set_dir, path, account_prefix))
    {
        builder.append_file(path)?;
    }
    let mut all_txns = parse_ledger_text(&builder.finish(&[])).transactions;

    // Sort by datetime desc (most recent first)
    all_txns.sort_by(|a, b| {
        b.datetime.cmp(&a.datetime).then_with(|| {
            b.meta
                .as_deref()
                .unwrap_or("")
                .cmp(a.meta.as_deref().unwrap_or(""))
        })
    });

    Ok(all_txns)
}

fn folder_matches_account_prefix(set_dir: &Path, ledger_path: &Path, account_prefix: &str) -> bool {
    if account_prefix.is_empty() {
        return true;
    }
    let Some(folder_path) = ledger_path.parent() else {
        return false;
    };
    let Ok(folder_relative) = folder_path.strip_prefix(set_dir) else {
        return false;
    };
    let folder_relative = folder_relative.to_string_lossy();
    if folder_relative.is_empty() {
        return false;
    }
    let derived = folder_to_account_name(&format!("__set__/{folder_relative}"));
    derived == account_prefix || derived.starts_with(&format!("{account_prefix}:"))
}

/// Read the `hidden_accounts` list from `config.json` in a generated set
/// directory. Bare names (no `:`) get a `:` appended so the returned strings
/// are always usable as `starts_with` prefixes — matching `ignore` against
/// `ignore:hidden` etc. Returns an empty vec when the file or key is absent.
pub fn load_hidden_accounts(set_dir: &Path) -> Vec<String> {
    let config_path = set_dir.join("config.json");
    let Ok(contents) = std::fs::read_to_string(&config_path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return Vec::new();
    };
    let Some(arr) = json.get("hidden_accounts").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|v| {
            v.as_str().map(|s| {
                let prefix = s.to_string();
                if prefix.contains(':') {
                    prefix
                } else {
                    format!("{prefix}:")
                }
            })
        })
        .collect()
}

/// Remove transactions that post to any hidden account prefix and recompute
/// balances from what remains: the surviving postings PLUS the opening
/// declarations of non-hidden accounts. Rebuilding from postings alone used to
/// silently drop every opening balance from the sidebar after any full
/// pipeline response.
pub fn filter_hidden_accounts(parse: &mut ParseResult, prefixes: &[String]) {
    let is_hidden = |account: &str| prefixes.iter().any(|pfx| account.starts_with(pfx.as_str()));
    parse
        .transactions
        .retain(|txn| !txn.postings.iter().any(|p| is_hidden(&p.account)));
    parse.opening_balances.retain(|b| !is_hidden(&b.account));
    parse.accounts_with_opening.retain(|a| !is_hidden(a));

    let mut balances_map: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, f64>,
    > = std::collections::BTreeMap::new();
    for txn in &parse.transactions {
        for posting in &txn.postings {
            *balances_map
                .entry(posting.account.clone())
                .or_default()
                .entry(posting.commodity.clone())
                .or_insert(0.0) += posting.amount;
        }
    }
    for opening in &parse.opening_balances {
        for t in &opening.totals {
            *balances_map
                .entry(opening.account.clone())
                .or_default()
                .entry(t.commodity.clone())
                .or_insert(0.0) += t.amount;
        }
    }
    parse.balances = crate::ledger_parser::account_balances_from_map(balances_map);
}

/// Extract the first `account:` prefix from a search string. Used to scope
/// which per-folder ledgers a query reads; empty means a whole-set scan.
///
/// Only a plain account name can prune folders. A regex term such as
/// `account:^assets:(assets:)?cash$` (what the CLI and scripts send) is not a
/// folder prefix — string-comparing it against derived folder names would
/// match nothing and silently return zero rows — so it yields a whole-set
/// scan and the regex still filters postings in the query engine.
pub fn account_prefix_from_search(search: &str) -> String {
    for part in search.split(" AND ") {
        if let Some(rest) = part.trim().strip_prefix("account:") {
            if rest.contains(|c: char| r"^$()[]{}|*+?\.".contains(c)) {
                return String::new();
            }
            return rest.to_string();
        }
    }
    String::new()
}

/// Does `account` fall under the folder scope `prefix` (empty = everything)?
fn in_account_scope(account: &str, prefix: &str) -> bool {
    prefix.is_empty() || account == prefix || account.starts_with(&format!("{prefix}:"))
}

/// Opening balances declared in the set-root `accounts.transactions`,
/// restricted to the folder scope `account_prefix` and, unless `show_hidden`,
/// minus the configured hidden accounts. Empty when the file is absent.
///
/// This is the single source of openings for both the app header total
/// ([`scoped_query`]'s `aggregated_balance`) and the CLI's posting-level
/// `--balances`, so the two cannot disagree on what an account opened with.
pub fn load_openings(
    set_dir: &Path,
    account_prefix: &str,
    show_hidden: bool,
) -> Vec<AccountBalance> {
    let Ok(text) = fs::read_to_string(set_dir.join("accounts.transactions")) else {
        return Vec::new();
    };
    let hidden_prefixes = if show_hidden {
        Vec::new()
    } else {
        load_hidden_accounts(set_dir)
    };
    parse_transactions(&text)
        .balances
        .into_iter()
        .filter(|ob| in_account_scope(&ob.account, account_prefix))
        .filter(|ob| {
            !hidden_prefixes
                .iter()
                .any(|pfx| ob.account.starts_with(pfx.as_str()))
        })
        .collect()
}

/// Parameters for [`scoped_query`]. `search` is the only required field.
#[derive(Default)]
pub struct ScopedQueryOptions<'a> {
    pub search: &'a str,
    /// Explicit folder scope (an account name such as `assets:cash:cba:bta`;
    /// `Some("")` = no pruning, scan the whole set). `None` derives it from
    /// the first plain `account:` term in `search`. The CLI sets it from the
    /// DIR positional so a top-level `OR` in the user's search cannot escape
    /// the folder.
    pub folder_scope: Option<&'a str>,
    pub show_hidden: bool,
    pub sort_field: Option<&'a str>,
    pub sort_order: Option<&'a str>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub min_value: Option<crate::query::MinValueFilter>,
}

impl ScopedQueryOptions<'_> {
    /// The account prefix used to prune per-folder ledgers and openings.
    pub fn account_prefix(&self) -> String {
        match self.folder_scope {
            Some(scope) => scope.to_string(),
            None => account_prefix_from_search(self.search),
        }
    }
}

/// Aggregate per-account, per-commodity balances from a transaction list.
fn balances_from_transactions(
    transactions: &[crate::ledger_parser::Transaction],
) -> Vec<AccountBalance> {
    let mut bmap: std::collections::BTreeMap<String, std::collections::BTreeMap<String, f64>> =
        std::collections::BTreeMap::new();
    for txn in transactions {
        for p in &txn.postings {
            *bmap
                .entry(p.account.clone())
                .or_default()
                .entry(p.commodity.clone())
                .or_insert(0.0) += p.amount;
        }
    }
    bmap.into_iter()
        .map(|(account, by_commodity)| AccountBalance {
            account,
            totals: by_commodity
                .into_iter()
                .map(|(commodity, amount)| CommodityAmount { commodity, amount })
                .collect(),
        })
        .collect()
}

/// Run a scoped query against a generated set: read only the per-folder
/// ledgers the `account:` prefix touches, drop hidden-account transactions
/// when `show_hidden` is off, then filter/sort/paginate via the query engine.
///
/// This is the single code path behind the desktop `query_search` command, the
/// web dispatch, and the BDD harness — they call it rather than re-deriving the
/// scope/filter/balance logic, so they can never silently diverge.
///
/// When `show_hidden` is on, the configured hidden prefixes are handed to the
/// query engine so a fully-hidden transaction still in the scoped folder
/// reappears in the row list. Balance aggregation always uses the original
/// search terms, so totals are unaffected by the toggle.
pub fn scoped_query(
    set_dir: &Path,
    opts: ScopedQueryOptions<'_>,
) -> Result<crate::query::QueryResult, String> {
    use crate::query;

    let account_prefix = opts.account_prefix();
    let ScopedQueryOptions {
        search,
        show_hidden,
        sort_field,
        sort_order,
        offset,
        limit,
        min_value,
        ..
    } = opts;
    let hidden_prefixes = load_hidden_accounts(set_dir);
    let mut transactions = load_scoped_transactions(set_dir, &account_prefix)?;
    if !show_hidden && !hidden_prefixes.is_empty() {
        transactions.retain(|txn| {
            !txn.postings.iter().any(|p| {
                hidden_prefixes
                    .iter()
                    .any(|pfx| p.account.starts_with(pfx.as_str()))
            })
        });
    }

    let mut balances = balances_from_transactions(&transactions);
    // Fold in opening balances from the set-root accounts.transactions so the
    // header prefix-query total matches the sidebar tree (load_account_tree);
    // global_query already includes them via load_ledger_text.
    let opening_balances = load_openings(set_dir, &account_prefix, show_hidden);
    for ob in &opening_balances {
        match balances.iter_mut().find(|b| b.account == ob.account) {
            Some(b) => {
                for t in &ob.totals {
                    match b.totals.iter_mut().find(|x| x.commodity == t.commodity) {
                        Some(x) => x.amount += t.amount,
                        None => b.totals.push(t.clone()),
                    }
                }
            }
            None => balances.push(ob.clone()),
        }
    }
    let parse = ParseResult {
        ok: true,
        diagnostics: Vec::new(),
        transactions,
        balances,
        opening_balances,
        accounts_with_opening: Vec::new(),
        account_properties: std::collections::HashMap::new(),
    };

    let expr = query::parse_search(search).map_err(|e| e.to_string())?;
    let sf = sort_field
        .map(|s| query::SortField::parse_str(s).ok_or_else(|| format!("Unknown sort field: {s}")))
        .transpose()?;
    let so = sort_order
        .map(|s| query::SortOrder::parse_str(s).ok_or_else(|| format!("Unknown sort order: {s}")))
        .transpose()?
        .unwrap_or(query::SortOrder::Asc);

    Ok(query::query(
        &parse,
        &query::QueryOptions {
            search: expr,
            sort_field: sf,
            sort_order: so,
            offset,
            limit,
            input_order: Some(query::InputOrder::DateDesc),
            min_value,
            hidden_prefixes: if show_hidden {
                hidden_prefixes
            } else {
                Vec::new()
            },
        },
    ))
}

/// Run a query against the UNION of every per-folder ledger in a set (no folder
/// pruning), so non-folder-backed accounts — `income:*`, `expenses:*`,
/// `equity:*`, `liabilities:*`, and the `assets` contras (staking, lending,
/// transfer, …) — are queryable. The folder-pruned [`scoped_query`] returns zero
/// rows for these because no source folder derives to their account name. Uses
/// [`load_active_ledger`] (union) + `query::query`.
///
/// NOTE: `load_active_ledger` sorts ascending, unlike `load_scoped_transactions`
/// (descending), so this passes `input_order: None` to force a real sort in
/// `query::query`. Passing `Some(InputOrder::DateDesc)` here would silently
/// reverse the default newest-first ordering.
pub fn global_query(
    set_dir: &Path,
    search: &str,
    show_hidden: bool,
    sort_field: Option<&str>,
    sort_order: Option<&str>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<crate::query::QueryResult, String> {
    use crate::query;

    let hidden_prefixes = load_hidden_accounts(set_dir);
    let mut parse = load_active_ledger(set_dir)?;
    if !show_hidden && !hidden_prefixes.is_empty() {
        filter_hidden_accounts(&mut parse, &hidden_prefixes);
    }

    let expr = query::parse_search(search).map_err(|e| e.to_string())?;
    let sf = sort_field
        .map(|s| query::SortField::parse_str(s).ok_or_else(|| format!("Unknown sort field: {s}")))
        .transpose()?;
    let so = sort_order
        .map(|s| query::SortOrder::parse_str(s).ok_or_else(|| format!("Unknown sort order: {s}")))
        .transpose()?
        .unwrap_or(query::SortOrder::Asc);

    Ok(query::query(
        &parse,
        &query::QueryOptions {
            search: expr,
            sort_field: sf,
            sort_order: so,
            offset,
            limit,
            input_order: None,
            min_value: None,
            hidden_prefixes: if show_hidden {
                hidden_prefixes
            } else {
                Vec::new()
            },
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_ledger(dir: &Path, folder: &str, contents: &str) {
        let path = dir.join(folder).join("ledger.transactions");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write ledger");
    }

    fn txn(payee: &str, account: &str) -> String {
        format!(
      "2025-01-01 * \"{payee}\" \"{payee}\"\n    {account} 1 USD\n    expenses:unknown -1 USD\n"
    )
    }

    /// Write `pipeline-metadata.json` naming `(set, account, folder)` as the
    /// folders the pipeline wrote, through the real struct so the test cannot
    /// drift from the file's shape.
    fn write_metadata(generated: &Path, account_folders: &[(&str, &str, &str)]) {
        let mut folders = crate::processing_pipeline::AccountFolders::new();
        for (set, account, folder) in account_folders {
            folders
                .entry((*set).to_string())
                .or_default()
                .insert((*account).to_string(), (*folder).to_string());
        }
        let metadata = PipelineMetadata {
            owner_accounts: HashMap::new(),
            account_folders: folders,
            account_properties: HashMap::new(),
            accounts_with_opening: Vec::new(),
        };
        let json = serde_json::to_string(&metadata).expect("serialize metadata");
        fs::write(generated.join("pipeline-metadata.json"), json).expect("write metadata");
    }

    /// A transaction block with the meta comment the pipeline always writes.
    fn txn_with_id(payee: &str, account: &str, id: &str) -> String {
        format!(
            "2025-01-01 * \"{payee}\" \"{payee}\" ; txn:{id}\n    {account} 1 USD\n    expenses:unknown -1 USD\n"
        )
    }

    fn payees(parse: &ParseResult) -> Vec<&str> {
        parse
            .transactions
            .iter()
            .filter_map(|t| t.payee.as_deref())
            .collect()
    }

    #[test]
    fn set_ledgers_keeps_only_the_folders_in_account_folders() {
        let generated = tmp_set_dir("set-ledgers");
        let set_dir = generated.join("alex");
        write_ledger(&set_dir, "crypto/wallet/foo", &txn_with_id("kept", "assets:crypto:wallet:foo", "kept"));
        write_ledger(
            &set_dir,
            "crypto/wallet/Foo-old",
            &txn_with_id("stale", "assets:crypto:wallet:foo", "stale"),
        );
        write_ledger(&set_dir, ".trash/crypto/wallet/foo", &txn_with_id("hidden", "assets:crypto:wallet:foo", "hidden"));
        write_metadata(
            &generated,
            &[("alex", "assets:crypto:wallet:foo", "alex/crypto/wallet/foo")],
        );

        let ledgers = set_ledgers(&set_dir);
        assert_eq!(
            ledgers.files,
            vec![set_dir.join("crypto/wallet/foo/ledger.transactions")]
        );
        assert_eq!(
            ledgers.orphans,
            vec![set_dir.join("crypto/wallet/Foo-old/ledger.transactions")]
        );

        let parse = load_active_ledger(&set_dir).expect("load active ledger");
        assert_eq!(payees(&parse), vec!["kept"]);
        assert_eq!(parse.diagnostics.len(), 1, "{:?}", parse.diagnostics);
        assert!(
            parse.diagnostics[0].message.contains("Foo-old"),
            "{}",
            parse.diagnostics[0].message
        );
        assert_eq!(
            parse.diagnostics[0].file.as_deref(),
            Some(ledgers.orphans[0].to_string_lossy().as_ref())
        );

        let _ = fs::remove_dir_all(&generated);
    }

    #[test]
    fn set_ledgers_without_metadata_keeps_every_ledger() {
        let set_dir = tmp_set_dir("no-metadata");
        write_ledger(&set_dir, "cash/a", &txn_with_id("a", "assets:cash:a", "a"));
        write_ledger(&set_dir, "cash/b", &txn_with_id("b", "assets:cash:b", "b"));

        let ledgers = set_ledgers(&set_dir);
        assert_eq!(ledgers.files.len(), 2);
        assert!(ledgers.orphans.is_empty());
        assert!(load_active_ledger(&set_dir)
            .expect("load active ledger")
            .diagnostics
            .is_empty());

        let _ = fs::remove_dir_all(&set_dir);
    }

    #[test]
    fn a_repeated_block_is_counted_once_and_reported_with_its_file_and_line() {
        let set_dir = tmp_set_dir("duplicate-block");
        let block = "2025-01-15 * \"Coffee\" \"imported\" ; txn:dup-001\n    assets:cash:bank -4.50 AUD\n    expenses:unknown 4.50 AUD\n";
        write_ledger(
            &set_dir,
            "cash/bank",
            &format!("{}\n{block}", txn_with_id("first", "assets:cash:bank", "first")),
        );
        write_ledger(&set_dir, "cash/bank-old", block);

        let parse = load_active_ledger(&set_dir).expect("load active ledger");

        assert_eq!(payees(&parse), vec!["first", "Coffee"]);
        assert_eq!(parse.diagnostics.len(), 1, "{:?}", parse.diagnostics);
        let diag = &parse.diagnostics[0];
        assert!(
            diag.message.starts_with("txn:dup-001 is already in"),
            "{}",
            diag.message
        );
        assert_eq!(
            diag.file.as_deref(),
            Some(
                set_dir
                    .join("cash/bank-old/ledger.transactions")
                    .to_string_lossy()
                    .as_ref()
            )
        );
        // cash/bank occupies lines 1-7, line 8 separates the files, so the
        // copy's header is line 9 of the merged text.
        assert_eq!(diag.line, 9);
        let aud = parse
            .balances
            .iter()
            .find(|b| b.account == "assets:cash:bank")
            .and_then(|b| b.totals.iter().find(|t| t.commodity == "AUD"))
            .map(|t| t.amount)
            .expect("AUD balance");
        assert!((aud + 4.5).abs() < 1e-9, "counted once, got {aud}");

        let _ = fs::remove_dir_all(&set_dir);
    }

    #[test]
    fn transaction_blocks_concatenate_back_to_the_input() {
        let text = "; preamble comment\naccount assets:cash\n2025-01-01 * \"a\" \"a\" ; txn:1\n    assets:cash 1 AUD\n2025-01-02 * \"b\" \"b\" ; txn:2\n    assets:cash 2 AUD\n\n; trailing\n2025-01-03 * \"c\"";
        let blocks = transaction_blocks(text);
        assert_eq!(blocks.concat(), text);
        assert_eq!(blocks.len(), 4, "{blocks:?}");
        assert!(blocks[0].starts_with("; preamble"));
        assert_eq!(block_txn_id(blocks[0]), None);
        assert_eq!(block_txn_id(blocks[1]), Some("txn:1"));
        assert_eq!(block_txn_id(blocks[2]), Some("txn:2"));
        assert_eq!(block_txn_id(blocks[3]), None);
    }

    #[test]
    fn scoped_load_only_reads_exact_and_descendant_folders() {
        let set_dir = std::env::temp_dir().join(format!(
            "emuacc-scoped-load-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        fs::create_dir_all(&set_dir).expect("create temp dir");

        write_ledger(&set_dir, "cash", &txn("parent", "assets:cash"));
        write_ledger(
            &set_dir,
            "cash/bank",
            &(txn("ancestor", "assets:cash:bank:savings")),
        );
        write_ledger(
            &set_dir,
            "cash/bank/savings",
            &txn("exact", "assets:cash:bank:savings"),
        );
        write_ledger(
            &set_dir,
            "cash/bank/savings/bonus",
            &txn("descendant", "assets:cash:bank:savings:bonus"),
        );
        write_ledger(
            &set_dir,
            "cash/bank/checking",
            &txn("sibling", "assets:cash:bank:checking"),
        );

        let txns = load_scoped_transactions(&set_dir, "assets:cash:bank:savings")
            .expect("load scoped transactions");
        let payees: Vec<&str> = txns.iter().filter_map(|t| t.payee.as_deref()).collect();

        assert_eq!(payees, vec!["descendant", "exact"]);

        let _ = fs::remove_dir_all(&set_dir);
    }

    #[test]
    fn global_query_finds_non_folder_backed_accounts_scoped_misses() {
        let set_dir = std::env::temp_dir().join(format!(
            "emuacc-global-query-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        fs::create_dir_all(&set_dir).expect("create temp dir");

        // A staking move lives INSIDE a wallet folder: the contra `assets:staking`
        // leg is recorded in the wallet's ledger, but no source folder derives to
        // the name `assets:staking`, so the folder-pruned scoped query can't reach it.
        write_ledger(
            &set_dir,
            "crypto/wallet/solana/ABC",
            "2025-01-01 * \"Stake\" \"Stake LFNTY\"\n    assets:crypto:wallet:solana:ABC -100 LFNTY\n    assets:staking 100 LFNTY\n",
        );

        let scoped = scoped_query(
            &set_dir,
            ScopedQueryOptions {
                search: "account:assets:staking",
                ..Default::default()
            },
        )
        .expect("scoped_query");
        assert_eq!(
            scoped.transaction_count, 0,
            "scoped_query folder-prunes and should find no staking rows"
        );

        let global = global_query(&set_dir, "account:assets:staking", false, None, None, None, None)
            .expect("global_query");
        assert_eq!(
            global.transaction_count, 1,
            "global_query unions all ledgers and should find the staking row"
        );
        let staking = global
            .aggregated_balance
            .iter()
            .find(|c| c.commodity == "LFNTY")
            .expect("LFNTY balance present in aggregated_balance");
        assert!(
            (staking.amount - 100.0).abs() < 1e-9,
            "staking balance should aggregate to 100 LFNTY, got {}",
            staking.amount
        );

        let _ = fs::remove_dir_all(&set_dir);
    }

    #[test]
    fn scoped_query_includes_opening_balances_in_aggregated_balance() {
        let set_dir = std::env::temp_dir().join(format!(
            "emuacc-scoped-opening-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        fs::create_dir_all(&set_dir).expect("create temp dir");

        fs::write(
            set_dir.join("accounts.transactions"),
            "account assets:cash:cba:bta AUD\n    opening 1,823.92 AUD\n",
        )
        .expect("write accounts");
        write_ledger(
            &set_dir,
            "cash/cba/bta",
            "2025-01-01 * \"Bank\" \"Deposit\"\n    assets:cash:cba:bta 680.10 AUD\n    expenses:unknown -680.10 AUD\n",
        );

        let result = scoped_query(
            &set_dir,
            ScopedQueryOptions {
                search: "account:assets:cash:cba:bta",
                ..Default::default()
            },
        )
        .expect("scoped_query");
        let aud = result
            .aggregated_balance
            .iter()
            .find(|c| c.commodity == "AUD")
            .expect("AUD balance present");
        assert!(
            (aud.amount - 2504.02).abs() < 1e-9,
            "header balance should include the opening (expected 2504.02, got {})",
            aud.amount
        );

        let _ = fs::remove_dir_all(&set_dir);
    }

    fn tmp_set_dir(suffix: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "emuacc-hidden-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            suffix
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_hidden_accounts_appends_colon_to_bare_prefix() {
        let dir = tmp_set_dir("bare");
        fs::write(
            dir.join("config.json"),
            r#"{"hidden_accounts": ["ignore"]}"#,
        )
        .unwrap();
        let prefixes = load_hidden_accounts(&dir);
        assert_eq!(prefixes, vec!["ignore:"]);
    }

    #[test]
    fn load_hidden_accounts_keeps_explicit_prefix_with_colon() {
        let dir = tmp_set_dir("explicit");
        fs::write(
            dir.join("config.json"),
            r#"{"hidden_accounts": ["ignore:hidden", "trash:bin"]}"#,
        )
        .unwrap();
        let prefixes = load_hidden_accounts(&dir);
        assert_eq!(prefixes, vec!["ignore:hidden", "trash:bin"]);
    }

    #[test]
    fn load_hidden_accounts_returns_empty_when_config_missing() {
        let dir = tmp_set_dir("missing");
        // No config.json written
        assert!(load_hidden_accounts(&dir).is_empty());
    }

    #[test]
    fn load_hidden_accounts_returns_empty_when_key_absent() {
        let dir = tmp_set_dir("no-key");
        fs::write(dir.join("config.json"), r#"{"base_currency": "AUD"}"#).unwrap();
        assert!(load_hidden_accounts(&dir).is_empty());
    }

    #[test]
    fn regex_account_term_does_not_prune_folders() {
        assert_eq!(
            account_prefix_from_search("account:assets:cash"),
            "assets:cash"
        );
        assert_eq!(
            account_prefix_from_search("account:^assets:(assets:)?cash$"),
            ""
        );
        assert_eq!(
            account_prefix_from_search("date:2025 AND account:assets:x"),
            "assets:x"
        );
    }

    #[test]
    fn load_openings_is_restricted_to_the_folder_scope_and_hidden_config() {
        let set_dir = tmp_set_dir("openings-scope");
        fs::write(
            set_dir.join("accounts.transactions"),
            "account assets:cash:cba:bta AUD\n    opening 1,823.92 AUD\n\
             account assets:cash:cba:other AUD\n    opening 500 AUD\n\
             account ignore:hidden AUD\n    opening 7 AUD\n",
        )
        .unwrap();
        fs::write(
            set_dir.join("config.json"),
            r#"{"hidden_accounts": ["ignore"]}"#,
        )
        .unwrap();

        let names = |v: Vec<AccountBalance>| v.into_iter().map(|b| b.account).collect::<Vec<_>>();
        assert_eq!(
            names(load_openings(&set_dir, "assets:cash:cba:bta", false)),
            vec!["assets:cash:cba:bta"]
        );
        assert_eq!(
            names(load_openings(&set_dir, "assets:cash", false)),
            vec!["assets:cash:cba:bta", "assets:cash:cba:other"]
        );
        assert_eq!(
            names(load_openings(&set_dir, "", true)),
            vec![
                "assets:cash:cba:bta",
                "assets:cash:cba:other",
                "ignore:hidden"
            ]
        );
        assert!(load_openings(&set_dir, "assets:cash:cba:bt", false).is_empty());
        let _ = fs::remove_dir_all(&set_dir);
    }

    /// Invariant: `load_active_ledger` MUST apply the same `(datetime, meta,
    /// txn_sort_key)` 3-key sort as `sort_tagged_txns` in the pipeline.
    /// Without the third key, two transactions sharing `(datetime, meta)`
    /// keep their file-input order — which depends on per-folder write order
    /// upstream and is not guaranteed stable. Then downstream report writers
    /// produce different output run-to-run.
    ///
    /// This test writes two same-(datetime, meta) transactions to disk in
    /// the OPPOSITE order in two separate runs and asserts the loaded result
    /// is identical.
    #[test]
    fn load_active_ledger_breaks_ties_deterministically_across_file_input_order() {
        // Two transactions with identical datetime and identical (empty) meta.
        // Distinguishable only by payee — `transaction_to_text` is the third
        // sort key, so the deterministic order is whichever payee-suffix
        // serialises first as ledger text.
        let txn_alpha = "2026-01-15 12:00:00 * \"Alpha payee\" \"alpha\"\n    assets:wallet:a  10 USD\n    income:other  -10 USD\n";
        let txn_omega = "2026-01-15 12:00:00 * \"Omega payee\" \"omega\"\n    assets:wallet:o  20 USD\n    income:other  -20 USD\n";

        // Run A: alpha file lex-sorts before omega → loader appends alpha first.
        let dir_a = tmp_set_dir("tiebreak-a");
        fs::create_dir_all(dir_a.join("folder_a")).unwrap();
        fs::create_dir_all(dir_a.join("folder_b")).unwrap();
        fs::write(dir_a.join("folder_a").join("ledger.transactions"), txn_alpha).unwrap();
        fs::write(dir_a.join("folder_b").join("ledger.transactions"), txn_omega).unwrap();
        let r_a = load_active_ledger(&dir_a).expect("load A");

        // Run B: swap which folder holds which transaction — now omega comes
        // off disk first. Without the txn-id tiebreak the 2-key sort would
        // keep that file-input order; with the tiebreak the output matches A.
        let dir_b = tmp_set_dir("tiebreak-b");
        fs::create_dir_all(dir_b.join("folder_a")).unwrap();
        fs::create_dir_all(dir_b.join("folder_b")).unwrap();
        fs::write(dir_b.join("folder_a").join("ledger.transactions"), txn_omega).unwrap();
        fs::write(dir_b.join("folder_b").join("ledger.transactions"), txn_alpha).unwrap();
        let r_b = load_active_ledger(&dir_b).expect("load B");

        let payees_a: Vec<_> = r_a
            .transactions
            .iter()
            .map(|t| t.payee.clone().unwrap_or_default())
            .collect();
        let payees_b: Vec<_> = r_b
            .transactions
            .iter()
            .map(|t| t.payee.clone().unwrap_or_default())
            .collect();

        assert_eq!(
            payees_a, payees_b,
            "load_active_ledger must produce deterministic order across file-input shuffles; A={payees_a:?} B={payees_b:?}"
        );

        let _ = fs::remove_dir_all(&dir_a);
        let _ = fs::remove_dir_all(&dir_b);
    }
}
