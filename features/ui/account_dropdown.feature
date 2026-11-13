Feature: The account suggestions dropdown goes away with its field

  The account fields in the rule editor, the inline category editor and the
  Add Account modal share one suggestions dropdown, attached to the page body
  so no table row can clip it. Before TASK-153 it could outlive its field: left
  open when a field revisited after a choice lost focus, or when the field left
  the page while it still had focus. It then sat over the page, moving to the
  window's top-left corner on the next scroll, until the app was reloaded.

  Background:
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/uncategorised_bank.transactions"
    And I select the account "assets:bank:everyday"
    And I expand the transaction row for payee "Bakery"
    And I click "Edit Rule" in the row detail

  Scenario: Leaving a field revisited after a choice closes its dropdown
    When I choose the suggestion for "income" in the rule's Amount Account field
    And I click into the rule's Comment field
    And I click into the rule's Amount Account field
    Then the account dropdown is open
    When I click into the rule's Comment field
    Then no account dropdown is left on the page

  Scenario: Cancelling the rule from a revisited field leaves no dropdown behind
    When I choose the suggestion for "income" in the rule's Amount Account field
    And I click into the rule's Comment field
    And I click into the rule's Amount Account field
    Then the account dropdown is open
    When I cancel the rule editor
    Then no account dropdown is left on the page

  Scenario: Going back while typing in the field leaves no dropdown behind
    When I type "income" into the rule's Amount Account field
    Then the account dropdown is open
    When I press Alt+Left
    Then the rule editor is closed
    And no account dropdown is left on the page
