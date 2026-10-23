#![deny(warnings)]

//! emuacc-ai-import — the Import button's AI path for PDF and text
//! statements, from the command line. The document is read, the model
//! configured in the app's Global Settings turns it into a normalised CSV,
//! and Rust re-validates every row (`ai_import`) exactly as the app does.
//! The CSV is printed on stdout and everything else goes to stderr; nothing
//! is written unless `--confirm` is passed, which stages the CSV through
//! `imports/`, files the original under `originals/` and rebuilds the ledger
//! and reports.
//!
//! Exit codes: 0 on success, 1 when extraction, validation or the import
//! fails, 2 for usage errors.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use emuacc_covid::ai_client::AiClient;
use emuacc_covid::ai_import::{
    self, DocumentImportContext, DocumentImportExtraction, DocumentImportResponse,
};
use emuacc_covid::display_config::vault_base_currency;
use emuacc_covid::processing_pipeline::{
    folder_to_account_name, process_imports, validate_folder_name, PipelineConfig,
};
use emuacc_covid::report_templates::{generate_all_reports, ALL_FORMATS};
use emuacc_covid::root_config::{self, platform_app_data_dir, RootConfig};

const NAME: &str = "emuacc-ai-import";

fn print_usage() {
    eprintln!("Usage: {NAME} [OPTIONS] --account FOLDER DOCUMENT");
    eprintln!();
    eprintln!("Interpret a PDF or text statement with the AI into a normalised CSV for one");
    eprintln!("account folder — the same extraction, validation and staging as the app's");
    eprintln!("Import button. The CSV is printed on stdout; nothing is written unless");
    eprintln!("--confirm is passed.");
    eprintln!();
    eprintln!("Vault resolution: EMUACC_SOURCES_DIR / EMUACC_GENERATED_DIR env overrides");
    eprintln!("win, otherwise the app config's current_root. The AI endpoint comes from");
    eprintln!("the app's Global Settings (EMUACC_APP_DATA_DIR overrides the config dir).");
    eprintln!();
    eprintln!("Options:");
    eprintln!(
        "  --account FOLDER     Account folder relative to sources/ (e.g. alex/bank/cdia)"
    );
    eprintln!(
        "  --currency CCY       Currency assumed in the prompt (default: the vault base currency)"
    );
    eprintln!(
        "  --confirm            Stage the CSV via imports/, keep the original in originals/,"
    );
    eprintln!("                       and rebuild the ledger and reports");
    eprintln!("  --sources-dir PATH   Override sources directory");
    eprintln!("  --generated-dir PATH Override generated directory");
    eprintln!("  --help               Show this help");
}

struct Args {
    account: String,
    document: PathBuf,
    currency: Option<String>,
    confirm: bool,
    sources_dir: Option<PathBuf>,
    generated_dir: Option<PathBuf>,
}

fn take_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, u8> {
    *i += 1;
    args.get(*i).cloned().ok_or_else(|| {
        eprintln!("{flag} requires a value");
        2
    })
}

/// `Err(code)` carries the exit code; the message has already been printed.
fn parse_args() -> Result<Args, u8> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut account = None;
    let mut document: Option<PathBuf> = None;
    let mut currency = None;
    let mut confirm = false;
    let mut sources_dir = None;
    let mut generated_dir = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return Err(0);
            }
            "--account" => account = Some(take_value(&args, &mut i, "--account")?),
            "--currency" => currency = Some(take_value(&args, &mut i, "--currency")?),
            "--confirm" => confirm = true,
            "--sources-dir" => {
                sources_dir = Some(PathBuf::from(take_value(&args, &mut i, "--sources-dir")?))
            }
            "--generated-dir" => {
                generated_dir = Some(PathBuf::from(take_value(&args, &mut i, "--generated-dir")?))
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {other}");
                print_usage();
                return Err(2);
            }
            path => {
                if document.replace(PathBuf::from(path)).is_some() {
                    eprintln!("exactly one DOCUMENT is expected");
                    print_usage();
                    return Err(2);
                }
            }
        }
        i += 1;
    }

    let (Some(account), Some(document)) = (account, document) else {
        eprintln!("--account FOLDER and DOCUMENT are required");
        print_usage();
        return Err(2);
    };
    Ok(Args {
        account,
        document,
        currency,
        confirm,
        sources_dir,
        generated_dir,
    })
}

fn resolve_dirs(args: &Args, root_cfg: &RootConfig, app_dir: &Path) -> (PathBuf, PathBuf) {
    let sources_dir = args.sources_dir.clone().unwrap_or_else(|| {
        let env_override = std::env::var("EMUACC_SOURCES_DIR").ok();
        root_config::resolve_sources(env_override.as_deref(), root_cfg, app_dir)
    });
    let generated_dir = args.generated_dir.clone().unwrap_or_else(|| {
        let env_override = std::env::var("EMUACC_GENERATED_DIR").ok();
        root_config::resolve_generated(env_override.as_deref(), root_cfg, app_dir)
    });
    (sources_dir, generated_dir)
}

fn step(s: &str) {
    eprintln!("{NAME}: {s}");
}

