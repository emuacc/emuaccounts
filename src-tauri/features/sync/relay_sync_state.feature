Feature: Relay sync state is not pipeline input

  A relay sync rewrites the metadata document, the relay config and the blob
  store under sources/. None of that is pipeline input, so it must not defeat
  the pipeline's early exit.

  Scenario: Relay sync state under sources does not defeat the early exit
    Given a clean sources directory with a CSV "commbank/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "commbank/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202501"
    And the relay sync rewrites its state under sources
    And I run the pipeline again
    Then the pipeline should have early-exited
