Feature: Typing in the search bar keeps the text and the caret where the user put them

  The accounts search waits for a pause in typing, runs the query in the
  backend and re-renders when the results land. Keys typed and caret moves
  made while a query is still running belong to the user: the results landing
  must not move the caret or reorder the text. Before TASK-150 the render put
  the caret back where it was when the query was scheduled, so the next keys
  went into the middle of the word.

  Background:
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/income_expenses.transactions"
    And I select the account "assets:bank:checking"
    And the search bar is empty
    And every transaction search takes 800 ms

  Scenario: Typing faster than the debounce plus the search keeps every character in order
    When I type "office" into the search bar with 400 ms between keys
    Then the search bar shows "office" with the caret at 6
    When the searches finish
    Then the search bar shows "office" with the caret at 6
    And the transaction table shows only the payee "Office Supply"

  Scenario: A caret move made while a search is running survives the results landing
    When I type "office" into the search bar with 400 ms between keys
    And I move the caret to 2 while a search is running
    And the searches finish
    Then the search bar shows "office" with the caret at 2
    And the transaction table shows only the payee "Office Supply"
