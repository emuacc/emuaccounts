Feature: Transaction list values a priced leg at its own price

  A broker fill carries its own price (`-100.0 NET @@ 31036.0 USD`). Seen
  from an AUD vault the Value column used to show 100 x the NET daily close
  converted to AUD, so two fills on one day at different prices read the same
  figure. A leg priced in another currency is now valued as that price
  converted to the base currency at the currency's rate on the transaction
  date, which is the proceeds the CGT report lists. A leg with no price keeps
  valuing at its quantity x the commodity's daily rate. The value cell says
  which basis it used, and the Set Price modal it opens is for the commodity
  whose rate the value depends on.

  Background:
    Given the app is running
    And the price file "USD.txt" contains:
      """
      P 2026-10-13 USD 1.4376 AUD
      P 2026-10-14 USD 1.4316 AUD
      P 2026-10-15 USD 1.4330 AUD
      """
    And the price file "NET.txt" contains:
      """
      P 2026-10-13T00:00:00.000000Z NET 347.7000 USD
      P 2026-10-14T00:00:00.000000Z NET 324.3300 USD
      P 2026-10-15T00:00:00.000000Z NET 316.3200 USD
      """
    When I parse the transactions file "src-tauri/features/fixtures/broker_trade_rows.transactions"
    And I select the account "assets:equity:broker:demo"

  Scenario: A USD-priced fill is valued at its own price converted to AUD
    Then the transaction row with notes "Sell -100.0 NET @ 310.36 USD" should show the value "44474.59 AUD"
    And the value of the transaction row with notes "Sell -100.0 NET @ 310.36 USD" should be based on "100.00 NET priced at 31,036.00 USD, converted to AUD at the USD rate on 2026-10-15"
    And the transaction row with notes "Buy 100.0 NET @ 312.21 USD" should show the value "44695.98 AUD"

  Scenario: An unpriced leg is still valued at its quantity times the daily rate
    Then the transaction row with notes "Position transfer in 10.0 NET" should show the value "4998.54 AUD"
    And the value of the transaction row with notes "Position transfer in 10.0 NET" should be based on "10.00 NET × the NET price on 2026-10-13"

  Scenario: The Set Price modal opens for the currency the value converted through
    When I click the value cell of the transaction row with notes "Sell -100.0 NET @ 310.36 USD"
    Then the set-price modal should show the commodity "USD" dated "2026-10-15"
    And the set-price modal should state the basis "100.00 NET priced at 31,036.00 USD, converted to AUD at the USD rate on 2026-10-15"
    When I cancel the set-price modal
    And I click the value cell of the transaction row with notes "Position transfer in 10.0 NET"
    Then the set-price modal should show the commodity "NET" dated "2026-10-13"
    And I cancel the set-price modal
