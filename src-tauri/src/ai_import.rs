//! Universal AI document import: extraction and validation core.
//!
//! Any document (text-layer PDF, plain text) is turned into text here, then an
//! AI engine (Phase 2) interprets it into a normalised CSV for the target
//! account folder. Everything the model produces is re-validated in Rust —
//! row shapes, dates, amounts, and (when the document carries a running
//! balance) the CBA-plugin-proven balance-chain check that rejects a whole
//! extraction when any amount disagrees with the balance delta.

use std::path::Path;
use std::sync::Arc;

use crate::ai_client::{AiClient, ChatMessage, ToolDef};

/// Below this many extracted characters a PDF is assumed to be scanned images.
const MIN_PDF_TEXT_CHARS: usize = 50;
/// Document text inlined into the prompt; the rest is reachable via the
/// `get_document_text` tool.
pub const DOC_PROMPT_TEXT_CHARS: usize = 24_000;
/// Slice size served per `get_document_text` call.
pub const DOC_TOOL_PAGE_CHARS: usize = 20_000;
/// Extraction attempts, feeding row-precise validation errors back.
pub const DOC_IMPORT_MAX_ATTEMPTS: usize = 3;
/// Cap on document text carried around in memory/prompts.
const MAX_DOC_TEXT_BYTES: usize = 2_000_000;
/// Balance-chain tolerance, matching the retired cba-statement-pdfs plugin.
pub const BALANCE_TOLERANCE: f64 = 0.005;
/// Distinct existing CSV shapes shown to the model as the folder's format.
const EXISTING_SHAPES_MAX: usize = 3;
/// Currency assumed for the prompt when the caller has none.
const DEFAULT_CURRENCY: &str = "USD";

/// Extract plain text from a document. PDFs go through `pdf-extract`
/// (text layer only — scanned PDFs are rejected with a clear error); other
/// files are read as UTF-8 text. Never panics: malformed PDFs are caught.
pub fn extract_document_text(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "pdf" => {
            let path_buf = path.to_path_buf();
            // pdf-extract can panic on malformed files; contain it.
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pdf_extract::extract_text(&path_buf)
            }));
            let text = match outcome {
                Ok(Ok(t)) => t,
                Ok(Err(e)) => return Err(format!("could not read PDF text: {e}")),
                Err(_) => {
                    return Err(
                        "could not read PDF (malformed file crashed the parser)".to_string()
                    )
                }
            };
            if text.trim().chars().count() < MIN_PDF_TEXT_CHARS {
                return Err(
                    "PDF appears to be scanned images (no text layer) — not supported yet"
                        .to_string(),
                );
            }
            Ok(cap_text(text))
        }
        "xlsx" | "xls" | "docx" | "doc" => Err(format!(
            ".{ext} files are not supported yet — export to CSV or PDF first"
        )),
        _ => {
            let bytes =
                std::fs::read(path).map_err(|e| format!("could not read file: {e}"))?;
            if bytes.contains(&0) {
                return Err("file appears to be binary — not a text document".to_string());
            }
            let text = String::from_utf8(bytes)
                .map_err(|_| "file is not valid UTF-8 text".to_string())?;
            Ok(cap_text(text))
        }
    }
}

fn cap_text(text: String) -> String {
    if text.len() <= MAX_DOC_TEXT_BYTES {
        return text;
    }
    let mut end = MAX_DOC_TEXT_BYTES;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n…[document truncated at {MAX_DOC_TEXT_BYTES} bytes]",
        &text[..end]
    )
}

/// Parse a money cell: strips `$`, commas, `+` and letters (CR/DR markers);
/// `(…)` means negative. None when nothing numeric remains.
pub fn parse_money(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let neg_paren = t.starts_with('(') && t.ends_with(')');
    let cleaned: String = t
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    if cleaned.is_empty() || cleaned == "-" || cleaned == "." {
        return None;
    }
    let v: f64 = cleaned.parse().ok()?;
    Some(if neg_paren { -v.abs() } else { v })
}

/// Accepts ISO (via `parse_date_to_iso`) plus the statement formats the model
/// may be told to reproduce when matching an existing folder shape
/// (`21 Mar 2018`, `21 March 2018`, `21/03/2018`).
fn date_parses(s: &str) -> bool {
    let s = s.trim();
    if crate::parse_date_to_iso(s).is_ok() {
        return true;
    }
    for fmt in ["%d %b %Y", "%d %B %Y", "%d/%m/%Y"] {
        if chrono::NaiveDate::parse_from_str(s, fmt).is_ok() {
            return true;
        }
    }
    false
}

