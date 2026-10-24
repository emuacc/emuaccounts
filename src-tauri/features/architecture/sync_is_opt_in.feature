Feature: Multi-device sync is compiled only into an opt-in build

  Device sync (relay, pairing, blob store) is half-baked, so a default build
  must not contain it. It is the `sync` cargo feature, off by default; its
  code lives under src/sync/, and only a few hook points outside that
  directory refer to it (docs/SYNC.md).

  Scenario: The default build leaves sync out
    Given the crate manifest
    Then the "sync" feature is off by default
    And the "emuacc-relay" binary requires the "sync" feature
    And the "sync" module of "lib" is compiled only with the "sync" feature

  Scenario: The default build has no Automerge dependency
    # Automerge backs only the sync build's metadata document. A default build
    # reads the trade links a pre-TASK-133 document holds with its own reader
    # (src/trade_links/automerge_doc.rs), so it never links automerge.
    Given the crate manifest
    Then the "automerge" dependency is optional and only the "sync" feature enables it
    And the "autosurgeon" dependency is optional and only the "sync" feature enables it

  Scenario: Only the hook points outside src/sync refer to the sync code
    Then outside "src/sync" only these files refer to the sync code:
      | src/lib.rs           |
      | src/main.rs          |
      | src/sources_state.rs |
