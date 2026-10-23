//! AI-assisted suggestion engines: per-transaction categorisation (and, with
//! Phase 4, Rhai transform generation). Pure of Tauri types — the commands in
//! `main.rs` load rules/ledger data, build a `*Context`, and forward step
//! strings to the frontend events; everything here is testable from BDD/unit
//! tests against a stub OpenAI-compatible server.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::ai_client::{AiClient, ChatMessage, ToolDef};
use crate::ledger_parser::Transaction;
use crate::rules::Rule;

/// Rules shown inline in the categorisation prompt; the rest are reachable
/// via the `search_rules` tool. Keeps the prompt bounded on rule-heavy folders.
pub const MAX_RULES_IN_PROMPT: usize = 30;
const TOOL_DEFAULT_LIMIT: usize = 20;
const TOOL_MAX_LIMIT: usize = 50;
const MATCHING_TXNS_CAP: usize = 20;
const NEARBY_TXNS_CAP: usize = 10;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiRuleSuggestion {
    pub pattern: String,
    pub amount_account: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payee: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_field: Option<String>,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AiSuggestResponse {
    pub suggestions: Vec<AiRuleSuggestion>,
    pub raw_output: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Everything the categorisation engine needs, pre-loaded by the caller.
pub struct CategorisationContext {
    pub account_name: String,
    /// Pretty JSON of the target transaction (date/datetime/payee/narration/
    /// amount/commodity/txn_id), as sent by the frontend sparkle button.
    pub uncategorised_json: String,
    pub rules: Vec<Rule>,
    /// Active-ledger transactions used for context sections and the
    /// `find_similar_transactions` / `list_categories` tools.
    pub transactions: Vec<Transaction>,
    /// The user's name/context from settings; empty = no context line.
    pub user_context: String,
}

/// The prompt sentence describing the user, or "" when no context is set.
/// Shared by the in-app categorisation prompt and the emuacc-classify
/// template ({user_context} placeholder).
pub fn user_context_section(user_context: &str) -> String {
    let user_context = user_context.trim();
    if user_context.is_empty() {
        return String::new();
    }
    format!(
        "About the user: {user_context}. Narrations or payees mentioning the user's \
         own name or initials usually indicate transfers between their own accounts."
    )
}

/// Extract a JSON array from text that may contain markdown code fences.
pub fn extract_json_array(text: &str) -> Option<&str> {
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim());
        }
    }
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        // Skip optional language tag on same line
        let after = if let Some(nl) = after.find('\n') {
            &after[nl + 1..]
        } else {
            after
        };
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim());
        }
    }
    // Try to find bare [ ... ] anywhere in the text
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if end > start {
                return Some(text[start..=end].trim());
            }
        }
    }
    None
}

/// One-line rendering of a transaction with its resolved category — the
/// non-asset/liability posting account.
pub fn fmt_txn(t: &Transaction) -> String {
    let payee = t
        .display_payee
        .as_deref()
        .or(t.payee.as_deref())
        .unwrap_or("—");
    let narration = t.narration.as_deref().unwrap_or("—");
    let category = t
        .postings
        .iter()
        .find(|p| !p.account.starts_with("assets:") && !p.account.starts_with("liabilities:"))
        .map(|p| p.account.as_str())
        .unwrap_or("—");
    format!(
        "  {} | {} | {} | {} {} → {}",
        t.date, payee, narration, t.amount, t.amount_commodity, category
    )
}

fn rule_summary_line(r: &Rule) -> String {
    let account = r.amount_account.as_deref().unwrap_or("—");
    let mut extras = Vec::new();
    if let Some(p) = r.payee.as_deref() {
        extras.push(format!("payee: {p}"));
    }
    if let Some(m) = r.match_field.as_deref() {
        extras.push(format!("match: {m}"));
    }
    let extras = if extras.is_empty() {
        String::new()
    } else {
        format!(" ({})", extras.join(", "))
    };
    format!("  \"{}\" → {}{}", r.pattern, account, extras)
}

