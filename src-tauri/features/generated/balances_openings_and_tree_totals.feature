Feature: Opening balances and tree totals share the Balances computation

  An account declaration can carry an `opening` balance — the balance before
  the first recorded transaction. It has no date and no postings, so it is
  invisible to anything that only walks transactions. The Balances report and
  the base-currency totals behind the Accounts sidebar rows and folder header
  are produced by ONE function, which folds openings in under the same scope,
  allowlist and ignore rules as postings — so those numbers cannot disagree.

  Background:
    Given a transactions file named "balances_with_openings.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given a prices file named "balances_with_openings.prices"
    When I run the prices parser on that file
    Then the prices parse should succeed

  Scenario: Opening balances count toward holdings
    # cba: 50,000 opening + 500 − 20,000 − 1,000 − 500 = 29,000 AUD.
    # btc: 0.10 opening + 0.40 bought = 0.50 BTC at 100,000 = 50,000 AUD.
    # ABC: 15 at 10 = 150 AUD. Total 79,150.
    When I generate a balances report as of "2026-06-30" in "AUD" scoped to "assets" restricted to accounts "assets:cash:bank:cba, assets:crypto:binance:btc, assets:equity:commsec, assets:equity:commsec:personal"
    Then balances holding "AUD" should have quantity "29000"
    And balances holding "BTC" should have quantity "0.5"
    And the balances report total value should be "79150.00"

  Scenario: Openings on non-primary accounts are excluded by the allowlist
    # assets:crypto:transfer opens with 5 BTC. Without the allowlist it inflates
    # BTC to 5.5; with it (previous scenario) only the primary's 0.5 BTC counts.
    When I generate a balances report as of "2026-06-30" in "AUD" scoped to "assets"
    Then balances holding "BTC" should have quantity "5.5"

  Scenario: Openings on ignore accounts never count
    # ignore:spam opens with 999 AUD. Unscoped, income:salary's −500 nets against
    # cba's 29,000 → 28,500; counting the ignored opening would give 29,499.
    When I generate a balances report as of "2026-06-30" in "AUD"
    Then balances holding "AUD" should have quantity "28500"

  Scenario: A parent account with its own postings stays in the breakdown
    # assets:equity:commsec holds 10 ABC itself and :personal holds 5. Both are
    # holdings; dropping the parent would leave a breakdown that doesn't sum.
    When I generate a balances report as of "2026-06-30" in "AUD" scoped to "assets"
    Then balances holding "ABC" should have quantity "15"
    And balances holding "ABC" should have 2 account breakdown rows
    And balances holding "ABC" account "assets:equity:commsec" should have quantity "10"
    And balances holding "ABC" account "assets:equity:commsec:personal" should have quantity "5"
    And balances holding "ABC" account quantities should sum to its total

  Scenario: Sidebar and header totals come from the Balances computation
    When I generate a balances report as of "2026-06-30" in "AUD" scoped to "assets" restricted to accounts "assets:cash:bank:cba, assets:crypto:binance:btc, assets:equity:commsec, assets:equity:commsec:personal"
    And I compute the tree base totals as of "2026-06-30" in "AUD" restricted to accounts "assets:cash:bank:cba, assets:crypto:binance:btc, assets:equity:commsec, assets:equity:commsec:personal"
    Then the primary tree total for "assets" should equal the balances report total value
    And the primary tree total for "assets:cash" should be "29000.00"
    And the primary tree total for "assets:crypto" should be "50000.00"
    And the primary tree total for "assets:equity:commsec" should be "150.00"
    And the primary tree totals should not include "income"
    And the unrestricted tree total for "assets:crypto" should be "550000.00"
    And the unrestricted tree total for "income" should be "-500.00"

  Scenario: Hiding accounts keeps opening balances in the recomputed balances
    When I filter hidden accounts with prefix "ignore:" on the parsed result
    Then the balance for account "assets:cash:bank:cba" should be "29000.00" "AUD"
    And the parsed balances should not include account "ignore:spam"
