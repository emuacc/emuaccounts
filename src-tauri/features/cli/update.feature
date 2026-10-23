Feature: emuacc-update CLI keeps a vault's data fresh headlessly

  A cron/launchd timer runs `emuacc-update` to refresh the configured vault
  without launching the app: it runs every daily-flagged plugin (the same
  batch as "update prices on startup"), then rebuilds the generated output
  and reports so balances reflect the fresh data. The vault is resolved like
  the other CLIs: EMUACC_SOURCES_DIR/EMUACC_GENERATED_DIR env overrides win,
  otherwise the app config's current_root. The command is non-interactive:
  exit 0 on success, non-zero when any plugin or the rebuild fails,
  per-plugin outcomes on stdout, failures on stderr. The once-per-day guard
  makes a second run on the same day a cheap no-op that still exits 0.

  Background:
    Given a vault root with a "bank" CSV source and a transform

  Scenario: Fresh run executes daily plugins then rebuilds ledger and reports
    Given the vault has a daily plugin "price-writer" that writes "_prices/PLG.txt"
    When I run emuacc-update
    Then emuacc-update should exit with code 0
    And the emuacc-update stdout should report "price-writer" as "ok"
    And the file "_prices/PLG.txt" should exist in sources
    And the active ledger should contain 1 transactions
    And the vault reports should exist

  Scenario: Second run the same day skips the plugin and still exits 0
    Given the vault has a daily plugin "price-writer" that writes "_prices/PLG.txt"
    And the plugin "price-writer" last succeeded today
    When I run emuacc-update
    Then emuacc-update should exit with code 0
    And the emuacc-update stdout should report "price-writer" as "skipped"
    And the file "_prices/PLG.txt" should not exist in sources

  Scenario: --force reruns a plugin that already succeeded today
    Given the vault has a daily plugin "price-writer" that writes "_prices/PLG.txt"
    And the plugin "price-writer" last succeeded today
    When I run emuacc-update with --force
    Then emuacc-update should exit with code 0
    And the emuacc-update stdout should report "price-writer" as "ok"
    And the file "_prices/PLG.txt" should exist in sources

  Scenario: A failing plugin exits non-zero without blocking others or the rebuild
    Given the vault has a daily plugin "a-failing" that fails with "boom"
    And the vault has a daily plugin "b-writer" that writes "_prices/B.txt"
    When I run emuacc-update
    Then emuacc-update should exit with a non-zero code
    And the emuacc-update stderr should mention "a-failing"
    And the emuacc-update stdout should report "a-failing" as "failed"
    And the emuacc-update stdout should report "b-writer" as "ok"
    And the file "_prices/B.txt" should exist in sources
    And the active ledger should contain 1 transactions

  Scenario: A partial-success plugin (exit 2) counts as OK for the run
    Given the vault has a daily plugin "partial-writer" that writes "_prices/P.txt" then exits 2
    When I run emuacc-update
    Then emuacc-update should exit with code 0
    And the emuacc-update stdout should report "partial-writer" as "partial"
    And the file "_prices/P.txt" should exist in sources
