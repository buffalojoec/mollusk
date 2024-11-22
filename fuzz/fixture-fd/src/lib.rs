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

pub mod account;
pub mod context;
pub mod effects;
pub mod feature_set;
pub mod instr_account;
pub mod metadata;
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/org.solana.sealevel.v1.rs"));
}

use {
    crate::{
        context::Context, effects::Effects, metadata::Metadata, proto::InstrFixture as ProtoFixture,
    },
    mollusk_svm_fuzz_fs::{FsHandler, IntoSerializableFixture, SerializableFixture},
};

/// A fixture for invoking a single instruction against a simulated SVM
/// program runtime environment, for a given program.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fixture {
    /// The fixture metadata.
    pub metadata: Option<Metadata>,
    /// The fixture inputs.
    pub input: Context,
    /// The fixture outputs.
    pub output: Effects,
}

impl Fixture {
    pub fn decode(blob: &[u8]) -> Self {
        let proto_fixture = <ProtoFixture as SerializableFixture>::decode(blob);
        proto_fixture.into()
    }

    pub fn load_from_blob_file(file_path: &str) -> Self {
        let proto_fixture: ProtoFixture = FsHandler::load_from_blob_file(file_path);
        proto_fixture.into()
    }

    pub fn load_from_json_file(file_path: &str) -> Self {
        let proto_fixture: ProtoFixture = FsHandler::load_from_json_file(file_path);
        proto_fixture.into()
    }
}

impl From<ProtoFixture> for Fixture {
    fn from(value: ProtoFixture) -> Self {
        // All blobs should have an input and output.
        Self {
            metadata: value.metadata.map(Into::into),
            input: value.input.unwrap().into(),
            output: value.output.unwrap().into(),
        }
    }
}

impl From<Fixture> for ProtoFixture {
    fn from(value: Fixture) -> Self {
        Self {
            metadata: value.metadata.map(Into::into),
            input: Some(value.input.into()),
            output: Some(value.output.into()),
        }
    }
}

impl SerializableFixture for ProtoFixture {}

impl IntoSerializableFixture for Fixture {
    type Fixture = ProtoFixture;

    fn into(self) -> Self::Fixture {
        Into::into(self)
    }
}
