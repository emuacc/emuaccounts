Feature: Transaction categorization rules

  Scenario: Rule sets payee based on narration match
    Given a clean sources directory with a CSV and transform
    And a rules file at "bank/_rules.json" matching "imported" with payee "Test Merchant"
    When I run the pipeline
    Then transactions with narration "imported" should have payee "Test Merchant"

  Scenario: Rule sets contra account
    Given a clean sources directory with a CSV and transform
    And a rules file at "bank/_rules.json" matching "imported" with contra "expenses:groceries"
    When I run the pipeline
    Then transactions with narration "imported" should use contra "expenses:groceries"

  Scenario: Wildcard rule matches prefix
    Given a clean sources directory with a CSV and transform
    And a rules file at "bank/_rules.json" matching "import*" with payee "Wildcard Match"
    When I run the pipeline
    Then transactions with narration "imported" should have payee "Wildcard Match"

  # The build cache holds the transform's raw rows; the rules are a pass over
  # them, so a new rule never re-runs the transform script.
  Scenario: A new rule re-categorises the cached rows without re-running the transform
    Given a pipeline has been run once with a transform
    When a rules file is added to "bank/_rules.json" matching "CacheTest" with payee "Ruled"
    And I run the pipeline again
    Then the CSV should be re-categorised from the cache, not re-transformed

  Scenario: Transform suggestion maps Description to narration not payee
    Given a CSV with headers "Date,Description,Amount"
    And the target account is "assets:bank:test"
    When I generate a transform suggestion
    Then the suggestion should map "narration" from "Description"
    And the suggestion should have blank payee
