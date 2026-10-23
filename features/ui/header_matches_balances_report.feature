Feature: Accounts header equals the Balances report

  The Accounts-page folder header, the sidebar row totals and the Balances
  report are all produced by ONE backend holdings computation over the
  primary accounts (source folders plus configured extra prefixes), with
  account opening balances folded in. The root folder header therefore shows
  the Balances report total — the same number, from the same function.

  Scenario: The root folder header total equals the Balances report total
    Given the app is running
    When I parse the transactions file "sample-data/example.transactions"
    And I open the root folder view
    And I note the folder header total
    And I open the Balances report
    Then the Balances report total equals the noted folder header total