/// Bounded one-line-per-rule summary. Rules beyond `max` are collapsed into a
/// "call search_rules" pointer so the prompt stays small on rule-heavy folders.
pub fn summarise_rules(rules: &[Rule], max: usize) -> String {
    if rules.is_empty() {
        return "  (no rules yet)".to_string();
    }
    let mut lines: Vec<String> = rules.iter().take(max).map(rule_summary_line).collect();
    if rules.len() > max {
        lines.push(format!(
            "  …and {} more rules — call search_rules to see others",
            rules.len() - max
        ));
    }
    lines.join("\n")
}

fn rule_json(r: &Rule) -> serde_json::Value {
    serde_json::json!({
        "pattern": r.pattern,
        "amount_account": r.amount_account,
        "payee": r.payee,
        "match_field": r.match_field,
        "comment": r.comment,
    })
}

fn tool_limit(args: &serde_json::Value) -> usize {
    args["limit"]
        .as_u64()
        .map(|n| (n as usize).min(TOOL_MAX_LIMIT))
        .unwrap_or(TOOL_DEFAULT_LIMIT)
}

/// The three inspection tools shared by categorisation and the batch
/// classifier: search_rules, find_similar_transactions, list_categories.
pub fn categorisation_tools(rules: Arc<Vec<Rule>>, txns: Arc<Vec<Transaction>>) -> Vec<ToolDef> {
    let mut tools = Vec::new();

    {
        let rules = rules.clone();
        tools.push(ToolDef {
            name: "search_rules".to_string(),
            description: "Search this account's existing categorisation rules. \
                Case-insensitive substring match against pattern, payee and target \
                account; an empty query lists the first rules."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "substring to match; empty lists the first rules"},
                    "limit": {"type": "integer", "description": "max rules to return (default 20, max 50)"}
                },
                "required": ["query"]
            }),
            handler: Box::new(move |args| {
                let query = args["query"].as_str().unwrap_or("").to_lowercase();
                let limit = tool_limit(args);
                let matched: Vec<serde_json::Value> = rules
                    .iter()
                    .filter(|r| {
                        query.is_empty()
                            || r.pattern.to_lowercase().contains(&query)
                            || r.payee
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query)
                            || r.amount_account
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query)
                    })
                    .take(limit)
                    .map(rule_json)
                    .collect();
                Ok(serde_json::json!(matched).to_string())
            }),
        });
    }

    {
        let txns = txns.clone();
        tools.push(ToolDef {
            name: "find_similar_transactions".to_string(),
            description: "Find transactions whose payee or narration contains the \
                query (case-insensitive), showing how they were categorised."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "substring of payee or narration"},
                    "limit": {"type": "integer", "description": "max transactions to return (default 20, max 50)"}
                },
                "required": ["query"]
            }),
            handler: Box::new(move |args| {
                let query = args["query"].as_str().unwrap_or("").to_lowercase();
                let limit = tool_limit(args);
                let lines: Vec<String> = txns
                    .iter()
                    .filter(|t| {
                        let payee = t
                            .display_payee
                            .as_deref()
                            .or(t.payee.as_deref())
                            .unwrap_or("")
                            .to_lowercase();
                        let narration = t.narration.as_deref().unwrap_or("").to_lowercase();
                        payee.contains(&query) || narration.contains(&query)
                    })
                    .take(limit)
                    .map(fmt_txn)
                    .collect();
                if lines.is_empty() {
                    Ok("no matching transactions".to_string())
                } else {
                    Ok(lines.join("\n"))
                }
            }),
        });
    }

    {
        tools.push(ToolDef {
            name: "list_categories".to_string(),
            description: "List the category accounts already in use in this ledger \
                (from transactions and rules). Prefer these over inventing new names."
                .to_string(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
            handler: Box::new(move |_args| {
                let mut categories = BTreeSet::new();
                for t in txns.iter() {
                    for p in &t.postings {
                        if !p.account.starts_with("assets:")
                            && !p.account.starts_with("liabilities:")
                        {
                            categories.insert(p.account.clone());
                        }
                    }
                }
                for r in rules.iter() {
                    if let Some(a) = r.amount_account.as_deref() {
                        categories.insert(a.to_string());
                    }
                }
                if categories.is_empty() {
                    Ok("no categories in use yet".to_string())
                } else {
                    Ok(categories.into_iter().collect::<Vec<_>>().join("\n"))
                }
            }),
        });
    }

    tools
}

