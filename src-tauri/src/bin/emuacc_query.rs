#![deny(warnings)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use emuacc_covid::generated_store::{load_openings, scoped_query, ScopedQueryOptions};
use emuacc_covid::ledger_parser::{CommodityAmount, PriceGraph};
use emuacc_covid::processing_pipeline::folder_to_account_name;
use emuacc_covid::query::{self, MinValueFilter, SortField, SortOrder};
use emuacc_covid::root_config;

fn platform_app_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join("Library/Application Support/com.cog32.emuacccovid");
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(data) = std::env::var("XDG_DATA_HOME").ok().or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{h}/.local/share"))
        }) {
            return PathBuf::from(data).join("com.cog32.emuacccovid");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("com.cog32.emuacccovid");
        }
    }
    PathBuf::from(".")
}

fn resolve_generated_dir() -> PathBuf {
    let env_override = std::env::var("EMUACC_GENERATED_DIR").ok();
    let app_dir = platform_app_data_dir();
    let config = root_config::load_root_config(&app_dir);
    root_config::resolve_generated(env_override.as_deref(), &config, &app_dir)
}

/// Resolve the directory containing `_prices/` for `--min-value-usd`.
/// Precedence: explicit flag > `EMUACC_SOURCES_DIR` env > root_config.
fn resolve_sources_dir(prices_dir_flag: Option<&str>) -> PathBuf {
    if let Some(p) = prices_dir_flag {
        return PathBuf::from(p);
    }
    let env_override = std::env::var("EMUACC_SOURCES_DIR").ok();
    let app_dir = platform_app_data_dir();
    let config = root_config::load_root_config(&app_dir);
    root_config::resolve_sources(env_override.as_deref(), &config, &app_dir)
}

fn build_min_value_filter(
    threshold: f64,
    prices_dir_flag: Option<&str>,
) -> Result<MinValueFilter, String> {
    let sources = resolve_sources_dir(prices_dir_flag);
    if !sources.join("_prices").is_dir() {
        return Err(format!(
            "--min-value-usd requires a `_prices/` directory; \
             checked {} — set EMUACC_SOURCES_DIR or pass --prices-dir PATH",
            sources.display()
        ));
    }
    Ok(MinValueFilter {
        threshold,
        currency: "USD".into(),
        price_graph: Arc::new(PriceGraph::load(&sources)),
    })
}

/// A positional arg is treated as DIR when it names an existing directory on disk.
/// Search terms (`key:value`, `AND`, `OR`, free text) never collide with real paths.
fn looks_like_dir(arg: &str) -> bool {
    Path::new(arg).is_dir()
}

/// Where a query runs: the account set the app would select, plus the folder
/// scope DIR narrows it to.
struct QueryScope {
    set_dir: PathBuf,
    /// Account name derived from DIR's position under the set (e.g.
    /// `assets:cash:cba:bta`); `None` at a set root (the search's own
    /// `account:` term prunes, as in the app); `Some("")` at the generated root,
    /// where folder names are set-qualified and cannot be pruned by account —
    /// every set is unioned and the search filters rows.
    folder_scope: Option<String>,
}

