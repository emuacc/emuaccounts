//! The runtime parser for `.transactions` ledgers and `P` price files.
//!
//! `Ledger.g4`, `LedgerLexer.g4` and `Prices.g4` (repository root) are the
//! specification of this syntax; this handwritten parser is the runtime
//! because it is 30-75x faster than the parser generated from the grammar
//! (decision-003). The two are held together by the differential parity
//! tests in [`grammar_parity`]: every fixture, a validator corpus and, opt-in,
//! a whole vault must get the same accept/reject decision from both. Change
//! the grammar first, regenerate, then this parser.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

// The parser generated from the authoritative grammar (`Ledger.g4`,
// `LedgerLexer.g4`, `Prices.g4`; regenerate with
// `scripts/generate_ledger_parsers.sh`). It is the oracle the parity tests
// hold this handwritten parser to, and nothing else: it only exists in test
// builds (decision-003).
#[cfg(test)]
#[allow(dead_code, nonstandard_style, clippy::all)]
mod generated;

#[cfg(test)]
mod grammar_parity;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriceAnnotation {
    pub is_total: bool,
    pub amount: f64,
    pub amount_text: String,
    pub commodity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostAnnotation {
    pub is_total: bool,
    pub amount: Option<f64>,
    pub amount_text: Option<String>,
    pub commodity: Option<String>,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PriceDirective {
    pub datetime: String,
    pub commodity: String,
    pub price_amount: f64,
    pub price_amount_text: String,
    pub quote_commodity: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PricesParseResult {
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub prices: Vec<PriceDirective>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Posting {
    pub account: String,
    pub amount: f64,
    pub amount_text: String,
    pub commodity: String,
    pub remainder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<CostAnnotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PriceAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub date: String,
    pub datetime: String,
    pub status: Option<char>,
    pub payee: Option<String>,
    pub narration: Option<String>,
    pub meta: Option<String>,
    pub postings: Vec<Posting>,
    /// Display-friendly payee set by rules (raw payee is immutable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_payee: Option<String>,
    /// Primary transaction amount (from first posting).
    pub amount: f64,
    /// Commodity of the primary amount (immutable raw value from CSV).
    pub amount_commodity: String,
    /// Display-friendly commodity set by rules (raw amount_commodity is immutable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_amount_commodity: Option<String>,
    /// Fee amount, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
    /// Commodity of the fee, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_commodity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommodityAmount {
    pub commodity: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountBalance {
    pub account: String,
    pub totals: Vec<CommodityAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountProperties {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParseResult {
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub transactions: Vec<Transaction>,
    /// Per-account totals: postings PLUS opening declarations.
    pub balances: Vec<AccountBalance>,
    /// The `opening` amounts declared on accounts, on their own. Openings have
    /// no date and no postings, so anything that aggregates transactions and
    /// wants the real balance (reports, hidden-account filtering) folds these
    /// in separately rather than rebuilding `balances` from postings alone.
    pub opening_balances: Vec<AccountBalance>,
    pub accounts_with_opening: Vec<String>,
    pub account_properties: HashMap<String, AccountProperties>,
}

#[derive(Debug, Clone, PartialEq)]
struct AccountDeclaration {
    account: String,
    default_commodity: Option<String>,
    opening: Option<CommodityAmount>,
    name: Option<String>,
}

/// Normalise a datetime string so date-only, ISO `T...Z`, and space-separated
/// forms are lexically comparable. Used by `PriceGraph` when sorting prices
/// and resolving the nearest-prior price at a target datetime.
///
/// - `"YYYY-MM-DD"` → `"YYYY-MM-DD 00:00:00"` (treated as start-of-day)
/// - `"YYYY-MM-DDTHH:MM:SS..."` → `"YYYY-MM-DD HH:MM:SS"` (T → space, fractional seconds and timezone dropped)
/// - `"YYYY-MM-DD HH:MM:SS"` → unchanged
fn normalize_datetime_for_compare(s: &str) -> String {
    let s = s.trim();
    if s.len() == 10 {
        return format!("{s} 00:00:00");
    }
    if s.len() > 10 && s.as_bytes()[10] == b'T' {
        let date = &s[..10];
        let after_t = &s[11..];
        let time: String = after_t
            .chars()
            .take_while(|c| *c != '.' && *c != 'Z' && *c != '+')
            .collect();
        return format!("{date} {time}");
    }
    if s.len() > 19 {
        return s[..19].to_string();
    }
    s.to_string()
}

fn diag(line: usize, column: usize, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        line,
        column,
        message: message.into(),
        file: None,
    }
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn is_directive(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    trimmed.starts_with(';')
}

// Digits are ASCII throughout, as the lexer's DIGIT is.
static HEADER_DATETIME_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Accepts both ISO T-separator (2022-08-15T14:30:00Z) and space separator (2022-08-15 14:30:00)
    Regex::new(
        r"^(?P<date>[0-9]{4}-[0-9]{2}-[0-9]{2})(?P<time>[T ][0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]{1,6})?(?:Z|[+-][0-9]{2}:[0-9]{2})?)?\s+",
    )
    .expect("datetime regex")
});

/// The datetime of a `P` directive: a date, optionally followed by a time
/// with `T` or a blank between them (one DATETIME token in the lexer).
static PRICE_DATETIME_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(
        r"^[0-9]{4}-[0-9]{2}-[0-9]{2}(?:[T ][0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]{1,6})?(?:Z|[+-][0-9]{2}:[0-9]{2})?)?$",
    )
    .expect("price datetime regex")
});

/// The time half of a blank-separated `P` directive datetime.
static TIME_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^[0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]{1,6})?(?:Z|[+-][0-9]{2}:[0-9]{2})?$")
        .expect("time regex")
});

static ACCOUNT_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Allow 2+ segments (e.g. `expenses:unknown`) for usability.
    // \p{L} = Unicode letter, \p{N} = Unicode digit.
    // NOTE: since commodities may carry a venue prefix (`xyz:WTIOIL`, see
    // COMMODITY_RE) an account and a commodity are no longer lexically
    // disjoint. Every construct places them positionally; never classify a
    // token as account-vs-commodity by shape.
    Regex::new(r"^[\p{L}\p{N}_.\-]+:[\p{L}\p{N}_.\-]+(?::[\p{L}\p{N}_.\-]+)*$")
        .expect("account regex")
});

static AMOUNT_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^[+-]?(?:[0-9]+|[0-9]{1,3}(?:,[0-9]{3})+)(?:\.[0-9]+)?$").expect("amount regex")
});

// The three lexer tokens that can occur inside a commodity or account name.
// `LedgerLexer.g4` takes the longest match at every token start, so a
// DATETIME is cut out of a name wherever `YYYY-MM-DD` begins a token.
static DATETIME_START_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}").expect("datetime start regex"));
static ALNUM_TOKEN_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^[\p{L}\p{N}_]+").expect("alnum token regex"));
static NUMBER_TOKEN_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^[0-9]+(?:\.[0-9]+)?").expect("number token regex"));

