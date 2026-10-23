Feature: Edit Rule pre-populates narration and payee

  Clicking "Edit Rule" inside an expanded transaction row opens the rule
  editor with a draft seeded from that transaction. The narration must
  populate the rule pattern, AND the payee must populate the rule's
  payee condition — the display label by default, or the raw address
  when no label exists. Without the payee condition the rule is too
  broad and would match unrelated counterparties.

  Wildcards in the seeded payee condition (`*Kraken*` for substring
  match) must remain visible in the pill so the user can see whether
  the rule will substring-match unrelated counterparties.

  Scenario: Edit Rule seeds both narration and payee
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I select the account "assets:exchange:kraken:btc"
    And I expand the transaction row for payee "Kraken"
    And I click "Edit Rule" in the row detail
    Then the rule editor pattern should contain "Sell BTC"
    And the rule editor should show the pill "payee:*Kraken*"

  # Regression: clicking a row whose (txn, datetime, narration, amount)
  # tuple collides with another row's (e.g. two legs of one Raydium swap)
  # must expand ONLY the clicked row. The previous behaviour expanded both
  # twins — and morphdom, which keys txn-row identity on `data-expand-key`,
  # collapsed the pair so the user could not reach Edit Rule on the second.
  #
  # @wip: with the harness now feeding fixtures through the REAL pipeline
  # (manual.transactions ingestion) instead of parse_transactions_file, the
  # twin rows double-expand again — the expand-key fix does not hold for
  # pipeline-ingested manual duplicates (no per-leg id allocation on this
  # path). Tracked in TASK-049; not part of the Edit Party work.
  @wip
  Scenario: clicking a row expands only that row, not duplicates with the same on-chain hash
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/duplicate_txn_legs.transactions"
    And I select the account "assets:crypto:wallet:solana:mngo"
    And I click the second transaction row
    Then exactly one transaction detail row should be visible
    And the detail row should follow the second transaction row
    And the detail row should contain an "Edit Rule" button

  # EXCHANGE:SYMBOL commodities (Hyperliquid `xyz:WTIOIL`) contain ':' — the
  # "keyword: value" form typed into the rule editor must still become one pill.
  Scenario: Rule editor accepts an EXCHANGE:SYMBOL commodity condition
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/example.transactions"
    And I select the account "assets:exchange:kraken:btc"
    And I expand the transaction row for payee "Kraken"
    And I click "Edit Rule" in the row detail
    And I type "commodity: xyz:WTIOIL " into the rule editor pattern
    Then the rule editor should show the pill "commodity:xyz:WTIOIL"
