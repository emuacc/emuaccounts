//! Correctness guarantee behind wiring `changed_folder_hint` into the
//! add/import/delete mutation handlers (`main.rs`): a **scoped** rebuild
//! (hint set to the changed folder) must produce **byte-identical generated
//! output** to a **full** rebuild for the same source mutation.
//!
//! Why this matters: a hint scopes the pipeline to the changed folder's
//! account-set — it skips loading other sets' caches AND restricts
//! `auto_link_equity_swaps`'s input (`tagged_txns`) to that set. That is only
//! safe if it changes nothing in the output. Asset accounts are namespaced by
//! account-set, so a full run never pairs swaps across sets anyway; this test
//! proves that equivalence empirically rather than by argument.
//!
//! Strategy (single vault, so absolute paths embedded in any generated file are
//! identical for both runs): baseline-build → mutate one set's folder → SCOPED
//! rebuild + snapshot → FORCED-FULL rebuild + snapshot → assert equal.

use emuaccounts::processing_pipeline::{run_pipeline, PipelineConfig};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn config(sources: &Path, generated: &Path, force: bool, hint: Option<Vec<String>>) -> PipelineConfig {
    PipelineConfig {
        sources_dir: sources.to_path_buf(),
        generated_dir: generated.to_path_buf(),
        now_yyyymm: "202502".to_string(),
        force,
        default_expense_account: emuaccounts::FALLBACK_EXPENSE_ACCOUNT.to_string(),
        changed_folder_hint: hint,
    }
}

fn write_folder(sources: &Path, folder_rel: &str, account: &str, manual: &str) {
    let dir = sources.join(folder_rel);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("accounts.transactions"),
        format!("account {account}\n  asset_class: \"cash\"\n"),
    )
    .unwrap();
    fs::write(dir.join("manual.transactions"), manual).unwrap();
}