/// Select the context transactions for the prompt: same payee/narration
/// (capped) and within ±60s of the target (capped), excluding the target.
fn select_context_txns(
    target: &serde_json::Value,
    txns: &[Transaction],
) -> (Vec<Transaction>, Vec<Transaction>) {
    let target_payee = target["payee"].as_str().unwrap_or("").to_lowercase();
    let target_narration = target["narration"].as_str().unwrap_or("").to_lowercase();
    let target_datetime = target["datetime"].as_str().unwrap_or("");
    let target_ts = chrono::NaiveDateTime::parse_from_str(target_datetime, "%Y-%m-%d %H:%M:%S").ok();

    let mut matching_txns = Vec::new();
    let mut nearby_txns = Vec::new();
    for txn in txns {
        // Skip the target transaction itself
        if txn.datetime == target_datetime
            && txn.narration.as_deref().map(|n| n.to_lowercase()) == Some(target_narration.clone())
            && (txn.display_payee.as_deref().or(txn.payee.as_deref())).map(|p| p.to_lowercase())
                == Some(target_payee.clone())
        {
            continue;
        }

        let txn_payee = txn
            .display_payee
            .as_deref()
            .or(txn.payee.as_deref())
            .unwrap_or("")
            .to_lowercase();
        let txn_narration = txn.narration.as_deref().unwrap_or("").to_lowercase();

        let payee_match = !target_payee.is_empty() && txn_payee == target_payee;
        let narration_match = !target_narration.is_empty() && txn_narration == target_narration;
        if payee_match || narration_match {
            matching_txns.push(txn.clone());
        }

        if let Some(ref t_ts) = target_ts {
            if let Ok(txn_ts) =
                chrono::NaiveDateTime::parse_from_str(&txn.datetime, "%Y-%m-%d %H:%M:%S")
            {
                let diff = (txn_ts - *t_ts).num_seconds().unsigned_abs();
                if diff <= 60 && diff > 0 && !(payee_match || narration_match) {
                    nearby_txns.push(txn.clone());
                }
            }
        }
    }
    matching_txns.truncate(MATCHING_TXNS_CAP);
    nearby_txns.truncate(NEARBY_TXNS_CAP);
    (matching_txns, nearby_txns)
}

