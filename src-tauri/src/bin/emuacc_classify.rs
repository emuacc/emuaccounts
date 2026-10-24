#![deny(warnings)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use emuaccounts::ai_client::{AiClient, ChatMessage};
use emuaccounts::ai_suggest::{self, AiRuleSuggestion};
use emuaccounts::generated_store::load_active_ledger;
use emuaccounts::ledger_parser::Transaction;
use emuaccounts::processing_pipeline::PipelineMetadata;
use emuaccounts::root_config::{self, platform_app_data_dir};
use emuaccounts::rules::{ai_rule_id, RulesFile};

fn resolve_generated_dir() -> PathBuf {
    let env_override = std::env::var("EMUACC_GENERATED_DIR").ok();
    let app_dir = platform_app_data_dir();
    let config = root_config::load_root_config(&app_dir);
    root_config::resolve_generated(env_override.as_deref(), &config, &app_dir)
}

fn resolve_sources_dir() -> PathBuf {
    let env_override = std::env::var("EMUACC_SOURCES_DIR").ok();
    let app_dir = platform_app_data_dir();
    let config = root_config::load_root_config(&app_dir);
    root_config::resolve_sources(env_override.as_deref(), &config, &app_dir)
}

const DEFAULT_CLASSIFY_PROMPT: &str = include_str!("../../prompts/classify.md");

/// Resolve the classify prompt template.
/// Order: EMUACC_CLASSIFY_PROMPT env var > config.classify_prompt_path > embedded default.
fn load_classify_prompt(config_path: Option<&str>) -> String {
    let from_env = std::env::var("EMUACC_CLASSIFY_PROMPT").ok();
    let path = from_env.as_deref().or(config_path);
    if let Some(p) = path {
        match std::fs::read_to_string(p) {
            Ok(s) => {
                eprintln!("[classify] Using prompt from {p}");
                return s;
            }
            Err(e) => {
                eprintln!("[classify] WARN: failed to read prompt {p}: {e} — using embedded default");
            }
        }
    }
    DEFAULT_CLASSIFY_PROMPT.to_string()
}

fn render_prompt(
    template: &str,
    acct: &str,
    txn_lines: &str,
    rules_json: &str,
    categorised_section: &str,
    user_context: &str,
) -> String {
    template
        .replace("{acct}", acct)
        .replace("{txn_lines}", txn_lines)
        .replace("{rules_json}", rules_json)
        .replace("{categorised_section}", categorised_section)
        .replace("{user_context}", user_context)
}

