Feature: Trade linking

  Scenario: Save and retrieve a trade link
    Given a trade link store
    When I save a trade link between "txn:sell-eth" and "txn:buy-usdc"
    Then get_trade_links should return 1 link
    And the trade link should pair "txn:sell-eth" with "txn:buy-usdc"

  Scenario: Deterministic trade link ID
    Given a trade link store
    When I save a trade link between "txn:aaa" and "txn:bbb"
    And I save a trade link between "txn:bbb" and "txn:aaa"
    Then get_trade_links should return 1 link

  Scenario: Delete a trade link
    Given a trade link store
    When I save a trade link between "txn:sell-eth" and "txn:buy-usdc"
    And I delete that trade link
    Then get_trade_links should return 0 links

  Scenario: Suggest trade links from exchange transactions
    Given a clean sources directory with exchange transactions
    When I run the pipeline
    And I request trade link suggestions
    Then I should receive 1 trade suggestion
    And the suggestion should pair the ETH sell with the USDC buy

  Scenario: Already-linked transactions are not re-suggested
    Given a clean sources directory with exchange transactions
    When I run the pipeline
    And I save a trade link between the two exchange transactions
    And I request trade link suggestions
    Then I should receive 0 trade suggestions

  Scenario: Zero-amount transactions are never suggested as trades
    Given a clean sources directory with exchange transactions including zero amounts
    When I run the pipeline
    And I request trade link suggestions
    Then I should receive 0 trade suggestions

  Scenario: Auto-linked swap adds price annotation from denominator side
    Given a transactions file named "fifo_pooled_equity_swap.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given the transactions are auto-linked for equity swaps with prices for "USDT" at "1.50" in "AUD"
    Then the SWELL posting should have a price annotation

  Scenario: Pre-linked swap (with :sell/:buy) adds price from counterpart
    Given a transactions file named "fifo_swap_prelinked.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given the transactions are auto-linked for equity swaps with prices for "SOL" at "200.00" in "AUD"
    Then the SKBDI posting should have a price annotation

  Scenario: Unpriced token swap with a 0-amount sibling indexed last stays unpriced
    # Three legs share one on-chain hash: the ABC sell, the XYZ buy and a
    # 0-amount ETH contract-interaction leg listed last. ETH has a price feed,
    # the two tokens do not. Resolving `swap:txn:0xshared` to the last leg
    # under that hash values the partner at convert(ETH, 0) = 0 and stamps
    # `@ 0.000000 AUD` on both swap legs: a zero cost base and zero proceeds
    # in the CGT report. The legs are linked, and stay unpriced.
    Given a transactions file named "swap_zero_sibling_unpriced.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given the transactions are auto-linked for equity swaps with prices for "ETH" at "5000.00" in "AUD"
    Then the first transaction meta should include "swap_partner_commodity:XYZ"
    And posting 1 of transaction 1 should have no price annotation
    And posting 1 of transaction 2 should have no price annotation

  Scenario: Swap leg indexed last under a shared hash is priced from its partner, not itself
    # USDC -> USDT: both sides are base-like, so only the final swap price
    # pass prices them. The USDC sell is the last leg under the shared hash;
    # a hash-only lookup of its own `swap:txn:0xshared` reference finds the
    # leg itself and skips pricing. It must be priced from the USDT leg:
    # 100 USDT x 1.50 AUD over 100.5 USDC.
    Given a transactions file named "swap_stable_self_ref.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given the transactions are auto-linked for equity swaps with prices for "USDT" at "1.50" in "AUD"
    Then posting 1 of transaction 3 should have a per-unit price of "1.492537" "AUD"

  Scenario: Two-hop convert in one second links each hop, never across hops
    # 0.01 BTC -> 600 USDT -> 0.2 ETH, four plain equity:trading legs at one
    # second under one narration. Ranked by raw amount across commodities
    # the sells are [BTC 0.01, USDT 600] and the buys [ETH 0.2, USDT 600],
    # which linked BTC with ETH and USDT with USDT. Each sell links to the
    # buy of a different commodity in its own hop, and to nothing else.
    Given a transactions file named "swap_two_hop_convert.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given the transactions are auto-linked for equity swaps
    Then transaction 1 should be a sell leg swapped with "txn:usdt-buy" in "USDT"
    And transaction 2 should be a buy leg swapped with "txn:btc-sell" in "BTC"
    And transaction 3 should be a sell leg swapped with "txn:eth-buy" in "ETH"
    And transaction 4 should be a buy leg swapped with "txn:usdt-sell" in "USDT"

  Scenario: Opposite-direction swaps in one second link within each direction
    # A BTC -> ETH fill and an ETH -> BTC fill at the same second, as Kraken
    # books them. Ranked by raw amount the sells are [BTC 0.5, ETH 2] and
    # the buys [BTC 0.1, ETH 10], which linked BTC with BTC and ETH with
    # ETH. Legs of one commodity are never linked as a swap.
    Given a transactions file named "swap_opposite_direction.transactions"
    When I run the ledger parser on that file
    Then the parse should succeed
    Given the transactions are auto-linked for equity swaps
    Then transaction 1 should be a sell leg swapped with "txn:eth-buy" in "ETH"
    And transaction 2 should be a buy leg swapped with "txn:btc-sell" in "BTC"
    And transaction 3 should be a sell leg swapped with "txn:btc-buy" in "BTC"
    And transaction 4 should be a buy leg swapped with "txn:eth-sell" in "ETH"

  Scenario: Same-txn-ID trade link is stored as a single link
    Given a trade link store
    When I save a trade link between "txn:same-hash" and "txn:same-hash"
    Then get_trade_links should return 1 link

  Scenario: Dust-value transactions are not suggested when price data available
    Given a clean sources directory with dust-value exchange transactions
    And price data valuing ETH at 3000 USD and USDC at 1 USD
    When I run the pipeline
    And I request trade link suggestions with base currency "USD"
    Then I should receive 0 trade suggestions

  Scenario: Suggest trade links for multi-fill exchange trades
    Given a clean sources directory with multi-fill exchange transactions
    When I run the pipeline
    And I request trade link suggestions
    Then I should receive 3 trade suggestions

  Scenario: Each account set's swap legs are priced in that set's own base currency
    # alice inherits the vault's AUD; bob overrides base_currency to USD. Each
    # swaps 100 USDC in one on-chain transaction at the same second from the
    # same-named wallet account. The pipeline resolved the base currency once,
    # for the alphabetically first set, and linked every set's legs with it:
    # bob's legs were stamped `@ … AUD`, so his ledger and CGT figures came
    # out in alice's currency. Each set is linked in its own base currency,
    # and a set's legs never pair with another set's.
    Given a clean sources directory
    And the vault config.json contains:
      """
      {"base_currency": "AUD"}
      """
    And the set "bob" config.json contains:
      """
      {"base_currency": "USD"}
      """
    And a prices file at "_prices/test.txt" in sources with content:
      """
      P 2024-05-01 USDC 1.00 USD
      P 2024-05-01 SOL 120.00 USD
      P 2024-05-01 ETH 3000.00 USD
      P 2024-05-01 USD 1.50 AUD
      """
    And a swap CSV at "alice/crypto/wallet/main/2024-05.csv" trading "USDC" for "SOL"
    And a swap CSV at "bob/crypto/wallet/main/2024-05.csv" trading "USDC" for "ETH"
    When I run the pipeline
    Then the ledger at "alice/crypto/wallet/main/ledger.transactions" should contain "@ 180.722892 AUD"
    And the ledger at "alice/crypto/wallet/main/ledger.transactions" should contain "swap_partner_commodity:SOL"
    And the ledger at "bob/crypto/wallet/main/ledger.transactions" should contain "@ 120.481928 USD"
    And the ledger at "bob/crypto/wallet/main/ledger.transactions" should contain "swap_partner_commodity:ETH"
    And the ledger at "bob/crypto/wallet/main/ledger.transactions" should not contain " AUD"