/// Snapshot every generated file (sorted by relative path), excluding the
/// `.cache/` build cache, which carries write timestamps and is by-design
/// run-to-run different (mirrors `snapshot_generated` in processing_pipeline.rs).
fn snapshot(generated: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, base, out);
            } else if p.is_file() {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().to_string();
                if rel.starts_with(".cache/") || rel.ends_with(".DS_Store") {
                    continue;
                }
                out.push((rel, fs::read(&p).expect("read generated file")));
            }
        }
    }
    let mut out = Vec::new();
    walk(generated, generated, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn scoped_rebuild_matches_full_for_manual_add() {
    let sources = tmp("scoped-eq-src");
    let generated = tmp("scoped-eq-gen");

    // Two account sets so scoping actually skips something ("alex" vs "wife").
    write_folder(
        &sources,
        "alex/cash/bank/cba",
        "assets:cash:bank:cba",
        "2025-01-15 * Payroll\n    assets:cash:bank:cba    1000.00 AUD\n    income:salary          -1000.00 AUD\n",
    );
    write_folder(
        &sources,
        "wife/cash/bank/anz",
        "assets:cash:bank:anz",
        "2025-01-20 * Rent\n    assets:cash:bank:anz    -800.00 AUD\n    expenses:rent            800.00 AUD\n",
    );

    // Baseline full build (warms the cache + generated tree).
    run_pipeline(&config(&sources, &generated, false, None)).expect("baseline build");

    // Mutate ONE set's folder, exactly as add_manual_transaction does on disk.
    let manual = sources.join("alex/cash/bank/cba/manual.transactions");
    let mut body = fs::read_to_string(&manual).unwrap();
    body.push_str("\n2025-02-01 * Coffee\n    assets:cash:bank:cba      -3.50 AUD\n    expenses:coffee            3.50 AUD\n");
    fs::write(&manual, &body).unwrap();

    // SCOPED rebuild (the new handler behaviour) → snapshot before it's overwritten.
    run_pipeline(&config(
        &sources,
        &generated,
        false,
        Some(vec!["alex/cash/bank/cba".to_string()]),
    ))
    .expect("scoped rebuild");
    let scoped = snapshot(&generated);

    // FORCED-FULL rebuild (ground truth: recompute all sets) → snapshot.
    run_pipeline(&config(&sources, &generated, true, None)).expect("forced full rebuild");
    let full = snapshot(&generated);

    assert_eq!(
        scoped.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        full.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        "scoped vs full produced a different set of generated files",
    );
    for ((sp, sb), (fp, fb)) in scoped.iter().zip(full.iter()) {
        assert_eq!(sp, fp, "path ordering mismatch");
        assert_eq!(
            sb,
            fb,
            "scoped rebuild output differs from full rebuild at {sp}\n--- scoped ---\n{}\n--- full ---\n{}",
            String::from_utf8_lossy(sb),
            String::from_utf8_lossy(fb),
        );
    }
}

/// The two snapshots must list the same files with the same bytes.
fn assert_same_output(scoped: &[(String, Vec<u8>)], full: &[(String, Vec<u8>)], what: &str) {
    assert_eq!(
        scoped.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        full.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        "{what}: a different set of generated files",
    );
    for ((sp, sb), (fp, fb)) in scoped.iter().zip(full.iter()) {
        assert_eq!(sp, fp, "path ordering mismatch");
        assert_eq!(
            sb,
            fb,
            "{what}: output differs at {sp}\n--- scoped ---\n{}\n--- full ---\n{}",
            String::from_utf8_lossy(sb),
            String::from_utf8_lossy(fb),
        );
    }
}

/// A rule file written into one folder: what saving a category from the
/// table does.
fn write_rule(sources: &Path, folder_rel: &str, pattern: &str, account: &str) {
    fs::write(
        sources.join(folder_rel).join("_rules.json"),
        format!(
            r#"{{"rules":[{{"id":"r1","pattern":"*{pattern}*","amount_account":"{account}"}}]}}"#
        ),
    )
    .unwrap();
}

/// A one-set vault of three bank folders, built once so every folder has a
/// ledger, a summary with its accounts list and a cache entry.
fn one_set_vault() -> (PathBuf, PathBuf) {
    let sources = tmp("scoped-one-src");
    let generated = tmp("scoped-one-gen");
    write_folder(
        &sources,
        "alex/cash/bank/cba",
        "assets:cash:bank:cba",
        "2025-01-15 * Payroll\n    assets:cash:bank:cba    1000.00 AUD\n    income:salary          -1000.00 AUD\n\n2025-01-16 * Coffee Shop\n    assets:cash:bank:cba      -4.50 AUD\n    expenses:unknown           4.50 AUD\n",
    );
    write_folder(
        &sources,
        "alex/cash/bank/anz",
        "assets:cash:bank:anz",
        "2025-01-20 * Rent\n    assets:cash:bank:anz    -800.00 AUD\n    expenses:rent            800.00 AUD\n",
    );
    write_folder(
        &sources,
        "alex/cash/bank/ubank",
        "assets:cash:bank:ubank",
        "2025-01-21 * Interest\n    assets:cash:bank:ubank    12.00 AUD\n    income:interest          -12.00 AUD\n",
    );
    run_pipeline(&config(&sources, &generated, false, None)).expect("baseline build");
    (sources, generated)
}

/// The scoped run of a one-folder change, then the forced full rebuild of
/// the same sources: the generated trees must match byte for byte.
fn scoped_then_full(sources: &Path, generated: &Path, hint: Option<Vec<String>>) -> (bool, Vec<(String, Vec<u8>)>, Vec<(String, Vec<u8>)>) {
    let result = run_pipeline(&config(sources, generated, false, hint)).expect("rebuild");
    let scoped = snapshot(generated);
    run_pipeline(&config(sources, generated, true, None)).expect("forced full rebuild");
    (result.scoped, scoped, snapshot(generated))
}

/// A rule saved into one folder of a one-set vault: the run is scoped (the
/// other folders stay on disk unread) and its output matches a full rebuild.
#[test]
fn a_rule_change_in_one_folder_is_scoped_and_matches_a_full_rebuild() {
    let (sources, generated) = one_set_vault();
    write_rule(&sources, "alex/cash/bank/cba", "Coffee", "expenses:coffee");

    let (scoped, scoped_out, full_out) = scoped_then_full(&sources, &generated, None);

    assert!(scoped, "a rule change in one folder should be a scoped run");
    assert_same_output(&scoped_out, &full_out, "scoped rule change vs full rebuild");
    let ledger = fs::read_to_string(generated.join("alex/cash/bank/cba/ledger.transactions")).unwrap();
    assert!(ledger.contains("expenses:coffee"), "the rule should have re-categorised the row:\n{ledger}");
}

/// The same with the folder hint the rule commands pass.
#[test]
fn a_hinted_rule_change_is_scoped_and_matches_a_full_rebuild() {
    let (sources, generated) = one_set_vault();
    write_rule(&sources, "alex/cash/bank/cba", "Coffee", "expenses:coffee");

    let (scoped, scoped_out, full_out) =
        scoped_then_full(&sources, &generated, Some(vec!["alex/cash/bank/cba".to_string()]));

    assert!(scoped, "a hinted rule change should be a scoped run");
    assert_same_output(&scoped_out, &full_out, "hinted rule change vs full rebuild");
}

/// A change that brings a trade into a folder that had none: the swap linker
/// needs the whole set, so the run loads it after the transform and still
/// matches a full rebuild.
#[test]
fn a_change_that_introduces_a_trade_loads_the_set_and_matches_a_full_rebuild() {
    let (sources, generated) = one_set_vault();
    let manual = sources.join("alex/cash/bank/cba/manual.transactions");
    let mut body = fs::read_to_string(&manual).unwrap();
    body.push_str("\n2025-02-01 12:00:00 * Buy ETH\n    assets:cash:bank:cba    -1000.00 AUD\n    equity:trading           1000.00 AUD\n");
    fs::write(&manual, &body).unwrap();

    let (scoped, scoped_out, full_out) = scoped_then_full(&sources, &generated, None);

    assert!(!scoped, "a folder that now holds a trade needs the whole set");
    assert_same_output(&scoped_out, &full_out, "new trade vs full rebuild");
}

/// A change to the ignored list can hide a row in any folder: a full run.
#[test]
fn an_ignored_list_change_is_a_full_run_and_matches_a_full_rebuild() {
    let (sources, generated) = one_set_vault();
    fs::write(sources.join("_ignored.txt"), "txn:nothing-matches\n").unwrap();

    let (scoped, scoped_out, full_out) = scoped_then_full(&sources, &generated, None);

    assert!(!scoped, "an ignored-list change needs every folder");
    assert_same_output(&scoped_out, &full_out, "ignored list vs full rebuild");
}

/// An unchanged folder whose ledger was clobbered is not what the pipeline
/// last wrote: the full run's recovery path restores it.
#[test]
fn a_clobbered_unchanged_ledger_is_a_full_run_that_restores_it() {
    let (sources, generated) = one_set_vault();
    write_rule(&sources, "alex/cash/bank/cba", "Coffee", "expenses:coffee");
    let anz = generated.join("alex/cash/bank/anz/ledger.transactions");
    fs::write(&anz, "; clobbered by an external tool\n").unwrap();

    let (scoped, scoped_out, full_out) = scoped_then_full(&sources, &generated, None);

    assert!(!scoped, "a clobbered ledger needs the recovery path");
    assert_same_output(&scoped_out, &full_out, "clobbered ledger vs full rebuild");
    assert!(fs::read_to_string(&anz).unwrap().contains("Rent"), "the clobbered ledger should be restored");
}

/// Exercises the `auto_link` skip directly: a vault that DOES contain
/// `equity:trading` (so auto_link's own fast-reject does not fire), where the
/// change is to a folder with no trade. The scoped rebuild must skip auto_link
/// yet produce output byte-identical to a forced-full rebuild that runs it —
/// proving the skip preserves existing swap links rather than dropping them.
#[test]
fn scoped_skip_of_auto_link_matches_full_when_change_has_no_trade() {
    let sources = tmp("scoped-eq2-src");
    let generated = tmp("scoped-eq2-gen");

    // Crypto folder with two equity:trading legs (a BTC→ETH swap) — gives
    // auto_link real work and defeats its has_any_equity_trading fast-reject.
    write_folder(
        &sources,
        "alex/crypto/kraken",
        "assets:crypto:kraken",
        "2025-01-10 12:00:00 * Sell BTC\n    assets:crypto:kraken    -0.50000000 BTC\n    equity:trading           25000.00 USD\n\n2025-01-10 12:00:00 * Buy ETH\n    assets:crypto:kraken     5.00000000 ETH\n    equity:trading          -25000.00 USD\n",
    );
    // Cash folder — the one we change (no equity:trading).
    write_folder(
        &sources,
        "alex/cash/bank/cba",
        "assets:cash:bank:cba",
        "2025-01-15 * Payroll\n    assets:cash:bank:cba    1000.00 USD\n    income:salary          -1000.00 USD\n",
    );

    // Baseline full build — auto_link runs and annotates the swap into the
    // crypto folder's cached ledger.
    run_pipeline(&config(&sources, &generated, false, None)).expect("baseline build");

    // Mutate the CASH folder (no equity:trading).
    let manual = sources.join("alex/cash/bank/cba/manual.transactions");
    let mut body = fs::read_to_string(&manual).unwrap();
    body.push_str("\n2025-02-01 * Coffee\n    assets:cash:bank:cba      -3.50 USD\n    expenses:coffee            3.50 USD\n");
    fs::write(&manual, &body).unwrap();

    // SCOPED rebuild (hint = the cash folder) → no changed folder trades →
    // auto_link SKIPPED; the crypto swap annotations come from cache.
    run_pipeline(&config(
        &sources,
        &generated,
        false,
        Some(vec!["alex/cash/bank/cba".to_string()]),
    ))
    .expect("scoped rebuild");
    let scoped = snapshot(&generated);

    // FORCED-FULL rebuild → force empties unchanged_folders → auto_link RUNS.
    run_pipeline(&config(&sources, &generated, true, None)).expect("forced full rebuild");
    let full = snapshot(&generated);

    assert_eq!(
        scoped.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        full.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        "scoped(skip) vs full(run) produced a different set of generated files",
    );
    for ((sp, sb), (fp, fb)) in scoped.iter().zip(full.iter()) {
        assert_eq!(sp, fp, "path ordering mismatch");
        assert_eq!(
            sb,
            fb,
            "skip-auto_link scoped output differs from full at {sp}\n--- scoped ---\n{}\n--- full ---\n{}",
            String::from_utf8_lossy(sb),
            String::from_utf8_lossy(fb),
        );
    }
}
