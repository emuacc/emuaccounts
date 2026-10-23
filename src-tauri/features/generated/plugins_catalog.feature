Feature: Plugin catalog install

  The app ships a read-only plugin catalog (bundled resources). "Add Plugins"
  lists catalog entries with their installed state and installs one by copying
  it into the vault's plugins/ directory, excluding runtime data.

  Scenario: List available plugins from the catalog with installed state
    Given a plugin catalog with a plugin "hello" with manifest:
      """
      [plugin]
      name = "Hello"
      version = "0.1.0"
      script = "sync.py"
      """
    And a plugin catalog with a plugin "prices" with manifest:
      """
      [plugin]
      name = "Prices"
      version = "0.2.0"
      script = "sync.py"
      daily = true
      """
    And a plugins directory with a plugin "prices" with manifest:
      """
      [plugin]
      name = "Prices"
      version = "0.2.0"
      script = "sync.py"
      daily = true
      """
    When I list available plugins
    Then available plugin "hello" should be listed as not installed
    And available plugin "prices" should be listed as installed

  Scenario: Installing a plugin copies it excluding runtime data
    Given a plugin catalog with a plugin "hello" with manifest:
      """
      [plugin]
      name = "Hello"
      version = "0.1.0"
      script = "sync.py"
      """
    And the catalog plugin "hello" has a file "lib/util.py"
    And the catalog plugin "hello" has a file ".data/token.json"
    And the catalog plugin "hello" has a file "__pycache__/x.pyc"
    And the catalog plugin "hello" has a file "wallets.json"
    And the catalog plugin "hello" has a file "accounts.json"
    And the catalog plugin "hello" has a file ".gitignore"
    When I install plugin "hello"
    Then the installed plugin "hello" should contain "plugin.toml"
    And the installed plugin "hello" should contain "lib/util.py"
    And the installed plugin "hello" should not contain ".data/token.json"
    And the installed plugin "hello" should not contain "__pycache__/x.pyc"
    And the installed plugin "hello" should not contain "wallets.json"
    And the installed plugin "hello" should not contain "accounts.json"
    And the installed plugin "hello" should not contain ".gitignore"

  Scenario: Installing an already-installed plugin is refused
    Given a plugin catalog with a plugin "hello" with manifest:
      """
      [plugin]
      name = "Hello"
      version = "0.1.0"
      script = "sync.py"
      """
    When I install plugin "hello"
    And I install plugin "hello"
    Then the install should fail with "already installed"

  Scenario: Installing a plugin missing from the catalog fails
    Given a plugin catalog with a plugin "hello" with manifest:
      """
      [plugin]
      name = "Hello"
      version = "0.1.0"
      script = "sync.py"
      """
    When I install plugin "nope"
    Then the install should fail with "not found"

  Scenario: Install rejects a path-traversal plugin name
    Given a plugin catalog with a plugin "hello" with manifest:
      """
      [plugin]
      name = "Hello"
      version = "0.1.0"
      script = "sync.py"
      """
    When I install plugin "../evil"
    Then the install should fail with "invalid plugin name"

  Scenario: The bundled catalog marks every price plugin daily
    The catalog is the deploy source: a vault that installs or updates a
    plugin gets the catalog manifest verbatim, so the daily flag has to be set
    here — not in a vault's installed copy — for the nightly emuacc-update /
    "update prices on startup" batch to keep running it after a redeploy.

    Given the bundled plugin catalog
    When I list available plugins
    Then available plugin "crypto-spot-prices" should be listed as daily
    And available plugin "fiat-prices" should be listed as daily
    And available plugin "stock-prices" should be listed as daily
    And available plugin "non-usd-prices" should be listed as daily

  Scenario: The transaction-sync plugins are daily wherever they are bundled
    The public repository carries only the price plugins and the example
    (PUBLIC_PLUGINS in copy.bara.sky): the transaction-sync plugins need
    per-provider API keys and stay in the private repository, whose nightly
    emuacc-update depends on their daily flag. So they are asserted only where
    the catalog has them, and this scenario passes unchanged in both repos.

    Given the bundled plugin catalog
    When I list available plugins
    Then available plugin "crypto-sync" should be listed as daily when bundled
    And available plugin "ibkr-flex-sync" should be listed as daily when bundled
    And available plugin "simplefin-sync" should be listed as daily when bundled
