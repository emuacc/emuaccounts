//! The vault's trade links — pairs of transactions the user linked as the two
//! legs of one trade — and the trade-link suggester.
//!
//! The links are kept in `sources/`[`TRADE_LINKS_FILE`], laid out like
//! `_rules.json`: each link's keys sorted, the links sorted by id, so saving or
//! deleting one link changes only its own lines. The file is the app's state,
//! not pipeline input ([`crate::sources_state`]): the `_rules.json` entries a
//! link writes are what change the ledger.
//!
//! Before TASK-133 the links lived in the Automerge document
//! `sources/`[`METADATA_FILE`]. While the file is absent, [`load`] reads that
//! document's links ([`automerge_doc`]) and writes them to the file; nothing
//! ever modifies the document, which the opt-in sync build still uses for its
//! device-sync metadata (docs/SYNC.md).

mod automerge_doc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// The trade-link file under `sources/`.
pub const TRADE_LINKS_FILE: &str = "_trade_links.json";
/// The Automerge metadata document under `sources/`: where the trade links
/// lived before TASK-133, and the sync build's device-sync metadata.
pub const METADATA_FILE: &str = "emuacc-metadata.automerge";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TradeLink {
    pub id: String,
    pub txn_id_a: String,
    pub txn_id_b: String,
    pub device_origin: String,
    pub created_at: i64,
}

/// A vault's links by id.
type Links = BTreeMap<String, TradeLink>;

#[derive(Deserialize)]
struct TradeLinksFile {
    trade_links: Vec<TradeLink>,
}

/// Held for every read-modify-write of the file, so two edits in this process
/// never drop each other's links.
static FILE_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

/// The vault's trade links, sorted by id. When `sources_dir` has no
/// [`TRADE_LINKS_FILE`] yet but its [`METADATA_FILE`] holds links, writes them
/// to the file first: the migration from the pre-TASK-133 store.
pub fn load(sources_dir: &Path) -> Result<Vec<TradeLink>, String> {
    let _lock = FILE_LOCK.lock();
    Ok(load_migrating(sources_dir)?.into_values().collect())
}

/// [`load`] without the migration: the file's links or, while there is no
/// file, the metadata document's. Writes nothing.
pub fn read(sources_dir: &Path) -> Result<Vec<TradeLink>, String> {
    let links = match read_file(sources_dir)? {
        Some(links) => links,
        None => read_document(sources_dir)?,
    };
    Ok(links.into_values().collect())
}

/// Link each pair of transaction ids, in either order: the link's id is
/// [`trade_link_id`] of the pair, and linking a pair again replaces its link.
pub fn save(sources_dir: &Path, pairs: &[(&str, &str)]) -> Result<(), String> {
    let _lock = FILE_LOCK.lock();
    let mut links = load_migrating(sources_dir)?;
    let device_origin = get_device_id();
    let created_at = now();
    for &(txn_id_a, txn_id_b) in pairs {
        let id = trade_link_id(txn_id_a, txn_id_b);
        links.insert(
            id.clone(),
            TradeLink {
                id,
                txn_id_a: txn_id_a.to_string(),
                txn_id_b: txn_id_b.to_string(),
                device_origin: device_origin.clone(),
                created_at,
            },
        );
    }
    write_file(sources_dir, &links)
}

/// Remove the link `link_id`, if there is one.
pub fn delete(sources_dir: &Path, link_id: &str) -> Result<(), String> {
    let _lock = FILE_LOCK.lock();
    let mut links = load_migrating(sources_dir)?;
    links.remove(link_id);
    write_file(sources_dir, &links)
}

/// The file's links, migrating the document's when there is no file. The
/// caller holds [`FILE_LOCK`].
fn load_migrating(sources_dir: &Path) -> Result<Links, String> {
    if let Some(links) = read_file(sources_dir)? {
        return Ok(links);
    }
    let links = read_document(sources_dir)?;
    if !links.is_empty() {
        write_file(sources_dir, &links)?;
        eprintln!(
            "[trade-links] migrated {} link(s) from {METADATA_FILE} to {TRADE_LINKS_FILE}",
            links.len()
        );
    }
    Ok(links)
}

/// The links in [`TRADE_LINKS_FILE`]; none when there is no file. Once written
/// the file is never removed, even when it lists no link, so the document is
/// read only until the first write.
fn read_file(sources_dir: &Path) -> Result<Option<Links>, String> {
    let path = sources_dir.join(TRADE_LINKS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("failed to read {}: {e}", path.display())),
    };
    let file: TradeLinksFile = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    Ok(Some(
        file.trade_links
            .into_iter()
            .map(|link| (link.id.clone(), link))
            .collect(),
    ))
}