/// Build the categorisation prompt: target txn + bounded rule summary +
/// matching/nearby context + the tools note + the JSON output contract.
pub fn categorisation_prompt(ctx: &CategorisationContext) -> String {
    let target: serde_json::Value = serde_json::from_str(&ctx.uncategorised_json).unwrap_or_default();
    let (matching_txns, nearby_txns) = select_context_txns(&target, &ctx.transactions);

    let matching_section = if matching_txns.is_empty() {
        String::new()
    } else {
        let lines: Vec<String> = matching_txns.iter().map(fmt_txn).collect();
        format!("\n\nTransactions with the same payee or narration (showing how they were categorised):\n{}", lines.join("\n"))
    };
    let nearby_section = if nearby_txns.is_empty() {
        String::new()
    } else {
        let lines: Vec<String> = nearby_txns.iter().map(fmt_txn).collect();
        format!("\n\nOther transactions within 1 minute of this one (may be part of the same action):\n{}", lines.join("\n"))
    };
    let rules_summary = summarise_rules(&ctx.rules, MAX_RULES_IN_PROMPT);
    let account_name = &ctx.account_name;
    let uncategorised_json = &ctx.uncategorised_json;
    let user_section = match user_context_section(&ctx.user_context) {
        s if s.is_empty() => String::new(),
        s => format!("\n\n{s}"),
    };

    format!(
        r#"You are analyzing a single financial transaction for the account "{account_name}" in a personal finance app called EmuAcc.

This transaction is currently uncategorised (expenses:unknown):

{uncategorised_json}{user_section}

Existing rules for this account (summary):
{rules_summary}{matching_section}{nearby_section}

The app uses a rules system where wildcard patterns match transaction narration/payee text and assign categories (accounts like "expenses:groceries", "expenses:subscriptions", "income:salary", "assets:transfer", etc.).

You can call tools to inspect this account: search_rules (find existing rules), find_similar_transactions (see how similar transactions were categorised), and list_categories (category accounts already in use). Check list_categories before inventing a new account name.

If this is a crypto wallet, particularly look at transactions within a minute of each other on the same account as they are often a group of transactions for a single action.

Please:
1. Use the existing rules and similar transactions above for context
2. Identify what this transaction likely represents based on the payee/narration
3. Suggest 1-2 categorisation rules. Return ONLY a JSON array, no other text. Each rule object must have:
   - "pattern": wildcard pattern to match (e.g. "*UBER*", "*Netflix*")
   - "amount_account": the category account (e.g. "expenses:transport", "expenses:subscriptions")
   - "payee": optional human-readable payee name (e.g. "Uber", "Netflix")
   - "explanation": brief reason for the categorization

Return the JSON array directly with no markdown formatting or code fences."#
    )
}

// ---------------------------------------------------------------------------
// Transform generation
// ---------------------------------------------------------------------------

