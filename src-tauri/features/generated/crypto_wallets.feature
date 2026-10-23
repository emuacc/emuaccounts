Feature: Crypto wallet registration

  Adding a Crypto Wallet account registers the wallet in the crypto-sync
  plugin's wallets.json (grouped form) so the next sync run picks it up.
  The upsert is non-destructive: existing entries, the binance section, and
  array-form files are preserved; duplicates are idempotent.

  Scenario: Registering a wallet creates wallets.json when missing
    Given a plugins directory with a plugin "other" with manifest:
      """
      [plugin]
      name = "Other"
      version = "0.1.0"
      script = "sync.py"
      """
    When I register crypto wallet "ethereum" address "0xAbCdEf1234567890aBcDeF1234567890AbCdEf12" label "main" network "Base"
    Then the wallets file should exist
    And the wallets entry for "ethereum" address "0xabcdef1234567890abcdef1234567890abcdef12" should have label "main"
    And the wallets entry for "ethereum" address "0xabcdef1234567890abcdef1234567890abcdef12" should have network "base"
    And the wallet outcome should be created_file

  Scenario: Solana addresses are stored verbatim
    When I register crypto wallet "solana" address "9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc" label "sol_defi"
    Then the wallets entry for "solana" address "9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc" should have label "sol_defi"

  Scenario: Registering preserves existing entries including the binance section
    Given a crypto-sync wallets file:
      """
      {
        "ethereum": [
          { "friendly_name": "main_eth", "address": "0x1111111111111111111111111111111111111111" }
        ],
        "binance": [
          { "friendly_name": "main_binance", "symbols": ["BTCUSDT"] }
        ],
        "custom_key": { "keep": true }
      }
      """
    When I register crypto wallet "solana" address "9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc" label "sol_defi"
    Then the wallets entry for "ethereum" address "0x1111111111111111111111111111111111111111" should have label "main_eth"
    And the wallets entry for "solana" address "9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc" should have label "sol_defi"
    And the wallets file should contain key "binance"
    And the wallets file should contain key "custom_key"

  Scenario: Registering the same wallet twice is idempotent and keeps the original label
    When I register crypto wallet "ethereum" address "0xAAAA111122223333444455556666777788889999" label "first"
    And I register crypto wallet "ethereum" address "0xaaaa111122223333444455556666777788889999" label "second"
    Then the wallets entry for "ethereum" address "0xaaaa111122223333444455556666777788889999" should have label "first"
    And the wallet outcome should be already_registered
    And the wallets chain "ethereum" should have 1 entry

  Scenario: Registering a wallet records its sources folder
    When I register crypto wallet "ethereum" address "0xCCCC111122223333444455556666777788889999" label "evm" folder "Acme/assets/crypto/wallet/ethereum/0xcccc111122223333444455556666777788889999"
    Then the wallets entry for "ethereum" address "0xcccc111122223333444455556666777788889999" should have folder "Acme/assets/crypto/wallet/ethereum/0xcccc111122223333444455556666777788889999"

  Scenario: Re-registering stamps a folder onto a legacy entry without losing data
    Given a crypto-sync wallets file:
      """
      {
        "ethereum": [
          { "friendly_name": "first", "address": "0xdddd111122223333444455556666777788889999", "provider": "zerion" }
        ]
      }
      """
    When I register crypto wallet "ethereum" address "0xDDDD111122223333444455556666777788889999" label "second" folder "alex/crypto/wallet/ethereum/0xdddd111122223333444455556666777788889999"
    Then the wallets entry for "ethereum" address "0xdddd111122223333444455556666777788889999" should have label "first"
    And the wallets entry for "ethereum" address "0xdddd111122223333444455556666777788889999" should have folder "alex/crypto/wallet/ethereum/0xdddd111122223333444455556666777788889999"
    And the wallet outcome should be already_registered

  Scenario: Re-registering never overwrites an existing folder
    Given a crypto-sync wallets file:
      """
      {
        "ethereum": [
          { "friendly_name": "first", "address": "0xeeee111122223333444455556666777788889999", "folder": "custom/place" }
        ]
      }
      """
    When I register crypto wallet "ethereum" address "0xEEEE111122223333444455556666777788889999" label "x" folder "other/place"
    Then the wallets entry for "ethereum" address "0xeeee111122223333444455556666777788889999" should have folder "custom/place"

  Scenario: Unknown chains are rejected
    When I register crypto wallet "dogecoin" address "D12345" label "much wow"
    Then the wallet registration should fail with "unsupported chain"
    And the wallets file should not exist

  Scenario: An array-form wallets file gains an array-form entry
    Given a crypto-sync wallets file:
      """
      [
        { "blockchain": "solana", "friendly_name": "sol_defi", "address": "9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc9Abc" }
      ]
      """
    When I register crypto wallet "ethereum" address "0xBBBB111122223333444455556666777788889999" label "evm"
    Then the wallets file should be an array with 2 entries
    And the wallets entry for "ethereum" address "0xbbbb111122223333444455556666777788889999" should have label "evm"
