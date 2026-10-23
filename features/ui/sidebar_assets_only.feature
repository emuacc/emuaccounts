Feature: Sidebar shows only asset accounts

  The sidebar navigation displays asset accounts directly (e.g., bitcoin, cba)
  without the "assets" group header. Equity, income and expenses are never
  shown in the sidebar, and liabilities only when backed by a source folder of
  their own (liabilities_sidebar.feature) — a liabilities account that only
  rules reference still exists as a posting counterpart but is not navigable.

  Scenario: Sidebar lists asset sub-accounts without the assets group
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    Then the sidebar should include the account "assets:exchange:kraken:btc"
    And the sidebar should not include the account group "liabilities"
    And the sidebar should not include the account group "expenses"
    And the sidebar should not include the account group "income"
