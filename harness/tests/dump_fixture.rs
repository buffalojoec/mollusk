#![cfg(feature = "fuzz")]

use {
    mollusk_svm::{result::Check, Mollusk},
    mollusk_svm_fuzz_fixture::Fixture,
    solana_sdk::{
        account::AccountSharedData, feature_set::FeatureSet, pubkey::Pubkey, system_instruction,
        system_program,
    },
    std::path::Path,
};

const EJECT_FUZZ_FIXTURES: &str = "./tests";

enum FileType {
    Blob,
    Json,
}

impl FileType {
    fn extension(&self) -> &'static str {
        match self {
            Self::Blob => ".fix",
            Self::Json => ".json",
        }
    }
}

fn is_fixture_file(path: &Path, file_type: &FileType) -> bool {
    if path.is_file() {
        let path = path.to_str().unwrap();
        if path.ends_with(file_type.extension()) {
            return true;
        }
    }
    false
}

// Find the first fixture in the `EJECT_FUZZ_FIXTURES` directory.
fn find_fixture(file_type: &FileType) -> Option<String> {
    let dir = std::fs::read_dir(EJECT_FUZZ_FIXTURES).unwrap();
    dir.filter_map(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if is_fixture_file(&path, file_type) {
            return Some(path.to_str().unwrap().to_string());
        }
        None
    })
    .next()
}

// Remove all fixture files in the `EJECT_FUZZ_FIXTURES` directory.
fn clear() {
    let dir = std::fs::read_dir(EJECT_FUZZ_FIXTURES).unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if is_fixture_file(&path, &FileType::Blob) || is_fixture_file(&path, &FileType::Json) {
            std::fs::remove_file(path).unwrap();
        }
    }
}

fn compare_feature_sets(from_fixture: &FeatureSet, from_mollusk: &FeatureSet) {
    assert_eq!(from_fixture.active.len(), from_mollusk.active.len());
    assert_eq!(from_fixture.inactive.len(), from_mollusk.inactive.len());
    for f in from_fixture.active.keys() {
        assert!(from_mollusk.active.contains_key(f));
    }
}

#[test]
fn test_dump() {
    clear();
    std::env::set_var("EJECT_FUZZ_FIXTURES", EJECT_FUZZ_FIXTURES);
    std::env::set_var("EJECT_FUZZ_FIXTURES_JSON", EJECT_FUZZ_FIXTURES);

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let mollusk = Mollusk::default();

    let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);
    let accounts = vec![
        (
            sender,
            AccountSharedData::new(base_lamports, 0, &system_program::id()),
        ),
        (
            recipient,
            AccountSharedData::new(base_lamports, 0, &system_program::id()),
        ),
    ];
    let checks = vec![
        Check::success(),
        Check::account(&sender)
            .lamports(base_lamports - transfer_amount)
            .build(),
        Check::account(&recipient)
            .lamports(base_lamports + transfer_amount)
            .build(),
    ];

    mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);

    // Validate the protobuf fixture matches the test environment.
    let blob_fixture_path = find_fixture(&FileType::Blob).unwrap();
    let blob_fixture = Fixture::load_from_blob_file(&blob_fixture_path);
    assert_eq!(blob_fixture.input.compute_budget, mollusk.compute_budget);
    // Feature set matches, but it can't guarantee sorting.
    compare_feature_sets(&blob_fixture.input.feature_set, &mollusk.feature_set);
    assert_eq!(blob_fixture.input.sysvars.clock, mollusk.sysvars.clock);
    assert_eq!(blob_fixture.input.sysvars.rent, mollusk.sysvars.rent);
    assert_eq!(blob_fixture.input.program_id, instruction.program_id);
    assert_eq!(
        blob_fixture.input.instruction_accounts,
        instruction.accounts
    );
    assert_eq!(blob_fixture.input.instruction_data, instruction.data);
    assert_eq!(blob_fixture.input.accounts, accounts);

    // Validate the JSON fixture matches the test environment.
    let json_fixture_path = find_fixture(&FileType::Json).unwrap();
    let json_fixture = Fixture::load_from_json_file(&json_fixture_path);
    assert_eq!(json_fixture.input.compute_budget, mollusk.compute_budget);
    // Feature set matches, but it can't guarantee sorting.
    compare_feature_sets(&json_fixture.input.feature_set, &mollusk.feature_set);
    assert_eq!(json_fixture.input.sysvars.clock, mollusk.sysvars.clock);
    assert_eq!(json_fixture.input.sysvars.rent, mollusk.sysvars.rent);
    assert_eq!(json_fixture.input.program_id, instruction.program_id);
    assert_eq!(
        json_fixture.input.instruction_accounts,
        instruction.accounts
    );
    assert_eq!(json_fixture.input.instruction_data, instruction.data);
    assert_eq!(json_fixture.input.accounts, accounts);

    // Ensure both files have the same name.
    assert_eq!(
        Path::new(&blob_fixture_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap(),
        Path::new(&json_fixture_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
    );

    std::env::remove_var("EJECT_FUZZ_FIXTURES");
    std::env::remove_var("EJECT_FUZZ_FIXTURES_JSON");
    clear();
}
