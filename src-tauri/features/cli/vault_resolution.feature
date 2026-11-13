Feature: The emuacc-* CLIs work on the vault the app has open

  Every CLI tool resolves the vault the same way: EMUACC_SOURCES_DIR /
  EMUACC_GENERATED_DIR env overrides win, otherwise the current_root in the
  app config (config.json in the platform app-data folder, or in the folder
  EMUACC_APP_DATA_DIR names, which is how these scenarios sandbox it).
  emuacc-import and emuacc-gaps used to look in the app-data folder of the
  bundle id from before the rename and never read current_root (TASK-088).

  Background:
    Given a vault root with a "bank" CSV source and a transform

  Scenario: emuacc-import imports into the vault the app config points at
    Given app settings whose current root is that vault
    And a pending import "2025-02.csv" in folder "bank" with payee "Imported Entry"
    When I run emuacc-import with no vault env overrides
    Then emuacc-import should exit with code 0
    And the file "bank/2025-02.csv" should exist in sources
    And the generated ledger should contain a transaction with payee "Imported Entry"

  Scenario: The env overrides win over the app config's current root
    Given app settings whose current root is an empty vault elsewhere
    And a pending import "2025-02.csv" in folder "bank" with payee "Imported Entry"
    When I run emuacc-import with EMUACC_SOURCES_DIR and EMUACC_GENERATED_DIR set to the vault
    Then emuacc-import should exit with code 0
    And the generated ledger should contain a transaction with payee "Imported Entry"
    And the vault in the app settings should still be empty

  Scenario: emuacc-gaps reads the generated folder the app config points at
    Given app settings whose current root is that vault
    And the generated archive holds "assets:bank:test" postings in 2025-01 and 2025-03
    When I run emuacc-gaps with no vault env overrides
    Then emuacc-gaps should exit with code 0
    And the emuacc-gaps stdout should mention "Missing: 2025-02"