/// How many times to regenerate a transform script that fails the compile+run
/// check, feeding the error back into the prompt. This outer loop is the
/// backstop for models that ignore the `validate_transform` tool.
pub const TRANSFORM_MAX_ATTEMPTS: usize = 4;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CsvTypeInfo {
    pub pattern: String,
    pub headers: Vec<String>,
    pub file_count: usize,
    pub sample_rows: Vec<Vec<String>>,
    /// First file of this type, for the `get_sample_rows` tool re-read.
    /// Not serialized — the frontend response shape is unchanged.
    #[serde(skip)]
    pub first_file: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AiTransformResponse {
    pub script: String,
    pub raw_output: String,
    pub success: bool,
    pub error: Option<String>,
    pub csv_types: Vec<CsvTypeInfo>,
}

/// Everything the transform engine needs, pre-loaded by the caller.
pub struct TransformContext {
    pub currency: String,
    pub csv_types: Vec<CsvTypeInfo>,
    /// Numbered example `_transform.rhai` scripts (from
    /// [`collect_transform_examples`]); empty when none exist.
    pub examples: String,
}

/// Build the transform-generation prompt. `compile_error` carries the previous
/// attempt's validation failure on retries.
pub fn build_ai_transform_prompt(
    currency: &str,
    branching_hint: &str,
    csv_sections: &str,
    examples: &str,
    compile_error: Option<&str>,
) -> String {
    let retry_section = match compile_error {
        Some(err) => format!(
            "\n\nIMPORTANT: Your previous script failed to compile with this error:\n  {err}\n\
             Fix the error and return a corrected script.\n"
        ),
        None => String::new(),
    };

    let examples_section = if examples.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nHere are working examples of Rhai transform scripts from this project. \
                 Study these carefully — they show correct Rhai syntax and patterns:\n{examples}\n"
        )
    };

    format!(
        r#"You are generating a Rhai transform script for a personal finance app called EmuAcc.

The script maps CSV columns to ledger transaction fields. It receives each CSV row as `row["ColumnName"]` and must return a Rhai map literal (#{{ ... }}).

CRITICAL Rhai language rules — trim() and replace() MUTATE in place and return ():
- NEVER chain trim(): `row["x"].trim().split(" ")` FAILS because trim() returns ().
  Correct: `let s = row["x"]; s.trim(); let parts = s.split(" ");`
- NEVER chain replace(): same issue. `s.replace(",","")` mutates s, returns ().
- split(delimiter) returns an array. Index with `parts[0]`, `parts[1]`.
- sub_string(start, len) extracts a substring (does NOT mutate).
- String concatenation: `"a" + "b"`.
- `if` is an expression: `let x = if cond {{ a }} else {{ b }};`
- Map literal: `#{{ key: value, key2: value2 }}`
- `contains(substring)` checks if a string contains another.
- Define a clean() helper to strip currency symbols:
  `fn clean(s) {{ s.replace("$", ""); s.replace("+", ""); s.replace(",", ""); s }}`
  Note: each replace() mutates s in place. The final bare `s` returns the cleaned string.
- For non-ISO dates, define a parse helper in the script. Do NOT use external libraries.
- SELF-CONTAINED: define every helper you call (write `fn clean(s) {{ ... }}` before using clean()). NEVER reference a variable or call a function you have not defined — a bare word like `narr` or `contra` compiles but crashes at runtime with "Variable not found".

Required output fields:
- date: ISO date string (YYYY-MM-DD)
- payee: who the transaction is with (or "" if not available)
- narration: transaction description
- amount: numeric amount (use clean() to strip $, +, commas)
- commodity: "{currency}"
- status: "*"

Optional fields (include a line ONLY if you assign a concrete string literal or a variable you defined — otherwise omit it entirely; never write `contra: contra`):
- txn_id: "" (leave empty for auto-generation)
- contra: contra account (e.g. "expenses:unknown", "income:sales"). Default is expenses:unknown.
- fee: fee amount if the CSV has a fee column
- datetime: full ISO datetime if available (e.g. "2024-01-15 10:30:00")

Available injected variables:
- row["_source_path"]: the filename of the current CSV (for branching on CSV type)

The account is auto-derived from the folder path — do NOT include an account field.

You can call tools: get_sample_rows (more rows of a CSV type when the samples below are ambiguous) and validate_transform (compile + run your script against a real row of every CSV type). Call validate_transform on your script and fix any error it reports BEFORE returning your final answer.
{examples_section}{branching_hint}{retry_section}
CSV data to analyze:
{csv_sections}

Return the script as a single ```rhai code block — any helper functions (clean, date parsers, `let` bindings) FIRST, then the #{{ ... }} map that uses them. Put nothing outside the code block."#
    )
}

/// Collect up to `max` existing `_transform.rhai` files from `sources_dir`,
/// formatted as numbered examples for the AI prompt.
pub fn collect_transform_examples(sources_dir: &std::path::Path, max: usize) -> String {
    let mut examples = Vec::new();
    let mut walker = vec![sources_dir.to_path_buf()];
    while let Some(dir) = walker.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walker.push(path);
            } else if path
                .file_name()
                .map(|n| n == "_transform.rhai")
                .unwrap_or(false)
            {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Skip very large scripts to avoid prompt bloat
                    if content.lines().count() <= 50 {
                        let rel = path
                            .strip_prefix(sources_dir)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();
                        examples.push((rel, content));
                    }
                }
            }
        }
        if examples.len() >= max * 2 {
            break; // enough candidates
        }
    }
    // Take up to max, preferring shorter ones first (simpler examples)
    examples.sort_by_key(|(_, c)| c.len());
    examples.truncate(max);
    examples
        .iter()
        .enumerate()
        .map(|(i, (path, content))| format!("\n--- Example {} ({}) ---\n{}", i + 1, path, content))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The per-type CSV sections and the multi-format branching hint.
pub fn transform_sections(csv_types: &[CsvTypeInfo]) -> (String, String) {
    let mut csv_sections = String::new();
    for (i, ct) in csv_types.iter().enumerate() {
        csv_sections.push_str(&format!(
            "\n--- CSV Type {} (e.g. \"{}\", {} file{}) ---\nHeaders: {}\nSample rows:\n",
            i + 1,
            ct.pattern,
            ct.file_count,
            if ct.file_count == 1 { "" } else { "s" },
            ct.headers.join(", "),
        ));
        for row in &ct.sample_rows {
            csv_sections.push_str(&format!("  {}\n", row.join(", ")));
        }
    }
    let branching_hint = if csv_types.len() > 1 {
        format!(
            "\n\nIMPORTANT: There are {} different CSV formats in this account folder. \
             Generate a SINGLE script that branches on `row[\"_source_path\"]` using `.contains(\"keyword\")` \
             to detect which CSV type is being processed. Each branch should return the appropriate field mapping.\n",
            csv_types.len()
        )
    } else {
        String::new()
    };
    (csv_sections, branching_hint)
}

