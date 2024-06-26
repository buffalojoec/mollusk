#![cfg(feature = "fuzz")]

use {
    mollusk_svm::{fuzz::fixture::Fixture, result::Check, Mollusk},
    solana_program_runtime::compute_budget::ComputeBudget,
    solana_sdk::{
        account::AccountSharedData, feature_set::FeatureSet, instruction::Instruction,
        pubkey::Pubkey, system_instruction, system_program,
    },
    std::path::Path,
};

const FIXTURES_DIR: &str = "./tests";

enum FileType {
    Fixture,
    Json,
}
impl FileType {
    fn extension(&self) -> &'static str {
        match self {
            Self::Fixture => ".fix",
            Self::Json => ".json",
        }
    }
}

fn is_file(path: &Path, file_type: &FileType) -> bool {
    if path.is_file() {
        let path = path.to_str().unwrap();
        if path.ends_with(file_type.extension()) {
            return true;
        }
    }
    false
}

// Find the first `.fix` file in the `EJECT_FUZZ_FIXTURES` directory.
fn find_file(file_type: FileType) -> Option<String> {
    let dir = std::fs::read_dir(FIXTURES_DIR).unwrap();
    dir.filter_map(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if is_file(&path, &file_type) {
            return Some(path.to_str().unwrap().to_string());
        }
        None
    })
    .next()
}

// Remove all `.fix` files in the `EJECT_FUZZ_FIXTURES` directory.
fn clear() {
    let dir = std::fs::read_dir(FIXTURES_DIR).unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if is_file(&path, &FileType::Fixture) || is_file(&path, &FileType::Json) {
            std::fs::remove_file(path).unwrap();
        }
    }
}

#[allow(clippy::field_reassign_with_default)]
fn mollusk_test() -> (Mollusk, Instruction, [(Pubkey, AccountSharedData); 2]) {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let mut mollusk = Mollusk::default();
    mollusk.feature_set = FeatureSet::all_enabled();

    let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);
    let accounts = [
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

    (mollusk, instruction, accounts)
}

#[test]
fn test_dump() {
    clear();

    // First try protobuf.
    {
        std::env::set_var("EJECT_FUZZ_FIXTURES", FIXTURES_DIR);

        let (mollusk, instruction, accounts) = mollusk_test();

        let fixture_path = find_file(FileType::Fixture).unwrap();
        let fixture = Fixture::read_from_binary_file(&fixture_path).unwrap();

        assert_eq!(
            ComputeBudget::from(fixture.input.compute_budget),
            mollusk.compute_budget
        );
        assert_eq!(
            FeatureSet::from(fixture.input.feature_set),
            mollusk.feature_set
        );
        assert_eq!(fixture.input.sysvar_context.clock, mollusk.sysvars.clock);
        assert_eq!(fixture.input.sysvar_context.rent, mollusk.sysvars.rent);
        assert_eq!(fixture.input.program_id, mollusk.program_id);
        assert_eq!(fixture.input.instruction_accounts, instruction.accounts);
        assert_eq!(fixture.input.instruction_data, instruction.data);
        assert_eq!(fixture.input.accounts, accounts);

        std::env::remove_var("EJECT_FUZZ_FIXTURES");
    }

    // Now try JSON.
    {
        std::env::set_var("EJECT_FUZZ_FIXTURES_JSON", FIXTURES_DIR);

        let (mollusk, instruction, accounts) = mollusk_test();

        let fixture_path = find_file(FileType::Json).unwrap();
        let fixture = Fixture::read_from_json_file(&fixture_path).unwrap();

        assert_eq!(
            ComputeBudget::from(fixture.input.compute_budget),
            mollusk.compute_budget
        );
        assert_eq!(
            FeatureSet::from(fixture.input.feature_set),
            mollusk.feature_set
        );
        assert_eq!(fixture.input.sysvar_context.clock, mollusk.sysvars.clock);
        assert_eq!(fixture.input.sysvar_context.rent, mollusk.sysvars.rent);
        assert_eq!(fixture.input.program_id, mollusk.program_id);
        assert_eq!(fixture.input.instruction_accounts, instruction.accounts);
        assert_eq!(fixture.input.instruction_data, instruction.data);
        assert_eq!(fixture.input.accounts, accounts);
    }

    clear();
}
