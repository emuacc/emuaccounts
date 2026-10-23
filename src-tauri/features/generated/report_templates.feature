Feature: Report template rendering

  Reports are generated as markdown files using Tera templates.
  The CGT report uses FIFO lot matching and the income report
  aggregates by account category.  Templates are rendered at
  pipeline rebuild time and stored as markdown files.

  Scenario: CGT report renders to markdown with FIFO data
    Given a transactions file named "cgt_trades.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    When I render the CGT report template for FY "2026" with base currency "AUD"
    Then the rendered report should contain "Capital Gains Tax Report"
    And the rendered report should contain "FY 2026"
    And the rendered report should contain "2025-08-20"
    And the rendered report should contain "ETH"
    And the rendered report should contain "1000.00"
    And the rendered report should contain "2000.00"
    And the rendered report should contain "Net Capital Gain"

  Scenario: CGT report with no events shows empty message
    Given a transactions file named "cgt_trades.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    When I render the CGT report template for FY "2020" with base currency "AUD"
    Then the rendered report should contain "No capital gains events"

  Scenario: Income report renders to markdown
    Given a transactions file named "income_expenses.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    When I render the income report template for FY "2026" with base currency "AUD"
    Then the rendered report should contain "Income Tax Report"
    And the rendered report should contain "FY 2026"

  Scenario: Pipeline generates report files for relevant FYs
    Given a clean sources directory with a file "alex/exchange/manual.transactions":
      """
      2025-01-15 * "Exchange" "Buy ETH"
          assets:exchange:eth 1 ETH {{ 1000.00 AUD }}
          equity:trading:buy -1000.00 AUD

      2025-08-20 * "Exchange" "Sell ETH"
          assets:exchange:eth -1 ETH @@ 2000.00 AUD
          equity:trading:sell 2000.00 AUD
      """
    And the vault config.json contains:
      """
      {"base_currency": "AUD"}
      """
    When I run the pipeline
    Then the generated reports directory should contain "cgt-2026.md"

  # Each cgt-YYYY report carries only its own year's warnings; the whole
  # list is written once, so one new warning touches two files, not two
  # per financial year.
  Scenario: The vault-wide CGT warnings are written once
    Given a clean sources directory with a file "alex/exchange/manual.transactions":
      """
      2025-01-10 * "Airdrop" "XYZ airdrop" ; txn:air-01
          assets:exchange 100 XYZ
          income:airdrop -100 XYZ

      2025-08-20 * "Exchange" "Sell ABC" ; txn:sell-abc
          assets:exchange -1 ABC @@ 10.00 AUD
          equity:trading:sell 10.00 AUD
      """
    And the vault config.json contains:
      """
      {"base_currency": "AUD"}
      """
    When I run the pipeline
    Then the generated report "cgt-warnings.md" should contain "No cost basis for acquisition of 100.0000 XYZ"
    And the generated report "cgt-warnings.md" should contain "Sold more than held for ABC"
    And the generated report "cgt-2025.md" should contain "No cost basis for acquisition of 100.0000 XYZ"
    And the generated report "cgt-2025.md" should not contain "Sold more than held"
    And the generated report "cgt-2026.md" should contain "Sold more than held for ABC"
    And the generated report "cgt-2026.md" should not contain "No cost basis"

  # The performance report used to exist only as a live command, so external
  # consumers (the dashboard widget, the read-only PWA, other agents) had no
  # artifact to read. It is now written per financial year, JSON only.
  Rule: The performance report is written as a per-FY JSON artifact

    # Holdings are posted to "assets:exchange" — the account the source folder
    # maps to — because the primary-accounts allowlist admits exactly those
    # folder accounts; a sub-account would be left out of the valuation.
    Background:
      Given a clean sources directory with a file "alex/exchange/manual.transactions":
        """
        2025-01-15 * "Exchange" "Buy ETH"
            assets:exchange 2 ETH {{ 2000.00 AUD }}
            equity:trading:buy -2000.00 AUD

        2025-08-20 * "Exchange" "Sell ETH"
            assets:exchange -1 ETH @@ 1500.00 AUD
            equity:trading:sell 1500.00 AUD
        """
      And a prices file at "_prices/ETH.txt" in sources with content:
        """
        P 2025-06-30 ETH 1000.00 AUD
        P 2026-06-30 ETH 1200.00 AUD
        """
      And the vault config.json contains:
        """
        {"base_currency": "AUD"}
        """

    Scenario: A performance artifact is written for each financial year
      When I run the pipeline
      Then the generated reports directory should contain "performance-2025.json"
      And the generated reports directory should contain "performance-2026.json"

    Scenario: The artifact spans the financial year and carries the window figures
      When I run the pipeline
      And I load the performance artifact for FY "2026"
      Then the performance artifact window should be "2025-07-01" to "2026-06-30"
      And the performance report should have 13 points
      And the performance report total realised should be "500.00"
      And the performance report unrealised change should be "200.00"
      And the performance report closing value should be "1200.00"
      And the performance report total return should be "700.00"
      And the performance artifact total return pct should be "0.35"
      And the performance artifact should have attribution for "ETH"

    Scenario: Total return pct is null when the window opens with nothing held
      When I run the pipeline
      And I load the performance artifact for FY "2025"
      Then the performance artifact total return pct should be null

    Scenario: The artifact matches the live report for the same window
      When I run the pipeline
      Then the performance artifact for FY "2026" should match a live performance report

    Scenario: A CSV-only run writes no performance artifact
      When I run the pipeline and regenerate reports in "csv" format
      Then the generated reports directory should contain "balances-2026.csv"
      And the generated reports directory should not contain "performance-2026.json"

    Scenario: A markdown-only run writes no performance artifact
      When I run the pipeline and regenerate reports in "md" format
      Then the generated reports directory should contain "cgt-2026.md"
      And the generated reports directory should not contain "performance-2026.json"
