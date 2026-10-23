Feature: Edit Party label editor

  Clicking "Edit Party" inside an expanded transaction row opens the
  party (label) editor: a glob pattern seeded from the row's payee, an
  output Party name, and a live preview of matching transactions. The
  seeded pattern covers only the clicked row's memo shape; widening the
  glob must raise the match count so one label can rename a merchant
  with several bank-descriptor shapes. The editor also lists the
  folder's existing party labels so stale one-off labels can be
  deleted.

  Scenario: detail row offers an Edit Party button
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/party_shapes.transactions"
    And I select the account "assets:cash:bank:cheq"
    And I expand the transaction row for payee "ANTHROPIC"
    Then the detail row should contain an "Edit Party" button

  Scenario: Edit Party opens the party editor seeded from the row
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/party_shapes.transactions"
    And I select the account "assets:cash:bank:cheq"
    And I expand the transaction row for payee "ANTHROPIC"
    And I click "Edit Party" in the row detail
    Then the party editor pattern should contain "ANTHROPIC"
    And the party editor should report 1 matching transaction
    And the party editor should show the existing labels section

  Scenario: widening the pattern raises the match count
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/party_shapes.transactions"
    And I select the account "assets:cash:bank:cheq"
    And I expand the transaction row for payee "ANTHROPIC"
    And I click "Edit Party" in the row detail
    And I replace the party editor pattern with "*ANTHROPIC*"
    Then the party editor should report 2 matching transactions

  Scenario: applying a label closes the editor
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/party_shapes.transactions"
    And I select the account "assets:cash:bank:cheq"
    And I expand the transaction row for payee "ANTHROPIC"
    And I click "Edit Party" in the row detail
    And I replace the party editor pattern with "*ANTHROPIC*"
    And I set the party name to "Anthropic"
    And I apply the party label
    Then the party editor should be closed
    And a transaction row should show the party "Anthropic"
