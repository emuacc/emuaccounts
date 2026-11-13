Feature: A folder's summary.json includes its declared opening balances

  The pipeline writes a summary.json beside every folder's ledger: the
  per-account totals the sidebar tree loads without parsing transactions,
  and the figure a script or a person reads to check a balance. An opening
  balance is declared in the folder's accounts.transactions, not in any
  transaction, so the summary folds the folder's openings into its totals
  and lists them under "openings". The tree takes every opening from the
  set-root accounts.transactions once: an opening declared in a folder that
  has no ledger still counts, and one a summary already holds never counts
  twice.

  Background:
    Given a clean sources directory
    And a CSV "alex/cash/cba/bta/2025-01.csv":
      | Date       | Description | Amount  |
      | 2025-01-15 | Xero        | -844.00 |
      | 2025-01-20 | Client      | 1524.10 |
    And a transform at "alex/cash/cba/bta/_transform.rhai" that maps Date/Description/Amount to AUD
    And an accounts file at "alex/cash/cba/bta/accounts.transactions" with:
      """
      account assets:cash:cba:bta AUD
          opening 1,823.92 AUD
      """

  Scenario: The summary, the account tree and a query agree on an account with an opening
    # 680.10 AUD of transactions plus the 1,823.92 AUD opening.
    When I run the pipeline
    Then the summary "alex/cash/cba/bta/summary.json" should total "2504.02" "AUD" for "assets:cash:cba:bta"
    And the summary "alex/cash/cba/bta/summary.json" should list an opening of "1823.92" "AUD" for "assets:cash:cba:bta"
    And the account tree should total "2504.02" "AUD" for "assets:cash:cba:bta"
    When I query "account:assets:cash:cba:bta" with limit 500
    Then the aggregated balance for "AUD" should be 2504.02

  Scenario: An opening declared in a folder with no ledger counts once in the tree
    Given an accounts file at "alex/cash/cba/goalsaver/accounts.transactions" with:
      """
      account assets:cash:cba:goalsaver AUD
          opening 500.00 AUD
      """
    When I run the pipeline
    Then the account tree should total "500" "AUD" for "assets:cash:cba:goalsaver"
    And the account tree should total "2504.02" "AUD" for "assets:cash:cba:bta"

  Scenario: Changing an opening rewrites the summary
    When I run the pipeline
    And I set the opening balance for "assets:cash:cba:bta" to "100.00" "AUD" in account set "alex"
    Then the summary "alex/cash/cba/bta/summary.json" should total "780.10" "AUD" for "assets:cash:cba:bta"
    And the account tree should total "780.10" "AUD" for "assets:cash:cba:bta"

  Scenario: A folder without openings writes no openings field
    Given a CSV "alex/cash/cba/everyday/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/cash/cba/everyday/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline
    Then the summary "alex/cash/cba/everyday/summary.json" should total "-4.5" "AUD" for "assets:cash:cba:everyday"
    And the summary "alex/cash/cba/everyday/summary.json" should have no openings field
    And the account tree should total "2504.02" "AUD" for "assets:cash:cba:bta"
