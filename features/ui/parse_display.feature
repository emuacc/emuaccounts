Feature: Display parsed transactions in a sidebar + table

  Scenario: Parsing a file shows accounts and transaction rows
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I select the account "assets:exchange:kraken:btc"
    Then the sidebar should include the account "assets:exchange:kraken:btc"
    And I should see a transaction row for payee "Kraken"
    And I should see a notes value of "Sell BTC"
    And that transaction row should show an amount of "0.0050 BTC"

  Scenario: Account declarations create accounts with opening balances
    Given the app is running
    When I stage the account declarations "src-tauri/features/fixtures/account_opening.transactions"
    And I select the account "assets:cash:bank:cba:smartaccess"
    Then the sidebar should include the account "assets:cash:bank:cba:smartaccess"
    And the account "assets:cash:bank:cba:smartaccess" should show an amount of "100.00"