static COMMODITY_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // The commodity grammar. \p{L} = Unicode letter, \p{N} = Unicode digit.
    //
    // Bare tickers for fungible assets (BTC, USD, BRK.B, RAY-SOL) — the venue
    // lives in the account path. `EXCHANGE:SYMBOL` (Google Finance / TradingView
    // convention, and Hyperliquid's own `<dex>:<coin>`) for venue-scoped
    // instruments: perps, builder-dex coins, cross-asset ticker collisions
    // (`ASX:ENA`). ':' is legal only after the first character.
    //
    // Everything else stays excluded because the format depends on it:
    //   whitespace  — every construct is whitespace-tokenised
    //   @ { }       — price / cost markers scanned by `parse_remainder`
    //   "           — header quoting
    //   ,           — cost bodies and meta tags split on it
    //   |           — rule alternation (`wildcard_match`), `leg_id` separator
    //   / \ % ...   — `_prices/<stem>.txt` file names (see `price_file_stem`)
    // Decision record: .backlog/decisions/decision-001.
    Regex::new(r"^[\p{L}\p{N}_][\p{L}\p{N}_.\-:]*$").expect("commodity regex")
});

fn header_datetime_re() -> &'static Regex { &HEADER_DATETIME_RE }
fn account_re() -> &'static Regex { &ACCOUNT_RE }
fn amount_re() -> &'static Regex { &AMOUNT_RE }
fn commodity_re() -> &'static Regex { &COMMODITY_RE }

/// Whether `LedgerLexer.g4` would cut a `DATETIME` token out of `text`.
///
/// The lexer takes the longest match at every token start, so `2026-01-15`
/// is a DATETIME wherever a token starts there: at the start of the name or
/// after `.`, `-` or `:`. Inside a longer ALNUM or NUMBER run it is not a
/// token boundary (`a_2026-01-15`, `2026.2026-01-15`). The grammar makes no
/// commodity or account piece out of a DATETIME, so neither do the validators.
fn contains_datetime_token(text: &str) -> bool {
    let mut rest = text;
    while let Some(first) = rest.chars().next() {
        if DATETIME_START_RE.is_match(rest) {
            return true;
        }
        let token_len = [&ALNUM_TOKEN_RE, &NUMBER_TOKEN_RE]
            .iter()
            .filter_map(|re| re.find(rest).map(|m| m.end()))
            .max()
            .unwrap_or(first.len_utf8());
        rest = &rest[token_len..];
    }
    false
}

/// The single definition of "is this a legal commodity name", mirroring the
/// `commodity` rule of `Ledger.g4` / `Prices.g4`. Every other check (parser,
/// transform sanitiser, `set_price`, rule save) delegates here so the grammar
/// cannot drift between call sites.
pub fn is_valid_commodity(text: &str) -> bool {
    commodity_re().is_match(text) && !contains_datetime_token(text)
}

/// The single definition of "is this a legal account path", mirroring the
/// `account` rule of `Ledger.g4`: two or more `:`-separated segments, none of
/// them shaped like a date.
pub fn is_valid_account(text: &str) -> bool {
    account_re().is_match(text) && !contains_datetime_token(text)
}

