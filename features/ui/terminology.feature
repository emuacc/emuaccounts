Feature: Global Settings — terminology

  The Global Settings dialog has a Terminology section that lists every
  configurable display term from src-tauri/terminology.json with its
  description and default wording, and lets the user override the singular
  and plural forms. Reports read the same terms, so the Balances and Tax
  Savings holdings are labelled with the "asset" term instead of "coin".

  Scenario: The Global Settings dialog lists the asset term with its defaults
    Given the app is running
    When I open the global settings dialog
    Then I should see the terminology list
    And the term "asset" should offer the defaults "asset" and "assets"
