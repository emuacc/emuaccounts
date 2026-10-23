Feature: Date bounds at month and year granularity

  A date bound shorter than a full YYYY-MM-DD stands for its whole period:
  an inclusive upper bound of 2025-03 keeps every day of March 2025 and an
  upper bound of 2025 keeps the whole year. Comparisons happen at the bound's
  granularity, so <2025-03 ends with February and >2025-03 starts with April.
  This is the same query language used by the UI search box and the
  emuacc-query CLI.

  Background:
    Given a clean sources directory
    And a CSV "alex/cash/bank/cba/savings/all.csv":
      | Date       | Description      | Amount |
      | 2025-01-15 | January deposit  | 100.00 |
      | 2025-02-10 | February deposit | 100.00 |
      | 2025-03-01 | March starts     | 100.00 |
      | 2025-03-31 | March ends       | 100.00 |
      | 2025-04-01 | April starts     | 100.00 |
      | 2026-01-05 | Next year        | 100.00 |
    And a transform at "alex/cash/bank/cba/savings/_transform.rhai" without an account field
    When I run the pipeline for month "202601"

  Scenario: A month range keeps every day of its final month
    Then searching "date:2025-01..2025-03" should return 4 transactions

  Scenario: A month upper bound keeps every day of that month
    Then searching "date:<=2025-03" should return 4 transactions

  Scenario: A year upper bound keeps the whole year
    Then searching "date:<=2025" should return 5 transactions

  Scenario: A year range keeps the whole year
    Then searching "date:2025..2025" should return 5 transactions

  Scenario: An exclusive month upper bound stops before that month
    Then searching "date:<2025-03" should return 2 transactions

  Scenario: An exclusive month lower bound starts after that month
    Then searching "date:>2025-03" should return 2 transactions

  Scenario: A bare year or month matches every date in it
    Then searching "date:2025" should return 5 transactions
    And searching "date:2025-03" should return 2 transactions