/// File stem for `_prices/<stem>.txt`. Percent-encodes the characters that are
/// unsafe in a file name on some platform (`:` is NTFS alternate-data-stream
/// syntax on Windows and renders as `/` in Finder): `xyz:WTIOIL` →
/// `xyz%3AWTIOIL`. Unambiguous because `%` is outside the commodity grammar;
/// existing names (no such characters) are unchanged. The `P` lines inside the
/// file carry the true commodity — `PriceGraph::load` keys on them, not on the
/// file name — so the mapping never needs to be reversed.
pub fn price_file_stem(commodity: &str) -> String {
    let mut out = String::with_capacity(commodity.len());
    for c in commodity.chars() {
        match c {
            ':' | '/' | '\\' | '*' | '?' | '"' | '<' | '>' | '|' => {
                out.push_str(&format!("%{:02X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Parse an amount token, accepting thousands-separator commas ("1,823.92").
/// Returns None when the token is not a valid amount.
pub fn parse_amount(text: &str) -> Option<f64> {
    if !amount_re().is_match(text) {
        return None;
    }
    text.replace(',', "").parse().ok()
}

fn take_token(input: &str) -> Option<(String, &str)> {
    let trimmed = input.trim_start_matches([' ', '\t']);
    if trimmed.is_empty() {
        return None;
    }

    if let Some(quoted) = trimmed.strip_prefix('"') {
        let mut chars = quoted.chars();
        let mut out = String::new();
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                }
                '"' => {
                    let rest = chars.as_str();
                    return Some((out, rest));
                }
                c => out.push(c),
            }
        }

        // Unterminated quote; treat the entire thing as a single token (minus the opening quote).
        return Some((quoted.to_string(), ""));
    }

    let end = trimmed.find([' ', '\t']).unwrap_or(trimmed.len());
    Some((trimmed[..end].to_string(), &trimmed[end..]))
}

/// Extract a named field (e.g. "source_payee") from a comma-separated meta string.
/// Returns (remaining meta without that field, extracted value).
fn extract_meta_field(meta: &Option<String>, key: &str) -> (Option<String>, Option<String>) {
    let meta_str = match meta {
        Some(s) => s,
        None => return (None, None),
    };
    let prefix = format!("{key}:");
    let parts: Vec<&str> = meta_str.split(',').map(|s| s.trim()).collect();
    let mut value = None;
    let mut remaining = Vec::new();
    for part in parts {
        if part.starts_with(&prefix) {
            value = Some(part[prefix.len()..].to_string());
        } else {
            remaining.push(part);
        }
    }
    let clean = remaining.join(", ");
    let clean_meta = if clean.is_empty() { None } else { Some(clean) };
    (clean_meta, value)
}

fn parse_header_fields(
    header_after_datetime: &str,
) -> (Option<char>, Option<String>, Option<String>) {
    let mut rest = header_after_datetime;
    let mut tokens: Vec<String> = Vec::new();
    while let Some((tok, next)) = take_token(rest) {
        tokens.push(tok);
        rest = next;
    }

    let mut idx = 0;
    let status = tokens.first().and_then(|t| match t.as_str() {
        "*" => {
            idx = 1;
            Some('*')
        }
        "!" => {
            idx = 1;
            Some('!')
        }
        _ => None,
    });

    let payee = tokens.get(idx).cloned();
    let narration = match tokens.get(idx + 1..) {
        Some([]) | None => None,
        Some([only]) => Some(only.clone()),
        Some(many) => Some(many.join(" ")),
    };

    (status, payee, narration)
}

fn flush_transaction(
    current: &mut Option<(usize, Transaction)>,
    diagnostics: &mut Vec<Diagnostic>,
    transactions: &mut Vec<Transaction>,
) {
    if let Some((header_line, mut txn)) = current.take() {
        if txn.postings.is_empty() {
            diagnostics.push(diag(header_line, 0, "transaction missing postings"));
        }
        // Populate first-class amount/fee fields from postings
        if let Some(first) = txn.postings.first() {
            txn.amount = first.amount;
            if txn.amount_commodity.is_empty() {
                // Normal case: populate from posting commodity
                txn.amount_commodity = first.commodity.clone();
            } else {
                // Re-parsed: amount_commodity already has raw value from source_commodity meta.
                // The posting commodity is the display (effective) value.
                txn.display_amount_commodity = Some(first.commodity.clone());
            }
        }
        // Fee heuristic: 3+ postings and last account starts with "expenses:fees"
        if txn.postings.len() >= 3 {
            if let Some(last) = txn.postings.last() {
                if last.account.starts_with("expenses:fees") {
                    txn.fee = Some(last.amount);
                    txn.fee_commodity = Some(last.commodity.clone());
                }
            }
        }
        transactions.push(txn);
    }
}

pub fn parse_transactions(contents: &str) -> ParseResult {
    let header_re = header_datetime_re();

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut current: Option<(usize, Transaction)> = None;
    let mut current_account: Option<(usize, AccountDeclaration)> = None;
    let mut transactions: Vec<Transaction> = Vec::new();
    let mut account_declarations: Vec<AccountDeclaration> = Vec::new();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end_matches('\r');

        if is_blank(line) {
            flush_transaction(&mut current, &mut diagnostics, &mut transactions);
            if let Some((_, decl)) = current_account.take() {
                account_declarations.push(decl);
            }
            continue;
        }

        if is_directive(line) {
            continue;
        }

        if line == "account" || line.starts_with("account ") || line.starts_with("account\t") {
            flush_transaction(&mut current, &mut diagnostics, &mut transactions);
            if let Some((_, decl)) = current_account.take() {
                account_declarations.push(decl);
            }

            let before_meta = line.split_once(';').map(|(l, _)| l).unwrap_or(line);
            let mut parts = before_meta.split_whitespace();
            let kw = parts.next().unwrap_or_default();
            if kw != "account" {
                diagnostics.push(diag(
                    line_no,
                    0,
                    "invalid line: expected transaction header, posting, or directive",
                ));
                continue;
            }
            let Some(account) = parts.next() else {
                diagnostics.push(diag(line_no, 0, "account declaration missing account path"));
                continue;
            };

            if !is_valid_account(account) {
                diagnostics.push(diag(line_no, 0, format!("invalid account path: {account}")));
            }

            let default_commodity = parts.next().map(|s| s.to_string());
            if let Some(c) = default_commodity.as_deref() {
                if !is_valid_commodity(c) {
                    diagnostics.push(diag(line_no, 0, format!("invalid commodity: {c}")));
                }
            }

            if parts.next().is_some() {
                diagnostics.push(diag(
          line_no,
          0,
          "unexpected extra tokens in account declaration (expected: account <path> [COMMODITY])",
        ));
            }

            current_account = Some((
                line_no,
                AccountDeclaration {
                    account: account.to_string(),
                    default_commodity,
                    opening: None,
                    name: None,
                },
            ));
            continue;
        }

        // Indented line: a posting, or an account declaration detail. Any run
        // of blanks indents (Ledger.g4 `WS`); blank lines were handled above.
        let stripped = line.trim_start_matches([' ', '\t']);
        if stripped.len() < line.len() {
            let indent = line.len() - stripped.len();
            let Some((_, txn)) = current.as_mut() else {
                let Some((_, decl)) = current_account.as_mut() else {
                    diagnostics.push(diag(line_no, 0, "unexpected indented line"));
                    continue;
                };

                let before_meta = stripped
                    .split_once(';')
                    .map(|(l, _)| l)
                    .unwrap_or(stripped);
                let mut parts = before_meta.split_whitespace();
                let Some(kind) = parts.next() else {
                    diagnostics.push(diag(line_no, indent, "invalid account declaration line"));
                    continue;
                };

                match kind {
                    "opening" => {
                        let Some(amount_text) = parts.next() else {
                            diagnostics.push(diag(line_no, indent, "opening missing amount"));
                            continue;
                        };

                        let Some(amount) = parse_amount(amount_text) else {
                            diagnostics.push(diag(
                                line_no,
                                indent,
                                format!("invalid amount: {amount_text}"),
                            ));
                            continue;
                        };

                        let commodity = match parts.next() {
                            Some(c) => Some(c.to_string()),
                            None => decl.default_commodity.clone(),
                        };

                        let Some(commodity) = commodity else {
                            diagnostics.push(diag(
                                line_no,
                                indent,
                                "opening missing commodity and account has no default commodity",
                            ));
                            continue;
                        };

                        if !is_valid_commodity(&commodity) {
                            diagnostics.push(diag(
                                line_no,
                                indent,
                                format!("invalid commodity: {commodity}"),
                            ));
                        }

                        if parts.next().is_some() {
                            diagnostics.push(diag(
                line_no,
                indent,
                "unexpected extra tokens in opening declaration (expected: opening <amount> [COMMODITY])",
              ));
                        }

                        if decl.opening.is_some() {
                            diagnostics.push(diag(line_no, indent, "duplicate opening declaration"));
                            continue;
                        }

                        decl.opening = Some(CommodityAmount { commodity, amount });
                    }
                    "name" => {
                        let rest_str: Vec<&str> = parts.collect();
                        if rest_str.is_empty() {
                            diagnostics.push(diag(line_no, indent, "name missing value"));
                            continue;
                        }
                        if decl.name.is_some() {
                            diagnostics.push(diag(line_no, indent, "duplicate name declaration"));
                            continue;
                        }
                        decl.name = Some(rest_str.join(" "));
                    }
                    _ => {
                        diagnostics.push(diag(
                            line_no,
                            indent,
                            format!("unknown account declaration entry: {kind}"),
                        ));
                    }
                }

                continue;
            };

            let mut parts = stripped.split_whitespace();
            let account = parts.next();
            let amount = parts.next();
            let commodity = parts.next();

            if account.is_none() {
                diagnostics.push(diag(line_no, indent, "missing account"));
                continue;
            }

            let account = account.unwrap();
            if !is_valid_account(account) {
                diagnostics.push(diag(line_no, indent, format!("invalid account path: {account}")));
            }

            if amount.is_none() {
                diagnostics.push(diag(line_no, indent + account.len() + 1, "missing amount"));
                continue;
            }

            let amount = amount.unwrap();
            let Some(parsed_amount) = parse_amount(amount) else {
                diagnostics.push(diag(
                    line_no,
                    indent + account.len() + 1,
                    format!("invalid amount: {amount}"),
                ));
                continue;
            };

            if commodity.is_none() {
                diagnostics.push(diag(
                    line_no,
                    indent + account.len() + 1 + amount.len() + 1,
                    "missing commodity",
                ));
                continue;
            }

            let commodity = commodity.unwrap();
            if !is_valid_commodity(commodity) {
                diagnostics.push(diag(
                    line_no,
                    indent + account.len() + 1 + amount.len() + 1,
                    format!("invalid commodity: {commodity}"),
                ));
            }

            let remainder_parts: Vec<&str> = parts.collect();
            let remainder = if remainder_parts.is_empty() {
                None
            } else {
                Some(remainder_parts.join(" "))
            };

            let (cost, price, annotation_error) = remainder
                .as_deref()
                .map(parse_remainder)
                .unwrap_or((None, None, None));
            if let Some(message) = annotation_error {
                diagnostics.push(diag(
                    line_no,
                    indent + account.len() + 1 + amount.len() + 1 + commodity.len() + 1,
                    message,
                ));
            }
            txn.postings.push(Posting {
                account: account.to_string(),
                amount: parsed_amount,
                amount_text: amount.to_string(),
                commodity: commodity.to_string(),
                remainder,
                cost,
                price,
            });

            continue;
        }

        // Header line
        if let Some(caps) = header_re.captures(line) {
            flush_transaction(&mut current, &mut diagnostics, &mut transactions);
            if let Some((_, decl)) = current_account.take() {
                account_declarations.push(decl);
            }

            let date = caps
                .name("date")
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "0000-00-00".to_string());
            let time = caps.name("time").map(|m| m.as_str()).unwrap_or("");
            let datetime = format!("{date}{time}");

            let datetime_span = caps.get(0).expect("datetime capture");
            let datetime_end = datetime_span.end();

            // The meta comment starts at the first `;` outside quotes.
            let (before_meta, meta) = match find_unquoted(line, ";") {
                Some(at) => (&line[..at], Some(line[at + 1..].trim().to_string())),
                None => (line, None),
            };

            let header_after_datetime = before_meta.get(datetime_end..).unwrap_or("");
            let (status, payee, narration) = parse_header_fields(header_after_datetime);
            // A status flag alone is not a transaction: Ledger.g4 `headerLine`
            // needs at least one header field after it.
            if payee.is_none() {
                diagnostics.push(diag(line_no, datetime_end, "missing transaction details"));
            }

            if meta.is_none() {
                diagnostics.push(diag(line_no, 0, "missing meta comment (expected ';')"));
            }

            // Extract source_payee / source_commodity from meta.
            // If source_payee exists, header payee is the display name and source_payee is the raw value.
            let (clean_meta, source_payee) = extract_meta_field(&meta, "source_payee");
            let (clean_meta, source_commodity) =
                extract_meta_field(&clean_meta, "source_commodity");
            let (raw_payee, display_payee) = if let Some(src) = source_payee {
                (Some(src), payee)
            } else {
                (payee, None)
            };
            current = Some((
                line_no,
                Transaction {
                    date,
                    datetime,
                    status,
                    payee: raw_payee,
                    narration,
                    meta: clean_meta,
                    postings: Vec::new(),
                    display_payee,
                    amount: 0.0,
                    amount_commodity: source_commodity.unwrap_or_default(),
                    display_amount_commodity: None,
                    fee: None,
                    fee_commodity: None,
                },
            ));
        } else {
            diagnostics.push(diag(
                line_no,
                0,
                "invalid line: expected transaction header, posting, or directive",
            ));
        }
    }

    flush_transaction(&mut current, &mut diagnostics, &mut transactions);
    if let Some((_, decl)) = current_account.take() {
        account_declarations.push(decl);
    }

    let mut balances_by_account: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    for txn in &transactions {
        for posting in &txn.postings {
            let entry = balances_by_account
                .entry(posting.account.clone())
                .or_default()
                .entry(posting.commodity.clone())
                .or_insert(0.0);
            *entry += posting.amount;
        }
    }

    let mut openings_by_account: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    for decl in &account_declarations {
        // Declared accounts with no postings still appear in `balances`.
        let _ = balances_by_account.entry(decl.account.clone()).or_default();
        if let Some(opening) = &decl.opening {
            *openings_by_account
                .entry(decl.account.clone())
                .or_default()
                .entry(opening.commodity.clone())
                .or_insert(0.0) += opening.amount;
            *balances_by_account
                .entry(decl.account.clone())
                .or_default()
                .entry(opening.commodity.clone())
                .or_insert(0.0) += opening.amount;
        }
    }

    let balances = account_balances_from_map(balances_by_account);
    let opening_balances = account_balances_from_map(openings_by_account);

    let accounts_with_opening: Vec<String> = account_declarations
        .iter()
        .filter(|d| d.opening.is_some())
        .map(|d| d.account.clone())
        .collect();

    let mut account_properties: HashMap<String, AccountProperties> = HashMap::new();
    for decl in &account_declarations {
        if decl.name.is_some() {
            account_properties.insert(
                decl.account.clone(),
                AccountProperties {
                    name: decl.name.clone(),
                },
            );
        }
    }

    ParseResult {
        ok: diagnostics.is_empty(),
        diagnostics,
        transactions,
        balances,
        opening_balances,
        accounts_with_opening,
        account_properties,
    }
}

/// Flatten an `account → commodity → amount` map into the serialisable
/// `AccountBalance` list. BTreeMap input keeps the output order deterministic.
pub fn account_balances_from_map(
    by_account: BTreeMap<String, BTreeMap<String, f64>>,
) -> Vec<AccountBalance> {
    by_account
        .into_iter()
        .map(|(account, by_commodity)| AccountBalance {
            account,
            totals: by_commodity
                .into_iter()
                .map(|(commodity, amount)| CommodityAmount { commodity, amount })
                .collect(),
        })
        .collect()
}

/// Length of the `QUOTED` token at the start of `text` (`"…"` with `\"` and
/// `\\` escapes, never spanning a line), or `None` when the quote is never
/// closed and the `"` is just a character, as the lexer sees it.
fn quoted_len(text: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in text.char_indices().skip(1) {
        match (escaped, c) {
            (true, _) => escaped = false,
            (false, '\\') => escaped = true,
            (false, '"') => return Some(i + 1),
            _ => {}
        }
    }
    None
}

/// Byte offset of the first `needle` in `text` outside a quoted string.
fn find_unquoted(text: &str, needle: &str) -> Option<usize> {
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with(needle) {
            return Some(at);
        }
        at += if rest.starts_with('"') {
            quoted_len(rest).unwrap_or(1)
        } else {
            rest.chars().next().map_or(1, char::len_utf8)
        };
    }
    None
}