/// The links in [`METADATA_FILE`]; none when there is no document. A document
/// that cannot be read is an error, never an empty list.
fn read_document(sources_dir: &Path) -> Result<Links, String> {
    let path = sources_dir.join(METADATA_FILE);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Links::new()),
        Err(e) => return Err(format!("failed to read {}: {e}", path.display())),
    };
    let links = automerge_doc::trade_links(&bytes)
        .map_err(|e| format!("cannot read the trade links in {}: {e}", path.display()))?;
    Ok(links
        .into_iter()
        .map(|link| (link.id.clone(), link))
        .collect())
}

fn write_file(sources_dir: &Path, links: &Links) -> Result<(), String> {
    let path = sources_dir.join(TRADE_LINKS_FILE);
    let json = crate::to_sorted_json_pretty(&serde_json::json!({
        "trade_links": links.values().collect::<Vec<_>>()
    }))
    .map_err(|e| e.to_string())?;
    fs::create_dir_all(sources_dir)
        .and_then(|()| fs::write(&path, json))
        .map_err(|e| format!("failed to write {}: {e}", path.display()))
}

// ── Helpers ─────────────────────────────────────────────────────────

pub(crate) fn extract_txn_id(meta: &Option<String>) -> Option<String> {
    let meta = meta.as_ref()?;
    for part in meta.split(',') {
        let trimmed = part.trim();
        if trimmed.starts_with("txn:") {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn extract_src_ref(meta: &Option<String>) -> Option<String> {
    let meta = meta.as_ref()?;
    for part in meta.split(',') {
        let trimmed = part.trim();
        if trimmed.starts_with("src:") {
            return Some(trimmed.to_string());
        }
    }
    None
}

pub fn get_device_id() -> String {
    machine_uid::get()
        .map(|id| {
            let short = if id.len() >= 8 { &id[..8] } else { &id };
            format!("device-{}", short)
        })
        .unwrap_or_else(|_| "device-unknown".to_string())
}

pub(crate) fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn trade_link_id(txn_id_a: &str, txn_id_b: &str) -> String {
    let mut pair = [txn_id_a, txn_id_b];
    pair.sort();
    let mut hasher = Sha256::new();
    hasher.update(pair[0].as_bytes());
    hasher.update(pair[1].as_bytes());
    hex::encode(hasher.finalize())[..12].to_string()
}

/// Bump when `suggest_trade_links` logic changes — it participates in the
/// persisted suggestion-cache key (generated/.cache/trade-suggestions-*.json),
/// so stale artifacts from older matcher logic are invalidated. Same pattern
/// as MATCHER_VERSION for the pipeline build cache.
pub const SUGGEST_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSuggestion {
    pub txn_id_a: String,
    pub txn_id_b: String,
    pub summary: String,
}

/// Suggest trade link candidates from parsed transactions.
/// Looks for pairs on the same date/within 60 seconds that have different
/// commodities, opposite directions, and the same primary account.
/// Prefers pairs that also share the same payee (e.g. "Uniswap V3: …").
///
/// When `price_graph` and `base_currency` are provided, rejects pairs whose
/// base-currency values differ by more than 80% (min/max < 0.2).
pub fn suggest_trade_links(
    transactions: &[crate::ledger_parser::Transaction],
    existing_links: &[TradeLink],
    price_graph: Option<&crate::ledger_parser::PriceGraph>,
    base_currency: Option<&str>,
) -> Vec<TradeSuggestion> {
    use std::collections::HashSet;

    // Build set of already-linked txn IDs
    let linked: HashSet<&str> = existing_links
        .iter()
        .flat_map(|l| [l.txn_id_a.as_str(), l.txn_id_b.as_str()])
        .collect();

    struct TxnInfo {
        txn_id: String,
        datetime: String,
        account: String,
        commodity: String,
        amount: f64,
        payee: String,
        src_ref: Option<String>,
    }

    let mut infos: Vec<TxnInfo> = Vec::new();
    for txn in transactions {
        let txn_id = extract_txn_id(&txn.meta).unwrap_or_default();
        if txn_id.is_empty() || linked.contains(txn_id.as_str()) {
            continue;
        }
        if let Some(posting) = txn.postings.first() {
            infos.push(TxnInfo {
                txn_id,
                datetime: txn.datetime.clone(),
                account: posting.account.clone(),
                commodity: posting.commodity.clone(),
                amount: posting.amount,
                payee: txn.payee.clone().unwrap_or_default(),
                src_ref: extract_src_ref(&txn.meta),
            });
        }
    }

    // Sort by datetime
    infos.sort_by(|a, b| a.datetime.cmp(&b.datetime));

    let mut suggestions = Vec::new();
    let mut used: HashSet<usize> = HashSet::new();

    fn is_swap_candidate(a: &TxnInfo, b: &TxnInfo) -> bool {
        let basic = a.amount.abs() > 1e-9
            && b.amount.abs() > 1e-9
            && a.commodity != b.commodity
            && a.account == b.account
            && ((a.amount < 0.0 && b.amount > 0.0) || (a.amount > 0.0 && b.amount < 0.0));
        let shared_src =
            a.src_ref.is_some() && a.src_ref == b.src_ref && a.commodity != b.commodity;
        basic || shared_src
    }

    for i in 0..infos.len() {
        if used.contains(&i) {
            continue;
        }

        // Collect all same-day/60s candidates
        let mut best: Option<usize> = None;
        let mut best_has_payee_match = false;

        for j in (i + 1)..infos.len() {
            if used.contains(&j) {
                continue;
            }
            if !within_seconds(&infos[i].datetime, &infos[j].datetime, 60) {
                break;
            }
            if is_swap_candidate(&infos[i], &infos[j]) {
                let payee_match = !infos[i].payee.is_empty() && infos[i].payee == infos[j].payee;
                // Prefer payee match; otherwise take first available
                if payee_match && !best_has_payee_match {
                    best = Some(j);
                    best_has_payee_match = true;
                } else if best.is_none() {
                    best = Some(j);
                }
            }
        }

        if let Some(j) = best {
            // Value-ratio filter: reject pairs with wildly mismatched base values
            if let (Some(pg), Some(base)) = (price_graph, base_currency) {
                let val_a = pg.convert_to_base(
                    &infos[i].commodity,
                    infos[i].amount.abs(),
                    &infos[i].datetime,
                    base,
                );
                let val_b = pg.convert_to_base(
                    &infos[j].commodity,
                    infos[j].amount.abs(),
                    &infos[j].datetime,
                    base,
                );
                if let (Some(va), Some(vb)) = (val_a, val_b) {
                    let min = va.min(vb);
                    let max = va.max(vb);
                    if max > 0.0 && min / max < 0.2 {
                        continue; // values differ by more than 80%
                    }
                }
            }

            let (sell, buy) = if infos[i].amount < 0.0 {
                (&infos[i], &infos[j])
            } else {
                (&infos[j], &infos[i])
            };
            suggestions.push(TradeSuggestion {
                txn_id_a: sell.txn_id.clone(),
                txn_id_b: buy.txn_id.clone(),
                summary: format!(
                    "{} {} {} → {} {}",
                    sell.datetime.split('T').next().unwrap_or(&sell.datetime),
                    format!("{:.6}", sell.amount.abs())
                        .trim_end_matches('0')
                        .trim_end_matches('.'),
                    sell.commodity,
                    format!("{:.6}", buy.amount.abs())
                        .trim_end_matches('0')
                        .trim_end_matches('.'),
                    buy.commodity,
                ),
            });
            used.insert(i);
            used.insert(j);
        }
    }

    suggestions
}

/// Check if two datetime strings are within N seconds of each other.
/// Supports "YYYY-MM-DD HH:MM:SS", "YYYY-MM-DDTHH:MM:SS", and date-only "YYYY-MM-DD".
/// Date-only values are treated as same-day (diff = 0).
fn within_seconds(a: &str, b: &str, max_secs: i64) -> bool {
    fn parse_secs(s: &str) -> Option<i64> {
        // Normalize T separator to space
        let s = s.replace('T', " ");
        let parts: Vec<&str> = s.split(' ').collect();
        let date_parts: Vec<i64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
        if date_parts.len() < 3 {
            return None;
        }
        let base = date_parts[0] * 365 * 86400 + date_parts[1] * 30 * 86400 + date_parts[2] * 86400;
        if parts.len() >= 2 {
            let time_parts: Vec<i64> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();
            if time_parts.len() >= 3 {
                return Some(base + time_parts[0] * 3600 + time_parts[1] * 60 + time_parts[2]);
            }
        }
        Some(base)
    }
    match (parse_secs(a), parse_secs(b)) {
        (Some(a_s), Some(b_s)) => (a_s - b_s).abs() <= max_secs,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_extract_txn_id() {
        assert_eq!(
            extract_txn_id(&Some("txn:abc123".to_string())),
            Some("txn:abc123".to_string())
        );
        assert_eq!(
            extract_txn_id(&Some("origin:device-1, txn:def456".to_string())),
            Some("txn:def456".to_string())
        );
        assert_eq!(extract_txn_id(&None), None);
        assert_eq!(extract_txn_id(&Some("no-txn".to_string())), None);
    }

    fn ids(links: &[TradeLink]) -> Vec<&str> {
        links.iter().map(|l| l.id.as_str()).collect()
    }

    #[test]
    fn a_vault_without_links_has_none_and_reading_writes_nothing() {
        let sources = tempfile::tempdir().unwrap();
        assert!(load(sources.path()).unwrap().is_empty());
        assert!(read(sources.path()).unwrap().is_empty());
        assert!(!sources.path().join(TRADE_LINKS_FILE).exists());
    }

    #[test]
    fn saved_links_load_back_sorted_by_id_whatever_the_pair_order() {
        let sources = tempfile::tempdir().unwrap();
        save(sources.path(), &[("txn:d", "txn:c"), ("txn:b", "txn:a")]).unwrap();
        let links = load(sources.path()).unwrap();
        let mut expected = vec![
            trade_link_id("txn:a", "txn:b"),
            trade_link_id("txn:c", "txn:d"),
        ];
        expected.sort();
        assert_eq!(ids(&links), expected);
        assert_eq!(
            trade_link_id("txn:b", "txn:a"),
            trade_link_id("txn:a", "txn:b")
        );
    }

    #[test]
    fn deleting_a_trade_link_removes_only_that_link() {
        let sources = tempfile::tempdir().unwrap();
        save(sources.path(), &[("txn:a", "txn:b"), ("txn:c", "txn:d")]).unwrap();
        delete(sources.path(), &trade_link_id("txn:c", "txn:d")).unwrap();
        delete(sources.path(), "no-such-link").unwrap();
        let keep = trade_link_id("txn:a", "txn:b");
        assert_eq!(ids(&load(sources.path()).unwrap()), vec![keep.as_str()]);
    }

    #[test]
    fn a_file_that_cannot_be_parsed_is_an_error_and_is_left_alone() {
        let sources = tempfile::tempdir().unwrap();
        let path = sources.path().join(TRADE_LINKS_FILE);
        fs::write(&path, "{ not json").unwrap();
        let err = save(sources.path(), &[("txn:a", "txn:b")]).unwrap_err();
        assert!(err.contains(TRADE_LINKS_FILE), "{err}");
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ not json");
    }

    #[test]
    fn test_suggest_trade_links_rejects_mismatched_values() {
        use crate::ledger_parser::{Posting, PriceDirective, PriceGraph, Transaction};

        fn test_posting(account: &str, amount: f64, commodity: &str) -> Posting {
            Posting {
                account: account.to_string(),
                amount,
                amount_text: format!("{}", amount),
                commodity: commodity.to_string(),
                remainder: None,
                cost: None,
                price: None,
            }
        }

        // Two transactions: sell 200 USDC, buy 0.002 ETH
        // At ETH=3000 USD, USDC=1 USD → $200 vs $6 → ratio 0.03 → rejected
        let txns = vec![
            Transaction {
                date: "2025-01-15".to_string(),
                datetime: "2025-01-15 10:00:00".to_string(),
                status: Some('*'),
                payee: Some("Swap".to_string()),
                narration: None,
                meta: Some("txn:sell-usdc".to_string()),
                postings: vec![test_posting("assets:exchange", -200.0, "USDC")],
                display_payee: None,
                amount: -200.0,
                amount_commodity: "USDC".to_string(),
                display_amount_commodity: None,
                fee: None,
                fee_commodity: None,
            },
            Transaction {
                date: "2025-01-15".to_string(),
                datetime: "2025-01-15 10:00:30".to_string(),
                status: Some('*'),
                payee: Some("Swap".to_string()),
                narration: None,
                meta: Some("txn:buy-eth".to_string()),
                postings: vec![test_posting("assets:exchange", 0.002, "ETH")],
                display_payee: None,
                display_amount_commodity: None,
                amount: 0.002,
                amount_commodity: "ETH".to_string(),
                fee: None,
                fee_commodity: None,
            },
        ];

        let pg = PriceGraph::from_entries(vec![
            PriceDirective {
                datetime: "2025-01-01".to_string(),
                commodity: "ETH".to_string(),
                price_amount: 3000.0,
                price_amount_text: "3000".to_string(),
                quote_commodity: "USD".to_string(),
            },
            PriceDirective {
                datetime: "2025-01-01".to_string(),
                commodity: "USDC".to_string(),
                price_amount: 1.0,
                price_amount_text: "1".to_string(),
                quote_commodity: "USD".to_string(),
            },
        ]);

        // With price data → should reject the mismatched pair
        let suggestions = suggest_trade_links(&txns, &[], Some(&pg), Some("USD"));
        assert!(
            suggestions.is_empty(),
            "expected no suggestions for mismatched values, got: {:?}",
            suggestions,
        );

        // Without price data → should still suggest the pair
        let suggestions_no_price = suggest_trade_links(&txns, &[], None, None);
        assert_eq!(
            suggestions_no_price.len(),
            1,
            "expected 1 suggestion without price data, got: {:?}",
            suggestions_no_price,
        );
    }

    #[test]
    fn test_suggest_trade_links_bybit_multi_pair() {
        use crate::ledger_parser::{Posting, Transaction};

        fn test_posting(account: &str, amount: f64, commodity: &str) -> Posting {
            Posting {
                account: account.to_string(),
                amount,
                amount_text: format!("{}", amount),
                commodity: commodity.to_string(),
                remainder: None,
                cost: None,
                price: None,
            }
        }

        // Reproduces: 5 HNT buys + 5 USDT sells at the same second on Bybit
        // All should pair up as trade suggestions
        let account = "assets:crypto:exchange:bybit:personal";
        let make_txn = |id: &str, amount: f64, commodity: &str, datetime: &str| -> Transaction {
            Transaction {
                date: datetime.split(' ').next().unwrap().to_string(),
                datetime: datetime.to_string(),
                status: Some('*'),
                payee: Some("Bybit".to_string()),
                narration: Some("BUY HNTUSDT".to_string()),
                meta: Some(format!("txn:{id}")),
                postings: vec![test_posting(account, amount, commodity)],
                display_payee: None,
                amount,
                amount_commodity: commodity.to_string(),
                display_amount_commodity: None,
                fee: None,
                fee_commodity: None,
            }
        };

        let txns = vec![
            // HNT buys (positive)
            make_txn("csv-hnt1", 26.75, "HNT", "2025-11-13 18:44:34"),
            make_txn("csv-hnt2", 9.96, "HNT", "2025-11-13 18:44:35"),
            make_txn("csv-hnt3", 3.19, "HNT", "2025-11-13 18:44:35"),
            make_txn("csv-hnt4", 7.20, "HNT", "2025-11-13 18:44:35"),
            make_txn("csv-hnt5", 6.39, "HNT", "2025-11-13 18:44:35"),
            // USDT sells (negative)
            make_txn("csv-usdt1", -59.76, "USDT", "2025-11-13 18:44:34"),
            make_txn("csv-usdt2", -22.15, "USDT", "2025-11-13 18:44:35"),
            make_txn("csv-usdt3", -14.22, "USDT", "2025-11-13 18:44:35"),
            make_txn("csv-usdt4", -7.09, "USDT", "2025-11-13 18:44:35"),
            make_txn("csv-usdt5", -16.03, "USDT", "2025-11-13 18:44:35"),
        ];

        let suggestions = suggest_trade_links(&txns, &[], None, None);
        assert_eq!(
            suggestions.len(),
            5,
            "expected 5 trade suggestions for 5 HNT/USDT pairs, got {}: {:?}",
            suggestions.len(),
            suggestions,
        );

        // Now test with a price graph where HNT has a known price
        use crate::ledger_parser::{PriceDirective, PriceGraph};
        let pg = PriceGraph::from_entries(vec![
            PriceDirective {
                datetime: "2025-11-01".to_string(),
                commodity: "HNT".to_string(),
                price_amount: 5.0,
                price_amount_text: "5".to_string(),
                quote_commodity: "AUD".to_string(),
            },
            PriceDirective {
                datetime: "2025-11-01".to_string(),
                commodity: "USDT".to_string(),
                price_amount: 1.5,
                price_amount_text: "1.5".to_string(),
                quote_commodity: "AUD".to_string(),
            },
        ]);
        let suggestions_priced = suggest_trade_links(&txns, &[], Some(&pg), Some("AUD"));
        assert_eq!(
            suggestions_priced.len(),
            5,
            "expected 5 suggestions even WITH price data (values should roughly match), got {}: {:?}",
            suggestions_priced.len(),
            suggestions_priced,
        );
    }
}
