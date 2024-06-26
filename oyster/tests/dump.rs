use {
    mollusk_svm_fuzz::fixture::Fixture,
    mollusk_svm_oyster::{result::Check, Mollusk, Oyster},
    solana_sdk::{
        account::AccountSharedData, feature_set::FeatureSet, pubkey::Pubkey, system_instruction,
        system_program,
    },
    std::path::Path,
};

const FIXTURES_DIR: &str = "./tests";

fn is_fixture_file(path: &Path) -> bool {
    if path.is_file() {
        let path = path.to_str().unwrap();
        if path.ends_with(".fix") {
            return true;
        }
    }
    false
}

// Find the first `.fix` file in the `EJECT_FUZZ_FIXTURES` directory.
fn find_fixture(dir_path: &str) -> Option<String> {
    let dir = std::fs::read_dir(dir_path).unwrap();
    dir.filter_map(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if is_fixture_file(&path) {
            return Some(path.to_str().unwrap().to_string());
        }
        None
    })
    .next()
}

// Remove all `.fix` files in the `EJECT_FUZZ_FIXTURES` directory.
fn clear_fixtures(dir_path: &str) {
    let dir = std::fs::read_dir(dir_path).unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if is_fixture_file(&path) {
            std::fs::remove_file(path).unwrap();
        }
    }
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn test_dump() {
    std::env::set_var("EJECT_FUZZ_FIXTURES", FIXTURES_DIR);
    clear_fixtures(FIXTURES_DIR);

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let mut mollusk = Mollusk::default();
    mollusk.feature_set = FeatureSet::default(); // `all_enabled` doesn't line up with `AGAVE_FEATURES`.

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

    Oyster::process_and_validate_instruction(&mollusk, &instruction, &accounts, &checks);

    let fixture_path = find_fixture(FIXTURES_DIR).unwrap();
    let fixture = Fixture::read_from_file(&fixture_path).unwrap();

    assert_eq!(fixture.input.compute_budget, mollusk.compute_budget);
    assert_eq!(fixture.input.feature_set, mollusk.feature_set);
    assert_eq!(fixture.input.sysvar_context.clock, mollusk.sysvars.clock);
    assert_eq!(fixture.input.sysvar_context.rent, mollusk.sysvars.rent);
    assert_eq!(fixture.input.program_id, mollusk.program_id);
    assert_eq!(fixture.input.instruction_accounts, instruction.accounts);
    assert_eq!(fixture.input.instruction_data, instruction.data);
    assert_eq!(fixture.input.accounts, accounts);

    clear_fixtures(FIXTURES_DIR);
}
