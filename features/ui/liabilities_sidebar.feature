Feature: Liabilities source folders appear in the Accounts sidebar

  A source folder directly under the entity's "liabilities" folder backs a
  liabilities: account exactly as "assets" folders back assets: accounts: the
  pipeline posts its rows there, and the Accounts sidebar lists it in a
  "liabilities" group beside the asset folders with the same drill-down, back
  navigation and header. An entity without a liabilities folder sees only its
  asset folders (sidebar_assets_only.feature), and a liabilities account that
  only rules reference stays in the Categories pane.

  Background:
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I stage the transactions file "src-tauri/features/fixtures/liabilities_amex.transactions" in the source folder "e2e/liabilities/credit-card/amex"

  Scenario: The liabilities group sits beside the asset folders and drills down to the account
    When I open the Accounts root folder
    Then the sidebar should include the account group "assets:exchange"
    And the sidebar should include the account group "liabilities"
    When I drill into the sidebar folder "liabilities"
    Then the folder header should show "liabilities"
    When I drill into the sidebar folder "liabilities:credit-card"
    Then the sidebar should include the account "liabilities:credit-card:amex"
    When I select the account "liabilities:credit-card:amex"
    Then I should see at least one transaction row
    When I go back one sidebar level
    Then the sidebar should include the account group "liabilities:credit-card"

  Scenario: A folder-backed liabilities account leaves the Categories pane
    When I open the Categories roots
    Then the sidebar should include the account group "expenses"
    And the sidebar should not include the account group "liabilities"