/// Compile + run a script against the first sample row of every CSV type — a
/// compile-only check lets undefined-variable bugs reach the pipeline, and
/// checking every type exercises each branch of a multi-format script.
fn validate_transform_script(script: &str, csv_types: &[CsvTypeInfo]) -> Result<(), String> {
    crate::transform_suggest::rhai_compile_check(script)?;
    for ct in csv_types {
        if let Some(row) = ct.sample_rows.first() {
            crate::transform_suggest::rhai_run_check(script, &ct.headers, row, &ct.pattern)?;
        }
    }
    Ok(())
}

/// The transform-generation tools: get_sample_rows + validate_transform.
pub fn transform_tools(csv_types: Arc<Vec<CsvTypeInfo>>) -> Vec<ToolDef> {
    let mut tools = Vec::new();

    {
        let csv_types = csv_types.clone();
        tools.push(ToolDef {
            name: "get_sample_rows".to_string(),
            description: "Fetch more sample rows of one CSV type when the rows in \
                the prompt are ambiguous. csv_type is the 1-based type number shown \
                in the prompt."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "csv_type": {"type": "integer", "description": "1-based CSV type number"},
                    "count": {"type": "integer", "description": "rows to return (default 10, max 50)"}
                },
                "required": ["csv_type"]
            }),
            handler: Box::new(move |args| {
                let index = args["csv_type"].as_u64().unwrap_or(0) as usize;
                let count = args["count"]
                    .as_u64()
                    .map(|n| (n as usize).min(TOOL_MAX_LIMIT))
                    .unwrap_or(10);
                let ct = csv_types
                    .get(index.wrapping_sub(1))
                    .ok_or_else(|| format!("no CSV type {index}; there are {}", csv_types.len()))?;
                let path = ct
                    .first_file
                    .as_ref()
                    .ok_or_else(|| "sample file not available for this CSV type".to_string())?;
                let (_headers, rows) = crate::transform_suggest::read_csv_sample(path, count)?;
                let lines: Vec<String> = rows.iter().map(|r| r.join(", ")).collect();
                Ok(lines.join("\n"))
            }),
        });
    }

    {
        let csv_types = csv_types.clone();
        tools.push(ToolDef {
            name: "validate_transform".to_string(),
            description: "Compile the given Rhai transform script and run it \
                against a real sample row of every CSV type. Returns OK or the \
                exact error to fix. Call this before returning your final answer."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "script": {"type": "string", "description": "the complete Rhai transform script"}
                },
                "required": ["script"]
            }),
            handler: Box::new(move |args| {
                let script = args["script"]
                    .as_str()
                    .ok_or_else(|| "missing script argument".to_string())?;
                match validate_transform_script(script, &csv_types) {
                    Ok(()) => Ok(format!(
                        "OK — compiles and runs against all {} CSV type(s)",
                        csv_types.len()
                    )),
                    Err(e) => Ok(format!("VALIDATION FAILED: {e}")),
                }
            }),
        });
    }

    tools
}