/// Parse cost and price annotations from a posting's remainder text.
///
/// Cost: `{ amount commodity, field, ... }` or `{{ amount commodity, field, ... }}`
/// Price: `@ amount commodity` or `@@ amount commodity`
///
/// The third element is a diagnostic message when an annotation marker is
/// present but malformed: an unclosed or stray brace, or a price marker not
/// followed by `amount commodity`. Dropping the annotation silently would
/// leave a trade with no cost basis and nothing in the issues panel.
fn parse_remainder(
    remainder: &str,
) -> (Option<CostAnnotation>, Option<PriceAnnotation>, Option<String>) {
    // A `;` outside quotes starts a comment the annotations never reach.
    let annotations = find_unquoted(remainder, ";").map_or(remainder, |at| &remainder[..at]);

    let (cost, after_cost) = match split_cost_annotations(annotations) {
        Ok(found) => found,
        Err(message) => return (None, None, Some(message)),
    };

    // --- Price extraction ---
    // `@@` (total price) or `@` (per-unit price), outside quotes
    let Some(at) = find_unquoted(after_cost, "@") else {
        return (cost, None, None);
    };
    let (marker, after, is_total) = if after_cost[at..].starts_with("@@") {
        ("@@", after_cost[at + 2..].trim(), true)
    } else {
        ("@", after_cost[at + 1..].trim(), false)
    };
    let price = parse_price_tokens(after, is_total);
    let price_error = price.is_none().then(|| {
        format!("invalid price annotation: {marker} {after} (expected `{marker} <amount> <commodity>`)")
    });
    (cost, price, price_error)
}

