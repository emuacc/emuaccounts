//! Multi-device sync: an Automerge metadata document merged through a
//! self-hosted relay, with a content-addressed blob store. It is half-baked,
//! so only a build with the `sync` cargo feature compiles it (docs/SYNC.md);
//! a default build has none of it.
//!
//! This directory also holds the sync build's two binary-side files, which are
//! not modules of the library: `commands.rs`, the app's sync Tauri commands
//! (a module of `main.rs`), and `relay_main.rs`, the `emuacc-relay` server.

pub mod content_store;
pub mod metadata;
pub mod protocol;
pub mod relay;
pub mod relay_client;
pub mod state;
