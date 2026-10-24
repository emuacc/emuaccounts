Feature: Payee labels

  Labels in _labels.json are pre-pass transforms that rename payees
  and commodities before categorization rules are applied.
  Clicking a payee in the UI should save a label, not a rule.

  Scenario: Label renames payee via _labels.json
    Given a clean sources directory with a CSV "bank/2025-01.csv":
      | Date       | Description     | Amount |
      | 2025-01-15 | Coffee purchase | -4.50  |
    And a transform at "bank/_transform.rhai" that maps Date/Description/Amount to AUD
    And a labels file at "bank/_labels.json" matching "*Coffee*" with payee "Coffee Shop"
    When I run the pipeline
    Then transactions with narration "imported" should have payee "Coffee Shop"

  Scenario: Label renames payee for manual transactions
    Given a clean sources directory
    And a "manual.transactions" file with payee "Coffee purchase"
    And a labels file at "bank/_labels.json" matching "*Coffee*" on field "payee" with payee "Coffee Shop"
    When I run the pipeline
    Then transactions with narration "manual test" should have payee "Coffee Shop"

  # Labels are first-match-wins over the merged chain (closest folder first,
  # file order within a folder). The party editor's preview mirrors this —
  # a row already renamed by an earlier label shows as shadowed.
  Scenario: Earlier label wins when two labels match
    Given a clean sources directory with a CSV "bank/2025-01.csv":
      | Date       | Description     | Amount |
      | 2025-01-15 | Coffee purchase | -4.50  |
    And a transform at "bank/_transform.rhai" that maps Date/Description/Amount to AUD
    And a labels file at "bank/_labels.json" with payee labels:
      | pattern        | field | payee  |
      | *Coffee*       | payee | Cafe A |
      | *Coffee purch* | payee | Cafe B |
    When I run the pipeline
    Then transactions with narration "imported" should have payee "Cafe A"

  Scenario: Label renames payee for OFX transactions
    Given a clean sources directory with an OFX file "bank/savings/statement.ofx":
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
      <TRNAMT>-10.40
      <FITID>TXN001
      <MEMO>Google Workspace_x.c Sydney AU Card xx1234 Value Date: 15/01/2025
      </STMTTRN>
      </BANKTRANLIST>
      </STMTRS>
      </STMTTRNRS>
      </BANKMSGSRSV1>
      </OFX>
      """
    And an accounts file declaring "assets:bank:savings AUD"
    And a labels file at "bank/_labels.json" matching "*Google Workspace*" on field "payee" with payee "Google Australia"
    When I run the pipeline for month "202501"
    Then transactions with narration "Google Workspace_x.c Sydney AU Card xx1234 Value Date: 15/01/2025" should have payee "Google Australia"

  # Saving, editing and deleting a label each load the folder's _labels.json,
  # change it and write it back. A file that is there but does not parse (a
  # hand edit with a trailing comma, sync conflict markers) must stop the
  # change, or the write would replace every other label with just this one.
  Scenario Outline: A label change leaves an unparseable labels file untouched
    Given a clean sources directory
    And a labels file at "bank/_labels.json" with raw contents:
      """
      {"labels": [{"id": "label-keep", "pattern": "*Coffee*", "payee": "Coffee Shop"},]}
      """
    When I try to <change> in folder "bank"
    Then the operation should fail with "bank/_labels.json"
    And the labels file "bank/_labels.json" should be unchanged

    Examples:
      | change                                                          |
      | save a label matching "*Tea*" with payee "Tea House"            |
      | edit label "label-keep" to match "*Coffee*" with payee "Cafe"   |
      | delete label "label-keep"                                       |
