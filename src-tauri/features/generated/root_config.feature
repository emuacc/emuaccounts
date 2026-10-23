Feature: Root folder configuration
  Users can choose a visible root folder (like Obsidian vaults) instead of
  using the hidden platform app-data directory.

  Scenario: No config file falls back to app_data_dir
    Given no root config file exists
    When I resolve the root directory
    Then the root should be None

  Scenario: Config file with current_root is loaded
    Given a root config file with current_root "/tmp/test-root"
    When I resolve the root directory
    Then the root should be "/tmp/test-root"
    And the known roots should contain "/tmp/test-root"

  Scenario: Setting a new root creates directories and persists config
    Given no root config file exists
    When I set the root directory to a temporary path
    Then the root config file should exist
    And the sources subdirectory should exist
    And the generated subdirectory should exist
    And a .gitignore should exist in the generated subdirectory

  Scenario: Setting a root adds it to known_roots
    Given a root config file with current_root "/tmp/old-root"
    When I set the root directory to a temporary path
    Then the known roots should contain the new path
    And the known roots should contain "/tmp/old-root"

  Scenario: Env var overrides config root for sources
    Given a root config file with current_root "/tmp/config-root"
    And the env var EMUACC_SOURCES_DIR is set to "/tmp/env-sources"
    When I resolve the sources directory
    Then the sources directory should be "/tmp/env-sources"

  Scenario: Env var overrides config root for generated
    Given a root config file with current_root "/tmp/config-root"
    And the env var EMUACC_GENERATED_DIR is set to "/tmp/env-generated"
    When I resolve the generated directory
    Then the generated directory should be "/tmp/env-generated"

  Scenario: Config root is used for sources when no env var
    Given a root config file with current_root "/tmp/config-root"
    When I resolve the sources directory without env var
    Then the sources directory should be "/tmp/config-root/sources"

  Scenario: Config root is used for generated when no env var
    Given a root config file with current_root "/tmp/config-root"
    When I resolve the generated directory without env var
    Then the generated directory should be "/tmp/config-root/generated"

  # --- strict resolution (app-data purity: GUI commands must never touch
  # app_data/sources|generated when no vault is selected) ---

  Scenario: Strict sources resolution with no root and no env var is an error
    Given no root config file exists
    When I strictly resolve the sources directory
    Then strict resolution should fail with "No vault selected"

  Scenario: Strict generated resolution with no root and no env var is an error
    Given no root config file exists
    When I strictly resolve the generated directory
    Then strict resolution should fail with "No vault selected"

  Scenario: Env var satisfies strict sources resolution without a root
    Given no root config file exists
    And the env var EMUACC_SOURCES_DIR is set to "/tmp/env-sources"
    When I strictly resolve the sources directory
    Then strict resolution should yield "/tmp/env-sources"

  Scenario: Env var satisfies strict generated resolution without a root
    Given no root config file exists
    And the env var EMUACC_GENERATED_DIR is set to "/tmp/env-generated"
    When I strictly resolve the generated directory
    Then strict resolution should yield "/tmp/env-generated"

  Scenario: Config root satisfies strict sources resolution
    Given a root config file with current_root "/tmp/config-root"
    When I strictly resolve the sources directory
    Then strict resolution should yield "/tmp/config-root/sources"

  Scenario: Config root satisfies strict generated resolution
    Given a root config file with current_root "/tmp/config-root"
    When I strictly resolve the generated directory
    Then strict resolution should yield "/tmp/config-root/generated"

  # --- update_prices_on_startup setting ---

  Scenario: Update-prices-on-startup defaults to false when no config exists
    Given no root config file exists
    When I resolve the root directory
    Then update prices on startup should be false

  Scenario: Update-prices-on-startup persists across save and reload
    Given a root config file with update prices on startup enabled
    When I resolve the root directory
    Then update prices on startup should be true

  Scenario: Setting a new root preserves the update-prices-on-startup preference
    Given a root config file with update prices on startup enabled
    When I set the root directory to a temporary path
    Then update prices on startup should be true

  Scenario: A legacy config without the startup flag still loads its root
    Given a raw config file containing:
      """
      {"current_root": "/tmp/legacy-root", "known_roots": ["/tmp/legacy-root"]}
      """
    When I resolve the root directory
    Then the root should be "/tmp/legacy-root"
    And update prices on startup should be false

  # --- generated/.gitignore ---
  # The ignore-all file is for a generated/ the app just created. A vault that
  # already tracks generated/ in git must not get one: it hid untracked
  # duplicate ledger folders from `git status` (TASK-093).

  Scenario: Setting a root whose generated folder is tracked by git writes no ignore-all .gitignore
    Given no root config file exists
    And a temporary root whose generated folder has a file tracked by git
    When I set the root directory to that path
    Then no .gitignore should exist in the generated subdirectory
