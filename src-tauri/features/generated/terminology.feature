Feature: Configurable display terminology
  Display wording such as the name for one holding ("asset") comes from a
  single catalogue, src-tauri/terminology.json, that both the Rust backend
  and the frontend read. The user can override any term's singular and
  plural form; overrides live under "terms" in the app config.json and are
  merged over the catalogue defaults, so a blank override means "default".

  Scenario: A fresh config uses the catalogue defaults
    Given no root config file exists
    When I resolve the display terms
    Then the term "asset" should read "asset" and "assets"
    And the term "asset" should carry a description

  Scenario: A configured override replaces the default wording
    Given a raw config file containing:
      """
      {"terms": {"asset": {"singular": "holding", "plural": "holdings"}}}
      """
    When I resolve the display terms
    Then the term "asset" should read "holding" and "holdings"
    And the default for term "asset" should still be "asset" and "assets"

  Scenario: A legacy config without terms loads with the defaults
    Given a raw config file containing:
      """
      {"current_root": "/tmp/legacy-root", "known_roots": ["/tmp/legacy-root"]}
      """
    When I resolve the display terms
    Then the term "asset" should read "asset" and "assets"

  Scenario: Saved overrides keep only real changes to known terms
    Given no root config file exists
    When I save term overrides:
      | key     | singular  | plural |
      | asset   |  holding  | assets |
      | unknown | foo       | foos   |
    Then the saved config should hold the term override "asset" with singular "holding" and plural ""
    And the saved config should hold no term override for "unknown"
    And the term "asset" should read "holding" and "assets"

  Scenario: Clearing an override restores the default and drops it from the config
    Given a raw config file containing:
      """
      {"terms": {"asset": {"singular": "holding", "plural": "holdings"}}}
      """
    When I save term overrides:
      | key   | singular | plural |
      | asset |          |        |
    Then the saved config should hold no term override for "asset"
    And the term "asset" should read "asset" and "assets"
