Feature: A page stays where the user scrolled it while rows load

  The Accounts and Categories tables fetch their next page of rows when the
  user nears the bottom. Reports keeps the window of rows the selection loaded.
  Before TASK-151 the scroll handler stayed live there: near the bottom of a
  long report it fetched rows nothing shows and put the scroll back where it
  started, again and again until every row had loaded, so the report stuck to
  the bottom and scrolling up jumped straight back down.

  Background:
    Given the app is running
    And the transactions file "src-tauri/target/e2e/busy.transactions" holds 600 purchases on the account "assets:bank:busy"
    When I parse the transactions file "src-tauri/target/e2e/busy.transactions"
    And I select the account "assets:bank:busy"
    Then the transaction table shows 500 rows

  Scenario: A long report scrolls to the bottom and back up without fetching rows
    When I open the Balances report
    And the report is long enough to scroll
    And every transaction search takes 800 ms
    And I scroll the page to the bottom
    And I scroll the page up by 600 pixels
    And the searches finish
    Then the page stays where I scrolled it
    And the transaction list still holds 500 rows

  Scenario: Reaching the bottom of the Accounts table loads the next page
    When I scroll the page to the bottom
    Then the transaction table shows 600 rows

  Scenario: Reaching the bottom of a category's table loads the next page
    When I switch to the Categories view
    And I drill into the category "expenses"
    And I select the category account "expenses:busy"
    Then the transaction table shows 500 rows
    When I scroll the page to the bottom
    Then the transaction table shows 600 rows

  Scenario: Scrolling up while the next page loads is not undone when it lands
    When every transaction search takes 800 ms
    And I scroll the page to the bottom
    And I scroll the page up by 600 pixels
    And the searches finish
    Then the page stays where I scrolled it
    And the transaction table shows 600 rows

  Scenario: A page of rows landing after the user moved to a report leaves the report alone
    When every transaction search takes 800 ms
    And I scroll the page to the bottom
    And I open the Balances report
    And the report is long enough to scroll
    And I scroll the page to 300 pixels
    And the searches finish
    Then the page stays where I scrolled it
