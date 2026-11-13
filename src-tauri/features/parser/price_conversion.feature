Feature: Multi-step price conversion

  Scenario: Direct conversion ETH to USD
    Given a clean sources directory
    And a prices file at "_prices/ETH.txt" in sources with content:
      """
      P 2026-01-15 ETH 3200.00 USD
      """
    When I convert 10 "ETH" to base currency "USD" at "2026-01-15"
    Then the converted value should be "32000.00"

  Scenario: Multi-step conversion ETH to AUD via USD
    Given a clean sources directory
    And a prices file at "_prices/ETH.txt" in sources with content:
      """
      P 2026-01-15 ETH 3200.00 USD
      """
    And a prices file at "_prices/USD.txt" in sources with content:
      """
      P 2026-01-15 USD 1.55 AUD
      """
    When I convert 10 "ETH" to base currency "AUD" at "2026-01-15"
    Then the converted value should be "49600.00"

  Scenario: Identity conversion AUD to AUD
    Given a clean sources directory
    When I convert 100 "AUD" to base currency "AUD" at "2026-01-15"
    Then the converted value should be "100.00"

  Scenario: No price data returns empty
    Given a clean sources directory
    When I convert 10 "ETH" to base currency "AUD" at "2026-01-15"
    Then the converted value should be empty

  Scenario: Batch conversion with mixed results
    Given a clean sources directory
    And a prices file at "_prices/ETH.txt" in sources with content:
      """
      P 2026-01-15 ETH 3200.00 USD
      """
    And a prices file at "_prices/USD.txt" in sources with content:
      """
      P 2026-01-15 USD 1.55 AUD
      """
    When I batch convert the following to base currency "AUD" at "2026-01-15":
      | amount | commodity |
      | 10     | ETH       |
      | 50     | AUD       |
      | 5      | DOGE      |
    Then the batch results should be:
      | value    |
      | 49600.00 |
      | 50.00    |
      | empty    |

  Scenario: Negative amounts preserve sign
    Given a clean sources directory
    And a prices file at "_prices/ETH.txt" in sources with content:
      """
      P 2026-01-15 ETH 3200.00 USD
      """
    When I convert -5 "ETH" to base currency "USD" at "2026-01-15"
    Then the converted value should be "-16000.00"

  # A _prices file can quote one date in several currencies (a merge keeps
  # every currency's quote). The conversion takes the nearest prior datetime,
  # then the quote in the base currency at it, whatever order the P lines are
  # in; it hops through another currency only when that datetime has no direct
  # quote. HNT via USD would be 10 × 2.70 × 1.50 = 40.50.
  Scenario: A direct quote wins over another currency's quote on the same date
    Given a clean sources directory
    And a prices file at "_prices/HNT.txt" in sources with content:
      """
      P 2024-06-15 HNT 4.00 AUD
      P 2024-06-15 HNT 2.70 USD
      """
    And a prices file at "_prices/USD.txt" in sources with content:
      """
      P 2024-06-15 USD 1.50 AUD
      """
    When I convert 10 "HNT" to base currency "AUD" at "2024-06-15"
    Then the converted value should be "40.00"

  Scenario: A direct quote wins whatever the order of the P lines
    Given a clean sources directory
    And a prices file at "_prices/HNT.txt" in sources with content:
      """
      P 2024-06-15 HNT 2.70 USD
      P 2024-06-15 HNT 4.00 AUD
      """
    And a prices file at "_prices/USD.txt" in sources with content:
      """
      P 2024-06-15 USD 1.50 AUD
      """
    When I convert 10 "HNT" to base currency "AUD" at "2024-06-15"
    Then the converted value should be "40.00"

  Scenario: A later quote in another currency is converted through one hop
    Given a clean sources directory
    And a prices file at "_prices/HNT.txt" in sources with content:
      """
      P 2024-06-14 HNT 4.00 AUD
      P 2024-06-15 HNT 2.70 USD
      """
    And a prices file at "_prices/USD.txt" in sources with content:
      """
      P 2024-06-15 USD 1.50 AUD
      """
    When I convert 10 "HNT" to base currency "AUD" at "2024-06-15"
    Then the converted value should be "40.50"
    When I convert 10 "HNT" to base currency "AUD" at "2024-06-14"
    Then the converted value should be "40.00"

  Scenario: The hop goes through the quote currency that reaches the base currency
    Given a clean sources directory
    And a prices file at "_prices/HNT.txt" in sources with content:
      """
      P 2024-06-15 HNT 2.70 USD
      P 2024-06-15 HNT 0.00004 BTC
      """
    And a prices file at "_prices/BTC.txt" in sources with content:
      """
      P 2024-06-15 BTC 67000.00 USD
      """
    And a prices file at "_prices/USD.txt" in sources with content:
      """
      P 2024-06-15 USD 1.50 AUD
      """
    When I convert 10 "HNT" to base currency "AUD" at "2024-06-15"
    Then the converted value should be "40.50"
