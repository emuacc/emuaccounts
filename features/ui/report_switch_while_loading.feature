Feature: Choosing a report while another is loading

  Reports opens on Capital Gains Tax. A report chosen before that load has
  finished is the one the page ends on, whichever of the two answers last.

  Scenario: The default report answers after the chosen one
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And the "Capital Gains Tax" report takes 3 seconds to generate
    And I choose the "Balances" report while the "Capital Gains Tax" report is loading
    Then the page shows the "Balances" report once the "Capital Gains Tax" report has answered

  Scenario: The default report answers while the chosen one is still loading
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And the "Capital Gains Tax" report takes 2 seconds to generate
    And the "Balances" report takes 6 seconds to generate
    And I choose the "Balances" report while the "Capital Gains Tax" report is loading
    Then the page still says the report is generating once the "Capital Gains Tax" report has answered
    And the page shows the "Balances" report once the "Balances" report has answered
