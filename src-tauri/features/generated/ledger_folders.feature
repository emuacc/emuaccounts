Feature: Reports read exactly the ledger folders the pipeline wrote

  `generated/` can hold ledger folders the pipeline did not write this run:
  a folder an older pipeline wrote under a different case, a folder a git
  checkout from another machine brought along, a source folder deleted
  elsewhere. Counting them reports an account twice, and which copies exist
  depends on how the filesystem treats case, so one vault gave different tax
  figures on macOS and Linux (TASK-093). The pipeline renames and removes
  what it can; the loader reads only the folders `pipeline-metadata.json`
  names in `account_folders` and reports the rest.

  Scenario: A ledger folder the pipeline did not write is left out and reported
    Given a clean sources directory with a CSV "alex/crypto/wallet/alpha/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/crypto/wallet/alpha/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline
    And a generated file at "alex/crypto/wallet/stray/ledger.transactions" containing:
      """
      2025-01-16 * "Stray" "not written by the pipeline" ; txn:stray-001
          assets:crypto:wallet:stray 10 AUD
          expenses:unknown -10 AUD
      """
    And a generated file at "alex/crypto/wallet/stray/summary.json" containing:
      """
      {"transaction_count": 1, "balances": [{"account": "assets:crypto:wallet:stray", "totals": [{"commodity": "AUD", "amount": 10.0}]}]}
      """
    Then the active ledger should contain 1 transactions
    And the active ledger should not include payee "Stray"
    And the active ledger diagnostics should mention "alex/crypto/wallet/stray"
    And the account tree should not include "assets:crypto:wallet:stray"

  Scenario: A byte-identical transaction in two ledger files counts once and is reported
    Given a clean generated ledger directory
    And a generated file at "cash/bank/ledger.transactions" containing:
      """
      2025-01-15 * "Coffee Shop" "imported" ; txn:dup-001
          assets:cash:bank -4.50 AUD
          expenses:unknown 4.50 AUD
      """
    And a generated file at "cash/bank-old/ledger.transactions" containing:
      """
      2025-01-15 * "Coffee Shop" "imported" ; txn:dup-001
          assets:cash:bank -4.50 AUD
          expenses:unknown 4.50 AUD
      """
    Then the active ledger should contain 1 transactions
    And the active ledger diagnostics should mention "txn:dup-001"
    And the active ledger balance for "assets:cash:bank" should be "-4.50" "AUD"

  Scenario: A self-transfer shares its txn id across two wallets and is not a duplicate
    # Each wallet's ledger carries its own leg of the one on-chain transaction
    # (cross_folder_txn_propagation.feature); identity is the whole block, not
    # the id alone.
    Given a clean generated ledger directory
    And a generated file at "crypto/wallet/alpha/ledger.transactions" containing:
      """
      2025-01-15 * "Incoming from B" "imported" ; txn:shared-tx-001
          assets:crypto:wallet:alpha 100 AUD
          assets:transfer -100 AUD
      """
    And a generated file at "crypto/wallet/beta/ledger.transactions" containing:
      """
      2025-01-15 * "Outgoing to A" "imported" ; txn:shared-tx-001
          assets:crypto:wallet:beta -100 AUD
          assets:transfer 100 AUD
      """
    Then the active ledger should contain 2 transactions
    And the active ledger should have no diagnostics

  Scenario: A stale ledger folder is removed by the pipeline even when nothing else changed
    Given a clean sources directory with a CSV "alex/crypto/wallet/alpha/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/crypto/wallet/alpha/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline
    And a generated file at "alex/crypto/wallet/old/ledger.transactions" containing:
      """
      2025-01-16 * "Old" "left by an older pipeline" ; txn:old-001
          assets:crypto:wallet:old 10 AUD
          expenses:unknown -10 AUD
      """
    And a generated file at "alex/crypto/wallet/old/summary.json" containing:
      """
      {"transaction_count": 1, "balances": []}
      """
    And I run the pipeline again
    Then the pipeline should have early-exited
    And no generated file should exist at "alex/crypto/wallet/old/ledger.transactions"
    And the pipeline warnings should mention "Removed stale ledger folder generated/alex/crypto/wallet/old"
    And the active ledger should contain 1 transactions
    And the active ledger should have no diagnostics

  Scenario: A wrong-case ledger folder becomes the source folder's case on every filesystem
    # On a case-insensitive filesystem the pipeline's write would land inside
    # Foo/ and git would never see a rename; on a case-sensitive one it would
    # create a foo/ sibling and both would be read. Either way the result must
    # be one folder named foo with one copy of the transactions.
    Given a clean sources directory with a CSV "alex/crypto/wallet/foo/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/crypto/wallet/foo/_transform.rhai" that maps Date/Description/Amount to AUD
    And a generated file at "alex/crypto/wallet/Foo/ledger.transactions" containing:
      """
      2025-01-15 * "Coffee Shop" "imported" ; txn:older-pipeline
          assets:crypto:wallet:foo -4.50 AUD
          expenses:unknown 4.50 AUD
      """
    When I run the pipeline
    Then the generated folder "alex/crypto/wallet" should contain exactly "foo"
    And account_folders should map "assets:crypto:wallet:foo" to "alex/crypto/wallet/foo"
    And the pipeline warnings should mention "Renamed generated/alex/crypto/wallet/Foo to generated/alex/crypto/wallet/foo"
    And the active ledger should contain 1 transactions
    And the active ledger should have no diagnostics
