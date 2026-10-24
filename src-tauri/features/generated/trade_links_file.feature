Feature: Trade links are kept in a plain JSON file

  A vault's trade links live in sources/_trade_links.json, laid out like
  _rules.json: each link's keys sorted, the links sorted by id, two-space
  indent. Saving or deleting one link changes only that link's lines, so the
  file reviews as a small diff in git.

  Before TASK-133 the links lived in the Automerge document
  sources/emuacc-metadata.automerge. While the JSON file is absent, the first
  use reads the document's links and writes them to the file without loss; the
  document itself is never modified, because the opt-in sync build keeps its
  device-sync metadata there. The fixture documents were written by the
  pre-TASK-133 store (automerge 0.7.4); each has the links it holds beside it
  as the expected file.

  Scenario: Trade links are written to the file in id order
    Given a trade link store
    When I save a trade link between "txn:sell-btc" and "txn:buy-aud"
    And I save a trade link between "txn:sell-eth" and "txn:buy-usdc"
    Then the trade-links file should read:
      """
      {
        "trade_links": [
          {
            "created_at": <created_at>,
            "device_origin": "<device>",
            "id": "372924585e97",
            "txn_id_a": "txn:sell-eth",
            "txn_id_b": "txn:buy-usdc"
          },
          {
            "created_at": <created_at>,
            "device_origin": "<device>",
            "id": "5b092dbe6f68",
            "txn_id_a": "txn:sell-btc",
            "txn_id_b": "txn:buy-aud"
          }
        ]
      }
      """

  Scenario: Links that exist only in the Automerge document are migrated on first use
    Given a vault whose trade links are only in the Automerge document "trade_links_doc.automerge"
    When I read the vault's trade links
    Then get_trade_links should return 2 links
    And the trade-links file should equal the fixture "trade_links_doc.json"
    And the Automerge document should be unchanged

  Scenario: A synced document's links are migrated, concurrent edits resolved as Automerge resolves them
    # Two devices edited the links concurrently (both re-linked one pair, one
    # dropped a link, the other added one) and the relay merged their
    # documents; the sync metadata's lists and maps sit beside the links, and
    # the document is large enough that its columns are deflated.
    Given a vault whose trade links are only in the Automerge document "trade_links_synced_doc.automerge"
    When I read the vault's trade links
    Then get_trade_links should return 40 links
    And the trade-links file should equal the fixture "trade_links_synced_doc.json"
    And the Automerge document should be unchanged

  Scenario: A document without trade links migrates nothing
    Given a vault whose trade links are only in the Automerge document "sync_only_doc.automerge"
    When I read the vault's trade links
    Then get_trade_links should return 0 links
    And there should be no trade-links file
    And the Automerge document should be unchanged

  Scenario: Once the file exists the document is not read again
    Given a vault whose trade links are only in the Automerge document "trade_links_doc.automerge"
    When I read the vault's trade links
    And I delete the trade link between "txn:sell-eth" and "txn:buy-usdc"
    And I delete the trade link between "txn:sell-btc" and "txn:buy-aud"
    Then the trade-links file should read:
      """
      {
        "trade_links": []
      }
      """
    And get_trade_links should return 0 links
    And the Automerge document should be unchanged

  Scenario: Looking at the links without migrating writes nothing
    Given a vault whose trade links are only in the Automerge document "trade_links_doc.automerge"
    When I look at the vault's trade links without migrating
    Then that look should find 2 links
    And there should be no trade-links file

  Scenario: A document that cannot be read is an error, never an empty list
    Given a vault whose Automerge document is not readable
    When I read the vault's trade links
    Then reading the trade links should fail naming "emuacc-metadata.automerge"
    And there should be no trade-links file
    And the Automerge document should be unchanged
