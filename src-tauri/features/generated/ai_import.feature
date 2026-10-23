@ai_import
Feature: AI document import

  Any document (text-layer PDF, plain text) is interpreted by the AI into a
  normalised CSV for the target account folder. Rust re-validates everything —
  row shapes, dates, amounts, and the running-balance chain — feeding
  row-precise errors back for a bounded number of retries. Nothing reaches
  sources/ without passing validation and the user's preview confirmation.

  Scenario: Valid extraction via a stub AI server
    Given a stub AI server that extracts a valid two-row statement
    When I run AI document import for a dividend statement
    Then the document import should succeed with 2 rows
    And the document import filename should be "test-statement-2026.csv"

  Scenario: Broken balance chain is retried with row-precise errors
    Given a stub AI server whose first extraction has a broken balance chain
    When I run AI document import for a dividend statement
    Then the document import should succeed with 2 rows
    And the stub server should have received 2 requests

  Scenario: Persistently broken extraction fails cleanly
    Given a stub AI server that always returns a broken balance chain
    When I run AI document import for a dividend statement
    Then the document import should fail mentioning "balance chain"

  Scenario: Unparseable reply is retried then fails cleanly
    Given a stub AI server that always replies with unparseable content
    When I run AI document import for a dividend statement
    Then the document import should fail mentioning "JSON"
    And the stub server should have received 3 requests

  Scenario: Confirmed extraction is staged and drained through the pipeline
    Given a clean sources directory
    And an institution transform at "assets-bank-test/_transform.rhai" using account "assets:bank:test"
    When I stage an AI import of 2 rows as "test-statement-2026.csv"
    Then the staged CSV should exist in the folder's imports directory
    And the original document should be filed under originals
    When the account folder imports are processed
    Then the active ledger should contain 2 transactions

  Scenario: Re-importing the same document filename is refused
    Given a clean sources directory
    And an institution transform at "assets-bank-test/_transform.rhai" using account "assets:bank:test"
    When I stage an AI import of 2 rows as "test-statement-2026.csv"
    And the account folder imports are processed
    Then staging the same AI import again should fail mentioning "already exists"

  Scenario: Prompt reproduces existing folder shape and offers tools
    Given an existing folder CSV shape with headers "Date,Transaction details,Amount,Balance"
    When the AI document import prompt is built
    Then the document import prompt should mention "Transaction details"
    And the document import prompt should mention "validate_rows"
    And the document import prompt should mention "get_document_text"
