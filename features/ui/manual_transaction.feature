Feature: Add manual transactions in the UI

  Scenario: Add a manual transaction and see it in the table
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/uncategorised_bank.transactions"
    And I select the account "assets:bank:everyday"
    And I add a manual transaction with payee "Manual UI" and notes "Coffee"
    Then I should see a transaction row for payee "Manual UI"
    And the status line should say "Added manual transaction"

  Scenario: A manual transaction that could not be saved says so in the status line
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/uncategorised_bank.transactions"
    And I select the account "assets:bank:everyday"
    And saving a manual transaction fails with "the disk is full"
    And I add a manual transaction with payee "Manual UI" and notes "Coffee"
    Then the status line should say "Error: the disk is full"

  Scenario: The manual modal shows the selected account as fixed context
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/uncategorised_bank.transactions"
    And I select the account "assets:bank:everyday"
    And I click the "Add New" button
    Then the account field should show "assets:bank:everyday"
