//! Differential parity between the handwritten runtime parser and the parser
//! generated from the authoritative grammar (`Ledger.g4`, `LedgerLexer.g4`,
//! `Prices.g4` at the repository root).
//!
//! Decision-003: the handwritten parser stays the runtime (the generated one
//! is 30-75x slower on real ledgers) and the grammar is the specification.
//! The two are kept in step not by sharing code but by these tests, which
//! fail on any accept/reject disagreement:
//!
//! - every checked-in fixture under `features/fixtures/` (a stem of `invalid`
//!   or ending in `_invalid` is expected to be rejected, anything else
//!   accepted) must get the same decision from both parsers;
//! - the commodity, account and amount validators must agree with the
//!   grammar's `commodityInput` / `accountInput` / `amountInput` entry points
//!   over `features/fixtures/validator_corpus.tsv`;
//! - opt-in, every `*.transactions`, `*.prices` and `_prices/*.txt` file in a
//!   vault: `EMUACC_PARITY_VAULT=<vault root>` and run the ignored test.
//!
//! "Accept" means the handwritten parser reports no diagnostic at all and the
//! generated parser reports no syntax error. The handwritten parser also
//! raises a few semantic diagnostics the grammar cannot express (a duplicate
//! `opening` or `name`, an `opening` with neither a commodity nor an account
//! default). No fixture or vault file exercises those; if one ever does, the
//! disagreement is reported here rather than hidden.
//!
//! This module only compiles under `cfg(test)`: `antlr4rust` is a
//! dev-dependency and release builds neither compile nor link the generated
//! parser.

use super::generated::ledger::Ledger;
use super::generated::ledgerlexer::LedgerLexer;
use super::generated::prices::Prices;
use super::{
    is_valid_account, is_valid_commodity, parse_amount, parse_prices, parse_transactions,
};
use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::error_listener::ErrorListener;
use antlr4rust::errors::ANTLRError;
use antlr4rust::parser::Parser;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::InputStream;
use std::cell::RefCell;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A syntax error reported by the generated parser (1-based line, 0-based
/// column, as ANTLR reports them).
#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntaxError {
    line: usize,
    column: usize,
    message: String,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}:{}: {}", self.line, self.column, self.message)
    }
}

/// Error listener that collects what the lexer and parser report instead of
/// printing it to stderr.
struct Collect(Rc<RefCell<Vec<SyntaxError>>>);

impl<'a, T: Recognizer<'a>> ErrorListener<'a, T> for Collect {
    fn syntax_error(
        &self,
        _recognizer: &T,
        _offending_symbol: Option<&<T::TF as TokenFactory<'a>>::Inner>,
        line: isize,
        column: isize,
        message: &str,
        _error: Option<&ANTLRError>,
    ) {
        self.0.borrow_mut().push(SyntaxError {
            line: line.max(0) as usize,
            column: column.max(0) as usize,
            message: message.to_string(),
        });
    }
}

/// A grammar start rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartRule {
    LedgerFile,
    LedgerAccount,
    LedgerCommodity,
    LedgerAmount,
    PricesFile,
    PricesCommodity,
    PricesAmount,
}

impl StartRule {
    fn name(self) -> &'static str {
        match self {
            StartRule::LedgerFile => "Ledger.file",
            StartRule::LedgerAccount => "Ledger.accountInput",
            StartRule::LedgerCommodity => "Ledger.commodityInput",
            StartRule::LedgerAmount => "Ledger.amountInput",
            StartRule::PricesFile => "Prices.pricesFile",
            StartRule::PricesCommodity => "Prices.commodityInput",
            StartRule::PricesAmount => "Prices.amountInput",
        }
    }
}

