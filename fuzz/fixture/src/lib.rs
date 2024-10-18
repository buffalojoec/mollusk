//! Mollusk SVM Fuzz: Mollusk-compatible fuzz fixture for SVM programs.
//!
//! Note: The fixtures defined in this library are compatible with Mollusk,
//! which means developers can fuzz programs using the Mollusk harness.
//! However, these fixtures (and this library) do not depend on the harness.
//! They can be used to fuzz a custom entrypoint of the developer's choice.

pub mod account;
pub mod compute_budget;
pub mod context;
pub mod effects;
pub mod feature_set;
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/org.mollusk.svm.rs"));
}
pub mod sysvars;

use crate::{context::Context, effects::Effects, proto::InstrFixture as ProtoFixture};

/// A fixture for invoking a single instruction against a simulated SVM
/// program runtime environment, for a given program.
pub struct Fixture {
    /// The fixture inputs.
    pub input: Context,
    /// The fixture outputs.
    pub output: Effects,
}

impl From<ProtoFixture> for Fixture {
    fn from(value: ProtoFixture) -> Self {
        // All blobs should have an input and output.
        Self {
            input: value.input.unwrap().into(),
            output: value.output.unwrap().into(),
        }
    }
}

impl From<Fixture> for ProtoFixture {
    fn from(value: Fixture) -> Self {
        Self {
            input: Some(value.input.into()),
            output: Some(value.output.into()),
        }
    }
}

impl Fixture {
    /// Decode a `Protobuf` blob into a `Fixture`.
    pub fn decode(blob: &[u8]) -> Self {
        <proto::InstrFixture as prost::Message>::decode(blob)
            .map(Into::into)
            .unwrap_or_else(|err| panic!("Failed to decode fixture: {}", err))
    }
}