fn find_col(headers: &[String], needle: &str) -> Option<usize> {
    headers
        .iter()
        .position(|h| h.to_lowercase().contains(needle))
}

/// Validate extracted rows. `Ok(warnings)` when structurally sound (warnings
/// cover unverifiable balance chains); `Err` carries row-precise errors the
/// retry prompt feeds back to the model.
pub fn validate_rows(headers: &[String], rows: &[Vec<String>]) -> Result<Vec<String>, String> {
    if headers.is_empty() {
        return Err("headers must not be empty".to_string());
    }
    if rows.is_empty() {
        return Err("no rows were extracted".to_string());
    }
    let mut errors: Vec<String> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        if row.len() != headers.len() {
            errors.push(format!(
                "row {} has {} cells but there are {} headers",
                i + 1,
                row.len(),
                headers.len()
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }

    let Some(date_col) = find_col(headers, "date") else {
        return Err("headers have no date column".to_string());
    };
    let amount_col = find_col(headers, "amount");
    let debit_col = find_col(headers, "debit");
    let credit_col = find_col(headers, "credit");
    let balance_col = find_col(headers, "balance");
    if amount_col.is_none() && (debit_col.is_none() || credit_col.is_none()) {
        return Err("headers need an amount column (or debit and credit columns)".to_string());
    }

    for (i, row) in rows.iter().enumerate() {
        let n = i + 1;
        if !date_parses(&row[date_col]) {
            errors.push(format!("row {n}: unparseable date '{}'", row[date_col]));
        }
        match amount_col {
            Some(ac) => {
                if parse_money(&row[ac]).is_none() {
                    errors.push(format!("row {n}: unparseable amount '{}'", row[ac]));
                }
            }
            None => {
                let d = debit_col.and_then(|c| parse_money(&row[c]));
                let c = credit_col.and_then(|c| parse_money(&row[c]));
                if d.is_none() && c.is_none() {
                    errors.push(format!("row {n}: neither debit nor credit is numeric"));
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }

    let mut warnings: Vec<String> = Vec::new();
    match (balance_col, amount_col) {
        (Some(bc), Some(ac)) => {
            let balances: Vec<Option<f64>> = rows.iter().map(|r| parse_money(&r[bc])).collect();
            if balances.iter().all(|b| b.is_some()) {
                for i in 1..rows.len() {
                    let prev = balances[i - 1].unwrap();
                    let bal = balances[i].unwrap();
                    let amt = parse_money(&rows[i][ac]).unwrap();
                    let expected = prev + amt;
                    if (bal - expected).abs() > BALANCE_TOLERANCE {
                        errors.push(format!(
                            "row {}: balance chain break — previous balance {prev} + amount {amt} = {expected}, but balance shows {bal}",
                            i + 1
                        ));
                    }
                }
                if !errors.is_empty() {
                    return Err(errors.join("; "));
                }
            } else {
                warnings.push(
                    "balance column is not fully populated — balance chain not verified"
                        .to_string(),
                );
            }
        }
        (Some(_), None) => warnings.push(
            "balance chain not verified (debit/credit columns instead of a signed amount)"
                .to_string(),
        ),
        (None, _) => warnings
            .push("no running balance to cross-check — review rows carefully".to_string()),
    }
    Ok(warnings)
}

/// Reject filenames the pipeline would mishandle: path separators, `..`,
/// leading `_`/`.` (silently skipped by the scanners), or a non-.csv suffix.
pub fn sanitize_csv_filename(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("filename is empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("filename must not contain path separators or '..'".to_string());
    }
    if name.starts_with('_') || name.starts_with('.') {
        return Err(
            "filename must not start with '_' or '.' (the pipeline ignores such files)"
                .to_string(),
        );
    }
    if !name.to_lowercase().ends_with(".csv") {
        return Err("filename must end with .csv".to_string());
    }
    Ok(name.to_string())
}

/// An existing CSV's shape in the target folder — the model must reproduce
/// the same headers and value formats so the folder's transform keeps working.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExistingCsvShape {
    pub filename: String,
    pub headers: Vec<String>,
    pub sample_rows: Vec<Vec<String>>,
}

/// Everything the document-import engine needs, pre-loaded by the caller.
pub struct DocumentImportContext {
    pub account_name: String,
    pub currency: String,
    /// From settings (`ai_user_name`); empty = no context line.
    pub user_context: String,
    pub document_name: String,
    /// Full extracted text; the prompt inlines only the first
    /// [`DOC_PROMPT_TEXT_CHARS`] chars.
    pub document_text: String,
    pub existing_shapes: Vec<ExistingCsvShape>,
    /// The folder's resolved `_transform.rhai`, when one exists — extracted
    /// rows are run through it so the confirm step is guaranteed modal-free.
    pub transform_script: Option<String>,
    /// Facts about the target folder the user must see alongside a
    /// successful extraction (see [`other_sources_warning`]).
    pub folder_warnings: Vec<String>,
}

impl DocumentImportContext {
    /// Assemble the context for one document bound for `account_folder`:
    /// extract the document text, read the folder's existing CSV shapes, find
    /// the transform that would apply and note folder-level warnings. The
    /// app's Import button and the `emuacc-ai-import` CLI both start here.
    pub fn load(
        sources_dir: &Path,
        account_folder: &str,
        account_name: String,
        currency: Option<String>,
        user_context: String,
        doc_path: &Path,
    ) -> Result<Self, String> {
        let document_text = extract_document_text(doc_path)?;
        let folder_dir = sources_dir.join(account_folder);
        Ok(Self {
            account_name,
            currency: currency.unwrap_or_else(|| DEFAULT_CURRENCY.to_string()),
            user_context,
            document_name: doc_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| doc_path.display().to_string()),
            document_text,
            existing_shapes: load_existing_shapes(&folder_dir, EXISTING_SHAPES_MAX),
            transform_script: folder_transform_script(sources_dir, account_folder),
            folder_warnings: other_sources_warning(&folder_dir).into_iter().collect(),
        })
    }
}

/// AI-imported rows are never de-duplicated against the CSV/OFX files a
/// folder already holds, so a folder with any says so as a warning.
pub fn other_sources_warning(folder_dir: &Path) -> Option<String> {
    let has_other_sources = std::fs::read_dir(folder_dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                matches!(
                    e.path().extension().and_then(|x| x.to_str()),
                    Some("csv") | Some("ofx")
                )
            })
        })
        .unwrap_or(false);
    has_other_sources.then(|| {
        "this account already has CSV/OFX data — AI-imported rows are not \
         de-duplicated; check for overlapping date ranges"
            .to_string()
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentImportExtraction {
    pub csv_filename: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    #[serde(default)]
    pub doc_kind: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct DocumentImportResponse {
    pub extraction: Option<DocumentImportExtraction>,
    pub raw_output: String,
    pub success: bool,
    pub error: Option<String>,
    /// Rust-side validation warnings (unverified balance chain, etc.).
    pub validation_warnings: Vec<String>,
    /// The document this result belongs to. A cancelled import's thread keeps
    /// running; the frontend matches this against the open draft so a late
    /// result can never bleed into a different document's preview.
    pub source_path: String,
}

/// Run a script against extracted rows in memory — compile once, eval per
/// row with the live pipeline's row shape (mirrors
/// `transform_suggest::rhai_run_check` without re-compiling per row).
pub fn transform_accepts_rows(
    script: &str,
    headers: &[String],
    rows: &[Vec<String>],
    source_name: &str,
) -> Result<(), String> {
    let engine = crate::csv_transform::new_transform_engine();
    let ast = engine.compile(script).map_err(|e| e.to_string())?;
    for (i, row) in rows.iter().enumerate() {
        let row_map =
            crate::csv_transform::build_row_map(headers, row, i, source_name, "assets:sample");
        let mut scope = rhai::Scope::new();
        scope.push("row", row_map);
        engine
            .eval_ast_with_scope::<rhai::Map>(&mut scope, &ast)
            .map_err(|e| format!("row {}: the folder's transform rejected it: {e}", i + 1))?;
    }
    Ok(())
}

fn char_slice(text: &str, offset: usize, len: usize) -> &str {
    let Some((start, _)) = text.char_indices().nth(offset) else {
        return "";
    };
    let end = text
        .char_indices()
        .nth(offset + len)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    &text[start..end]
}

fn render_shapes(shapes: &[ExistingCsvShape]) -> String {
    shapes
        .iter()
        .map(|s| {
            let rows = s
                .sample_rows
                .iter()
                .map(|r| format!("  {}", r.join(", ")))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "--- {} ---\nHeaders: {}\nSample rows:\n{}",
                s.filename,
                s.headers.join(","),
                rows
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build the document-import prompt: right-sized context inline, the rest
/// reachable via tools.
pub fn document_import_prompt(ctx: &DocumentImportContext) -> String {
    let user_section = match crate::ai_suggest::user_context_section(&ctx.user_context) {
        s if s.is_empty() => String::new(),
        s => format!("\n\n{s}"),
    };
    let shape_section = if ctx.existing_shapes.is_empty() {
        format!(
            "Output format: headers `Date,Payee,Description,Amount,Balance`. Omit the \
             Balance column if the document carries no running balance; omit Payee if \
             not meaningful. Dates ISO YYYY-MM-DD. Amounts are signed plain decimals \
             in {} (negative = money out) with no currency symbols or thousands \
             separators.",
            ctx.currency
        )
    } else {
        format!(
            "This account folder's existing CSVs use the shapes below. Your output \
             MUST reproduce exactly the same headers and the same value formats \
             (dates, signs, separators) so the folder's import transform keeps \
             working:\n{}",
            render_shapes(&ctx.existing_shapes)
        )
    };
    let total_chars = ctx.document_text.chars().count();
    let doc_slice = char_slice(&ctx.document_text, 0, DOC_PROMPT_TEXT_CHARS);
    let continuation = if total_chars > DOC_PROMPT_TEXT_CHARS {
        format!(
            "\n…(document continues — {total_chars} chars total; call get_document_text \
             with an offset to read more)"
        )
    } else {
        String::new()
    };
    let account_name = &ctx.account_name;
    let document_name = &ctx.document_name;

    format!(
        r#"You are extracting financial transactions from a document into a normalised CSV for the account "{account_name}" in a personal finance app called EmuAcc.{user_section}

Extract EVERY transaction that belongs to this account. If the document covers multiple accounts, extract only this account's rows and add a warning naming the others. Never invent rows; if you cannot read part of the document, say so in a warning rather than guessing.

{shape_section}

Name the CSV like `<account>-<kind>-<first-date>--<last-date>.csv` (e.g. "cdia-pdf-2018-03-03--2018-09-02.csv"); it must end in .csv and must not start with '_' or '.'.

You can call tools: get_document_text (read more of a long document), list_existing_files (this folder's existing CSV shapes with more sample rows), and validate_rows (check your extracted rows — dates, amounts, running-balance chain). Call validate_rows on your rows and fix any error it reports BEFORE returning your final answer.

Return ONLY a JSON object, no other text:
{{"csv_filename": "...", "headers": [...], "rows": [["...", ...], ...], "doc_kind": "bank statement|dividend statement|...", "warnings": ["..."]}}
Rows are arrays of strings in document order.

Document "{document_name}":
{doc_slice}{continuation}"#
    )
}

/// The document-import tools: page through the text, inspect folder shapes,
/// and pre-validate rows with the exact checker the engine applies afterward.
pub fn document_import_tools(ctx: Arc<DocumentImportContext>) -> Vec<ToolDef> {
    let mut tools = Vec::new();

    {
        let ctx = ctx.clone();
        tools.push(ToolDef {
            name: "get_document_text".to_string(),
            description: "Read a slice of the document text starting at a character \
                offset. Use to page through documents longer than the excerpt in \
                the prompt."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "offset": {"type": "integer", "description": "0-based character offset"}
                },
                "required": ["offset"]
            }),
            handler: Box::new(move |args| {
                let offset = args["offset"].as_u64().unwrap_or(0) as usize;
                let total = ctx.document_text.chars().count();
                if offset >= total {
                    return Ok(format!("(past end of document — {total} chars total)"));
                }
                let slice = char_slice(&ctx.document_text, offset, DOC_TOOL_PAGE_CHARS);
                Ok(format!(
                    "(chars {offset}..{} of {total})\n{slice}",
                    (offset + DOC_TOOL_PAGE_CHARS).min(total)
                ))
            }),
        });
    }

    {
        let ctx = ctx.clone();
        tools.push(ToolDef {
            name: "list_existing_files".to_string(),
            description: "List this account folder's existing CSV files with their \
                headers and sample rows — match their shape exactly."
                .to_string(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
            handler: Box::new(move |_args| {
                if ctx.existing_shapes.is_empty() {
                    Ok("no existing CSV files in this folder".to_string())
                } else {
                    Ok(render_shapes(&ctx.existing_shapes))
                }
            }),
        });
    }

    {
        tools.push(ToolDef {
            name: "validate_rows".to_string(),
            description: "Validate extracted rows: header/row shapes, dates, \
                amounts, and the running-balance chain. Returns OK or the exact \
                errors to fix. Call this before returning your final answer."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "headers": {"type": "array", "items": {"type": "string"}},
                    "rows": {"type": "array", "items": {"type": "array", "items": {"type": "string"}}}
                },
                "required": ["headers", "rows"]
            }),
            handler: Box::new(move |args| {
                let headers: Vec<String> =
                    serde_json::from_value(args["headers"].clone()).map_err(|e| e.to_string())?;
                let rows: Vec<Vec<String>> =
                    serde_json::from_value(args["rows"].clone()).map_err(|e| e.to_string())?;
                match validate_rows(&headers, &rows) {
                    Ok(warnings) if warnings.is_empty() => Ok("OK".to_string()),
                    Ok(warnings) => Ok(format!("OK with warnings: {}", warnings.join("; "))),
                    Err(e) => Ok(format!("VALIDATION FAILED: {e}")),
                }
            }),
        });
    }

    tools
}

fn parse_and_validate(
    raw: &str,
    transform_script: Option<&str>,
    document_name: &str,
) -> Result<(DocumentImportExtraction, Vec<String>), String> {
    let json = extract_json_object(raw).unwrap_or(raw);
    let extraction: DocumentImportExtraction = serde_json::from_str(json)
        .map_err(|e| format!("reply is not a parseable JSON object: {e}"))?;
    sanitize_csv_filename(&extraction.csv_filename)?;
    let warnings = validate_rows(&extraction.headers, &extraction.rows)?;
    if let Some(script) = transform_script {
        transform_accepts_rows(script, &extraction.headers, &extraction.rows, document_name)?;
    }
    Ok((extraction, warnings))
}

/// Run the document-import engine: prompt + tools through the client's tool
/// loop, then Rust-side validation regardless of whether the model used the
/// validate_rows tool; row-precise errors are fed back for up to
/// [`DOC_IMPORT_MAX_ATTEMPTS`] attempts.
pub fn run_document_import(
    client: &AiClient,
    ctx: DocumentImportContext,
    on_step: &(dyn Fn(&str) + Send + Sync),
) -> DocumentImportResponse {
    let prompt = document_import_prompt(&ctx);
    let transform_script = ctx.transform_script.clone();
    let document_name = ctx.document_name.clone();
    let ctx = Arc::new(ctx);
    let tools = document_import_tools(ctx.clone());

    let mut messages = vec![ChatMessage::user(prompt)];
    let mut last_raw = String::new();
    let mut last_err = String::new();

    for attempt in 0..DOC_IMPORT_MAX_ATTEMPTS {
        if attempt == 0 {
            on_step("Calling AI model…");
        } else {
            on_step(&format!(
                "Retry {}/{} — fixing: {last_err}",
                attempt,
                DOC_IMPORT_MAX_ATTEMPTS - 1
            ));
        }
        let raw = match client.run(messages.clone(), &tools, on_step) {
            Ok(r) => r,
            Err(e) => {
                return DocumentImportResponse {
                    extraction: None,
                    raw_output: last_raw,
                    success: false,
                    error: Some(e.to_string()),
                    validation_warnings: Vec::new(),
                    source_path: String::new(),
                }
            }
        };
        on_step("Validating extraction…");
        match parse_and_validate(&raw, transform_script.as_deref(), &document_name) {
            Ok((extraction, mut validation_warnings)) => {
                on_step(&format!(
                    "Extraction valid ✓ — {} rows",
                    extraction.rows.len()
                ));
                validation_warnings.extend(ctx.folder_warnings.iter().cloned());
                return DocumentImportResponse {
                    extraction: Some(extraction),
                    raw_output: raw,
                    success: true,
                    error: None,
                    validation_warnings,
                    source_path: String::new(),
                };
            }
            Err(e) => {
                messages.push(ChatMessage::assistant(raw.clone()));
                messages.push(ChatMessage::user(format!(
                    "Your reply failed validation: {e}. Return ONLY the corrected JSON object."
                )));
                last_raw = raw;
                last_err = e;
            }
        }
    }

    DocumentImportResponse {
        extraction: None,
        raw_output: last_raw,
        success: false,
        error: Some(format!(
            "extraction still invalid after {DOC_IMPORT_MAX_ATTEMPTS} attempts: {last_err}"
        )),
        validation_warnings: Vec::new(),
        source_path: String::new(),
    }
}

/// Load up to `max_shapes` distinct CSV shapes (by header signature) from an
/// account folder — the guidance the model uses to match existing formats.
pub fn load_existing_shapes(folder_dir: &Path, max_shapes: usize) -> Vec<ExistingCsvShape> {
    let Ok(entries) = std::fs::read_dir(folder_dir) else {
        return Vec::new();
    };
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map(|e| e == "csv").unwrap_or(false)
                && !p
                    .file_name()
                    .map(|n| n.to_string_lossy().starts_with('_'))
                    .unwrap_or(true)
        })
        .collect();
    paths.sort();
    let mut shapes: Vec<ExistingCsvShape> = Vec::new();
    for p in paths {
        if shapes.len() >= max_shapes {
            break;
        }
        if let Ok((headers, sample_rows)) = crate::transform_suggest::read_csv_sample(&p, 3) {
            if shapes.iter().any(|s| s.headers == headers) {
                continue;
            }
            shapes.push(ExistingCsvShape {
                filename: p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                headers,
                sample_rows,
            });
        }
    }
    shapes
}

/// The `_transform.rhai` that would apply to a CSV in `account_folder` —
/// the folder's own, else the nearest ancestor's up to (and including) the
/// sources root. Mirrors `suggest_transform_for_import`'s ancestor walk.
pub fn folder_transform_script(sources_dir: &Path, account_folder: &str) -> Option<String> {
    let mut dir = sources_dir.join(account_folder);
    loop {
        if let Ok(script) = std::fs::read_to_string(dir.join("_transform.rhai")) {
            return Some(script);
        }
        if dir == *sources_dir {
            return None;
        }
        match dir.parent() {
            Some(p) if p.starts_with(sources_dir) => dir = p.to_path_buf(),
            _ => return None,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct StagedAiImport {
    pub csv_path: std::path::PathBuf,
    pub original_copied: bool,
    pub warnings: Vec<String>,
}

/// Stage a confirmed extraction: re-validate server-side (never trust the
/// frontend payload), write the CSV into `imports/` (overwriting staging
/// leftovers is fine — it makes cancel/retry idempotent), refuse when the
/// FINAL `sources/<folder>/<name>` already exists, and file the original
/// document under `originals/` (inert — the pipeline only drains csv/ofx).
pub fn stage_ai_import(
    sources_dir: &Path,
    account_folder: &str,
    csv_filename: &str,
    headers: &[String],
    rows: &[Vec<String>],
    original_path: &Path,
) -> Result<StagedAiImport, String> {
    crate::processing_pipeline::validate_folder_name(account_folder)?;
    let csv_filename = sanitize_csv_filename(csv_filename)?;
    let mut warnings = validate_rows(headers, rows)?;

    let folder_dir = sources_dir.join(account_folder);
    let final_path = folder_dir.join(&csv_filename);
    if final_path.exists() {
        return Err(format!(
            "{csv_filename} already exists in this account folder — already imported?"
        ));
    }

    let imports_dir = folder_dir.join("imports");
    std::fs::create_dir_all(&imports_dir)
        .map_err(|e| format!("failed to create imports dir: {e}"))?;
    let csv_path = imports_dir.join(&csv_filename);
    let mut writer = csv::Writer::from_path(&csv_path)
        .map_err(|e| format!("failed to write staged CSV: {e}"))?;
    writer
        .write_record(headers)
        .map_err(|e| format!("failed to write CSV headers: {e}"))?;
    for row in rows {
        writer
            .write_record(row)
            .map_err(|e| format!("failed to write CSV row: {e}"))?;
    }
    writer
        .flush()
        .map_err(|e| format!("failed to flush staged CSV: {e}"))?;

    let mut original_copied = false;
    if let Some(name) = original_path.file_name() {
        let originals_dir = folder_dir.join("originals");
        std::fs::create_dir_all(&originals_dir)
            .map_err(|e| format!("failed to create originals dir: {e}"))?;
        let dest = originals_dir.join(name);
        if dest.exists() {
            warnings.push(format!(
                "original {} already exists in originals/ — not overwritten",
                name.to_string_lossy()
            ));
        } else {
            std::fs::copy(original_path, &dest)
                .map_err(|e| format!("failed to copy original document: {e}"))?;
            original_copied = true;
        }
    }

    Ok(StagedAiImport {
        csv_path,
        original_copied,
        warnings,
    })
}

/// Extract a JSON object from text that may contain markdown code fences —
/// the `{…}` analogue of [`crate::ai_suggest::extract_json_array`].
pub fn extract_json_object(text: &str) -> Option<&str> {
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim());
        }
    }
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        let after = if let Some(nl) = after.find('\n') {
            &after[nl + 1..]
        } else {
            after
        };
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim());
        }
    }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return Some(text[start..=end].trim());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_pdf() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("features/fixtures/minimal-text.pdf")
    }

    fn h(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn r(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn extracts_text_from_pdf_fixture() {
        let text = extract_document_text(&fixture_pdf()).expect("pdf text");
        assert!(text.contains("DIVIDEND PAYMENT"), "got: {text}");
        assert!(text.contains("123.45"), "got: {text}");
    }

    #[test]
    fn reads_plain_text_files() {
        let path = std::env::temp_dir().join("emuacc-ai-import-test.txt");
        std::fs::write(&path, "Dividend statement\n2026-01-02 BHP 123.45\n").unwrap();
        let text = extract_document_text(&path).expect("txt");
        assert!(text.contains("BHP"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn rejects_binary_and_office_files() {
        let path = std::env::temp_dir().join("emuacc-ai-import-test.bin");
        std::fs::write(&path, b"abc\x00def").unwrap();
        let err = extract_document_text(&path).unwrap_err();
        assert!(err.contains("binary"), "got: {err}");
        std::fs::remove_file(&path).ok();

        let err = extract_document_text(Path::new("statement.xlsx")).unwrap_err();
        assert!(err.contains("not supported yet"), "got: {err}");
    }

    #[test]
    fn parse_money_variants() {
        assert_eq!(parse_money("$1,234.56"), Some(1234.56));
        assert_eq!(parse_money("-15.00"), Some(-15.0));
        assert_eq!(parse_money("(42.50)"), Some(-42.5));
        assert_eq!(parse_money("$35,803.66CR"), Some(35803.66));
        assert_eq!(parse_money(""), None);
        assert_eq!(parse_money("n/a"), None);
    }

    #[test]
    fn validate_rows_happy_path_with_chain() {
        let headers = h(&["Date", "Description", "Amount", "Balance"]);
        let rows = vec![
            r(&["2026-01-01", "OPENING", "0.00", "1000.00"]),
            r(&["2026-01-02", "DIVIDEND", "123.45", "1123.45"]),
            r(&["2026-01-03", "FEE", "-23.45", "1100.00"]),
        ];
        let warnings = validate_rows(&headers, &rows).expect("valid");
        assert!(warnings.is_empty(), "got: {warnings:?}");
    }

    #[test]
    fn validate_rows_detects_chain_break() {
        let headers = h(&["Date", "Description", "Amount", "Balance"]);
        let rows = vec![
            r(&["2026-01-01", "OPENING", "0.00", "1000.00"]),
            r(&["2026-01-02", "DIVIDEND", "123.45", "1999.99"]),
        ];
        let err = validate_rows(&headers, &rows).unwrap_err();
        assert!(err.contains("row 2"), "got: {err}");
        assert!(err.contains("balance chain break"), "got: {err}");
    }

    #[test]
    fn validate_rows_statement_date_format_accepted() {
        let headers = h(&["Date", "Transaction details", "Amount", "Balance"]);
        let rows = vec![
            r(&["21 Mar 2018", "OPENING", "0.00", "500.00"]),
            r(&["22 Mar 2018", "TRANSFER", "-100.00", "400.00"]),
        ];
        assert!(validate_rows(&headers, &rows).is_ok());
    }

    #[test]
    fn validate_rows_errors_are_row_precise() {
        let headers = h(&["Date", "Description", "Amount"]);
        let rows = vec![
            r(&["2026-01-01", "OK", "10.00"]),
            r(&["not a date", "BAD", "x"]),
        ];
        let err = validate_rows(&headers, &rows).unwrap_err();
        assert!(err.contains("row 2: unparseable date"), "got: {err}");
        assert!(err.contains("row 2: unparseable amount"), "got: {err}");
    }

    #[test]
    fn validate_rows_ragged_and_missing_columns() {
        let headers = h(&["Date", "Amount"]);
        let err = validate_rows(&headers, &[r(&["2026-01-01"])]).unwrap_err();
        assert!(err.contains("1 cells"), "got: {err}");

        let err = validate_rows(&h(&["Description"]), &[r(&["x"])]).unwrap_err();
        assert!(err.contains("no date column"), "got: {err}");

        let err = validate_rows(&h(&["Date"]), &[r(&["2026-01-01"])]).unwrap_err();
        assert!(err.contains("amount column"), "got: {err}");
    }

    #[test]
    fn validate_rows_warns_without_balance() {
        let headers = h(&["Date", "Description", "Amount"]);
        let rows = vec![r(&["2026-01-01", "DIVIDEND", "123.45"])];
        let warnings = validate_rows(&headers, &rows).expect("valid");
        assert!(warnings.iter().any(|w| w.contains("no running balance")));
    }

    #[test]
    fn validate_rows_warns_on_partial_balance() {
        let headers = h(&["Date", "Description", "Amount", "Balance"]);
        let rows = vec![
            r(&["2026-01-01", "A", "10.00", "100.00"]),
            r(&["2026-01-02", "B", "5.00", ""]),
        ];
        let warnings = validate_rows(&headers, &rows).expect("valid");
        assert!(warnings.iter().any(|w| w.contains("not fully populated")));
    }

    #[test]
    fn sanitize_csv_filename_rules() {
        assert!(sanitize_csv_filename("cdia-pdf-2026-01-01--2026-06-30.csv").is_ok());
        assert!(sanitize_csv_filename("a/b.csv").is_err());
        assert!(sanitize_csv_filename("..\\evil.csv").is_err());
        assert!(sanitize_csv_filename("_hidden.csv").is_err());
        assert!(sanitize_csv_filename(".dot.csv").is_err());
        assert!(sanitize_csv_filename("statement.pdf").is_err());
        assert!(sanitize_csv_filename("  ").is_err());
    }

    #[test]
    fn extract_json_object_variants() {
        assert_eq!(extract_json_object("```json\n{\"a\":1}\n```"), Some("{\"a\":1}"));
        assert_eq!(extract_json_object("```\n{\"a\":1}\n```"), Some("{\"a\":1}"));
        assert_eq!(extract_json_object("text {\"a\":1} more"), Some("{\"a\":1}"));
        assert_eq!(extract_json_object("no object"), None);
    }

    #[test]
    fn load_context_reads_document_shapes_transform_and_folder_warnings() {
        let sources = tempfile::tempdir().expect("tempdir");
        let folder = sources.path().join("alex/bank/cdia");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join("2025-01.csv"),
            "Date,Description,Amount\n2025-01-02,X,1.00\n",
        )
        .unwrap();
        std::fs::write(
            sources.path().join("alex/_transform.rhai"),
            "#{ date: row[\"Date\"] }",
        )
        .unwrap();

        let ctx = DocumentImportContext::load(
            sources.path(),
            "alex/bank/cdia",
            "assets:bank:cdia".to_string(),
            None,
            String::new(),
            &fixture_pdf(),
        )
        .expect("context");
        assert_eq!(ctx.currency, DEFAULT_CURRENCY);
        assert_eq!(ctx.document_name, "minimal-text.pdf");
        assert!(ctx.document_text.contains("DIVIDEND PAYMENT"));
        assert_eq!(ctx.existing_shapes.len(), 1);
        assert_eq!(
            ctx.existing_shapes[0].headers,
            h(&["Date", "Description", "Amount"])
        );
        assert!(ctx.transform_script.is_some(), "ancestor transform applies");
        assert_eq!(ctx.folder_warnings.len(), 1, "folder already holds a CSV");
        assert!(ctx.folder_warnings[0].contains("not de-duplicated"));
    }

    #[test]
    fn load_context_for_an_empty_folder_has_no_warnings_or_transform() {
        let sources = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(sources.path().join("bank")).unwrap();
        let ctx = DocumentImportContext::load(
            sources.path(),
            "bank",
            "assets:bank".to_string(),
            Some("AUD".to_string()),
            String::new(),
            &fixture_pdf(),
        )
        .expect("context");
        assert_eq!(ctx.currency, "AUD");
        assert!(ctx.existing_shapes.is_empty());
        assert!(ctx.transform_script.is_none());
        assert!(ctx.folder_warnings.is_empty());
        assert_eq!(other_sources_warning(&sources.path().join("missing")), None);
    }
}
