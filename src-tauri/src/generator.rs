//! The build that produced `generated/`.
//!
//! A vault that commits `generated/` wants its diffs to read as "this data
//! changed, and this is its impact". Several machines regenerate the same
//! vault (the nightly `emuacc-update`, the installed app, a dev build) and
//! each writes its own build's output, so a diff can be a generator change
//! wearing a data change's clothes — and two builds can flip a figure back
//! and forth across commits. `generator.json` names the producing build in
//! one small tracked file, so the diff says when the generator changed, and
//! a build older than the one recorded warns before it overwrites newer
//! output.

use std::path::Path;

use serde::{Deserialize, Serialize};

pub const FILE: &str = "generator.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratorStamp {
    /// Full git commit sha of the producing build; "unknown" outside a checkout.
    pub git_sha: String,
    /// Committer time of that commit, unix seconds; 0 when unknown. The
    /// ordering key for "older build" — shas don't order.
    pub commit_time: i64,
    /// Crate version (`CARGO_PKG_VERSION`).
    pub version: String,
    /// The pipeline's `MATCHER_VERSION` at build time.
    pub matcher_version: String,
}

impl GeneratorStamp {
    /// This binary's build, as exported by `build.rs`.
    pub fn current() -> Self {
        GeneratorStamp {
            git_sha: option_env!("EMUACC_GIT_SHA").unwrap_or("unknown").to_string(),
            commit_time: option_env!("EMUACC_GIT_COMMIT_TIME")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            version: env!("CARGO_PKG_VERSION").to_string(),
            matcher_version: crate::rules::MATCHER_VERSION.to_string(),
        }
    }

    pub fn load(generated_dir: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(generated_dir.join(FILE)).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn save(&self, generated_dir: &Path) -> Result<(), String> {
        let json = crate::to_sorted_json_pretty(self)
            .map_err(|e| format!("failed to serialize {FILE}: {e}"))?;
        std::fs::create_dir_all(generated_dir)
            .map_err(|e| format!("failed to create {}: {e}", generated_dir.display()))?;
        std::fs::write(generated_dir.join(FILE), json)
            .map_err(|e| format!("failed to write {FILE}: {e}"))
    }

    /// The warning to raise before this build overwrites output that
    /// `recorded` produced, if `recorded` is the newer build.
    pub fn older_than(&self, recorded: &GeneratorStamp) -> Option<String> {
        if recorded.commit_time <= self.commit_time {
            return None;
        }
        Some(format!(
            "generated/ was last produced by build {} ({}); this build is {} ({}), which is \
             older — regenerating with it may revert what the newer build changed",
            recorded.short_sha(),
            recorded.commit_date(),
            self.short_sha(),
            self.commit_date(),
        ))
    }

    fn short_sha(&self) -> &str {
        let n = self.git_sha.len().min(7);
        &self.git_sha[..n]
    }

    fn commit_date(&self) -> String {
        match chrono::DateTime::from_timestamp(self.commit_time, 0) {
            Some(dt) if self.commit_time > 0 => dt.format("committed %Y-%m-%d %H:%M UTC").to_string(),
            _ => "commit time unknown".to_string(),
        }
    }
}

/// Record this build as the producer of `generated_dir`. Returns the
/// older-build warning (already printed to stderr) when the file recorded a
/// newer build — the caller decides whether to keep it with its own warnings.
pub fn stamp(generated_dir: &Path) -> Result<Option<String>, String> {
    let current = GeneratorStamp::current();
    let warning = GeneratorStamp::load(generated_dir)
        .and_then(|recorded| current.older_than(&recorded));
    if let Some(w) = &warning {
        eprintln!("[generator] warning: {w}");
    }
    current.save(generated_dir)?;
    Ok(warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp_at(commit_time: i64) -> GeneratorStamp {
        GeneratorStamp {
            git_sha: format!("{commit_time:040x}"),
            commit_time,
            version: "0.1.0".to_string(),
            matcher_version: "v6".to_string(),
        }
    }

    #[test]
    fn current_build_carries_version_and_matcher() {
        let current = GeneratorStamp::current();
        assert_eq!(current.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(current.matcher_version, crate::rules::MATCHER_VERSION);
        assert!(!current.git_sha.is_empty());
    }

    #[test]
    fn only_a_newer_recorded_build_warns() {
        let this = stamp_at(1_700_000_000);
        assert!(this.older_than(&stamp_at(1_700_000_000)).is_none());
        assert!(this.older_than(&stamp_at(1_600_000_000)).is_none());
        let warning = this.older_than(&stamp_at(1_800_000_000)).expect("newer build warns");
        assert!(warning.contains("which is older"), "{warning}");
        assert!(warning.contains(&stamp_at(1_800_000_000).git_sha[..7]), "{warning}");
    }

    #[test]
    fn unknown_build_warns_against_a_known_one() {
        // An unstamped build (commit time 0) overwriting a known build's
        // output is exactly the provenance gap the file exists to close.
        assert!(stamp_at(0).older_than(&stamp_at(1_700_000_000)).is_some());
        assert!(stamp_at(1_700_000_000).older_than(&stamp_at(0)).is_none());
    }

    #[test]
    fn stamp_writes_the_current_build_and_round_trips() {
        let dir = std::env::temp_dir().join(format!(
            "emuacc-generator-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(stamp(&dir).expect("stamp").is_none(), "no prior stamp, no warning");
        assert_eq!(GeneratorStamp::load(&dir), Some(GeneratorStamp::current()));
        let mut newer = GeneratorStamp::current();
        newer.commit_time += 86_400;
        newer.git_sha = "f".repeat(40);
        newer.save(&dir).expect("save newer");
        let warning = stamp(&dir).expect("stamp").expect("older build warns");
        assert!(warning.contains("fffffff"), "{warning}");
        assert_eq!(GeneratorStamp::load(&dir), Some(GeneratorStamp::current()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