/// The generated root is the ancestor of `dir` that holds the pipeline's
/// `pipeline-metadata.json`; falling back to the configured generated dir when
/// `dir` lies under it (fixtures without metadata). Sets are its direct
/// children, so `generated/Acme/cash/cba/bta` resolves to set
/// `generated/Acme` scoped to `assets:cash:cba:bta` — the same
/// (set, account) pair the app queries — so the set-root
/// `accounts.transactions` openings and `config.json` are in view even when
/// DIR points deep inside the set.
fn resolve_scope(dir: &Path) -> QueryScope {
    let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let generated_root = dir
        .ancestors()
        .find(|a| a.join("pipeline-metadata.json").is_file())
        .map(Path::to_path_buf)
        .or_else(|| {
            let configured = resolve_generated_dir();
            let configured = configured.canonicalize().unwrap_or(configured);
            dir.starts_with(&configured).then_some(configured)
        });
    let Some(generated_root) = generated_root else {
        // Not inside any generated tree: treat DIR itself as a set root.
        return QueryScope {
            set_dir: dir,
            folder_scope: None,
        };
    };
    let rel: Vec<String> = dir
        .strip_prefix(&generated_root)
        .map(|r| {
            r.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    let Some(set_name) = rel.first() else {
        return QueryScope {
            set_dir: generated_root,
            folder_scope: Some(String::new()),
        };
    };
    let folder_scope =
        (rel.len() > 1).then(|| folder_to_account_name(&format!("__set__/{}", rel[1..].join("/"))));
    QueryScope {
        set_dir: generated_root.join(set_name),
        folder_scope,
    }
}

fn print_usage() {
    eprintln!("Usage: emuacc-query [DIR] [SEARCH...] [OPTIONS]");
    eprintln!();
    eprintln!("Query transactions under DIR (defaults to the vault's generated/ folder).");
    eprintln!("All `ledger.transactions` files beneath DIR are unioned — DIR may point at");
    eprintln!("the vault root, a user subtree, or a single account folder. DIR resolves to");
    eprintln!("the account set it belongs to (generated/<set>), so the set's declared");
    eprintln!("opening balances and hidden-account config apply exactly as in the app.");
    eprintln!();
    eprintln!("Search terms:");
    eprintln!("  account:PREFIX        Filter by account (regex match on postings)");
    eprintln!("  payee:PATTERN         Filter by payee (regex)");
    eprintln!("  narration:PATTERN     Filter by narration (regex)");
    eprintln!("  date:PATTERN          Filter by date: a prefix (2025-01), a bound (<=2025-03,");
    eprintln!("                        >2025) or a range (2025-01..2025-03); a month or year");
    eprintln!("                        bound covers its whole period");
    eprintln!("  amount:CONDITION      Filter by amount (e.g. >100, <50, >=0)");
    eprintln!("  commodity:PATTERN     Filter by commodity (exact, or *glob*; venue-prefixed");
    eprintln!("                        names work as-is: commodity:xyz:WTIOIL)");
    eprintln!("  meta:PATTERN          Filter by meta field (regex)");
    eprintln!("  fee:CONDITION         Filter by fee amount");
    eprintln!("  FREE_TEXT             Search across all fields (an unknown prefix such as");
    eprintln!("                        xyz:WTIOIL is free text, not a field)");
    eprintln!();
    eprintln!("  Combine with AND / OR. Negate with - prefix (e.g. -commodity:SPAM).");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --balances            Output per-commodity balances (postings + declared");
    eprintln!("                        openings) instead of transactions");
    eprintln!("  --show-hidden         Include transactions on hidden accounts (config.json");
    eprintln!("                        hidden_accounts); hidden by default, as in the app");
    eprintln!("  --sort FIELD ORDER    Sort by: date, amount/value, payee, account");
    eprintln!("                        Order: asc, desc (default: asc)");
    eprintln!("  --limit N             Limit number of transactions listed (--balances");
    eprintln!("                        always covers every match)");
    eprintln!("  --format FORMAT       Output format: json, summary, register (default: register)");
    eprintln!("  --min-value-usd N     Drop balances worth less than N USD at latest price.");
    eprintln!("                        Unpriced commodities (typical of spam) are dropped.");
    eprintln!("  --prices-dir PATH     Override sources dir (parent of _prices/); usually");
    eprintln!("                        resolved via EMUACC_SOURCES_DIR or the vault root.");
    eprintln!("  --help                Show this help");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  emuacc-query account:expenses:unknown --sort value desc");
    eprintln!("  emuacc-query generated/alex commodity:HNT --format json");
    eprintln!("  emuacc-query account:crypto AND amount:>1000 --sort date desc --limit 20");
    eprintln!("  emuacc-query -commodity:SPAM --sort amount desc");
    eprintln!("  emuacc-query --balances account:assets:crypto date:<=2026-06-30 --format json");
    eprintln!("  emuacc-query account:assets:crypto:wallet:ethereum --min-value-usd 1");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Validated here for a clear CLI error; passed as text to the shared engine.
    let mut sort_field: Option<String> = None;
    let mut sort_order: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut format = "register".to_string();
    let mut base_dir: Option<PathBuf> = None;
    let mut search_parts: Vec<String> = Vec::new();
    let mut balances_mode = false;
    let mut show_hidden = false;
    let mut min_value_usd: Option<f64> = None;
    let mut prices_dir: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--balances" => {
                balances_mode = true;
            }
            "--show-hidden" => {
                show_hidden = true;
            }
            "--sort" => {
                i += 1;
                let field_str = args.get(i).unwrap_or_else(|| {
                    eprintln!("--sort requires a field (date, amount, value, payee, account)");
                    std::process::exit(2);
                });
                if SortField::parse_str(field_str).is_none() {
                    eprintln!("Unknown sort field: {field_str}. Valid: date, amount, value, payee, account");
                    std::process::exit(2);
                }
                sort_field = Some(field_str.clone());
                // Optional order argument (peek next arg)
                if let Some(next) = args.get(i + 1) {
                    if SortOrder::parse_str(next).is_some() {
                        sort_order = Some(next.clone());
                        i += 1;
                    }
                }
            }
            "--limit" => {
                i += 1;
                let n_str = args.get(i).unwrap_or_else(|| {
                    eprintln!("--limit requires a number");
                    std::process::exit(2);
                });
                limit = Some(n_str.parse().unwrap_or_else(|_| {
                    eprintln!("--limit must be a positive integer, got: {n_str}");
                    std::process::exit(2);
                }));
            }
            "--format" => {
                i += 1;
                format = args
                    .get(i)
                    .unwrap_or_else(|| {
                        eprintln!("--format requires a value");
                        std::process::exit(2);
                    })
                    .clone();
            }
            "--min-value-usd" => {
                i += 1;
                let n_str = args.get(i).unwrap_or_else(|| {
                    eprintln!("--min-value-usd requires a number (USD threshold)");
                    std::process::exit(2);
                });
                min_value_usd = Some(n_str.parse().unwrap_or_else(|_| {
                    eprintln!("--min-value-usd must be a number, got: {n_str}");
                    std::process::exit(2);
                }));
            }
            "--prices-dir" => {
                i += 1;
                prices_dir = Some(
                    args.get(i)
                        .unwrap_or_else(|| {
                            eprintln!("--prices-dir requires a path (parent of _prices/)");
                            std::process::exit(2);
                        })
                        .clone(),
                );
            }
            // First positional arg that names an existing directory is DIR; everything
            // else is part of the search expression.
            arg if base_dir.is_none() && looks_like_dir(arg) => {
                base_dir = Some(PathBuf::from(arg));
            }
            _ => {
                search_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    let base_dir = base_dir.unwrap_or_else(resolve_generated_dir);
    if !base_dir.is_dir() {
        eprintln!("Directory not found: {}", base_dir.display());
        std::process::exit(1);
    }
    let scope = resolve_scope(&base_dir);

    // Parse search expression
    let search_str = search_parts.join(" ");
    let search = match query::parse_search(&search_str) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("Search error: {e}");
            std::process::exit(2);
        }
    };

    // --min-value-usd applies to the default-mode aggregated_balance / per-account
    // balances (via the query engine) and to --balances (aggregate_posting_balances).
    let min_value: Option<MinValueFilter> = match min_value_usd {
        Some(threshold) => match build_min_value_filter(threshold, prices_dir.as_deref()) {
            Ok(f) => Some(f),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(2);
            }
        },
        None => None,
    };

    // The same engine the desktop/web/BDD `query_search` use: folder-pruned
    // load, hidden-account filtering, openings folded into balances.
    let opts = ScopedQueryOptions {
        search: &search_str,
        folder_scope: scope.folder_scope.as_deref(),
        show_hidden,
        sort_field: sort_field.as_deref(),
        sort_order: sort_order.as_deref(),
        offset: None,
        // --limit caps the listing; balances cover every match, so they are
        // summed from an unpaged result.
        limit: if balances_mode { None } else { limit },
        min_value: min_value.clone(),
    };
    let account_prefix = opts.account_prefix();
    let result = match scoped_query(&scope.set_dir, opts) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading ledger: {e}");
            std::process::exit(1);
        }
    };

    if balances_mode {
        let openings = load_openings(&scope.set_dir, &account_prefix, show_hidden);
        let balances =
            query::aggregate_posting_balances(&result, &search, &openings, min_value.as_ref());
        render_balances(&balances, result.transaction_count, &format, &search_str);
        return;
    }

    match format.as_str() {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("JSON serialization")
            );
        }
        "summary" => {
            println!(
                "Search: {}",
                if search_str.is_empty() {
                    "(all)"
                } else {
                    &search_str
                }
            );
            println!(
                "Transactions: {} (showing {})",
                result.transaction_count,
                result.transactions.len()
            );
            println!("Accounts: {}", result.accounts.len());
            for acct in &result.accounts {
                println!("  {acct}");
            }
            if !result.aggregated_balance.is_empty() {
                println!("Balances:");
                for b in &result.aggregated_balance {
                    println!("  {:.4} {}", b.amount, b.commodity);
                }
            }
        }
        "register" => {
            // Human-readable tabular output
            if result.transactions.is_empty() {
                println!("No transactions found.");
                if !search_str.is_empty() {
                    println!("Search: {search_str}");
                }
                std::process::exit(0);
            }

            // Header
            println!(
                "{:<12} {:>14} {:<8} DESCRIPTION",
                "DATE", "AMOUNT", "CURR"
            );
            println!("{}", "-".repeat(70));

            for txn in &result.transactions {
                let desc = txn
                    .display_payee
                    .as_deref()
                    .or(txn.payee.as_deref())
                    .or(txn.narration.as_deref())
                    .unwrap_or("");
                let commodity = txn
                    .display_amount_commodity
                    .as_deref()
                    .unwrap_or(&txn.amount_commodity);
                println!(
                    "{:<12} {:>14.4} {:<8} {}",
                    txn.date, txn.amount, commodity, desc,
                );
            }

            println!("{}", "-".repeat(70));
            println!(
                "{} transactions (of {})",
                result.transactions.len(),
                result.transaction_count
            );
        }
        other => {
            eprintln!("Unknown format: {other}. Use 'json', 'summary', or 'register'.");
            std::process::exit(2);
        }
    }
}

