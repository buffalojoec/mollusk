//! A fixture for invoking a single instruction against a simulated Solana
//! program runtime environment, for a given program.

pub mod account;
pub mod compute_budget;
pub mod context;
pub mod effects;
pub mod error;
pub mod feature_set;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/org.mollusk.svm.rs"));
}
pub mod sysvars;

use {
    context::FixtureContext,
    effects::FixtureEffects,
    error::FixtureError,
    prost::Message,
    std::{fs::File, io::Read},
};

/// A fixture for invoking a single instruction against a simulated Solana
/// program runtime environment, for a given program.
#[derive(Debug)]
pub struct Fixture {
    /// The fixture inputs.
    pub input: FixtureContext,
    /// The fixture outputs.
    pub output: FixtureEffects,
}

impl TryFrom<proto::InstrFixture> for Fixture {
    type Error = FixtureError;

    fn try_from(fixture: proto::InstrFixture) -> Result<Self, Self::Error> {
        // All blobs should have an input and output.
        let input: FixtureContext = fixture
            .input
            .ok_or::<FixtureError>(FixtureError::InvalidFixtureInput)?
            .try_into()?;
        let output: FixtureEffects = fixture
            .output
            .ok_or::<FixtureError>(FixtureError::InvalidFixtureOutput)?
            .try_into()?;
        Ok(Self { input, output })
    }
}

impl From<&Fixture> for proto::InstrFixture {
    fn from(fixture: &Fixture) -> Self {
        let Fixture { input, output } = fixture;
        proto::InstrFixture {
            input: Some(input.into()),
            output: Some(output.into()),
        }
    }
}

impl Fixture {
    /// Decode a `Protobuf` blob into a `Fixture`.
    pub fn decode(blob: &[u8]) -> Result<Self, FixtureError> {
        let fixture: proto::InstrFixture = proto::InstrFixture::decode(blob)?;
        fixture.try_into()
    }

    /// Loads a `Fixture` from a protobuf binary blob file.
    pub fn load_from_blob_file(file_path: &str) -> Result<Self, FixtureError> {
        if !file_path.ends_with(".fix") {
            panic!("Invalid fixture file extension: {}", file_path);
        }
        let mut file = File::open(file_path).expect("Failed to open fixture file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .expect("Failed to read fixture file");
        Self::decode(&buf)
    }
}
