Feature: Base currency setup gate

  A data folder is valued in one base currency, stored as `base_currency` in
  `generated/config.json` and inherited by every account set unless the set's
  own `generated/<set>/config.json` overrides it. A folder without one cannot
  value holdings — the header and sidebar would degrade to raw single-commodity
  figures — so the app refuses to show the main page until one is chosen, the
  same way it refuses without a data folder.

  Scenario: A vault without a base currency is gated until one is chosen
    Given the app is running
    And the vault has no base currency
    When the vault is reopened
    Then I should see the base currency setup screen
    And the main page should not be shown
    When I choose the base currency "AUD"
    Then the main page should be shown
    And the vault config should have base currency "AUD"

  Scenario: A set-up vault goes straight to the main page
    Given the app is running
    And the vault base currency is "AUD"
    When the vault is reopened
    Then the main page should be shown