/// Run the transform engine: generate → extract → compile+run validate →
/// retry with the error fed back, up to [`TRANSFORM_MAX_ATTEMPTS`]. Exhaustion
/// returns the last script with `success: true` plus an error note so the user
/// can hand-fix it. A client-level error (connect/HTTP/...) aborts immediately.
pub fn run_transform_generation(
    client: &AiClient,
    ctx: TransformContext,
    on_step: &(dyn Fn(&str) + Send + Sync),
) -> AiTransformResponse {
    let (csv_sections, branching_hint) = transform_sections(&ctx.csv_types);
    let csv_types = Arc::new(ctx.csv_types);
    let tools = transform_tools(csv_types.clone());

    let mut last_err = String::new();
    let mut last_script = String::new();
    let mut last_raw = String::new();

    for attempt in 0..TRANSFORM_MAX_ATTEMPTS {
        let prompt = build_ai_transform_prompt(
            &ctx.currency,
            &branching_hint,
            &csv_sections,
            &ctx.examples,
            if attempt == 0 {
                None
            } else {
                Some(last_err.as_str())
            },
        );
        if attempt == 0 {
            on_step("Calling AI model…");
        } else {
            on_step(&format!(
                "Retry {}/{} — fixing: {last_err}",
                attempt,
                TRANSFORM_MAX_ATTEMPTS - 1
            ));
        }

        let raw = match client.run(vec![ChatMessage::user(prompt)], &tools, on_step) {
            Ok(s) => s,
            Err(e) => {
                return AiTransformResponse {
                    script: String::new(),
                    raw_output: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                    csv_types: csv_types.as_ref().clone(),
                }
            }
        };

        on_step("Parsing response…");
        let script = crate::transform_suggest::extract_rhai_block(&raw);
        match validate_transform_script(&script, &csv_types) {
            Ok(()) => {
                on_step("Script compiles & runs ✓");
                return AiTransformResponse {
                    script,
                    raw_output: raw,
                    success: true,
                    error: None,
                    csv_types: csv_types.as_ref().clone(),
                };
            }
            Err(err) => {
                on_step(&format!("Attempt {} failed: {err}", attempt + 1));
                last_err = err;
                last_script = script;
                last_raw = raw;
            }
        }
    }

    // No clean script after all attempts — return the last one (with the
    // error) so the user can fix it manually.
    AiTransformResponse {
        script: last_script,
        raw_output: last_raw,
        success: true,
        error: Some(format!(
            "Script still has errors after {TRANSFORM_MAX_ATTEMPTS} attempts: {last_err}"
        )),
        csv_types: csv_types.as_ref().clone(),
    }
}

fn parse_suggestions(raw: &str) -> Result<Vec<AiRuleSuggestion>, String> {
    let json_str = extract_json_array(raw).unwrap_or(raw);
    serde_json::from_str::<Vec<AiRuleSuggestion>>(json_str).map_err(|e| e.to_string())
}

