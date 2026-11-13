Feature: Add a new account via the UI

  Scenario: Add a new account using the "+ Add Account" button
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I click the "Add Account" button
    And I enter account name "assets:bank:savings"
    And I submit the new account form
    And I select the account "assets:bank:savings"
    Then the sidebar should include the account "assets:bank:savings"

  Scenario: Add an expenses account using the Account Type dropdown
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I click the "Add Account" button
    And I select account type "expenses"
    And I enter account name "office:supplies"
    And I submit the new account form

  Scenario: Entity select defaults to the selected account set
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I click the "Add Account" button
    Then the entity select should show the account set "e2e"

  Scenario: Add an account under a new entity
    Given the app is running
    When I click the "Add Account" button
    And I choose to create entity "TestCo"
    And I enter account name "cba:smartaccess"
    And I submit the new account form
    And I select the account "assets:cash:bank:cba:smartaccess"
    Then the sidebar should include the account "assets:cash:bank:cba:smartaccess"

  Scenario: Add a crypto wallet account
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I click the "Add Account" button
    And I select account type "crypto"
    And I select wallet chain "ethereum"
    And I enter wallet address "0xABCDEF1234567890ABCDEF1234567890ABCDEF12"
    And I submit the new account form
    And I select the account "assets:crypto:wallet:ethereum:0xabcdef1234567890abcdef1234567890abcdef12"
    Then the sidebar should include the account "assets:crypto:wallet:ethereum:0xabcdef1234567890abcdef1234567890abcdef12"

  Scenario: The first account in an empty vault can name its entity
    Given the app is running
    And the vault has no account sets
    When I click the "Add Account" button
    Then the new entity name field should be shown
