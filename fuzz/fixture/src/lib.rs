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

use {
    crate::{context::Context, effects::Effects, proto::InstrFixture as ProtoFixture},
    prost::Message,
    std::{
        fs::{self, File},
        io::{Read, Write},
        path::Path,
    },
};

/// A fixture for invoking a single instruction against a simulated SVM
/// program runtime environment, for a given program.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fixture {
    /// The fixture inputs.
    pub input: Context,
    /// The fixture outputs.
    pub output: Effects,
}

impl Fixture {
    /// Decode a `Protobuf` blob into a `Fixture`.
    pub fn decode(blob: &[u8]) -> Self {
        ProtoFixture::decode(blob)
            .map(Into::into)
            .unwrap_or_else(|err| panic!("Failed to decode fixture: {}", err))
    }

    /// Encode the `Fixture` into a `Protobuf` blob.
    pub fn encode(self) -> Vec<u8> {
        let mut buf = Vec::new();
        ProtoFixture::from(self)
            .encode(&mut buf)
            .expect("Failed to encode fixture");
        buf
    }

    /// Dumps the `Fixture` to a protobuf binary blob file.
    /// The file name is a hash of the fixture with the `.fix` extension.
    pub fn dump_to_blob_file(self, dir: &str) {
        let blob = self.encode();

        let hash = solana_sdk::hash::hash(&blob);
        let file_name = format!("instr-{}.fix", bs58::encode(hash).into_string());

        write_file(Path::new(dir), &file_name, &blob);
    }

    /// Dumps the `Fixture` to a JSON file.
    /// The file name is a hash of the fixture with the `.json` extension.
    pub fn dump_to_json_file(self, dir_path: &str) {
        let blob = self.clone().encode();
        let json = serde_json::to_string_pretty(&ProtoFixture::from(self))
            .expect("Failed to serialize fixture to JSON");

        let hash = solana_sdk::hash::hash(&blob);
        let file_name = format!("instr-{}.json", bs58::encode(hash).into_string());

        write_file(Path::new(dir_path), &file_name, json.as_bytes());
    }

    /// Loads a `Fixture` from a protobuf binary blob file.
    pub fn load_from_blob_file(file_path: &str) -> Self {
        if !file_path.ends_with(".fix") {
            panic!("Invalid fixture file extension: {}", file_path);
        }
        let mut file = File::open(file_path).expect("Failed to open fixture file");
        let mut blob = Vec::new();
        file.read_to_end(&mut blob)
            .expect("Failed to read fixture file");
        Self::decode(&blob)
    }

    /// Loads a `Fixture` from a JSON file.
    pub fn load_from_json_file(file_path: &str) -> Self {
        if !file_path.ends_with(".json") {
            panic!("Invalid fixture file extension: {}", file_path);
        }
        let mut file = File::open(file_path).expect("Failed to open fixture file");
        let mut json = String::new();
        file.read_to_string(&mut json)
            .expect("Failed to read fixture file");
        let fixture: ProtoFixture =
            serde_json::from_str(&json).expect("Failed to deserialize fixture from JSON");
        fixture.into()
    }
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

fn write_file(dir: &Path, file_name: &str, data: &[u8]) {
    fs::create_dir_all(dir).expect("Failed to create directory");
    let file_path = dir.join(file_name);
    let mut file = File::create(file_path).unwrap();
    file.write_all(data)
        .expect("Failed to write fixture to file");
}
