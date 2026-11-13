Feature: Global Settings — accounts included in balance, Show all holdings, clearing accounts

  A gear button in the sidebar opens a Global Settings dialog. It lets the user
  add nominal account prefixes (e.g. assets:staking) to the set of accounts
  counted in the Balances, Performance and Tax Savings reports (source-folder
  accounts always count), turn on Show all holdings so the Accounts pane also
  lists holdings with no source folder, and declare the vault's clearing
  accounts (assets:transfer when none are declared).

  Scenario: The Global Settings dialog shows the included-accounts editor
    Given the app is running
    When I open the global settings dialog
    Then I should see the included-accounts prefix input
    And I should see the included-accounts list

  Scenario: The Global Settings dialog has a Show all holdings checkbox
    Given the app is running
    When I open the global settings dialog
    Then I should see the Show all holdings checkbox

  Scenario: The Global Settings dialog lists the vault's clearing accounts
    Given the app is running
    When I open the global settings dialog
    Then the clearing accounts list should show "assets:transfer"
    And I should see the clearing account input
