Feature: Parse .transactions files

  Scenario: Valid file parses successfully
    Given a transactions file named "valid.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    And the first transaction payee should be "Binance"
    And the first transaction narration should be "Buy SOL"
    And the first transaction meta should include "txn:"

  Scenario: Invalid file returns diagnostics
    Given a transactions file named "invalid.transactions"
    When I run the ledger parser on that file
    Then the parse should fail
    And diagnostics should include "missing amount"

  Scenario: Example file parses header fields correctly
    Given a transactions file named "example.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    And the first transaction payee should be "Kraken"
    And the first transaction narration should be "Sell BTC"
    And the first transaction meta should include "src:kraken:trade:def456"

  Scenario: Account declarations set opening balances
    Given a transactions file named "account_opening.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    And the balance for account "assets:cash:bank:cba:smartaccess" should be "100.0" "AUD"

  Scenario: Hyphenated commodity names parse successfully
    Given a transactions file named "hyphenated_commodity.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed

  Scenario: Opening balances accept thousands-separator commas
    Given a transactions file named "account_opening_commas.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    And the balance for account "assets:cash:cba:bta" should be "1823.92" "AUD"

  Scenario: Unparseable opening amounts fail with a diagnostic
    Given a transactions file named "account_opening_invalid.transactions"
    When I run the ledger parser on that file
    Then the parse should fail
    And diagnostics should include "invalid amount"

  # EXCHANGE:SYMBOL tickers (Google/TradingView convention; Hyperliquid's own
  # `<dex>:<coin>` ids). The venue prefix is part of the commodity, not the
  # account: `xyz:WTIOIL` is a distinct instrument from a bare `WTIOIL`.
  Scenario: EXCHANGE:SYMBOL commodity names parse successfully
    Given a transactions file named "colon_commodity.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    And the balance for account "assets:crypto:exchange:hyperliquid:llm-gemini" should be "926.92" "xyz:WTIOIL"

  # Ledger.g4 is the specification (decision-003). These pin the handwritten
  # parser to shapes the grammar accepts or rejects; the same fixtures go
  # through both parsers in src/ledger_parser/grammar_parity.rs.
  Scenario: Postings and account details may be indented by any run of blanks
    Given a transactions file named "posting_indent_any.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    And the balance for account "assets:cash:bank" should be "1100.0" "AUD"

  Scenario: A header without a meta comment fails
    Given a transactions file named "header_no_meta_invalid.transactions"
    When I run the ledger parser on that file
    Then the parse should fail
    And diagnostics should include "missing meta comment"

  Scenario: A header with only a status flag fails
    Given a transactions file named "header_status_only_invalid.transactions"
    When I run the ledger parser on that file
    Then the parse should fail
    And diagnostics should include "missing transaction details"

  Scenario: A commodity shaped like a date fails
    Given a transactions file named "commodity_date_shaped_invalid.transactions"
    When I run the ledger parser on that file
    Then the parse should fail
    And diagnostics should include "invalid commodity: 2026-01-15"

  Scenario: An account segment shaped like a date fails
    Given a transactions file named "account_date_segment_invalid.transactions"
    When I run the ledger parser on that file
    Then the parse should fail
    And diagnostics should include "invalid account path: assets:2026-01-15"