fn render_balances(
    balances: &[CommodityAmount],
    transaction_count: usize,
    format: &str,
    search_str: &str,
) {
    match format {
        "json" => {
            let entries: Vec<serde_json::Value> = balances
                .iter()
                .map(|b| serde_json::json!({ "commodity": b.commodity, "quantity": b.amount }))
                .collect();
            let payload = serde_json::json!({
                "transaction_count": transaction_count,
                "balances": entries,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("JSON serialization")
            );
        }
        "summary" => {
            println!("Balances ({transaction_count} transactions):");
            for b in balances {
                println!("  {:.4} {}", b.amount, b.commodity);
            }
        }
        "register" => {
            if balances.is_empty() {
                println!("No balances.");
                if !search_str.is_empty() {
                    println!("Search: {search_str}");
                }
                return;
            }
            println!("{:<12} {:>14}", "COMMODITY", "QUANTITY");
            println!("{}", "-".repeat(28));
            for b in balances {
                println!("{:<12} {:>14.4}", b.commodity, b.amount);
            }
            println!("{}", "-".repeat(28));
            let noun = if balances.len() == 1 {
                "commodity"
            } else {
                "commodities"
            };
            println!(
                "{} {} · {transaction_count} transactions",
                balances.len(),
                noun,
            );
        }
        other => {
            eprintln!("Unknown format: {other}. Use 'json', 'summary', or 'register'.");
            std::process::exit(2);
        }
    }
}
