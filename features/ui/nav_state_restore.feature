@wip
Feature: Navigation state is restored per vault

  Navigation state (selected account, drill path, search) is persisted in the
  vault at generated/.cache/ui-state.json — never in browser storage, which is
  shared across app instances and vaults. Reloading the app on the same vault
  restores where you were; a different vault (or no vault) never shows another
  vault's state.

  Scenario: Heading matches sidebar after reload
    Given the app is running
    When I parse the transactions file "sample-data/example.transactions"
    And I select the account "assets:cash:usd"
    Then the heading should show "cash:usd"
    And the sidebar should highlight "assets:cash:usd"
    When the app is reloaded
    Then the heading should show "cash:usd"
    And the sidebar should highlight "assets:cash:usd"

  Scenario: Saved account that no longer exists falls back with matching sidebar
    Given the app is running
    When I parse the transactions file "sample-data/example.transactions"
    And the vault ui-state has selectedAccount "assets:deleted:account" and drillPath ["deleted"]
    When the app is reloaded
    Then the heading and sidebar should show the same account

  Scenario: Fresh instance with no vault shows the picker with no account heading
    # The original cross-vault leak: another instance's nav state must never
    # paint before (or instead of) the vault picker.
    Given a previous session on another vault selected the account "assets:equity:broker:ibkr"
    And the app is running with no vault configured
    Then the vault picker should be shown
    And no account heading should be rendered

  Scenario: Switching vaults never carries the previous vault's selection
    Given the app is running
    When I parse the transactions file "sample-data/example.transactions"
    And I select the account "assets:cash:usd"
    And I switch to an empty vault
    Then the heading should not show "cash:usd"
