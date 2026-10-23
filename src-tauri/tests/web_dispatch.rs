//! Exercises the web-mode command dispatcher without spawning a server or
//! depending on a real vault. Confirms routing, argument parsing, and that a
//! report command returns serializable JSON on an empty vault.

use serde_json::json;
use emuacc_covid::web::context::WebCtx;
use emuacc_covid::web::dispatch::dispatch;

fn empty_ctx() -> (tempfile::TempDir, WebCtx) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sources = tmp.path().join("sources");
    let generated = tmp.path().join("generated");
    std::fs::create_dir_all(&sources).unwrap();
    std::fs::create_dir_all(&generated).unwrap();
    (tmp, WebCtx::from_dirs(sources, generated))
}

#[test]
fn unknown_command_is_rejected() {
    let (_tmp, ctx) = empty_ctx();
    // rebuild_pipeline used to be the probe here, but it gained a read-only
    // web handler; use a command that can never exist.
    let err = dispatch(&ctx, "no_such_command", &json!({})).unwrap_err();
    assert!(err.contains("not supported"), "got: {err}");
}

#[test]
fn list_account_sets_empty_vault() {
    let (_tmp, ctx) = empty_ctx();
    let v = dispatch(&ctx, "list_account_sets", &json!({})).unwrap();
    assert_eq!(v, json!([]));
}

#[test]
fn list_report_years_empty_vault() {
    let (_tmp, ctx) = empty_ctx();
    // A report command with no generated reports yet returns an empty list.
    let v = dispatch(
        &ctx,
        "list_report_years_cmd",
        &json!({ "accountSet": "", "reportType": "cgt" }),
    )
    .unwrap();
    assert_eq!(v, json!([]));
}

#[test]
fn vault_status_reads_config() {
    let (_tmp, ctx) = empty_ctx();
    // Default RootConfig has no current_root.
    assert_eq!(dispatch(&ctx, "has_root_dir", &json!({})).unwrap(), json!(false));
    assert_eq!(dispatch(&ctx, "get_known_roots", &json!({})).unwrap(), json!([]));
}

#[test]
fn trade_links_degrade_to_empty() {
    let (_tmp, ctx) = empty_ctx();
    assert_eq!(dispatch(&ctx, "get_trade_links", &json!({})).unwrap(), json!([]));
}

#[test]
fn load_ui_state_reads_vault_file() {
    let (_tmp, ctx) = empty_ctx();
    // No ui-state.json yet → null (frontend treats it as "no saved state").
    assert_eq!(dispatch(&ctx, "load_ui_state", &json!({})).unwrap(), json!(null));

    let value = json!({ "version": 1, "nav": { "sidebarView": "accounts" } });
    emuacc_covid::ui_state::save_ui_state(&ctx.generated_dir, &value).unwrap();
    assert_eq!(dispatch(&ctx, "load_ui_state", &json!({})).unwrap(), value);
}

#[test]
fn save_ui_state_is_not_dispatched() {
    // The web server is read-only; the frontend's persister swallows the error.
    let (_tmp, ctx) = empty_ctx();
    let err = dispatch(&ctx, "save_ui_state", &json!({ "state": {} })).unwrap_err();
    assert!(err.contains("not supported"), "got: {err}");
}