/// Cuts every `{ ... }` / `{{ ... }}` cost annotation out of `text`, returning
/// the first one parsed and the text left for the price scan.
///
/// Ledger.g4 `lotCostSpec`: a cost is closed by the matching brace pair and
/// its body holds no brace outside quotes, so an unclosed opener, a nested
/// brace or a stray closer is a malformed annotation.
fn split_cost_annotations(text: &str) -> Result<(Option<CostAnnotation>, &str), String> {
    let malformed = || {
        format!(
            "invalid cost annotation: {} (expected `{{ <amount> <commodity>, ... }}` or `{{{{ ... }}}}`)",
            text.trim()
        )
    };
    let mut cost: Option<CostAnnotation> = None;
    let mut rest = text;
    while let Some(start) = find_unquoted(rest, "{") {
        let (open, close) = if rest[start..].starts_with("{{") {
            ("{{", "}}")
        } else {
            ("{", "}")
        };
        let body_start = start + open.len();
        let Some(body_len) = find_unquoted(&rest[body_start..], close) else {
            return Err(malformed());
        };
        let body = &rest[body_start..body_start + body_len];
        if find_unquoted(body, "{").is_some() || find_unquoted(body, "}").is_some() {
            return Err(malformed());
        }
        if cost.is_none() {
            cost = Some(parse_cost_body(body, open == "{{"));
        }
        rest = &rest[body_start + body_len + close.len()..];
    }
    if find_unquoted(rest, "}").is_some() {
        return Err(malformed());
    }
    Ok((cost, rest))
}

fn parse_cost_body(body: &str, is_total: bool) -> CostAnnotation {
    let parts: Vec<&str> = body.split(',').collect();
    let mut amount: Option<f64> = None;
    let mut amount_text: Option<String> = None;
    let mut commodity: Option<String> = None;
    let mut fields: Vec<String> = Vec::new();

    if let Some(first) = parts.first() {
        let tokens: Vec<&str> = first.split_whitespace().collect();
        // Try to parse leading "amount commodity"
        let leading = match tokens.as_slice() {
            [amount_token, commodity_token, ..] if is_valid_commodity(commodity_token) => {
                parse_amount(amount_token).map(|value| (value, *amount_token, *commodity_token))
            }
            _ => None,
        };
        if let Some((value, amount_token, commodity_token)) = leading {
            amount = Some(value);
            amount_text = Some(amount_token.to_string());
            commodity = Some(commodity_token.to_string());
        } else {
            // Not a recognisable amount+commodity; treat whole first part as a field
            let trimmed = first.trim();
            if !trimmed.is_empty() {
                fields.push(trimmed.to_string());
            }
        }
    }

    // Remaining comma-separated parts are raw metadata fields
    for part in parts.iter().skip(1) {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            fields.push(trimmed.to_string());
        }
    }

    CostAnnotation {
        is_total,
        amount,
        amount_text,
        commodity,
        fields,
    }
}

fn parse_price_tokens(text: &str, is_total: bool) -> Option<PriceAnnotation> {
    let mut tokens = text.split_whitespace();
    let amount_str = tokens.next()?;
    let commodity = tokens.next()?;
    let amount = parse_amount(amount_str)?;
    if !is_valid_commodity(commodity) {
        return None;
    }
    Some(PriceAnnotation {
        is_total,
        amount,
        amount_text: amount_str.to_string(),
        commodity: commodity.to_string(),
    })
}

