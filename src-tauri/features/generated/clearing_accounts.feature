Feature: Clearing accounts
  A clearing account is a pass-through account such as assets:transfer: money
  goes in on one leg and comes out on another, so its postings should net to
  zero. It is not a holding. The vault declares its clearing accounts as
  `clearing_accounts` in generated/config.json; with none declared the list is
  assets:transfer, the account self-transfers are booked to. Each entry covers
  its sub-accounts.

  A clearing account raises no Unverified Balance issue. One whose postings in
  a commodity don't net to zero across the whole vault is reported as an
  Unbalanced clearing account, with the amount left over. A transfer between
  two entities passes through assets:transfer in both sets, so the sum is taken
  over every set together. The zero test rounds to the most decimals written in
  that account's postings in that commodity.

  Background:
    Given an issues vault

  Scenario: With none declared, assets:transfer is the clearing account
    Then the vault clearing accounts should be "assets:transfer"

  Scenario: Declaring clearing accounts keeps the other vault config keys
    Given the vault config.json contains:
      """
      {"base_currency": "AUD", "default_decimals": 4}
      """
    When I declare the clearing accounts "assets:transfer, assets:crypto:bridge"
    Then the vault clearing accounts should be "assets:transfer, assets:crypto:bridge"
    And the vault base currency should be "AUD"
    And the vault config.json should contain default_decimals 4

  Scenario: A clearing account left over in a commodity is reported per account and commodity
    Given the set "alex" ledger:
      """
      2026-01-05 * "Bank" "To exchange"
          assets:bank        -100.00 AUD
          assets:transfer     100.00 AUD

      2026-01-06 * "Exchange" "From bank"
          assets:exchange      60.00 AUD
          assets:transfer     -60.00 AUD

      2026-01-07 * "Card" "To wallet"
          assets:card         -25.5 USD
          assets:transfer      25.5 USD
      """
    When I collect the issues
    Then the "Unbalanced clearing account" issues should be:
      | message                                  |
      | assets:transfer — 40.00 AUD left over    |
      | assets:transfer — 25.5 USD left over     |

  Scenario: A leftover that is zero at the written precision is not reported
    Given the set "alex" ledger:
      """
      2026-01-05 * "Bank" "Out"
          assets:bank         -0.10 AUD
          assets:transfer      0.10 AUD

      2026-01-06 * "Bank" "Out"
          assets:bank         -0.20 AUD
          assets:transfer      0.20 AUD

      2026-01-07 * "Exchange" "In"
          assets:exchange      0.30 AUD
          assets:transfer     -0.30 AUD
      """
    When I collect the issues
    Then there should be no "Unbalanced clearing account" issues

  Scenario: Amounts written to 18 decimals are summed exactly
    Given the set "alex" ledger:
      """
      2026-01-05 * "Wallet" "Out"
          assets:wallet       -0.100000000000000000 ETH
          assets:transfer      0.100000000000000000 ETH

      2026-01-06 * "Wallet" "Out"
          assets:wallet       -0.200000000000000000 ETH
          assets:transfer      0.200000000000000000 ETH

      2026-01-07 * "Exchange" "In"
          assets:exchange      0.300000000000000000 ETH
          assets:transfer     -0.300000000000000000 ETH

      2026-01-08 * "Wallet" "Out"
          assets:wallet       -1,000,000.000000000000000001 WETH
          assets:transfer      1,000,000.000000000000000001 WETH

      2026-01-09 * "Exchange" "In"
          assets:exchange      1,000,000 WETH
          assets:transfer     -1,000,000 WETH
      """
    When I collect the issues
    Then the "Unbalanced clearing account" issues should be:
      | message                                                  |
      | assets:transfer — 0.000000000000000001 WETH left over    |

  Scenario: A transfer between two sets nets to zero across the vault
    Given the set "alex" ledger:
      """
      2026-01-05 * "Bank" "To Suzi"
          assets:bank        -100.00 AUD
          assets:transfer     100.00 AUD
      """
    And the set "suzi" ledger:
      """
      2026-01-05 * "Bank" "From Alex"
          assets:savings      100.00 AUD
          assets:transfer    -100.00 AUD
      """
    When I collect the issues
    Then there should be no "Unbalanced clearing account" issues

  Scenario: A declared entry covers its sub-accounts and replaces the default
    Given the vault config.json contains:
      """
      {"clearing_accounts": ["assets:crypto:bridge"]}
      """
    And the set "alex" ledger:
      """
      2026-01-05 * "Wallet" "Bridge out"
          assets:wallet                    -1.5 ETH
          assets:crypto:bridge:arbitrum     1.5 ETH

      2026-01-06 * "Wallet" "Not a bridge"
          assets:wallet                    -2.0 ETH
          assets:crypto:bridgework          2.0 ETH

      2026-01-07 * "Bank" "Out"
          assets:bank                      -9.00 AUD
          assets:transfer                   9.00 AUD
      """
    When I collect the issues
    Then the "Unbalanced clearing account" issues should be:
      | message                                              |
      | assets:crypto:bridge:arbitrum — 1.5 ETH left over    |

  Scenario: A clearing account raises no Unverified Balance issue or badge
    Given the set "alex" ledger:
      """
      2026-01-05 * "Bank" "To exchange"
          assets:bank        -100.00 AUD
          assets:transfer     100.00 AUD

      2026-01-06 * "Exchange" "From bank"
          assets:exchange     100.00 AUD
          assets:transfer    -100.00 AUD
      """
    When I collect the issues
    Then the "Unverified Balance" issues should be:
      | message                                |
      | assets:bank — no opening balance       |
      | assets:exchange — no opening balance   |
    And the issue badge count for "assets:bank" should be 1
    And there should be no issue badge for "assets:transfer"