/// Every syntax error the generated parser reports for `input` under `rule`.
/// The default (recovering) error strategy is used so all errors are
/// collected, not just the first.
fn grammar_errors(rule: StartRule, input: &str) -> Vec<SyntaxError> {
    let errors = Rc::new(RefCell::new(Vec::new()));
    let mut lexer = LedgerLexer::new(InputStream::new(input));
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(Collect(Rc::clone(&errors))));
    let tokens = CommonTokenStream::new(lexer);

    let outcome = match rule {
        StartRule::LedgerFile
        | StartRule::LedgerAccount
        | StartRule::LedgerCommodity
        | StartRule::LedgerAmount => {
            let mut parser = Ledger::new(tokens);
            parser.remove_error_listeners();
            parser.add_error_listener(Box::new(Collect(Rc::clone(&errors))));
            match rule {
                StartRule::LedgerFile => parser.file().map(|_| ()),
                StartRule::LedgerAccount => parser.accountInput().map(|_| ()),
                StartRule::LedgerCommodity => parser.commodityInput().map(|_| ()),
                _ => parser.amountInput().map(|_| ()),
            }
        }
        StartRule::PricesFile | StartRule::PricesCommodity | StartRule::PricesAmount => {
            let mut parser = Prices::new(tokens);
            parser.remove_error_listeners();
            parser.add_error_listener(Box::new(Collect(Rc::clone(&errors))));
            match rule {
                StartRule::PricesFile => parser.pricesFile().map(|_| ()),
                StartRule::PricesCommodity => parser.commodityInput().map(|_| ()),
                _ => parser.amountInput().map(|_| ()),
            }
        }
    };

    let mut collected = errors.borrow().clone();
    if let Err(error) = outcome {
        // An unrecoverable error aborts the parse; it normally reaches the
        // listener first, but never let a rejection go unreported.
        if collected.is_empty() {
            let (line, column) = error
                .get_offending_token()
                .map(|token| (token.line.max(0) as usize, token.column.max(0) as usize))
                .unwrap_or((0, 0));
            collected.push(SyntaxError {
                line,
                column,
                message: error.to_string(),
            });
        }
    }
    collected
}

/// Which runtime parser a file goes through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileKind {
    Transactions,
    Prices,
}

impl FileKind {
    /// `*.transactions` and `*.prices` by extension; `_prices/*.txt` is the
    /// vault's price-file layout and goes through `parse_prices` too.
    fn of(path: &Path) -> Option<Self> {
        let extension = path.extension().and_then(|e| e.to_str())?;
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str());
        match extension {
            "transactions" => Some(FileKind::Transactions),
            "prices" => Some(FileKind::Prices),
            "txt" if parent == Some("_prices") => Some(FileKind::Prices),
            _ => None,
        }
    }

    fn start_rule(self) -> StartRule {
        match self {
            FileKind::Transactions => StartRule::LedgerFile,
            FileKind::Prices => StartRule::PricesFile,
        }
    }
}

/// One parser's decision on a file, with where and why when it rejected.
struct Verdict {
    accepted: bool,
    detail: Option<String>,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.accepted {
            return f.write_str("accept");
        }
        match &self.detail {
            Some(detail) => write!(f, "reject ({detail})"),
            None => f.write_str("reject"),
        }
    }
}

/// Both parsers' decisions on the same contents.
struct Comparison {
    handwritten: Verdict,
    grammar: Verdict,
}

impl Comparison {
    fn agree(&self) -> bool {
        self.handwritten.accepted == self.grammar.accepted
    }
}

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "handwritten={} grammar={}",
            self.handwritten, self.grammar
        )
    }
}

fn compare(kind: FileKind, contents: &str) -> Comparison {
    let handwritten = match kind {
        FileKind::Transactions => {
            let result = parse_transactions(contents);
            Verdict {
                accepted: result.ok,
                detail: result
                    .diagnostics
                    .first()
                    .map(|d| format!("line {}: {}", d.line, d.message)),
            }
        }
        FileKind::Prices => {
            let result = parse_prices(contents);
            Verdict {
                accepted: result.ok,
                detail: result
                    .diagnostics
                    .first()
                    .map(|d| format!("line {}: {}", d.line, d.message)),
            }
        }
    };
    let errors = grammar_errors(kind.start_rule(), contents);
    let grammar = Verdict {
        accepted: errors.is_empty(),
        detail: errors.first().map(ToString::to_string),
    };
    Comparison {
        handwritten,
        grammar,
    }
}

fn decision_word(accepted: bool) -> &'static str {
    if accepted {
        "accept"
    } else {
        "reject"
    }
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join("fixtures")
}