/// Parse a `.prices` file containing market price directives.
///
/// Format (hledger-compatible; Prices.g4 is the specification):
/// ```text
/// P 2026-01-15 BTC 30000.00 USD
/// P 2026-01-15T10:05:03.123456Z BTC 30150.00 USD
/// P 2026-01-15 10:05:03 BTC 30150.00 USD
/// ; comments and blank lines allowed
/// ```
pub fn parse_prices(contents: &str) -> PricesParseResult {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut prices: Vec<PriceDirective> = Vec::new();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end();

        if line.is_empty() || line.trim_start().starts_with(';') {
            continue;
        }

        // `P` at the start of the line, followed by a blank.
        let Some(after_p) = line
            .strip_prefix('P')
            .filter(|rest| rest.starts_with([' ', '\t']))
        else {
            diagnostics.push(diag(
                line_no,
                0,
                format!("expected 'P' directive, got: {line}"),
            ));
            continue;
        };
        let mut tokens = after_p.split_whitespace().peekable();

        let Some(date_str) = tokens.next() else {
            diagnostics.push(diag(line_no, 0, "P directive missing datetime"));
            continue;
        };
        // A blank-separated time belongs to the datetime (one DATETIME token).
        let mut datetime_str = date_str.to_string();
        if date_str.len() == 10 {
            if let Some(time) = tokens.next_if(|token| TIME_RE.is_match(token)) {
                datetime_str = format!("{date_str} {time}");
            }
        }

        if !PRICE_DATETIME_RE.is_match(&datetime_str) {
            diagnostics.push(diag(
                line_no,
                0,
                format!("invalid datetime: {datetime_str}"),
            ));
            continue;
        }

        let Some(commodity) = tokens.next() else {
            diagnostics.push(diag(line_no, 0, "P directive missing commodity"));
            continue;
        };

        if !is_valid_commodity(commodity) {
            diagnostics.push(diag(line_no, 0, format!("invalid commodity: {commodity}")));
            continue;
        }

        let Some(price_amount_str) = tokens.next() else {
            diagnostics.push(diag(line_no, 0, "P directive missing price amount"));
            continue;
        };

        let Some(price_amount) = parse_amount(price_amount_str) else {
            diagnostics.push(diag(
                line_no,
                0,
                format!("invalid price amount: {price_amount_str}"),
            ));
            continue;
        };

        let Some(quote_commodity) = tokens.next() else {
            diagnostics.push(diag(line_no, 0, "P directive missing quote commodity"));
            continue;
        };

        if !is_valid_commodity(quote_commodity) {
            diagnostics.push(diag(
                line_no,
                0,
                format!("invalid quote commodity: {quote_commodity}"),
            ));
            continue;
        }

        prices.push(PriceDirective {
            datetime: datetime_str,
            commodity: commodity.to_string(),
            price_amount,
            price_amount_text: price_amount_str.to_string(),
            quote_commodity: quote_commodity.to_string(),
        });
    }

    PricesParseResult {
        ok: diagnostics.is_empty(),
        diagnostics,
        prices,
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PriceLookupResult {
    pub price_amount: f64,
    pub price_amount_text: String,
    pub quote_commodity: String,
    pub datetime: String,
}

/// Parse a CSV file containing price data.
/// Expected columns (case-insensitive): Date, Commodity, Price, Currency
pub fn parse_prices_csv(contents: &str) -> PricesParseResult {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut prices: Vec<PriceDirective> = Vec::new();

    let mut lines = contents.lines();
    let Some(header_line) = lines.next() else {
        return PricesParseResult {
            ok: false,
            diagnostics: vec![diag(1, 0, "empty CSV")],
            prices,
        };
    };

    let headers: Vec<String> = header_line
        .split(',')
        .map(|h| h.trim().to_lowercase())
        .collect();
    let date_idx = headers.iter().position(|h| h == "date");
    let commodity_idx = headers.iter().position(|h| h == "commodity");
    let price_idx = headers.iter().position(|h| h == "price");
    let currency_idx = headers.iter().position(|h| h == "currency");

    if date_idx.is_none()
        || commodity_idx.is_none()
        || price_idx.is_none()
        || currency_idx.is_none()
    {
        diagnostics.push(diag(
            1,
            0,
            "CSV header must contain Date, Commodity, Price, Currency columns",
        ));
        return PricesParseResult {
            ok: false,
            diagnostics,
            prices,
        };
    }

    let date_idx = date_idx.unwrap();
    let commodity_idx = commodity_idx.unwrap();
    let price_idx = price_idx.unwrap();
    let currency_idx = currency_idx.unwrap();

    let datetime_re = Regex::new(
        r"^\d{4}-\d{2}-\d{2}(?:T\d{2}:\d{2}:\d{2}(?:\.\d{1,6})?(?:Z|[+-]\d{2}:\d{2})?)?$",
    )
    .expect("csv price datetime regex");

    for (idx, raw_line) in lines.enumerate() {
        let line_no = idx + 2; // 1-based, skip header
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();
        let max_idx = *[date_idx, commodity_idx, price_idx, currency_idx]
            .iter()
            .max()
            .unwrap();
        if fields.len() <= max_idx {
            diagnostics.push(diag(line_no, 0, format!("not enough columns: {line}")));
            continue;
        }

        let date_str = fields[date_idx];
        let commodity = fields[commodity_idx];
        let price_str = fields[price_idx];
        let currency = fields[currency_idx];

        if !datetime_re.is_match(date_str) {
            diagnostics.push(diag(line_no, 0, format!("invalid date: {date_str}")));
            continue;
        }
        if !is_valid_commodity(commodity) {
            diagnostics.push(diag(line_no, 0, format!("invalid commodity: {commodity}")));
            continue;
        }
        let Some(price_amount) = parse_amount(price_str) else {
            diagnostics.push(diag(
                line_no,
                0,
                format!("invalid price amount: {price_str}"),
            ));
            continue;
        };
        if !is_valid_commodity(currency) {
            diagnostics.push(diag(line_no, 0, format!("invalid currency: {currency}")));
            continue;
        }

        prices.push(PriceDirective {
            datetime: date_str.to_string(),
            commodity: commodity.to_string(),
            price_amount,
            price_amount_text: price_str.to_string(),
            quote_commodity: currency.to_string(),
        });
    }

    PricesParseResult {
        ok: diagnostics.is_empty(),
        diagnostics,
        prices,
    }
}

/// Look up the closest price for a commodity at a given datetime.
/// Finds the last directive with datetime <= target, or the earliest as fallback.
pub fn lookup_price(
    sources_dir: &Path,
    commodity: &str,
    target_datetime: &str,
) -> Option<PriceLookupResult> {
    let price_file = sources_dir
        .join("_prices")
        .join(format!("{}.txt", price_file_stem(commodity)));
    let contents = std::fs::read_to_string(&price_file).ok()?;
    let result = parse_prices(&contents);
    if result.prices.is_empty() {
        return None;
    }

    let mut sorted = result.prices;
    sorted.sort_by(|a, b| a.datetime.cmp(&b.datetime));

    // Find last directive with datetime <= target
    let mut best: Option<&PriceDirective> = None;
    for p in &sorted {
        if p.datetime.as_str() <= target_datetime {
            best = Some(p);
        }
    }

    // If target is before all prices, return earliest as fallback
    let chosen = best.unwrap_or(&sorted[0]);

    Some(PriceLookupResult {
        price_amount: chosen.price_amount,
        price_amount_text: chosen.price_amount_text.clone(),
        quote_commodity: chosen.quote_commodity.clone(),
        datetime: chosen.datetime.clone(),
    })
}

/// Graph-based price converter that loads all price files once and supports
/// multi-step conversion (e.g. ETH→USD→AUD).
pub struct PriceGraph {
    /// commodity → list of (normalized-datetime-key, directive), sorted by key.
    /// Pre-normalising at construction means `lookup` does one normalise of the
    /// target and a binary search — no per-comparison allocation.
    prices: HashMap<String, Vec<(String, PriceDirective)>>,
}

impl PriceGraph {
    fn build(entries: Vec<PriceDirective>) -> Self {
        let mut prices: HashMap<String, Vec<(String, PriceDirective)>> = HashMap::new();
        for p in entries {
            let key = normalize_datetime_for_compare(&p.datetime);
            prices.entry(p.commodity.clone()).or_default().push((key, p));
        }
        for list in prices.values_mut() {
            list.sort_by(|a, b| a.0.cmp(&b.0));
        }
        PriceGraph { prices }
    }

    /// Load all `_prices/*.txt` files from the sources directory.
    pub fn load(sources_dir: &Path) -> Self {
        let mut all: Vec<PriceDirective> = Vec::new();
        let prices_dir = sources_dir.join("_prices");
        if let Ok(entries) = std::fs::read_dir(&prices_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("txt") {
                    continue;
                }
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    all.extend(parse_prices(&contents).prices);
                }
            }
        }
        Self::build(all)
    }

    /// Construct a PriceGraph from a list of price directives (for testing).
    pub fn from_entries(entries: Vec<PriceDirective>) -> Self {
        Self::build(entries)
    }

    /// Look up the nearest-prior price for a commodity at a given datetime.
    ///
    /// Returns `None` when no entry exists at or before `target_datetime`.
    /// (The previous behaviour silently fell back to the earliest entry,
    /// which fabricated wildly wrong prices when timestamp formats didn't
    /// match — e.g. ISO `T...Z` entries vs space-separated txn datetimes.)
    pub fn lookup(&self, commodity: &str, target_datetime: &str) -> Option<&PriceDirective> {
        let sorted = self.prices.get(commodity)?;
        if sorted.is_empty() {
            return None;
        }
        let target_norm = normalize_datetime_for_compare(target_datetime);
        // partition_point returns the first index where key > target_norm;
        // the entry just before it is the nearest-prior price.
        let idx = sorted.partition_point(|(k, _)| k.as_str() <= target_norm.as_str());
        if idx == 0 {
            None
        } else {
            Some(&sorted[idx - 1].1)
        }
    }

    /// Most recent price for a commodity (last entry in the date-sorted vec).
    pub fn latest(&self, commodity: &str) -> Option<&PriceDirective> {
        self.prices.get(commodity)?.last().map(|(_, p)| p)
    }

    /// Convert using the most recent prices (latest direct + latest one-hop).
    /// Mirrors `convert_to_base` semantics but without a date cutoff.
    pub fn convert_to_base_latest(
        &self,
        commodity: &str,
        amount: f64,
        base_currency: &str,
    ) -> Option<f64> {
        if commodity == base_currency {
            return Some(amount);
        }
        let p = self.latest(commodity)?;
        if p.quote_commodity == base_currency {
            return Some(amount * p.price_amount);
        }
        let p2 = self.latest(&p.quote_commodity)?;
        if p2.quote_commodity == base_currency {
            return Some(amount * p.price_amount * p2.price_amount);
        }
        None
    }

    /// Convert an amount from `commodity` to `base_currency` at `datetime`.
    ///
    /// Supports:
    /// 1. Identity (commodity == base)
    /// 2. Direct: commodity→base
    /// 3. One-hop: commodity→intermediate→base
    pub fn convert_to_base(
        &self,
        commodity: &str,
        amount: f64,
        datetime: &str,
        base_currency: &str,
    ) -> Option<f64> {
        // Identity
        if commodity == base_currency {
            return Some(amount);
        }

        // Direct: commodity→base
        if let Some(p) = self.lookup(commodity, datetime) {
            if p.quote_commodity == base_currency {
                return Some(amount * p.price_amount);
            }

            // One-hop: commodity→intermediate→base
            let intermediate = &p.quote_commodity;
            if let Some(p2) = self.lookup(intermediate, datetime) {
                if p2.quote_commodity == base_currency {
                    return Some(amount * p.price_amount * p2.price_amount);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_fixture() {
        let input = r#"2026-01-15 * \"Binance\" \"Buy SOL\" ; txn:01J2N9R9, src:binance:order:999
    assets:exchange:binance:sol    10.000000 SOL {{ 230.00 USD, fee:0.10 USD, fee_to:expenses:fees:trading, venue:binance, note:\"maker fee\" }}
    assets:cash:usd              -230.10 USD
"#;

        let result = parse_transactions(input);
        assert!(
            result.ok,
            "expected ok, got diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn parses_account_declaration_opening_with_default_commodity() {
        let input = r#"account assets:cash:bank:cba:smartaccess AUD
    opening 100.00
"#;

        let result = parse_transactions(input);
        assert!(
            result.ok,
            "expected ok, got diagnostics: {:?}",
            result.diagnostics
        );

        let balance = result
            .balances
            .iter()
            .find(|b| b.account == "assets:cash:bank:cba:smartaccess")
            .expect("expected declared account balance");
        assert_eq!(
            balance.totals,
            vec![CommodityAmount {
                commodity: "AUD".to_string(),
                amount: 100.0
            }]
        );
    }

    #[test]
    fn parses_opening_amount_with_thousands_commas() {
        let input = r#"account assets:cash:cba:bta AUD
    opening 1,823.92
"#;

        let result = parse_transactions(input);
        assert!(
            result.ok,
            "expected ok, got diagnostics: {:?}",
            result.diagnostics
        );

        let balance = result
            .balances
            .iter()
            .find(|b| b.account == "assets:cash:cba:bta")
            .expect("expected declared account balance");
        assert_eq!(
            balance.totals,
            vec![CommodityAmount {
                commodity: "AUD".to_string(),
                amount: 1823.92
            }]
        );
    }

    #[test]
    fn rejects_opening_with_unparseable_amount() {
        let input = r#"account assets:cash:cba:bta AUD
    opening 1,8x3.92
"#;

        let result = parse_transactions(input);
        assert!(!result.ok);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("invalid amount")));
        // The opening must be skipped, not silently treated as zero.
        assert!(result.accounts_with_opening.is_empty());
    }

    #[test]
    fn rejects_misgrouped_comma_amount_in_opening() {
        let input = r#"account assets:cash:cba:bta AUD
    opening 1,23.45
"#;

        let result = parse_transactions(input);
        assert!(!result.ok);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("invalid amount")));
    }

    #[test]
    fn parses_posting_amount_with_thousands_commas() {
        let input = r#"2026-01-15 * "Bank" "Deposit" ; txn:01J2N9R9
    assets:cash:cba:bta 1,000.50 AUD
"#;

        let result = parse_transactions(input);
        assert!(
            result.ok,
            "expected ok, got diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.transactions[0].postings[0].amount, 1000.50);
    }

    #[test]
    fn rejects_posting_with_unparseable_amount() {
        let input = r#"2026-01-15 * "Bank" "Deposit" ; txn:01J2N9R9
    assets:cash:cba:bta 1,0x0.50 AUD
    income:salary -1000.50 AUD
"#;

        let result = parse_transactions(input);
        assert!(!result.ok);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("invalid amount")));
        // The bad posting is skipped rather than kept with a silent 0.0 amount.
        let postings = &result.transactions[0].postings;
        assert_eq!(postings.len(), 1);
        assert_eq!(postings[0].account, "income:salary");
    }

    #[test]
    fn unparseable_price_annotation_is_a_diagnostic_not_a_silent_none() {
        // (Trailing tokens after `amount commodity` are still tolerated — that is
        // pre-existing leniency, not the silent-drop bug this guards against.)
        for remainder in ["@ notanumber USD", "@@ 100", "@@", "@ 100 BTC@USD"] {
            let input = format!(
                "2026-01-15 * \"Kraken\" \"Buy BTC\" ; txn:t1\n    assets:crypto:btc 0.5 BTC {remainder}\n"
            );
            let result = parse_transactions(&input);
            assert!(!result.ok, "{remainder:?} must not parse ok");
            assert!(
                result.diagnostics.iter().any(|d| d.line == 2
                    && d.message.contains("invalid price annotation")
                    && d.message.contains(remainder.trim_start_matches('@').trim())),
                "{remainder:?}: {:?}",
                result.diagnostics
            );
            // The posting itself is still recorded (only its price is missing).
            assert!(result.transactions[0].postings[0].price.is_none());
        }
    }

    #[test]
    fn parses_price_annotation_with_thousands_commas() {
        let input = r#"2026-01-15 * "Kraken" "Buy BTC" ; txn:01J2N9R9
    assets:crypto:btc 0.5 BTC @ 60,000.00 USD
"#;

        let result = parse_transactions(input);
        assert!(
            result.ok,
            "expected ok, got diagnostics: {:?}",
            result.diagnostics
        );
        let price = result.transactions[0].postings[0]
            .price
            .as_ref()
            .expect("price annotation");
        assert_eq!(price.amount, 60000.0);
    }

    #[test]
    fn parses_price_directive_with_thousands_commas() {
        let result = parse_prices("P 2026-01-15 BTC 60,000.00 USD\n");
        assert!(
            result.ok,
            "expected ok, got diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.prices[0].price_amount, 60000.0);
    }

    #[test]
    fn rejects_posting_without_amount() {
        let input = r#"2026-01-15 * \"Binance\" \"Buy SOL\" ; txn:01J2N9R9
    assets:exchange:binance:sol
"#;

        let result = parse_transactions(input);
        assert!(!result.ok);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("missing amount")));
    }

    #[test]
    fn take_token_preserves_unicode_in_quoted_strings() {
        // é is multi-byte UTF-8 (0xC3 0xA9)
        let (tok, rest) = take_token(r#""Café" rest"#).unwrap();
        assert_eq!(tok, "Café");
        assert_eq!(rest, " rest");
    }

    #[test]
    fn take_token_preserves_cjk_characters() {
        let (tok, rest) = take_token(r#""比特币交易" next"#).unwrap();
        assert_eq!(tok, "比特币交易");
        assert_eq!(rest, " next");
    }

    #[test]
    fn take_token_handles_escaped_char_after_unicode() {
        let (tok, _) = take_token(r#""Café \"Latte\"""#).unwrap();
        assert_eq!(tok, "Café \"Latte\"");
    }

    #[test]
    fn commodity_re_accepts_unicode_letters() {
        let re = commodity_re();
        assert!(re.is_match("比特币"));
        assert!(re.is_match("UЅDС")); // Cyrillic homoglyphs
        assert!(re.is_match("ETH")); // ASCII still works
        assert!(re.is_match("BTC_v2"));
        assert!(!re.is_match(""));
        assert!(!re.is_match(" ETH")); // leading space
        assert!(!re.is_match(".ETH")); // leading dot
    }

    #[test]
    fn commodity_re_accepts_exchange_symbol_form() {
        // EXCHANGE:SYMBOL (Google/TradingView) and Hyperliquid's `<dex>:<coin>`.
        let re = commodity_re();
        assert!(re.is_match("xyz:WTIOIL"));
        assert!(re.is_match("HYPERLIQUID:BTC"));
        assert!(re.is_match("ASX:ENA"));
        assert!(is_valid_commodity("xyz:WTIOIL"));
        // ':' only after the first character.
        assert!(!re.is_match(":ETH"));
        // Everything the ledger tokeniser, cost/meta splitters, rule alternation
        // and the price-file encoder rely on stays excluded.
        assert!(!re.is_match("BTC USDT"));
        assert!(!re.is_match("BTC@USD"));
        assert!(!re.is_match("A,B"));
        assert!(!re.is_match("A|B"));
        assert!(!re.is_match("BTC/USDT"));
        assert!(!re.is_match("A%B"));
        assert!(!re.is_match("A{B}"));
        assert!(!re.is_match("A\"B"));
    }

    #[test]
    fn price_file_stem_encodes_only_filename_unsafe_chars() {
        // ':' is NTFS ADS syntax on Windows and shown as '/' by Finder.
        assert_eq!(price_file_stem("xyz:WTIOIL"), "xyz%3AWTIOIL");
        assert_eq!(price_file_stem("HYPERLIQUID:BTC"), "HYPERLIQUID%3ABTC");
        // Existing files keep their names.
        assert_eq!(price_file_stem("BTC"), "BTC");
        assert_eq!(price_file_stem("BRK.B"), "BRK.B");
        assert_eq!(price_file_stem("RAY-SOL"), "RAY-SOL");
        assert_eq!(price_file_stem("aEthUSDC"), "aEthUSDC");
        assert_eq!(price_file_stem("比特币"), "比特币");
    }

    #[test]
    fn account_re_accepts_unicode_segments() {
        let re = account_re();
        assert!(re.is_match("资产:银行:储蓄"));
        assert!(re.is_match("assets:café"));
        assert!(re.is_match("assets:bank:savings")); // ASCII still works
        assert!(!re.is_match("single_segment")); // needs colon
    }

    #[test]
    fn parses_transaction_with_unicode_commodity() {
        let input = "2026-01-15 * \"Binance\" \"Buy\" ; txn:test1\n    assets:wallet 10.00 比特币\n    expenses:unknown -10.00 比特币\n";
        let result = parse_transactions(input);
        assert!(
            result.ok,
            "expected ok, got diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.transactions[0].postings[0].commodity, "比特币");
    }

    #[test]
    fn unicode_payee_round_trips_through_quote_and_parse() {
        use crate::generated_store::quote;
        let original = "Café Müller";
        let quoted = quote(original);
        let (parsed, _) = take_token(&quoted).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn price_graph_one_hop_conversion() {
        let graph = PriceGraph::from_entries(vec![
            PriceDirective {
                datetime: "2025-01-01".into(),
                commodity: "USDC".into(),
                price_amount: 1.0,
                price_amount_text: "1.00".into(),
                quote_commodity: "USD".into(),
            },
            PriceDirective {
                datetime: "2025-01-01".into(),
                commodity: "USD".into(),
                price_amount: 1.55,
                price_amount_text: "1.55".into(),
                quote_commodity: "AUD".into(),
            },
        ]);
        // One-hop: USDC → USD → AUD
        let val = graph.convert_to_base("USDC", 100.0, "2025-03-05T01:03:44", "AUD");
        assert!(val.is_some(), "one-hop conversion should succeed");
        assert!((val.unwrap() - 155.0).abs() < 0.01);
    }

    #[test]
    fn price_graph_returns_none_for_unknown_commodity() {
        let graph = PriceGraph::from_entries(vec![PriceDirective {
            datetime: "2025-01-01".into(),
            commodity: "USD".into(),
            price_amount: 1.55,
            price_amount_text: "1.55".into(),
            quote_commodity: "AUD".into(),
        }]);
        let val = graph.convert_to_base("CHEEMS", 100.0, "2025-03-05", "AUD");
        assert!(val.is_none(), "unknown commodity should return None");
    }

    #[test]
    fn price_graph_latest_returns_last_chronological_entry() {
        let graph = PriceGraph::from_entries(vec![
            PriceDirective {
                datetime: "2024-06-01".into(),
                commodity: "HNT".into(),
                price_amount: 4.00,
                price_amount_text: "4.00".into(),
                quote_commodity: "USD".into(),
            },
            PriceDirective {
                datetime: "2026-01-01".into(),
                commodity: "HNT".into(),
                price_amount: 6.50,
                price_amount_text: "6.50".into(),
                quote_commodity: "USD".into(),
            },
            PriceDirective {
                datetime: "2025-03-01".into(),
                commodity: "HNT".into(),
                price_amount: 5.25,
                price_amount_text: "5.25".into(),
                quote_commodity: "USD".into(),
            },
        ]);
        let latest = graph.latest("HNT").expect("HNT should be present");
        assert_eq!(latest.datetime, "2026-01-01");
        assert!((latest.price_amount - 6.50).abs() < f64::EPSILON);
        assert!(graph.latest("UNKNOWN").is_none());
    }
}
