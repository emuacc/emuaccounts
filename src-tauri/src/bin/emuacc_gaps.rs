#![deny(warnings)]

use std::path::PathBuf;

use emuaccounts::processing_pipeline::detect_account_gaps;
use emuaccounts::root_config::CliVault;

fn print_usage() {
    eprintln!("Usage: emuacc-gaps [OPTIONS]");
    eprintln!();
    eprintln!("Detect missing monthly statements per account.");
    eprintln!();
    eprintln!("Vault resolution: the EMUACC_GENERATED_DIR env override wins, otherwise");
    eprintln!("the app config's current_root.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --generated-dir PATH  Override generated directory");
    eprintln!("  --help                Show this help");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut generated_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--generated-dir" => {
                i += 1;
                generated_dir = Some(PathBuf::from(args.get(i).unwrap_or_else(|| {
                    eprintln!("--generated-dir requires a value");
                    std::process::exit(2);
                })));
            }
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {arg}");
                print_usage();
                std::process::exit(2);
            }
            _ => {
                eprintln!("Unexpected argument: {}", args[i]);
                print_usage();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let generated_dir = generated_dir.unwrap_or_else(|| CliVault::load().generated_dir);

    if !generated_dir.exists() {
        eprintln!(
            "Generated directory does not exist: {}",
            generated_dir.display()
        );
        std::process::exit(1);
    }

    match detect_account_gaps(&generated_dir) {
        Ok(gaps) => {
            if gaps.is_empty() {
                println!("No accounts found.");
                return;
            }
            for gap in &gaps {
                let first = format!("{}-{}", &gap.first_month[0..4], &gap.first_month[4..6]);
                let last = format!("{}-{}", &gap.last_month[0..4], &gap.last_month[4..6]);
                println!("{}", gap.account);
                println!("  First: {}  Last: {}", first, last);
                if gap.missing_months.is_empty() {
                    println!("  No gaps");
                } else {
                    let formatted: Vec<String> = gap
                        .missing_months
                        .iter()
                        .map(|m| format!("{}-{}", &m[0..4], &m[4..6]))
                        .collect();
                    println!("  Missing: {}", formatted.join(", "));
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