/// Fixtures named `invalid.*` or `*_invalid.*` are the intentionally broken
/// ones; every other fixture is expected to parse.
fn fixture_expected_accept(path: &Path) -> bool {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("UTF-8 fixture name");
    !(stem == "invalid" || stem.ends_with("_invalid"))
}

fn ledger_fixtures() -> Vec<(PathBuf, FileKind)> {
    let mut fixtures: Vec<(PathBuf, FileKind)> = fs::read_dir(fixture_dir())
        .expect("fixture directory")
        .map(|entry| entry.expect("fixture entry").path())
        .filter_map(|path| FileKind::of(&path).map(|kind| (path, kind)))
        .collect();
    fixtures.sort_by(|a, b| a.0.cmp(&b.0));
    fixtures
}

#[test]
fn reported_syntax_errors_carry_the_offending_line() {
    let errors = grammar_errors(
        StartRule::LedgerFile,
        "2026-01-15 * \"Binance\" \"Buy SOL\" ; txn:1\n    assets:exchange:binance:sol\n",
    );
    assert!(!errors.is_empty(), "a posting without an amount is a syntax error");
    assert_eq!(errors[0].line, 2, "{errors:?}");
    assert!(
        grammar_errors(StartRule::LedgerFile, "").is_empty(),
        "an empty file is a valid ledger"
    );
}

#[test]
fn every_fixture_gets_the_same_decision_from_both_parsers() {
    let fixtures = ledger_fixtures();
    let mut failures = Vec::new();
    let mut accepted = 0;
    let mut rejected = 0;

    for (path, kind) in &fixtures {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("UTF-8 fixture name");
        let contents = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read fixture {name}: {e}"));
        let expected = fixture_expected_accept(path);
        if expected {
            accepted += 1;
        } else {
            rejected += 1;
        }
        let comparison = compare(*kind, &contents);
        if comparison.handwritten.accepted != expected || comparison.grammar.accepted != expected {
            failures.push(format!(
                "{name}: expected {}; {comparison}",
                decision_word(expected)
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "parser disagreement on {} of {} fixture(s):\n{}",
        failures.len(),
        fixtures.len(),
        failures.join("\n")
    );
    // This scans the directory, not the git index, so a shared working tree
    // may carry extra fixtures from in-flight work: tolerate additions and
    // only guard against the corpus silently shrinking. Baseline: 52 valid
    // and 4 invalid fixtures when parity was introduced.
    assert!(accepted >= 52, "expected at least 52 valid fixtures, got {accepted}");
    assert!(rejected >= 4, "expected at least 4 invalid fixtures, got {rejected}");
}

/// Which validator a corpus row exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    Commodity,
    Account,
    Amount,
}

impl InputKind {
    fn handwritten(self, input: &str) -> bool {
        match self {
            InputKind::Commodity => is_valid_commodity(input),
            InputKind::Account => is_valid_account(input),
            InputKind::Amount => parse_amount(input).is_some(),
        }
    }

    /// The grammar entry points that define this kind of input. Commodities
    /// and amounts are defined in both grammars and must agree in both.
    fn entry_points(self) -> &'static [StartRule] {
        match self {
            InputKind::Commodity => &[StartRule::LedgerCommodity, StartRule::PricesCommodity],
            InputKind::Account => &[StartRule::LedgerAccount],
            InputKind::Amount => &[StartRule::LedgerAmount, StartRule::PricesAmount],
        }
    }
}

struct CorpusRow {
    line: usize,
    kind: InputKind,
    valid: bool,
    input: String,
}

/// `features/fixtures/validator_corpus.tsv`: `kind<TAB>valid|invalid<TAB>input`.
/// The input is taken verbatim, or decoded as a JSON string when it starts
/// with `"` (for inputs that are empty or carry leading or trailing blanks,
/// which editors strip). `#` lines and empty lines are ignored.
fn validator_corpus() -> Vec<CorpusRow> {
    let path = fixture_dir().join("validator_corpus.tsv");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(index, line)| {
            let line_no = index + 1;
            let mut fields = line.splitn(3, '\t');
            let kind = match fields.next() {
                Some("commodity") => InputKind::Commodity,
                Some("account") => InputKind::Account,
                Some("amount") => InputKind::Amount,
                other => panic!("{}:{line_no}: unknown input kind {other:?}", path.display()),
            };
            let valid = match fields.next() {
                Some("valid") => true,
                Some("invalid") => false,
                other => panic!("{}:{line_no}: expected valid|invalid, got {other:?}", path.display()),
            };
            let raw = fields.next().unwrap_or_default();
            let input = if raw.starts_with('"') {
                serde_json::from_str::<String>(raw).unwrap_or_else(|e| {
                    panic!("{}:{line_no}: input is not a JSON string: {e}", path.display())
                })
            } else {
                raw.to_string()
            };
            CorpusRow {
                line: line_no,
                kind,
                valid,
                input,
            }
        })
        .collect()
}

