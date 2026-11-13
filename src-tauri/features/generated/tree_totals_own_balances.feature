Feature: Every account is valued by its own balance

  The unrestricted tree totals behind the Categories rows and the revealed
  holdings of "Show all holdings" value each account's OWN balance in the base
  currency, commodity by commodity, rolled up to every ancestor. Across the
  whole ledger every commodity nets to zero (double entry), so valuing a
  commodity's vault-wide total would leave almost every account without a
  value. The primary totals stay the Balances report's holdings computation.

  Background:
    Given a transactions file named "tree_totals_double_entry.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given a prices file named "tree_totals_double_entry.prices"
    When I run the prices parser on that file
    Then the prices parse should succeed

  Scenario: Accounts are valued even though every commodity nets to zero
    # ETH at 300 AUD: wallet 2.00 - 0.50 = 1.50 ETH = 450, wrap 0.50 ETH = 150.
    # cba 1000 - 600 - 5 = 395 AUD. equity:trading:buy -2 ETH + 600 AUD = 0.
    When I compute the tree base totals as of "2026-06-30" in "AUD" restricted to accounts "assets:cash:bank:cba, assets:crypto:wallet"
    Then the unrestricted tree total for "assets:crypto:wrap" should be "150.00"
    And the unrestricted tree total for "assets:crypto:wallet" should be "450.00"
    And the unrestricted tree total for "assets:crypto" should be "600.00"
    And the unrestricted tree total for "assets" should be "995.00"
    And the unrestricted tree total for "equity:trading:buy" should be "0.00"

  Scenario: Categories are valued by their own balances
    When I compute the tree base totals as of "2026-06-30" in "AUD" restricted to accounts "assets:cash:bank:cba, assets:crypto:wallet"
    Then the unrestricted tree total for "income:salary" should be "-1000.00"
    And the unrestricted tree total for "income" should be "-1000.00"
    And the unrestricted tree total for "expenses:food" should be "5.00"

  Scenario: The primary totals are unchanged and leave the other accounts out
    When I generate a balances report as of "2026-06-30" in "AUD" scoped to "assets" restricted to accounts "assets:cash:bank:cba, assets:crypto:wallet"
    And I compute the tree base totals as of "2026-06-30" in "AUD" restricted to accounts "assets:cash:bank:cba, assets:crypto:wallet"
    Then the primary tree total for "assets" should equal the balances report total value
    And the primary tree total for "assets" should be "845.00"
    And the primary tree totals should not include "assets:crypto:wrap"
