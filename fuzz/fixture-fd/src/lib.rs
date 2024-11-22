//! Mollusk SVM Fuzz: Mollusk-compatible Firedancer fuzz fixture for SVM
//! programs.
//!
//! Similar to the `mollusk-svm-fuzz-fixture` library, but built around
//! Firedancer's protobuf layouts.
//!
//! Note: The fixtures defined in this library are compatible with Mollusk,
//! which means developers can fuzz programs using the Mollusk harness.
//! However, these fixtures (and this library) do not depend on the harness.
//! They can be used to fuzz a custom entrypoint of the developer's choice.

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/org.solana.sealevel.v1.rs"));
}