#[test]
fn validators_agree_with_the_grammar_entry_points_over_the_corpus() {
    let corpus = validator_corpus();
    let mut failures = Vec::new();

    for row in &corpus {
        let handwritten = row.kind.handwritten(&row.input);
        let grammar: Vec<(StartRule, bool)> = row
            .kind
            .entry_points()
            .iter()
            .map(|rule| (*rule, grammar_errors(*rule, &row.input).is_empty()))
            .collect();
        let agree = handwritten == row.valid && grammar.iter().all(|(_, ok)| *ok == row.valid);
        if !agree {
            let verdicts: Vec<String> = grammar
                .iter()
                .map(|(rule, ok)| format!("{}={}", rule.name(), decision_word(*ok)))
                .collect();
            failures.push(format!(
                "line {}: {:?} {:?}: expected {}; handwritten={}; {}",
                row.line,
                row.kind,
                row.input,
                decision_word(row.valid),
                decision_word(handwritten),
                verdicts.join("; ")
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "validator/grammar disagreement on {} of {} corpus input(s):\n{}",
        failures.len(),
        corpus.len(),
        failures.join("\n")
    );
    for kind in [InputKind::Commodity, InputKind::Account, InputKind::Amount] {
        let valid = corpus.iter().filter(|r| r.kind == kind && r.valid).count();
        let invalid = corpus.iter().filter(|r| r.kind == kind && !r.valid).count();
        assert!(
            valid >= 10 && invalid >= 10,
            "{kind:?} corpus is too thin: {valid} valid, {invalid} invalid"
        );
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.depth() > 0
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with('.'))
}

/// Opt-in: every ledger and price file in a real vault through both parsers.
///
/// ```text
/// EMUACC_PARITY_VAULT=~/vault \
///   cargo test --manifest-path src-tauri/Cargo.toml --lib \
///   grammar_parity::whole_vault -- --ignored --nocapture
/// ```
///
/// Reports every disagreement with the file's path, the first diagnostic
/// (line and message) from each parser, and both decisions.
#[test]
#[ignore = "opt-in: set EMUACC_PARITY_VAULT=<vault root> and run with --ignored"]
fn whole_vault_gets_the_same_decision_from_both_parsers() {
    let root = std::env::var_os("EMUACC_PARITY_VAULT")
        .map(PathBuf::from)
        .expect("EMUACC_PARITY_VAULT must name the vault root to compare");
    let mut files = 0;
    let mut disagreements = Vec::new();

    let walk = walkdir::WalkDir::new(&root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| !is_hidden(entry));
    for entry in walk {
        let entry = entry.unwrap_or_else(|e| panic!("walk {}: {e}", root.display()));
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(kind) = FileKind::of(entry.path()) else {
            continue;
        };
        let shown = entry.path().strip_prefix(&root).unwrap_or(entry.path());
        let contents = match fs::read_to_string(entry.path()) {
            Ok(contents) => contents,
            Err(e) => {
                disagreements.push(format!("{}: unreadable ({e})", shown.display()));
                continue;
            }
        };
        files += 1;
        let comparison = compare(kind, &contents);
        if !comparison.agree() {
            disagreements.push(format!("{}: {comparison}", shown.display()));
        }
    }

    eprintln!("compared {files} ledger file(s) under {}", root.display());
    assert!(
        files > 0,
        "no *.transactions, *.prices or _prices/*.txt files under {}",
        root.display()
    );
    assert!(
        disagreements.is_empty(),
        "parser disagreement on {} of {files} file(s) under {}:\n{}",
        disagreements.len(),
        root.display(),
        disagreements.join("\n")
    );
}
