Feature: Categories page lists income and expenses

  The Categories view (parallel to Accounts) lists what money was earned and
  spent on: income and expenses, plus ignore while Show Ignored is on. It holds
  no assets, liabilities or equity: holdings belong in the Accounts pane, and
  clearing accounts such as assets:transfer appear in neither. The Categories
  view reuses the same drill-down tree and transaction table, and its rows are
  loaded via the union query (query_global) because no source folder derives to
  a category (the folder-pruned query_search would return nothing).

  Scenario: Nominal accounts are navigable under Categories
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I switch to the Categories view
    Then the Categories sidebar should show the root "income"
    When I drill into the category "income"
    And I drill into the category "income:trading"
    Then the sidebar should include the account "income:trading:pnl"

  Scenario: Selecting a category loads its transactions via the union query
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I switch to the Categories view
    And I drill into the category "income"
    And I drill into the category "income:trading"
    And I select the category account "income:trading:pnl"
    Then I should see at least one transaction row

  Scenario: The Categories pane holds no holdings
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I open the Categories roots
    Then the sidebar should include the account group "income"
    And the sidebar should not include the account group "assets"
