Feature: Vault-level display config with per-set override
  The vault's generated/config.json carries a default base_currency that
  every account set inherits. A set's generated/<set>/config.json may
  override it. Until the vault has a base_currency the app cannot value
  holdings, so the resolver must report that state distinctly rather than
  silently defaulting.

  Scenario: Set inherits the vault base currency
    Given a generated directory
    And the vault config.json contains:
      """
      {"base_currency": "USD", "default_decimals": 2}
      """
    And the set "acme" config.json contains:
      """
      {"commodities": {"AUD": {"decimals": 2}}}
      """
    When I resolve the display config for set "acme"
    Then the resolved base_currency should be "USD"
    And the resolved commodities should include "AUD"

  Scenario: Set base currency overrides the vault default
    Given a generated directory
    And the vault config.json contains:
      """
      {"base_currency": "USD"}
      """
    And the set "alex" config.json contains:
      """
      {"base_currency": "AUD"}
      """
    When I resolve the display config for set "alex"
    Then the resolved base_currency should be "AUD"

  Scenario: Neither vault nor set defines a base currency
    Given a generated directory
    And the set "acme" config.json contains:
      """
      {"default_decimals": 2}
      """
    When I resolve the display config for set "acme"
    Then the resolved base_currency should be absent
    And the vault base currency should be absent

  Scenario: Set override does not count as a vault default
    Given a generated directory
    And the set "alex" config.json contains:
      """
      {"base_currency": "AUD"}
      """
    When I resolve the display config for set "alex"
    Then the resolved base_currency should be "AUD"
    And the vault base currency should be absent

  Scenario: Setting the vault base currency preserves other keys
    Given a generated directory
    And the vault config.json contains:
      """
      {"default_decimals": 4, "commodities": {"EUR": {"decimals": 2}}}
      """
    When I set the vault base currency to "EUR"
    Then the vault base currency should be "EUR"
    And the vault config.json should contain default_decimals 4
    And the resolved display config for set "anything" should have base_currency "EUR"

  Scenario: Setting the vault base currency creates config.json when absent
    Given a generated directory
    When I set the vault base currency to "AUD"
    Then the vault base currency should be "AUD"

  Scenario: Blank base currency is rejected
    Given a generated directory
    When I set the vault base currency to "  "
    Then setting the vault base currency should fail

  Scenario: Base currency candidates come from price quotes and configured commodities
    Given a generated directory
    And a sources directory with a prices file "spot.txt" containing:
      """
      P 2026-01-01 BTC 70000.00 USD
      P 2026-01-01 AUD 0.65 USD
      P 2026-01-01 ETH 2000.00 EUR
      """
    And the vault config.json contains:
      """
      {"commodities": {"GBP": {"decimals": 2}, "AUD": {"decimals": 2}}}
      """
    When I list the base currency candidates
    Then the base currency candidates should be "AUD, EUR, GBP, USD"