/// Run the categorisation engine: prompt + tools through the client's tool
/// loop, parse the JSON-array answer, with one retry when the reply is not
/// parseable. A still-unparseable retry returns `success: false`.
pub fn run_categorisation(
    client: &AiClient,
    ctx: CategorisationContext,
    on_step: &(dyn Fn(&str) + Send + Sync),
) -> AiSuggestResponse {
    let prompt = categorisation_prompt(&ctx);
    on_step(&format!("Prompt:\n{prompt}"));
    let tools = categorisation_tools(Arc::new(ctx.rules), Arc::new(ctx.transactions));

    on_step("Calling AI model…");
    let first = match client.run(vec![ChatMessage::user(prompt.clone())], &tools, on_step) {
        Ok(raw) => raw,
        Err(e) => {
            return AiSuggestResponse {
                suggestions: vec![],
                raw_output: String::new(),
                success: false,
                error: Some(e.to_string()),
            }
        }
    };

    on_step("Parsing response…");
    let parse_err = match parse_suggestions(&first) {
        Ok(suggestions) => {
            return AiSuggestResponse {
                suggestions,
                raw_output: first,
                success: true,
                error: None,
            }
        }
        Err(e) => e,
    };

    on_step("Reply was not a parseable JSON array; asking the model to fix it…");
    let retry_messages = vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant(first.clone()),
        ChatMessage::user(format!(
            "Your previous reply was not a parseable JSON array ({parse_err}). \
             Return ONLY the JSON array, no other text."
        )),
    ];
    match client.run(retry_messages, &tools, on_step) {
        Ok(raw2) => match parse_suggestions(&raw2) {
            Ok(suggestions) => AiSuggestResponse {
                suggestions,
                raw_output: raw2,
                success: true,
                error: None,
            },
            Err(e2) => AiSuggestResponse {
                suggestions: vec![],
                raw_output: raw2,
                success: false,
                error: Some(format!("Could not parse suggestions: {e2}")),
            },
        },
        Err(e) => AiSuggestResponse {
            suggestions: vec![],
            raw_output: first,
            success: false,
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(pattern: &str, account: &str) -> Rule {
        serde_json::from_value(serde_json::json!({
            "id": format!("test-{pattern}"),
            "pattern": pattern,
            "amount_account": account,
        }))
        .expect("rule from json")
    }

    #[test]
    fn summarise_rules_bounds_output() {
        let rules: Vec<Rule> = (0..500)
            .map(|i| rule(&format!("*P{i}*"), &format!("expenses:c{i}")))
            .collect();
        let summary = summarise_rules(&rules, MAX_RULES_IN_PROMPT);
        assert_eq!(summary.lines().count(), MAX_RULES_IN_PROMPT + 1);
        assert!(summary.contains("470 more rules"));
        assert!(summary.contains("search_rules"));
    }

    #[test]
    fn summarise_rules_small_list_has_no_pointer() {
        let rules = vec![rule("*A*", "expenses:a"), rule("*B*", "expenses:b")];
        let summary = summarise_rules(&rules, MAX_RULES_IN_PROMPT);
        assert!(summary.contains("\"*A*\" → expenses:a"));
        assert!(!summary.contains("more rules"));
    }

    #[test]
    fn extract_json_array_variants() {
        assert_eq!(extract_json_array("```json\n[1]\n```"), Some("[1]"));
        assert_eq!(extract_json_array("```\n[1]\n```"), Some("[1]"));
        assert_eq!(extract_json_array("text [1, 2] trailing"), Some("[1, 2]"));
        assert_eq!(extract_json_array("no array"), None);
    }

    #[test]
    fn user_context_section_in_prompt() {
        let mut ctx = CategorisationContext {
            account_name: "assets:test".to_string(),
            uncategorised_json: "{}".to_string(),
            rules: Vec::new(),
            transactions: Vec::new(),
            user_context: "Alex Example (initials AE)".to_string(),
        };
        let prompt = categorisation_prompt(&ctx);
        assert!(prompt.contains("Alex Example (initials AE)"));
        assert!(prompt.contains("own name or initials"));

        ctx.user_context = String::new();
        let prompt = categorisation_prompt(&ctx);
        assert!(!prompt.contains("About the user"));
    }

    #[test]
    fn search_rules_tool_filters_case_insensitively() {
        let rules = Arc::new(vec![rule("*UBER*", "expenses:transport"), rule("*NETFLIX*", "expenses:subscriptions")]);
        let tools = categorisation_tools(rules, Arc::new(Vec::new()));
        let search = tools.iter().find(|t| t.name == "search_rules").unwrap();
        let out = (search.handler)(&serde_json::json!({"query": "uber"})).unwrap();
        assert!(out.contains("*UBER*"));
        assert!(!out.contains("*NETFLIX*"));
    }

    #[test]
    fn list_categories_tool_includes_rule_accounts() {
        let rules = Arc::new(vec![rule("*A*", "expenses:groceries")]);
        let tools = categorisation_tools(rules, Arc::new(Vec::new()));
        let list = tools.iter().find(|t| t.name == "list_categories").unwrap();
        let out = (list.handler)(&serde_json::json!({})).unwrap();
        assert!(out.contains("expenses:groceries"));
    }
}
