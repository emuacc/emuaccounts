Feature: Processing pipeline

  Scenario: Simple CSV is transformed into ledger transactions
    Given a clean sources directory with a CSV "commbank/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
      | 2025-01-16 | Salary      | 3500   |
    And a transform at "commbank/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202501"
    Then the active ledger should contain 2 transactions
    And the active ledger should include payee "Coffee Shop"
    And the active ledger should include payee "Salary"

  Scenario: Transform hierarchy — child folder transform overrides parent
    Given a clean sources directory
    And an institution transform at "commbank/_transform.rhai" using account "assets:bank:commbank"
    And an account transform at "commbank/credit/_transform.rhai" using account "liabilities:credit:commbank"
    And a CSV "commbank/credit/2025-01.csv" with one row
    When I run the pipeline
    Then the transaction should use account "assets:credit"

  Scenario: Manual transactions are merged with CSV transactions
    Given a clean sources directory with a CSV and transform
    And a "manual.transactions" file with payee "ManualEntry"
    When I run the pipeline
    Then the active ledger should include payee from CSV
    And the active ledger should include payee "ManualEntry"

  Scenario: Build cache skips unchanged CSVs
    Given a clean sources directory with a CSV and transform
    When I run the pipeline twice
    Then the second run should report 0 CSVs transformed and all cached

  Scenario: Changing a transform invalidates cache
    Given a pipeline has been run once with a transform
    When the transform is modified
    And I run the pipeline again
    Then the CSV should be re-transformed (not cached)

  Scenario: Stable transaction IDs across rebuilds
    Given a clean sources directory with a CSV and transform
    When I run the pipeline twice
    Then all transaction txn: IDs should be identical between runs

  Scenario: Per-folder summary.json is byte-identical across reruns
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount  |
      | 2025-01-15 | Coffee Shop |   -4.50 |
      | 2025-01-16 | Salary      | 3500.00 |
      | 2025-01-17 | Refund      |   12.34 |
      | 2025-01-18 | Groceries   |  -82.10 |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline twice
    Then the per-folder summary.json should be byte-identical between runs

  Scenario: Deterministic output ordering
    Given CSVs from two sources with interleaved dates
    When I run the pipeline
    Then transactions should be sorted by date, then by source path

  Scenario: Monthly rotation of output
    Given CSVs with transactions in January and February
    When I run the pipeline for month "202502"
    Then January transactions should be in archive/ledger-202501.transactions
    And February transactions should be in ledger.transactions

  Scenario: Loading ledger includes archived transactions
    Given CSVs with transactions in January and February
    When I run the pipeline for month "202502"
    Then loading all ledgers should include payee "January Item"
    And loading all ledgers should include payee "February Item"

  Scenario: Account declarations are written to output
    Given an "accounts.transactions" file declaring "assets:bank:commbank AUD"
    When I run the pipeline
    Then the output should include the account declaration

  Scenario: Import CSV copies file to account folder and rebuilds
    Given a clean sources directory with a transform at "bank/commbank/_transform.rhai"
    And a CSV fixture file "sample.csv"
    When I import the CSV to account folder "bank/commbank"
    Then the file should exist in "bank/commbank/sample.csv"
    And the pipeline should have transformed 1 CSV

  Scenario: Pipeline early-exits when nothing changed
    Given a clean sources directory with a CSV "commbank/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "commbank/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202501"
    And I run the pipeline again
    Then the pipeline should have early-exited

  Scenario: The trade-link store under sources does not defeat the early exit
    Given a clean sources directory with a CSV "commbank/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "commbank/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202501"
    And the trade-link store is rewritten under sources
    And I run the pipeline again
    Then the pipeline should have early-exited

  Scenario: Frontend rebuild_pipeline command respects cache
    Given a clean sources directory with a CSV "commbank/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "commbank/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202501"
    And I run the pipeline again as the frontend would
    Then the pipeline should have early-exited

  Scenario: Add account creates nested explicit-layout folder and appears in owner_accounts
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I add account "assets:crypto:wallet:ethereum" with currency "ETH" to account set "alex"
    Then the folder "alex/assets/crypto/wallet/ethereum/imports" should exist under sources
    And owner_accounts for "alex" should include "assets:crypto:wallet:ethereum"
    And the ledger for set "alex" should include a balance for "assets:crypto:wallet:ethereum"

  Scenario: Explicit-root folder derives its account without set stripping
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    And an empty folder "acme/assets/crypto/ethereum/0xAbC" under sources
    When I run the pipeline for month "202501"
    Then owner_accounts for "acme" should include "assets:crypto:ethereum:0xAbC"

  Scenario: Bare entity assets folder does not become an account
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    And an empty folder "acme/assets" under sources
    When I run the pipeline for month "202501"
    Then owner_accounts for "acme" should not include "assets"

  Scenario: Bare entity liabilities folder does not become an account
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    And an empty folder "acme/liabilities" under sources
    When I run the pipeline for month "202501"
    Then owner_accounts for "acme" should not include "liabilities"

  Scenario: CSV under an entity liabilities folder posts to the liabilities account
    Given a clean sources directory with a CSV "alex/liabilities/credit-card/amex/2025-01.csv":
      | Date       | Description | Amount  |
      | 2025-01-15 | Groceries   | -120.00 |
    And a transform at "alex/liabilities/credit-card/amex/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202501"
    Then the transaction should use account "liabilities:credit-card:amex"
    And the active ledger should include payee "Groceries"
    And account_folders should map "liabilities:credit-card:amex" to "alex/liabilities/credit-card/amex"
    And owner_accounts for "alex" should include "liabilities:credit-card:amex"

  Scenario: OFX under an entity liabilities folder resolves the declared liabilities account
    Given a clean sources directory with an OFX file "alex/liabilities/credit-card/amex/statement.ofx":
      """
      OFXHEADER:100
      DATA:OFXSGML

      <OFX>
      <BANKMSGSRSV1>
      <STMTTRNRS>
      <STMTRS>
      <CURDEF>AUD
      <BANKTRANLIST>
      <STMTTRN>
      <TRNTYPE>DEBIT
      <DTPOSTED>20250115
      <TRNAMT>-120.00
      <FITID>AMEX001
      <MEMO>Groceries
      </STMTTRN>
      </BANKTRANLIST>
      </STMTRS>
      </STMTTRNRS>
      </BANKMSGSRSV1>
      </OFX>
      """
    When an account declaration "account liabilities:credit-card:amex AUD" is appended to "alex/liabilities/credit-card/amex/accounts.transactions"
    And I run the pipeline for month "202501"
    Then the transaction should use account "liabilities:credit-card:amex"
    And account_folders should map "liabilities:credit-card:amex" to "alex/liabilities/credit-card/amex"

  Scenario: Add account with a liabilities name creates its explicit-layout folder
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I add account "liabilities:credit-card:amex" with currency "AUD" to account set "alex"
    Then the folder "alex/liabilities/credit-card/amex/imports" should exist under sources
    And account_folders should map "liabilities:credit-card:amex" to "alex/liabilities/credit-card/amex"
    And owner_accounts for "alex" should include "liabilities:credit-card:amex"
    And the ledger for set "alex" should include a balance for "liabilities:credit-card:amex"

  Scenario: Liabilities declaration in the set's _accounts folder stays declaration-only
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When an account declaration "account liabilities:loan:car AUD" is appended to "alex/_accounts/accounts.transactions"
    And I run the pipeline
    Then account_folders should not contain "liabilities:loan:car"
    And loading the account tree should return balance for "liabilities:loan:car"

  Scenario: Three-level empty folder appears in owner_accounts
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    And an empty folder "alex/crypto/wallet/solana/9Abc9AbcWallet" under sources
    When I run the pipeline for month "202501"
    Then owner_accounts for "alex" should include "assets:crypto:wallet:solana:9Abc9AbcWallet"

  Scenario: Pipeline detects source changes on restart
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline
    Then the pipeline should have early-exited is false
    When I run the pipeline again
    Then the pipeline should have early-exited
    When a "manual.transactions" file is added to "alex/savings" with payee "OfflineAdd"
    And I run the pipeline again
    Then the pipeline should have early-exited is false
    And the active ledger should include payee "OfflineAdd"

  Scenario: Manual transaction is saved to the specified account folder
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I append a manual transaction with payee "BuyETH" to account folder "alex/ethereum"
    Then a "manual.transactions" file should exist in "alex/ethereum"
    And no "manual.transactions" file should exist at the sources root
    When I run the pipeline
    Then the active ledger should include payee "BuyETH"

  Scenario: Manual transactions get stable txn IDs for trade linking
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I append a manual transaction with payee "BuyETH" to account folder "alex/ethereum"
    And I run the pipeline
    Then the active ledger should contain "txn:man-"
    When I run the pipeline again
    Then the active ledger should contain "txn:man-"

  Scenario: Pipeline does not create stray top-level folders for nested accounts
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I append a manual transaction with payee "BuyETH" to account folder "alex/ethereum"
    And I run the pipeline
    Then the only top-level source folders should be "alex"

  Scenario: Second pipeline run skips unchanged output files
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202502"
    And the sources directory is touched to bypass global cache
    And I run the pipeline again
    Then the pipeline should report 0 output files written
    And the pipeline should report output files skipped > 0

  Scenario: Changing one CSV only rewrites affected output files
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    And a CSV "alex/checking/2025-02.csv":
      | Date       | Description | Amount |
      | 2025-02-15 | Rent        | -1200  |
    And a transform at "alex/checking/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202503"
    Then the pipeline should report output files written > 0
    When a CSV "alex/savings/2025-01.csv" is modified:
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -5.00  |
    And I run the pipeline again
    Then the pipeline should report output files written > 0
    And the pipeline should report output files skipped > 0

  Scenario: Stale output files are pruned when source folder is removed
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    And a CSV "alex/checking/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-20 | Groceries   | -50    |
    And a transform at "alex/checking/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202502"
    Then the active ledger should include payee "Groceries"
    When the source folder "alex/checking" is removed
    And I run the pipeline again
    Then the active ledger should not include payee "Groceries"

  Scenario: Manual-only folder is registered in account_folders and ledger
    Given a clean sources directory
    When a "manual.transactions" file is added to "alex/crypto/wallet/generic" with payee "Osmosis DEX"
    And I run the pipeline
    Then owner_accounts for "alex" should include "assets:crypto:wallet:generic"
    And account_folders should map "assets:crypto:wallet:generic" to "alex/crypto/wallet/generic"
    And the active ledger should include payee "Osmosis DEX"

  Scenario: Declaration-only account is registered in account_folders and account tree
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I add account "assets:crypto:wallet:ethereum" with currency "ETH" to account set "alex"
    Then account_folders should map "assets:crypto:wallet:ethereum" to "alex/assets/crypto/wallet/ethereum"
    And loading the account tree should return balance for "assets:crypto:wallet:ethereum"

  Scenario: Declaration-only account_folders mapping survives pipeline early-exit
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I add account "assets:crypto:wallet:ethereum" with currency "ETH" to account set "alex"
    And I run the pipeline again as the frontend would
    Then the pipeline should have early-exited
    And account_folders should map "assets:crypto:wallet:ethereum" to "alex/assets/crypto/wallet/ethereum"

  Scenario: Non-asset account is declaration-only under the set's _accounts folder
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When I add account "expenses:office:supplies" with currency "AUD" to account set "alex"
    Then the folder "alex/_accounts" should exist under sources
    And the folder "alex/_accounts/imports" should not exist under sources
    And account_folders should not contain "expenses:office:supplies"
    And loading the account tree should return balance for "expenses:office:supplies"

  Scenario: Sub-account declared in a parent folder maps to that folder
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When an account declaration "account assets:savings:goal" is appended to "alex/savings/accounts.transactions"
    And I run the pipeline
    Then account_folders should map "assets:savings:goal" to "alex/savings"
    And owner_accounts for "alex" should include "assets:savings:goal"

  Scenario: Two account sets with the same bank folder each keep their folder and declaration
    # folder_to_account_name drops the entity, so alex/cash/bank/cba and
    # acme/cash/bank/cba both derive assets:cash:bank:cba. Keyed by that bare
    # name, one folder overwrote the other in pipeline-metadata.json: the loser's
    # ledger was read as an orphan and its account declaration never written to
    # its own accounts.transactions (TASK-107). Which one lost depended on
    # processing order, so both the cold build and a rebuild with one folder
    # changed (the other goes through the unchanged-folder path) are checked.
    Given a clean sources directory with a CSV "acme/cash/bank/cba/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "acme/cash/bank/cba/_transform.rhai" that maps Date/Description/Amount to AUD
    And a CSV "alex/cash/bank/cba/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-20 | Groceries   | -50.00 |
    And a transform at "alex/cash/bank/cba/_transform.rhai" that maps Date/Description/Amount to AUD
    When I run the pipeline for month "202501"
    Then account_folders should map "assets:cash:bank:cba" to "acme/cash/bank/cba"
    And account_folders should map "assets:cash:bank:cba" to "alex/cash/bank/cba"
    And the generated file "acme/accounts.transactions" should contain "account assets:cash:bank:cba"
    And the generated file "alex/accounts.transactions" should contain "account assets:cash:bank:cba"
    And the ledger for set "acme" should include a balance for "assets:cash:bank:cba"
    And the ledger for set "alex" should include a balance for "assets:cash:bank:cba"
    And the ledger for set "acme" should have no diagnostics
    And the ledger for set "alex" should have no diagnostics
    When a "manual.transactions" file is added to "alex/cash/bank/cba" with payee "Rent"
    And I run the pipeline again
    Then account_folders should map "assets:cash:bank:cba" to "acme/cash/bank/cba"
    And account_folders should map "assets:cash:bank:cba" to "alex/cash/bank/cba"
    And the generated file "acme/accounts.transactions" should contain "account assets:cash:bank:cba"
    And the ledger for set "acme" should have no diagnostics
    And the ledger for set "alex" should have no diagnostics

  Scenario: Unchanged manual.transactions uses cache
    Given a clean sources directory with a CSV "alex/savings/2025-01.csv":
      | Date       | Description | Amount |
      | 2025-01-15 | Coffee Shop | -4.50  |
    And a transform at "alex/savings/_transform.rhai" that maps Date/Description/Amount to AUD
    When a "manual.transactions" file is added to "alex/savings" with payee "ManualEntry"
    And I run the pipeline twice
    Then the second run should report 0 CSVs transformed and all cached
    And the active ledger should include payee "ManualEntry"

  Scenario: Shared txn_id rows are not cloned by equity swap propagation
    Given a clean sources directory with a CSV "alex/eth/txns.csv":
      | Date       | Description | Amount | TxHash  |
      | 2025-01-10 | PoolRouter  | -5.00  | 0xabc01 |
      | 2025-01-10 | DEXPool     | -80.00 | 0xabc01 |
      | 2025-01-10 | PoolRouter  | 3.00   | 0xabc01 |
    And a txn_id-aware transform at "alex/eth/_transform.rhai"
    And a rules file at "alex/eth/_rules.json" matching "*DEXPool*" with account "equity:trading:sell"
    When I run the pipeline
    Then the folder ledger "eth" should contain 3 transactions
    And the folder ledger "eth" should contain payee "PoolRouter"
    And the folder ledger "eth" should contain payee "DEXPool"
