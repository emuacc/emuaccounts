Feature: Hide and delete transactions

  Scenario: Ignored CSV transaction is excluded from pipeline output
    Given a clean sources directory with a CSV and transform
    And a "manual.transactions" file with payee "ShouldStay"
    When I run the pipeline
    Then the active ledger should include payee from CSV
    And the active ledger should include payee "ShouldStay"
    When I hide the CSV transaction
    And I run the pipeline
    Then the active ledger should not include payee from CSV
    And the active ledger should include payee "ShouldStay"

  Scenario: Per-folder ledger file no longer contains a hidden CSV transaction
    Given a clean sources directory with a CSV and transform
    When I run the pipeline
    And I hide the CSV transaction
    And I run the pipeline
    Then the per-folder ledger at "bank" should not contain payee "CSV Entry"

  Scenario: Hiding the same CSV transaction twice does not duplicate the ignored entry
    Given a clean sources directory with a CSV and transform
    When I run the pipeline
    And I hide the CSV transaction
    And I hide the CSV transaction
    Then the ignored file should have 1 entry

  Scenario: Hiding a transaction cleans up pre-existing duplicates in the ignored file
    Given a clean sources directory with a CSV and transform
    And the ignored file already contains "txn:dup-a" twice and "txn:dup-b" once
    When I run the pipeline
    And I hide the CSV transaction
    Then the ignored file should have 3 entry

  Scenario: Deleted manual transaction is excluded from pipeline output
    Given a clean sources directory with a CSV and transform
    And a "manual.transactions" file with payee "ManualDelete"
    When I run the pipeline
    Then the active ledger should include payee "ManualDelete"
    When I delete the manual transaction with payee "ManualDelete"
    And I run the pipeline
    Then the active ledger should not include payee "ManualDelete"
    And the active ledger should include payee from CSV

  Scenario: Hidden manual transaction stays hidden after an earlier one in the same file is deleted
    Given a clean sources directory with a CSV and transform
    And a "manual.transactions" file with transactions:
      | Date       | Payee  | Narration |
      | 2025-01-10 | First  | one       |
      | 2025-01-11 | Second | two       |
      | 2025-01-12 | Third  | three     |
    When I run the pipeline
    And I hide the manual transaction with payee "Third"
    And I run the pipeline
    Then exactly 1 transaction with payee "Third" should post to "ignore:hidden"
    When I note the txn ids of the manual transactions
    And I delete the manual transaction with payee "First"
    Then the active ledger should not include payee "First"
    And the noted txn ids of the remaining manual transactions should be unchanged
    And exactly 1 transaction with payee "Third" should post to "ignore:hidden"

  Scenario: Identical manual transactions get distinct ids and can be hidden one at a time
    Given a clean sources directory with a CSV and transform
    And a "manual.transactions" file with transactions:
      | Date       | Payee | Narration |
      | 2025-01-10 | Dup   | same      |
      | 2025-01-10 | Dup   | same      |
    When I run the pipeline
    Then the active ledger should contain 2 distinct txn ids for payee "Dup"
    When I hide the manual transaction with payee "Dup"
    And I run the pipeline
    Then exactly 1 transaction with payee "Dup" should post to "ignore:hidden"
    When I run the pipeline again
    Then the active ledger should contain 2 distinct txn ids for payee "Dup"
    And exactly 1 transaction with payee "Dup" should post to "ignore:hidden"

  Scenario: Hiding a transaction leaves an unparseable rules file untouched
    Given a clean sources directory
    And a rules file at "bank/_rules.json" with raw contents:
      """
      {"rules": [{"id": "keep", "pattern": "*coffee*", "amount_account": "expenses:coffee"},]}
      """
    When I try to hide txn id "txn:abc123" in folder "bank"
    Then the operation should fail with "bank/_rules.json"
    And the rules file "bank/_rules.json" should be unchanged

  Scenario: Ignoring one leg of an auto-linked swap keeps the partner leg
    # A legacy _ignored.txt id must equal a whole meta segment. The partner
    # leg carries swap:txn:<ignored id>, which a substring check also drops.
    # The second run takes the intact-folder path: _ignored.txt lives at the
    # sources root, so the wallet folder's fingerprint is unchanged.
    Given a clean sources directory
    And a swap CSV at "wallet/2024-05.csv" trading "USDC" for "SOL" with a distinct txn id per leg
    When I run the pipeline
    Then transactions with narration "on-chain swap" should have meta containing "swap:txn:"
    When I add the txn id of the transaction with payee "Sell USDC" to the ignored file
    And I run the pipeline
    Then the active ledger should not include payee "Sell USDC"
    And the active ledger should include payee "Buy SOL"

  Scenario: An ignored id does not drop the ids it is a prefix of
    Given a clean sources directory
    And a CSV "bank/2025-01.csv" with columns "Date,Description,Amount,TxHash":
      | Date       | Description | Amount | TxHash |
      | 2025-01-15 | Drop Me     | -10.00 | abc    |
      | 2025-01-16 | Keep Me     | -20.00 | abc1   |
    And a transform with txn_id at "bank/_transform.rhai"
    When I run the pipeline
    And I add "txn:abc" to the ignored file
    And I run the pipeline
    Then the active ledger should not include payee "Drop Me"
    And the active ledger should include payee "Keep Me"
