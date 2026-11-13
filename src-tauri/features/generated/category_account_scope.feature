Feature: A category lists only the transactions with a leg under it

  An `account:` search term is a case-insensitive regex matched anywhere in an
  account name, the emuacc-query semantics: `account:income` also matches
  `expenses:tax:income`. The Categories view needs the account and its
  children only, as the sidebar tree groups them, so it anchors its term at a
  segment boundary: `account:(?-i)^income(:|$)`. Selecting income then no
  longer lists the tax paid on income (TASK-139).

  Background:
    Given a clean generated ledger directory
    And a generated file at "cash/bank/ledger.transactions" containing:
      """
      2025-01-10 * "Bank" "interest" ; txn:interest-001
          assets:cash:bank 12.50 AUD
          income:interest -12.50 AUD

      2025-01-20 * "ATO" "income tax" ; txn:tax-001
          assets:cash:bank -400.00 AUD
          expenses:tax:income 400.00 AUD

      2025-01-25 * "Bank" "fx loss" ; txn:forex-001
          assets:cash:bank -3.00 AUD
          income:forex 3.00 AUD
      """

  Scenario: A plain account term matches anywhere in the account name
    Then searching "account:income" should return 3 transactions

  Scenario: An anchored account term matches the category and its children only
    Then searching "account:(?-i)^income(:|$)" should return 2 transactions
    And searching "account:(?-i)^income(:|$) AND payee:ATO" should return 0 transactions

  Scenario: An anchored account term matches a child category
    Then searching "account:(?-i)^income:interest(:|$)" should return 1 transactions

  Scenario: An anchored account term does not match part of a segment
    Then searching "account:(?-i)^inc(:|$)" should return 0 transactions

  Scenario: An anchored account term is case-sensitive
    Then searching "account:(?-i)^Income(:|$)" should return 0 transactions
