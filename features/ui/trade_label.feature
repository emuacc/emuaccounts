Feature: Transaction list labels a broker trade "trade"

  A broker buy or sell is one balanced transaction: the shares leg on the
  broker account, the cash leg on its :cash sub-account and the commission on
  expenses:fees:brokerage. With the broker account selected both asset legs
  are in scope, so the commission was the only candidate for the category
  column and every fill read as a fee. A transaction whose asset legs move one
  commodity in and a different one out is a trade and is labelled so; like a
  swap row the label is not a category-rule click target, and the commission
  shows on the fee line under the amount. Rows without that shape keep their
  contra account.

  Scenario: Broker fills read "trade" with their commission
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/broker_trade_rows.transactions"
    And I select the account "assets:equity:broker:demo"
    Then the transaction row with notes "Sell -100.0 NET @ 310.36 USD" should show the category "trade"
    And the transaction row with notes "Sell -100.0 NET @ 310.36 USD" should show the fee "fee 0.64 USD"
    And the category of the transaction row with notes "Sell -100.0 NET @ 310.36 USD" should not be editable
    And the transaction row with notes "Buy 100.0 NET @ 312.21 USD" should show the category "trade"

  Scenario: A single-commodity row keeps its contra account
    Given the app is running
    When I parse the transactions file "src-tauri/features/fixtures/broker_trade_rows.transactions"
    And I select the account "assets:equity:broker:demo"
    Then the transaction row with notes "Credit interest" should show the category "income:interest:broker"
    And the category of the transaction row with notes "Credit interest" should be editable
