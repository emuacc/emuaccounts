@wip
Feature: Swap transaction value consistency

  A swap transaction's AUD value should be the same whether the
  swap is linked (showing both legs) or unlinked (showing a single leg).
  A leg's own price annotation is its value: in the base currency (set
  by the pipeline's swap linking) it is summed as is, in another
  currency it is converted at that currency's rate on the day. Only an
  unpriced leg uses the PriceGraph market price of the commodity.

  Scenario: Unlinked swap leg shows price-annotation value, not PriceGraph value
    Given the app is running with base currency "AUD"
    And I select an account with a PENGU swap that has a price annotation of "0.010594 AUD"
    And the PriceGraph returns 151.52 AUD for 19029.00 PENGU
    Then the value cell should show "201.61 AUD"

  Scenario: An annotation in another currency is converted at that currency's rate
    Given the app is running with base currency "AUD"
    And I select an account with a PENGU swap that has a price annotation of "0.0067 USD"
    And the PriceGraph returns 151.52 AUD for 19029.00 PENGU
    And the PriceGraph returns 1.52 AUD for 1.00 USD
    Then the value cell should show "193.79 AUD"
