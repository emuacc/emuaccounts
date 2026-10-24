Feature: Sidebar brand tagline

  The top of the sidebar carries the EmuAcc wordmark and the product tagline,
  which matches the README headline: "Understand your finances".

  Scenario: The sidebar tagline reads "Understand your finances"
    Given the app is running
    Then the sidebar brand tagline should read "Understand your finances"
