Feature: Ledger.g4 is the specification; the handwritten parser is the runtime

  decision-003: the parser generated from Ledger.g4 / LedgerLexer.g4 /
  Prices.g4 (antlr4rust) is 30-75x slower than the handwritten parser on
  real ledgers, so the handwritten parser stays the runtime and the grammar
  is enforced as its specification by differential parity tests
  (src/ledger_parser/grammar_parity.rs): every fixture, a validator corpus
  through the grammar entry points, and an opt-in whole-vault run must get
  the same accept/reject decision from both parsers.

  Those tests are the only consumer of the generated parser, so it exists
  only in test builds: release binaries neither compile nor link it.

  Scenario: The generated parser is confined to test builds
    Given the crate manifest
    Then "antlr4rust" is a dev-dependency
    And "antlr4rust" is not a dependency of normal or build targets
    And the "generated" module of "ledger_parser" is compiled only for tests
