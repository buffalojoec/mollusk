//! A fixture for invoking a single instruction against a simulated Solana
//! program runtime environment, for a given program.

pub mod account;
pub mod compute_budget;
pub mod context;
pub mod effects;
pub mod error;
pub mod feature_set;
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/org.mollusk.svm.rs"));
}
pub mod sysvars;

use {
    context::FixtureContext,
    effects::FixtureEffects,
    error::FixtureError,
    prost::Message,
    serde::{Deserialize, Serialize},
    std::{
        fs::{self, File},
        io::{Read, Write},
        path::Path,
    },
};

/// A fixture for invoking a single instruction against a simulated Solana
/// program runtime environment, for a given program.
#[derive(Debug, Deserialize, Serialize)]
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

    /// Dumps the `Fixture` to a protobuf binary blob file.
    /// The file name is a hash of the fixture with the `.fix` extension.
    pub fn dump(&self, dir_path: &str) {
        let proto_fixture: proto::InstrFixture = self.into();

        let mut buf = Vec::new();
        proto_fixture
            .encode(&mut buf)
            .expect("Failed to encode fixture");

        let hash = solana_sdk::hash::hash(&buf);
        let file_name = format!("instr-{}.fix", bs58::encode(hash).into_string());

        fs::create_dir_all(dir_path).expect("Failed to create directory");
        let file_path = Path::new(dir_path).join(file_name);

        let mut file = File::create(file_path).unwrap();
        file.write_all(&buf)
            .expect("Failed to write fixture to file");
    }

    /// Reads a `Fixture` from a protobuf binary blob file.
    pub fn read_from_binary_file(file_path: &str) -> Result<Self, FixtureError> {
        if !file_path.ends_with(".fix") {
            panic!("Invalid fixture file extension: {}", file_path);
        }

        let mut file = File::open(file_path).expect("Failed to open fixture file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .expect("Failed to read fixture file");

        Self::decode(&buf)
    }

    /// Dumps the `Fixture` to a JSON file.
    /// The file name is a hash of the fixture with the `.json` extension.
    pub fn dump_json(&self, dir_path: &str) {
        let json = serde_json::to_string_pretty(&self).expect("Failed to serialize fixture");

        let hash = solana_sdk::hash::hash(json.as_bytes());
        let file_name = format!("instr-{}.json", bs58::encode(hash).into_string());

        fs::create_dir_all(dir_path).expect("Failed to create directory");
        let file_path = Path::new(dir_path).join(file_name);

        let mut file = File::create(file_path).unwrap();
        file.write_all(json.as_bytes())
            .expect("Failed to write fixture to file");
    }

    /// Reads a `Fixture` from a JSON file.
    pub fn read_from_json_file(file_path: &str) -> Result<Self, FixtureError> {
        if !file_path.ends_with(".json") {
            panic!("Invalid fixture file extension: {}", file_path);
        }

        let mut file = File::open(file_path).expect("Failed to open fixture file");
        let mut json = String::new();
        file.read_to_string(&mut json)
            .expect("Failed to read fixture file");

        serde_json::from_str(&json).map_err(|_| FixtureError::InvalidJsonFixture)
    }
}