fn warn(w: &str) {
    eprintln!("{NAME}: warning: {w}");
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(code) => return ExitCode::from(code),
    };
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{NAME}: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    let app_dir = platform_app_data_dir();
    let root_cfg = root_config::load_root_config(&app_dir);
    let (sources_dir, generated_dir) = resolve_dirs(&args, &root_cfg, &app_dir);

    // Fail fast on an unconfigured AI: the error names Global Settings.
    let client = AiClient::from_root_config(&root_cfg).map_err(|e| e.to_string())?;

    validate_folder_name(&args.account)?;
    if !args.document.is_file() {
        return Err(format!("document not found: {}", args.document.display()));
    }
    // stage_ai_import would create a missing folder, which turns a typo into
    // a new account — only ever import into a folder that already exists.
    if !sources_dir.join(&args.account).is_dir() {
        return Err(format!(
            "account folder {} does not exist under {}",
            args.account,
            sources_dir.display()
        ));
    }

    let currency = match args.currency {
        Some(c) => Some(c),
        None => vault_base_currency(&generated_dir)?,
    };
    step("Reading document…");
    let ctx = DocumentImportContext::load(
        &sources_dir,
        &args.account,
        folder_to_account_name(&args.account),
        currency,
        root_cfg.ai_user_name.clone(),
        &args.document,
    )?;
    if ctx.transform_script.is_none() {
        let hint = format!(
            "no _transform.rhai applies to {} — bring this folder's first document in \
             through the app, which writes the transform",
            args.account
        );
        if args.confirm {
            return Err(format!(
                "{hint}; --confirm needs it so the staged CSV can be imported"
            ));
        }
        warn(&format!("{hint}; --confirm will be refused until then"));
    }

    let DocumentImportResponse {
        extraction,
        raw_output,
        success,
        error,
        validation_warnings,
        ..
    } = ai_import::run_document_import(&client, ctx, &step);
    let extraction = match extraction {
        Some(extraction) if success => extraction,
        _ => {
            if !raw_output.trim().is_empty() {
                eprintln!("{NAME}: last model output:\n{raw_output}");
            }
            return Err(error.unwrap_or_else(|| "extraction failed".to_string()));
        }
    };

    let doc_kind = if extraction.doc_kind.trim().is_empty() {
        "document"
    } else {
        extraction.doc_kind.as_str()
    };
    eprintln!(
        "{NAME}: proposed file {} — {} rows from a {doc_kind}",
        extraction.csv_filename,
        extraction.rows.len()
    );
    for w in extraction.warnings.iter().chain(&validation_warnings) {
        warn(w);
    }
    write_csv(&extraction)?;

    if !args.confirm {
        eprintln!("{NAME}: preview only — rerun with --confirm to import");
        return Ok(());
    }
    confirm_import(
        &sources_dir,
        &generated_dir,
        &root_cfg,
        &args.account,
        &extraction,
        &args.document,
    )
}

/// The CSV exactly as `--confirm` would stage it, on stdout alone.
fn write_csv(extraction: &DocumentImportExtraction) -> Result<(), String> {
    let mut writer = csv::Writer::from_writer(std::io::stdout().lock());
    writer
        .write_record(&extraction.headers)
        .map_err(|e| format!("failed to write CSV headers: {e}"))?;
    for row in &extraction.rows {
        writer
            .write_record(row)
            .map_err(|e| format!("failed to write CSV row: {e}"))?;
    }
    writer
        .flush()
        .map_err(|e| format!("failed to flush CSV: {e}"))
}

/// What the app's confirm button does: stage, file the original, drain
/// imports/ through the pipeline, then refresh the reports.
fn confirm_import(
    sources_dir: &Path,
    generated_dir: &Path,
    root_cfg: &RootConfig,
    account: &str,
    extraction: &DocumentImportExtraction,
    document: &Path,
) -> Result<(), String> {
    let staged = ai_import::stage_ai_import(
        sources_dir,
        account,
        &extraction.csv_filename,
        &extraction.headers,
        &extraction.rows,
        document,
    )?;
    for w in &staged.warnings {
        warn(w);
    }
    step(&format!("Staged {}", staged.csv_path.display()));
    if staged.original_copied {
        step("Original filed under originals/");
    }

    // No changed-folder hint: unlike the app, a CLI run cannot assume the
    // cache already covers every other folder, so let the pipeline
    // fingerprint everything (cheap when nothing else changed).
    let config = PipelineConfig {
        sources_dir: sources_dir.to_path_buf(),
        generated_dir: generated_dir.to_path_buf(),
        now_yyyymm: chrono::Local::now().format("%Y%m").to_string(),
        force: false,
        default_expense_account: root_cfg.default_expense_account.clone(),
        changed_folder_hint: None,
    };
    let (import_result, pipeline) = process_imports(&config, account)?;
    for f in &import_result.files_processed {
        step(&format!("Imported {account}/{f}"));
    }
    for w in import_result.warnings.iter().chain(&pipeline.warnings) {
        warn(w);
    }
    step(&format!(
        "Pipeline: {} CSV transformed, {} CSV cached, {} manual, {} total written",
        pipeline.csv_transformed,
        pipeline.csv_cached,
        pipeline.manual_count,
        pipeline.total_written
    ));

    if pipeline.output_files_written > 0 {
        let mut account_sets: Vec<String> = pipeline.owner_accounts.keys().cloned().collect();
        account_sets.sort();
        generate_all_reports(
            sources_dir,
            generated_dir,
            &account_sets,
            ALL_FORMATS,
            &root_cfg.extra_primary_account_prefixes,
        )
        .map_err(|e| format!("report generation failed: {e}"))?;
        step("Reports regenerated");
    }
    Ok(())
}
