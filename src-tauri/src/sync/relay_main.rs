#![deny(warnings)]

use emuaccounts::sync::relay::server::{RelayConfig, RelayServer};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut bind = "0.0.0.0:8384".to_string();
    let mut data_dir = PathBuf::from("/tmp/emuacc-relay");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" => {
                i += 1;
                if i < args.len() {
                    bind = args[i].clone();
                }
            }
            "--data-dir" => {
                i += 1;
                if i < args.len() {
                    data_dir = PathBuf::from(&args[i]);
                }
            }
            "--help" | "-h" => {
                eprintln!("emuacc-relay — Self-hosted relay for multi-device sync");
                eprintln!();
                eprintln!("USAGE:");
                eprintln!("  emuacc-relay [OPTIONS]");
                eprintln!();
                eprintln!("OPTIONS:");
                eprintln!("  --bind <ADDR>       Bind address (default: 0.0.0.0:8384)");
                eprintln!("  --data-dir <PATH>   Data directory (default: /tmp/emuacc-relay)");
                eprintln!("  --help              Show this help");
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let config = RelayConfig {
        bind: bind.clone(),
        data_dir: data_dir.clone(),
        pairing_ttl_secs: 600,
    };

    let server = RelayServer::new(config).unwrap_or_else(|e| {
        eprintln!("Failed to start relay: {}", e);
        std::process::exit(1);
    });

    eprintln!("EmuAcc relay listening on {}", server.server_addr());
    eprintln!("Data directory: {}", data_dir.display());

    server.run();
}
