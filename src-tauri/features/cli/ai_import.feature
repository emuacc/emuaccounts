Feature: emuacc-ai-import CLI interprets a PDF or text statement into an account folder

  The app's Import button hands PDF and text statements to the AI, which
  returns a normalised CSV that Rust re-validates (dates, amounts, the
  running-balance chain and the folder's transform) before a preview-and-
  confirm step stages it through imports/ and files the original under
  originals/. `emuacc-ai-import` is the same engine from the command line.
  The vault resolves like the other CLIs (EMUACC_SOURCES_DIR /
  EMUACC_GENERATED_DIR env overrides, else the app config's current_root)
  and the AI endpoint comes from the app's Global Settings. The extracted
  CSV is printed on stdout and everything else goes to stderr; nothing is
  written unless --confirm is passed, and --confirm needs a transform already
  in place for the folder — the app is where a folder's first transform gets
  written.

  Background:
    Given a vault root with a "alex/bank/cdia" account folder
    And an app config whose AI endpoint is a stub that extracts:
      | Date       | Description      | Amount | Balance |
      | 2026-01-02 | DIVIDEND PAYMENT | 123.45 | 1123.45 |
      | 2026-01-03 | ACCOUNT FEE      | -23.45 | 1100.00 |

  Scenario: Preview prints the extracted CSV on stdout and writes nothing
    Given the folder "alex/bank/cdia" has a transform
    When I run emuacc-ai-import for "alex/bank/cdia" with the fixture PDF
    Then emuacc-ai-import should exit with code 0
    And the emuacc-ai-import stdout should be the CSV:
      | Date       | Description      | Amount | Balance |
      | 2026-01-02 | DIVIDEND PAYMENT | 123.45 | 1123.45 |
      | 2026-01-03 | ACCOUNT FEE      | -23.45 | 1100.00 |
    And the emuacc-ai-import stderr should mention "cdia-pdf-2026-01-02--2026-01-03.csv"
    And the folder "alex/bank/cdia" should hold no CSV and no originals

  Scenario: --confirm stages the CSV, keeps the original and rebuilds the ledger
    Given the folder "alex/bank/cdia" has a transform
    When I run emuacc-ai-import for "alex/bank/cdia" with the fixture PDF and --confirm
    Then emuacc-ai-import should exit with code 0
    And the file "alex/bank/cdia/cdia-pdf-2026-01-02--2026-01-03.csv" should exist in sources
    And the file "alex/bank/cdia/originals/minimal-text.pdf" should exist in sources
    And the generated ledger should contain a transaction with payee "DIVIDEND PAYMENT"

  Scenario: --confirm is refused before the AI is called when no transform applies
    When I run emuacc-ai-import for "alex/bank/cdia" with the fixture PDF and --confirm
    Then emuacc-ai-import should exit with a non-zero code
    And the emuacc-ai-import stderr should mention "transform"
    And the AI stub should not have been called
    And the folder "alex/bank/cdia" should hold no CSV and no originals

  Scenario: Without an AI endpoint the CLI exits with the Global Settings hint
    Given the folder "alex/bank/cdia" has a transform
    And an app config with no AI provider settings
    When I run emuacc-ai-import for "alex/bank/cdia" with the fixture PDF
    Then emuacc-ai-import should exit with a non-zero code
    And the emuacc-ai-import stderr should mention "Global Settings"
    And the AI stub should not have been called
