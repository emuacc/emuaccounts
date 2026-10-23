@ai_suggest
Feature: AI-assisted suggestions

  The categorisation and transform-generation engines send right-sized
  context to the configured OpenAI-compatible endpoint and expose tools
  (search_rules, find_similar_transactions, list_categories, ...) so the
  model can pull more data on demand.

  Scenario: Categorisation prompt is bounded and offers tools
    Given an account with 500 categorisation rules
    When the AI categorisation prompt is built
    Then the categorisation prompt should mention "470 more rules"
    And the categorisation prompt should mention "search_rules"

  Scenario: Categorisation suggestion via a stub AI server
    Given a stub AI server that suggests pattern "*UBER*" with account "expenses:transport"
    When I run AI categorisation for an uncategorised transaction with payee "UBER TRIP"
    Then the categorisation result should have 1 suggestion with pattern "*UBER*"

  Scenario: Unparseable categorisation reply retries once then fails cleanly
    Given a stub AI server that replies with content "no json here" and then with content "still no json"
    When I run AI categorisation for an uncategorised transaction with payee "UBER TRIP"
    Then the categorisation result should be a failure mentioning "parse"
    And the stub server should have received 2 requests

  Scenario: Transform generation retries until the script passes the run check
    Given a stub AI server that returns a broken transform script and then a working one
    When I run AI transform generation for a CSV with headers "Date,Description,Amount"
    Then the transform result should be a script that compiles and runs
    And the stub server should have received 2 requests

  Scenario: Categorisation prompt includes the configured user context
    Given the AI user context is "Alex Example (initials AE)"
    When the AI categorisation prompt is built
    Then the categorisation prompt should mention "Alex Example (initials AE)"

  Scenario: Transform prompt offers the validation tool
    Given a CSV with transform headers "Date,Description,Amount"
    When the AI transform prompt is built
    Then the transform prompt should mention "validate_transform"
    And the transform prompt should mention "get_sample_rows"