fn print_usage() {
    eprintln!("Usage: emuacc-classify [OPTIONS]");
    eprintln!();
    eprintln!("Batch-classify expenses:unknown transactions using the configured AI endpoint.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --account-set NAME    Account set to classify (default: first available)");
    eprintln!("  --dry-run             Show what would be classified without calling the AI");
    eprintln!("  --batch-size N        Transactions per AI call (default: 30)");
    eprintln!("  --account FILTER      Only classify for this asset account (prefix match)");
    eprintln!("  --txid ID             Classify a single transaction by meta substring (prints, does not save)");
    eprintln!("  --print-prompt        Render and print the prompt(s) sent to the AI; do not call it");
    eprintln!("  --help                Show this help");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return;
    }

    let mut account_set = String::new();
    let mut dry_run = false;
    let mut batch_size: usize = 30;
    let mut account_filter: Option<String> = None;
    let mut txid_filter: Option<String> = None;
    let mut print_prompt = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--account-set" => {
                i += 1;
                account_set = args.get(i).cloned().unwrap_or_default();
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--batch-size" => {
                i += 1;
                batch_size = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(30);
            }
            "--account" => {
                i += 1;
                account_filter = args.get(i).cloned();
            }
            "--txid" => {
                i += 1;
                txid_filter = args.get(i).cloned();
            }
            "--print-prompt" => {
                print_prompt = true;
            }
            other => {
                eprintln!("Unknown option: {other}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let generated_dir = resolve_generated_dir();
    let sources_dir = resolve_sources_dir();
    let app_dir = platform_app_data_dir();
    let root_cfg = root_config::load_root_config(&app_dir);
    let prompt_template = load_classify_prompt(root_cfg.classify_prompt_path.as_deref());
    let user_context = ai_suggest::user_context_section(&root_cfg.ai_user_name);

    // Resolve account set
    let set_dir = if account_set.is_empty() {
        // Find first available account set
        if let Ok(entries) = std::fs::read_dir(&generated_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name != "prices" && !name.starts_with('.') {
                        account_set = name;
                        break;
                    }
                }
            }
        }
        generated_dir.join(&account_set)
    } else {
        generated_dir.join(&account_set)
    };

    eprintln!("[classify] Account set: {account_set}");
    eprintln!("[classify] Sources: {}", sources_dir.display());
    eprintln!("[classify] Generated: {}", set_dir.display());

    // Load ledger
    let parse = match load_active_ledger(&set_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load ledger: {e}");
            std::process::exit(1);
        }
    };

    // Load pipeline metadata for this set's account→folder mapping
    let account_folders: HashMap<String, String> = PipelineMetadata::load(&generated_dir)
        .and_then(|mut m| m.account_folders.remove(&account_set))
        .unwrap_or_default();

    // Find all expenses:unknown transactions, grouped by source asset account
    let mut by_source: HashMap<String, Vec<&Transaction>> = HashMap::new();
    for txn in &parse.transactions {
        if let Some(ref id) = txid_filter {
            if !txn.meta.as_deref().is_some_and(|m| m.contains(id.as_str())) {
                continue;
            }
        } else {
            let has_unknown = txn.postings.iter().any(|p| p.account == "expenses:unknown");
            if !has_unknown {
                continue;
            }
        }

        // Find the asset-side account
        let asset_acct = txn
            .postings
            .iter()
            .find(|p| {
                p.account.starts_with("assets:") || p.account.starts_with("liabilities:")
            })
            .or_else(|| {
                txn.postings
                    .iter()
                    .find(|p| p.account != "expenses:unknown")
            })
            .map(|p| p.account.clone())
            .unwrap_or_default();

        // Apply account filter
        if let Some(ref filter) = account_filter {
            if !asset_acct.starts_with(filter.as_str()) {
                continue;
            }
        }

        by_source.entry(asset_acct).or_default().push(txn);
    }

    if by_source.is_empty() {
        eprintln!("[classify] No expenses:unknown transactions found.");
        return;
    }

    // Summary
    let total: usize = by_source.values().map(|v| v.len()).sum();
    eprintln!(
        "\n[classify] Found {total} uncategorised transactions across {} accounts:\n",
        by_source.len()
    );
    for (acct, txns) in by_source.iter() {
        let folder = account_folders.get(acct).map(|s| s.as_str()).unwrap_or("?");
        eprintln!("  {:>5}  {acct:<55} folder: {folder}", txns.len());
    }

    if dry_run {
        eprintln!(
            "\n[dry-run] Would classify these transactions. Run without --dry-run to proceed."
        );
        // In dry-run mode, show sample transactions per account
        for (acct, txns) in by_source.iter() {
            eprintln!("\n--- {acct} ({} txns) ---", txns.len());
            // Deduplicate by narration to show unique patterns
            let mut seen = std::collections::HashSet::new();
            let mut unique = 0;
            for t in txns.iter() {
                let key = t.narration.as_deref().unwrap_or("").to_string();
                if seen.insert(key) {
                    unique += 1;
                    if unique <= 10 {
                        eprintln!("{}", ai_suggest::fmt_txn(t));
                    }
                }
            }
            if unique > 10 {
                eprintln!("  ... and {} more unique patterns", unique - 10);
            }
        }
        return;
    }

    // Fail fast when the AI endpoint is unconfigured (unless we're only
    // printing prompts, which needs no client).
    let client = if print_prompt {
        None
    } else {
        match AiClient::from_disk() {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("[classify] ERROR: {e}");
                eprintln!("[classify] Configure the endpoint in the app's Global Settings (stored in config.json).");
                std::process::exit(1);
            }
        }
    };
    // One shared copy of the ledger for the AI inspection tools.
    let txns_for_tools = Arc::new(parse.transactions.clone());
    let mut total_rules_added = 0;

    // Process each source account
    for (acct, txns) in &by_source {
        let folder_rel = match account_folders.get(acct) {
            Some(f) => f,
            None => {
                eprintln!("[classify] No folder mapping for {acct}, skipping");
                continue;
            }
        };
        let folder_path = sources_dir.join(folder_rel);

        eprintln!("\n{}", "=".repeat(60));
        eprintln!("[classify] Processing {acct} ({} txns)", txns.len());
        eprintln!("[classify] Folder: {folder_rel}");

        // Load existing rules for context. The prompt gets a bounded one-line
        // summary; the full list is reachable through the search_rules tool.
        let existing_rules = RulesFile::load(&folder_path);
        let rules_summary =
            ai_suggest::summarise_rules(&existing_rules.rules, ai_suggest::MAX_RULES_IN_PROMPT);
        let rules_for_tools = Arc::new(existing_rules.rules.clone());

        // Deduplicate transactions by narration pattern to avoid redundancy
        let mut seen_narrations = std::collections::HashSet::new();
        let mut unique_txns: Vec<&Transaction> = Vec::new();
        for t in txns {
            let key = format!(
                "{}|{}",
                t.narration.as_deref().unwrap_or(""),
                t.amount_commodity
            );
            if seen_narrations.insert(key) {
                unique_txns.push(t);
            }
        }
        eprintln!(
            "[classify] {} unique narration patterns (from {} total)",
            unique_txns.len(),
            txns.len()
        );

        // Build context: categorised transactions from this account (for examples)
        let categorised: Vec<String> = parse
            .transactions
            .iter()
            .filter(|t| {
                t.postings.iter().any(|p| p.account == *acct)
                    && !t.postings.iter().any(|p| p.account == "expenses:unknown")
            })
            .take(30)
            .map(ai_suggest::fmt_txn)
            .collect();

        let categorised_section = if categorised.is_empty() {
            String::new()
        } else {
            format!(
                "\n\nExamples of already-categorised transactions on this account:\n{}",
                categorised.join("\n")
            )
        };

        // Process in batches
        for (batch_idx, chunk) in unique_txns.chunks(batch_size).enumerate() {
            eprintln!(
                "[classify] Batch {}/{} ({} transactions)",
                batch_idx + 1,
                unique_txns.len().div_ceil(batch_size),
                chunk.len()
            );

            let txn_lines: Vec<String> = chunk
                .iter()
                .map(|t| {
                    let payee = t
                        .display_payee
                        .as_deref()
                        .or(t.payee.as_deref())
                        .unwrap_or("—");
                    let narration = t.narration.as_deref().unwrap_or("—");
                    format!(
                        "  {} | {} | {} | {} {}",
                        t.date, payee, narration, t.amount, t.amount_commodity
                    )
                })
                .collect();

            let prompt = render_prompt(
                &prompt_template,
                acct,
                &txn_lines.join("\n"),
                &rules_summary,
                &categorised_section,
                &user_context,
            );

            if print_prompt {
                println!(
                    "===== prompt: {acct} batch {}/{} =====",
                    batch_idx + 1,
                    unique_txns.len().div_ceil(batch_size)
                );
                println!("{prompt}");
                continue;
            }

            let client = client.as_ref().expect("client exists unless print_prompt");
            let tools =
                ai_suggest::categorisation_tools(rules_for_tools.clone(), txns_for_tools.clone());
            let raw = match client.run(
                vec![ChatMessage::user(prompt)],
                &tools,
                &|s: &str| eprintln!("[classify] {s}"),
            ) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[classify] ERROR: AI call failed: {e}");
                    continue;
                }
            };

            let json_str = ai_suggest::extract_json_array(&raw).unwrap_or(&raw);

            let suggestions: Vec<AiRuleSuggestion> = match serde_json::from_str(json_str) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[classify] ERROR: Could not parse suggestions: {e}");
                    eprintln!("[classify] Raw output: {}", &raw[..raw.len().min(500)]);
                    continue;
                }
            };

            eprintln!("[classify] Got {} suggestions", suggestions.len());
            for s in &suggestions {
                eprintln!("  {} → {} ({})", s.pattern, s.amount_account, s.explanation);
            }

            if txid_filter.is_some() {
                // Print-only mode: emit suggestions as JSON and skip rules save.
                match serde_json::to_string_pretty(&suggestions.iter().map(|s| {
                    serde_json::json!({
                        "pattern": s.pattern,
                        "amount_account": s.amount_account,
                        "payee": s.payee,
                        "match_field": s.match_field,
                        "explanation": s.explanation,
                    })
                }).collect::<Vec<_>>()) {
                    Ok(j) => println!("{j}"),
                    Err(e) => eprintln!("[classify] ERROR serialising: {e}"),
                }
                continue;
            }

            // Append to rules file
            let mut rules = match RulesFile::try_load(&folder_path) {
                Ok(rules) => rules,
                Err(e) => {
                    eprintln!("[classify] ERROR loading rules, folder skipped: {e}");
                    continue;
                }
            };
            let mut added = 0;
            for s in &suggestions {
                // Skip if a rule with the same pattern already exists (covers
                // the manual-rule-shadowing case; an identical AI rule is
                // caught downstream by the content-derived id check).
                if rules.rules.iter().any(|r| r.pattern == s.pattern) {
                    eprintln!("  [skip] pattern already exists: {}", s.pattern);
                    continue;
                }
                let mut rule = emuaccounts::rules::Rule {
                    id: String::new(),
                    pattern: s.pattern.clone(),
                    match_field: s.match_field.clone(),
                    payee: s.payee.clone(),
                    commodity: None,
                    comment: Some(format!("ai: {}", s.explanation)),
                    amount_condition: None,
                    fee_condition: None,
                    amount_account: Some(s.amount_account.clone()),
                    fee_account: None,
                    postings: vec![],
                    payee_condition: None,
                    narration_condition: None,
                    commodity_condition: None,
                    meta_condition: None,
                };
                rule.id = ai_rule_id(&rule);
                // Skip if an identical rule (same content-derived id) already exists.
                if rules.rules.iter().any(|r| r.id == rule.id) {
                    eprintln!("  [skip] identical rule already exists: {}", rule.pattern);
                    continue;
                }
                rules.insert_rule(rule);
                added += 1;
            }

            if added > 0 {
                if let Err(e) = rules.save(&folder_path) {
                    eprintln!("[classify] ERROR saving rules: {e}");
                } else {
                    eprintln!("[classify] Saved {added} new rules to {folder_rel}/_rules.json");
                    total_rules_added += added;
                }
            }
        }
    }

    eprintln!("\n[classify] Done. Added {total_rules_added} rules total.");
    if total_rules_added > 0 {
        eprintln!("[classify] Run emuacc-regenerate to apply the new rules.");
    }
}
