Feature: A new account shows in the sidebar and the account dropdowns

  An account the app writes (a rule's category, an inline category edit, a
  manual transaction's other account) is listed in the sidebar and offered by
  every account dropdown at once, before the rebuild finishes; the refresh
  after it fills in the totals. Before TASK-148 a rule's new category reached
  neither list until the app was restarted.

  Scenario: A category an inline edit creates shows in the Categories pane and the dropdowns
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/income_expenses.transactions"
    And I select the account "assets:bank:checking"
    And I recategorise the first transaction row as "expenses:pets"
    And I open the Categories roots
    And I drill into the category "expenses"
    Then the sidebar should include the account "expenses:pets"
    And the account dropdowns should offer "expenses:pets"

  Scenario: A category a manual transaction creates shows in the Categories pane and the dropdowns
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I add a manual transaction posting to the new category "expenses:gifts"
    And I open the Categories roots
    And I drill into the category "expenses"
    Then the sidebar should include the account "expenses:gifts"
    And the account dropdowns should offer "expenses:gifts"
