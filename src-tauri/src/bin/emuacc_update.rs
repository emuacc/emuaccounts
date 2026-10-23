#![deny(warnings)]

//! Cron-safe headless vault refresh: run every daily-flagged plugin (the same
//! batch as the app's "update prices on startup"), then rebuild the generated
//! output and reports so balances reflect the fresh data.
//!
//! Exit code 0 when every plugin succeeded (exit 0), partially succeeded
//! (exit 2 — wrote data with warnings), or was skipped by the once-per-day
//! guard, AND the rebuild succeeded. Non-zero otherwise. Per-plugin outcomes
//! go to stdout; failures also go to stderr.

use std::path::PathBuf;

use emuacc_covid::plugins;
use emuacc_covid::processing_pipeline::{run_pipeline, PipelineConfig};
use emuacc_covid::root_config::{self, platform_app_data_dir};

fn resolve_sources_dir() -> PathBuf {
    let env_override = std::env::var("EMUACC_SOURCES_DIR").ok();
    let app_dir = platform_app_data_dir();
    let config = root_config::load_root_config(&app_dir);
    root_config::resolve_sources(env_override.as_deref(), &config, &app_dir)
}

fn resolve_generated_dir() -> PathBuf {
    let env_override = std::env::var("EMUACC_GENERATED_DIR").ok();
    let app_dir = platform_app_data_dir();
    let config = root_config::load_root_config(&app_dir);
    root_config::resolve_generated(env_override.as_deref(), &config, &app_dir)
}

fn print_usage() {
    eprintln!("Usage: emuacc-update [OPTIONS]");
    eprintln!();
    eprintln!("Update the configured vault's data without launching the app: run every");
    eprintln!("daily-flagged plugin (the 'update prices on startup' batch), then rebuild");
    eprintln!("the generated ledger and reports. Non-interactive and cron-safe: exits 0");
    eprintln!("on success, non-zero when any plugin or the rebuild fails.");
    eprintln!();
    eprintln!("Vault resolution: EMUACC_SOURCES_DIR / EMUACC_GENERATED_DIR env overrides");
    eprintln!("win, otherwise the app config's current_root.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --sources-dir PATH   Override sources directory");
    eprintln!("  --generated-dir PATH Override generated directory");
    eprintln!("  --force              Run daily plugins even if they already succeeded today");
    eprintln!("  --help               Show this help");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut sources_dir: Option<PathBuf> = None;
    let mut generated_dir: Option<PathBuf> = None;
    let mut force = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--sources-dir" => {
                i += 1;
                sources_dir = Some(PathBuf::from(args.get(i).unwrap_or_else(|| {
                    eprintln!("--sources-dir requires a value");
                    std::process::exit(2);
                })));
            }
            "--generated-dir" => {
                i += 1;
                generated_dir = Some(PathBuf::from(args.get(i).unwrap_or_else(|| {
                    eprintln!("--generated-dir requires a value");
                    std::process::exit(2);
                })));
            }
            "--force" => {
                force = true;
            }
            arg => {
                eprintln!("Unknown argument: {arg}");
                print_usage();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let app_data_dir = platform_app_data_dir();
    let root_cfg = root_config::load_root_config(&app_data_dir);
    let sources_dir = sources_dir.unwrap_or_else(resolve_sources_dir);
    let generated_dir = generated_dir.unwrap_or_else(resolve_generated_dir);
    let plugins_dir = match sources_dir.parent() {
        Some(root) => root.join("plugins"),
        None => {
            eprintln!(
                "emuacc-update: sources dir {} has no parent — cannot locate plugins/",
                sources_dir.display()
            );
            std::process::exit(1);
        }
    };

    println!(
        "emuacc-update: {} — sources {}, generated {}",
        chrono::Local::now().to_rfc3339(),
        sources_dir.display(),
        generated_dir.display()
    );

    let summary = plugins::run_daily_plugins(
        &plugins_dir,
        &sources_dir,
        !force,
        |plugin, stream, line| {
            if stream == "stderr" {
                eprintln!("[{plugin}] {line}");
            } else {
                println!("[{plugin}] {line}");
            }
        },
    );

    if summary.outcomes.is_empty() {
        println!("no daily plugins found in {}", plugins_dir.display());
    }
    for o in &summary.outcomes {
        if o.skipped_ran_today {
            println!("{}: skipped (already succeeded today)", o.dir_name);
        } else if o.success {
            println!("{}: ok ({} ms)", o.dir_name, o.duration_ms);
        } else if o.exit_code == Some(2) {
            println!(
                "{}: partial (exit 2 — wrote data with warnings, {} ms)",
                o.dir_name, o.duration_ms
            );
        } else {
            let reason = match o.exit_code {
                Some(code) => format!("exit {code}"),
                None => "could not run".to_string(),
            };
            println!("{}: failed ({reason})", o.dir_name);
            eprintln!("emuacc-update: plugin {} failed ({reason})", o.dir_name);
        }
    }

    // Always rebuild: the incremental pipeline is cheap when nothing changed,
    // and a cron run should also pick up source files added since the last
    // app launch — not just plugin output.
    let config = PipelineConfig {
        sources_dir,
        generated_dir,
        now_yyyymm: chrono::Local::now().format("%Y%m").to_string(),
        force: false,
        default_expense_account: root_cfg.default_expense_account.clone(),
        changed_folder_hint: None,
    };

    let mut ok = summary.all_ok();
    match run_pipeline(&config) {
        Ok(result) => {
            println!(
                "pipeline: {} transformed, {} cached, {} manual, {} total written",
                result.csv_transformed,
                result.csv_cached,
                result.manual_count,
                result.total_written,
            );
            // Reports read sources/_prices at generation time, so a price-only
            // plugin run must regenerate them even when no ledger output changed.
            if summary.any_ran() || result.output_files_written > 0 {
                let mut account_sets: Vec<String> =
                    result.owner_accounts.keys().cloned().collect();
                account_sets.sort();
                match emuacc_covid::report_templates::generate_all_reports(
                    &config.sources_dir,
                    &config.generated_dir,
                    &account_sets,
                    emuacc_covid::report_templates::ALL_FORMATS,
                    &root_cfg.extra_primary_account_prefixes,
                ) {
                    Ok(()) => println!("reports: regenerated"),
                    Err(e) => {
                        eprintln!("emuacc-update: report generation failed: {e}");
                        ok = false;
                    }
                }
            } else {
                println!("reports: unchanged (no plugin ran, no ledger output written)");
            }
        }
        Err(e) => {
            eprintln!("emuacc-update: pipeline rebuild failed: {e}");
            ok = false;
        }
    }

    if ok {
        println!("emuacc-update: done");
    } else {
        eprintln!("emuacc-update: completed with failures");
    }
    std::process::exit(if ok { 0 } else { 1 });
}
