use std::path::PathBuf;
use std::process::Command;

/// Run `git` in the package directory and return trimmed stdout, or `None`
/// when git is missing or this is not a checkout (a source tarball).
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Export the producing commit so `generated/generator.json` can record it
/// (see `src/generator.rs`). Falls back to "unknown" / 0 outside a checkout.
/// Re-runs when HEAD moves: the git dir's HEAD file and, when HEAD is a
/// symbolic ref, the branch ref it points at.
fn export_git_build() {
    println!("cargo:rerun-if-env-changed=EMUACC_GIT_SHA");
    println!("cargo:rerun-if-env-changed=EMUACC_GIT_COMMIT_TIME");
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        let git_dir = PathBuf::from(git_dir);
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
        if let Some(branch_ref) = git(&["symbolic-ref", "-q", "HEAD"]) {
            println!("cargo:rerun-if-changed={}", git_dir.join(&branch_ref).display());
            // In a worktree the branch refs live in the main repository's dir.
            if let Some(common) = git(&["rev-parse", "--git-common-dir"]) {
                let common = PathBuf::from(common);
                let common = if common.is_absolute() {
                    common
                } else {
                    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default())
                        .join(common)
                };
                println!("cargo:rerun-if-changed={}", common.join(&branch_ref).display());
            }
        }
    }
    let sha = std::env::var("EMUACC_GIT_SHA")
        .ok()
        .or_else(|| git(&["rev-parse", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());
    let commit_time = std::env::var("EMUACC_GIT_COMMIT_TIME")
        .ok()
        .or_else(|| git(&["show", "-s", "--format=%ct", "HEAD"]))
        .unwrap_or_else(|| "0".to_string());
    println!("cargo:rustc-env=EMUACC_GIT_SHA={sha}");
    println!("cargo:rustc-env=EMUACC_GIT_COMMIT_TIME={commit_time}");
}

fn main() {
    export_git_build();
    tauri_build::build()
}
