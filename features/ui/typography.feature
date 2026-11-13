Feature: Typography

  The app reads as one typeface. Text uses the UI font family, with the one
  monospace family for the search box, patterns, ids and code. Every size and
  weight comes from the type scale in the shared CSS variables (src/style.css
  :root). Buttons, selects and inputs inherit the page font instead of the
  platform's control font. The EmuAcc wordmark is the logo and keeps its own
  typeface and size.

  Scenario: Main screen text uses the UI and monospace fonts and the type scale
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I select the account "assets:exchange:kraken:btc"
    Then the "main screen" text should use only the UI and monospace font families
    And the "main screen" text should use only the sizes and weights of the type scale

  @dialog
  Scenario: The Global Settings dialog uses the UI and monospace fonts and the type scale
    Given the app is running
    When I open the global settings dialog
    Then the "dialog" text should use only the UI and monospace font families
    And the "dialog" text should use only the sizes and weights of the type scale

  @dialog
  Scenario: The Add New dialog uses the UI and monospace fonts and the type scale
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I select the account "assets:exchange:kraken:btc"
    And I click the "Add New" button
    Then the "dialog" text should use only the UI and monospace font families
    And the "dialog" text should use only the sizes and weights of the type scale

  Scenario: The Balances report uses the UI and monospace fonts and the type scale
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I open the Balances report
    Then the "report" text should use only the UI and monospace font families
    And the "report" text should use only the sizes and weights of the type scale
