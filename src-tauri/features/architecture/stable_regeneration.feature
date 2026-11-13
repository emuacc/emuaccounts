Feature: Regeneration changes only what a data change affects

  A vault that commits generated/ should get a diff that reads as "this data
  changed, and this is its impact": the touched ledgers, the reports
  downstream of them, and nothing else. A full rebuild is fine as long as the
  output is a stable function of (sources, prices, config, generator version)
  and each report contains only what is in its scope — then every unaffected
  file is byte-identical after a rebuild.

  Background:
    Given a clean sources directory with a file "alex/exchange/manual.transactions":
      """
      2025-01-15 * "Exchange" "Buy ETH" ; txn:buy-eth
          assets:exchange 2 ETH {{ 2000.00 AUD }}
          equity:trading:buy -2000.00 AUD

      2025-01-16 * "Airdrop" "XYZ airdrop" ; txn:air-xyz
          assets:exchange 100 XYZ
          income:airdrop -100 XYZ

      2025-08-20 * "Exchange" "Sell ETH" ; txn:sell-eth
          assets:exchange -1 ETH @@ 1500.00 AUD
          equity:trading:sell 1500.00 AUD
      """
    And a prices file at "_prices/ETH.txt" in sources with content:
      """
      P 2025-06-30 ETH 1000.00 AUD
      P 2026-06-30 ETH 1200.00 AUD
      """
    And a CSV "alex/bank/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-20 | Salary      | 3000   |
    And a transform at "alex/bank/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    And the vault config.json contains:
      """
      {"base_currency": "AUD"}
      """

  Scenario: A full rebuild from the same sources is byte-identical
    When I run the pipeline
    And I rebuild from scratch into a fresh generated directory
    Then every generated file should be byte-identical between the two builds

  # A change to one folder rebuilds that folder from its own sources and
  # leaves every other folder on disk unread (a scoped run), and the tree it
  # leaves is the one a rebuild from scratch produces. The cases that still
  # need the whole set are listed on `full_run_blockers` in
  # processing_pipeline.rs: a forced run, a changed ignored list, a root-level
  # manual.transactions, a changed folder with no set or one that held trades,
  # an unchanged folder whose outputs moved or whose summary predates the
  # accounts list; and a trade the change introduces, found after the
  # transform. tests/scoped_rebuild_equivalence.rs proves each against a
  # forced full rebuild, byte for byte.
  Scenario: A rule change rebuilds one folder and matches a rebuild from scratch
    When I run the pipeline
    And a rules file is added to "alex/bank/savings/_rules.json" matching "Salary" with payee "Employer"
    And I run the pipeline again
    Then the last rebuild should have been scoped
    And the ledger at "alex/bank/savings/ledger.transactions" should contain "Employer"
    When I rebuild from scratch into a fresh generated directory
    Then every generated file should be byte-identical between the two builds

  Scenario: A change that brings a trade into a bank folder reads the whole set
    When I run the pipeline
    Given a swap CSV at "alex/bank/savings/2026-02.csv" trading "USDC" for "SOL"
    When I run the pipeline again
    Then the last rebuild should have been a full run
    When I rebuild from scratch into a fresh generated directory
    Then every generated file should be byte-identical between the two builds

  Scenario: Adding one bank transaction changes only that account's ledger and the reports downstream of it
    When I run the pipeline
    And I snapshot the generated directory
    And a new CSV "alex/bank/savings/2026-01.csv" is added:
      | Date       | Description | Amount |
      | 2026-01-15 | Bonus       | 500    |
    And I run the pipeline again
    Then the generated files that changed should be exactly:
      | alex/bank/savings/ledger.transactions |
      | alex/bank/savings/summary.json        |
      | alex/reports/.stamp                   |
      | alex/reports/balances-2026.csv        |
      | alex/reports/balances-2026.json       |
      | alex/reports/income-2026.csv          |
      | alex/reports/income-2026.json         |
      | alex/reports/income-2026.md           |
      | alex/reports/performance-2026.json    |
