#![deny(warnings)]
//! One-off migration: backfill missing _rules.json entries for the vault's
//! existing trade links. The bulk of the logic lives in
//! `emuaccounts::trade_link_repair` so it can be unit-tested.
//!
//! Usage:
//!   emuacc-repair-trade-links            # dry-run; reports what would change
//!   emuacc-repair-trade-links --write    # apply the changes
//!
//! The dry run writes nothing: in a vault whose links are still only in the
//! pre-TASK-133 metadata document it reads them from there. `--write` first
//! migrates them to the trade-link file, as the app does.

use std::path::PathBuf;

use emuaccounts::root_config::{self, platform_app_data_dir};
use emuaccounts::trade_link_repair::{load_per_folder_ledgers, repair_links};
use emuaccounts::trade_links::{self, METADATA_FILE, TRADE_LINKS_FILE};

fn resolve_dirs() -> (PathBuf, PathBuf) {
    let sources_override = std::env::var("EMUACC_SOURCES_DIR").ok();
    let generated_override = std::env::var("EMUACC_GENERATED_DIR").ok();
    let app_dir = platform_app_data_dir();
    let config = root_config::load_root_config(&app_dir);
    (
        root_config::resolve_sources(sources_override.as_deref(), &config, &app_dir),
        root_config::resolve_generated(generated_override.as_deref(), &config, &app_dir),
    )
}

fn main() -> Result<(), String> {
    let write = std::env::args().any(|a| a == "--write");
    let (sources_dir, generated_dir) = resolve_dirs();
    println!("sources:   {}", sources_dir.display());
    println!("generated: {}", generated_dir.display());
    println!("mode:      {}", if write { "WRITE" } else { "dry-run" });
    println!();

    let from = if sources_dir.join(TRADE_LINKS_FILE).exists() || write {
        TRADE_LINKS_FILE
    } else {
        METADATA_FILE
    };
    let links = if write {
        trade_links::load(&sources_dir)?
    } else {
        trade_links::read(&sources_dir)?
    };
    println!("loaded {} trade link(s) from {from}", links.len());

    let by_folder = load_per_folder_ledgers(&generated_dir, &sources_dir);
    println!("scanned {} folder ledger(s)\n", by_folder.len());

    let mut log = Vec::new();
    let report = repair_links(&links, &by_folder, &sources_dir, write, &mut log);
    for line in &log {
        println!("{line}");
    }

    println!();
    println!("== summary ==");
    println!("  links total:        {}", report.links_total);
    println!("  links already ok:   {}", report.links_already_ok);
    println!("  links unresolved:   {}", report.links_unresolved);
    println!("  rules to add:       {}", report.rules_added);
    println!("  folders to change:  {}", report.folders_changed.len());
    if !write && report.rules_added > 0 {
        println!();
        println!("(dry-run — re-run with --write to apply)");
    }
    Ok(())
}
