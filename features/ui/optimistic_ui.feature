Feature: Edits in the transactions table show at once and land in place

  Re-categorising, renaming a party, hiding or unhiding a transaction
  changes the vault's rules and rebuilds the pipeline, which takes seconds
  on a large vault. The table shows the change at once, before that
  rebuild has finished. When the rebuild lands the rows refresh in place:
  the table is not cleared or reloaded, so the other rows keep their
  place, expanded rows stay expanded, and only rows that changed fade.
  Show Ignored brings the hidden rows in and takes them out the same way.

  Background:
    Given the app is running
    And the "uncategorised_bank" fixture folders have no rules or labels
    When I parse the transactions file "src-tauri/features/fixtures/uncategorised_bank.transactions"
    And I select the account "assets:bank:everyday"

  Scenario: Re-categorising an uncategorised row shows before the rebuild finishes
    When I re-categorise the transaction row for payee "Coffee Club" to "expenses:food"
    Then before the rebuild finished the row for payee "Coffee Club" showed category "expenses:food"
    And before the rebuild finished the row for payee "Coffee Club" was no longer marked uncategorised
    And once the rebuild lands the row for payee "Coffee Club" shows category "expenses:food"

  Scenario: Renaming a party shows before the rebuild finishes
    When I rename the party on the transaction row for payee "Grocer" to "Local Grocer"
    Then before the rebuild finished the row for payee "Grocer" showed party "Local Grocer"
    And once the rebuild lands the row for payee "Grocer" shows party "Local Grocer"

  Scenario: The rebuild lands in place without reloading the table
    When I expand the transaction row for payee "Employer"
    And I tag the transaction row for payee "Employer" with marker "kept"
    And I re-categorise the transaction row for payee "Coffee Club" to "expenses:food"
    Then once the rebuild lands the row for payee "Coffee Club" shows category "expenses:food"
    And the transaction table never emptied while the rebuild landed
    And the transaction row for payee "Employer" still has marker "kept"
    And the transaction row for payee "Employer" is still expanded

  Scenario: Hiding and unhiding a row show at once
    When I hide the transaction row for payee "Grocer"
    Then before the rebuild finished the row for payee "Grocer" was gone
    And once the rebuild lands there is no row for payee "Grocer"
    When I open the hidden transactions with Show Ignored
    And I unhide the transaction row for payee "Grocer"
    Then before the rebuild finished the row for payee "Grocer" was gone
    And once the rebuild lands there is no row for payee "Grocer"
    When I turn off Show Ignored in the Accounts view
    And I select the account "assets:bank:everyday"
    Then once the rebuild lands the row for payee "Grocer" shows category "expenses:unknown"

  Scenario: Show Ignored brings a hidden row in and takes it out in place
    Given the data folder leaves out the "ignore" accounts unless Show Ignored is on
    And I hid the transaction row for payee "Grocer"
    And I tag the transaction row for payee "Employer" with marker "kept"
    When I turn on Show Ignored
    Then once its rows land the row for payee "Grocer" fades in showing category "ignore:hidden"
    And the row for payee "Employer" did not fade
    And the transaction row for payee "Employer" still has marker "kept"
    When I turn off Show Ignored
    Then the row for payee "Grocer" was gone in the same frame as the click
    And once its rows land there is no row for payee "Grocer"
    And the transaction row for payee "Employer" still has marker "kept"
