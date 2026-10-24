#![deny(warnings)]

pub const FALLBACK_ASSET_ACCOUNT: &str = "assets:unknown";
pub const FALLBACK_EXPENSE_ACCOUNT: &str = "expenses:unknown";

/// Validate and normalize a date string to YYYY-MM-DD.
/// Accepts YYYY-MM-DD (with optional trailing content) or YYYYMMDD formats.
pub fn parse_date_to_iso(s: &str) -> Result<String, String> {
    let s = s.trim();
    // YYYY-MM-DD format (possibly with time suffix)
    if s.len() >= 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' {
        let date_part = &s[..10];
        let year = &date_part[..4];
        let month = &date_part[5..7];
        let day = &date_part[8..10];
        if year.chars().all(|c| c.is_ascii_digit())
            && month.chars().all(|c| c.is_ascii_digit())
            && day.chars().all(|c| c.is_ascii_digit())
        {
            return Ok(date_part.to_string());
        }
        return Err(format!("date contains non-digits: '{s}'"));
    }
    // YYYYMMDD format (OFX-style)
    if s.len() >= 8 && s[..8].chars().all(|c| c.is_ascii_digit()) {
        let year = &s[0..4];
        let month = &s[4..6];
        let day = &s[6..8];
        return Ok(format!("{year}-{month}-{day}"));
    }
    Err(format!("unrecognized date format: '{s}'"))
}

/// Serialize to pretty-printed JSON with deterministically sorted keys.
///
/// Direct `serde_json::to_string_pretty(&struct)` outputs fields in Rust struct
/// declaration order, which changes when struct fields are reordered. This two-step
/// approach (struct → Value → string) produces alphabetically sorted keys, giving
/// stable output regardless of struct layout. HashMap keys are also sorted.
pub fn to_sorted_json_pretty(value: &impl serde::Serialize) -> Result<String, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    serde_json::to_string_pretty(&v)
}

pub mod ai_client;
pub mod ai_import;
pub mod ai_suggest;
pub mod build_cache;
pub mod csv_transform;
pub mod dedupe;
pub mod derived_cache;
pub mod display_config;
pub mod generated_ledger;
pub mod generated_store;
pub mod generator;
pub mod issues;
pub mod ledger_parser;
pub mod ofx_parser;
pub mod plugins;
pub mod processing_pipeline;
pub mod query;
pub mod report_csv;
pub mod report_templates;
pub mod reports;
pub mod root_config;
pub mod rules;
pub mod sources_state;
/// Multi-device sync, compiled only with the `sync` feature (docs/SYNC.md).
#[cfg(feature = "sync")]
pub mod sync;
pub mod terminology;
pub mod trade_link_repair;
pub mod trade_links;
pub mod transform_suggest;
pub mod ui_state;
pub mod web;
