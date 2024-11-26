#![cfg(any(feature = "fuzz", feature = "fuzz-fd"))]

use {
    mollusk_svm::Mollusk,
    solana_sdk::{account::AccountSharedData, pubkey::Pubkey, system_instruction, system_program},
};

const BASE_LAMPORTS: u64 = 100_000_000;

#[cfg(feature = "fuzz")]
#[test]
fn test_process_mollusk() {
    let ok_transfer_amount = 42_000;
    let too_much = BASE_LAMPORTS + 1;

    let mollusk = Mollusk::default();

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let accounts = vec![
        (
            sender,
            AccountSharedData::new(BASE_LAMPORTS, 0, &system_program::id()),
        ),
        (
            recipient,
            AccountSharedData::new(BASE_LAMPORTS, 0, &system_program::id()),
        ),
    ];

    // First try the success case.
    let instruction = system_instruction::transfer(&sender, &recipient, ok_transfer_amount);
    let result = mollusk.process_instruction(&instruction, &accounts);

    let fixture = mollusk_svm::fuzz::mollusk::build_fixture_from_mollusk_test(
        &mollusk,
        &instruction,
        &accounts,
        &result,
        &[],
    );

    Mollusk::process_and_validate_fixture(&fixture);

    // Now the error case.
    let instruction = system_instruction::transfer(&sender, &recipient, too_much);
    let result = mollusk.process_instruction(&instruction, &accounts);

    let fixture = mollusk_svm::fuzz::mollusk::build_fixture_from_mollusk_test(
        &mollusk,
        &instruction,
        &accounts,
        &result,
        &[],
    );

    Mollusk::process_and_validate_fixture(&fixture);
}

#[cfg(feature = "fuzz-fd")]
#[test]
fn test_process_firedancer() {
    let ok_transfer_amount = 42_000;
    let too_much = BASE_LAMPORTS + 1;

    let mollusk = Mollusk::default();

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let accounts = vec![
        (
            sender,
            AccountSharedData::new(BASE_LAMPORTS, 0, &system_program::id()),
        ),
        (
            recipient,
            AccountSharedData::new(BASE_LAMPORTS, 0, &system_program::id()),
        ),
    ];

    // First try the success case.
    let instruction = system_instruction::transfer(&sender, &recipient, ok_transfer_amount);
    let result = mollusk.process_instruction(&instruction, &accounts);

    let fixture = mollusk_svm::fuzz::firedancer::build_fixture_from_mollusk_test(
        &mollusk,
        &instruction,
        &accounts,
        &result,
        &[],
    );

    Mollusk::process_and_validate_firedancer_fixture(&fixture);

    // Now the error case.
    let instruction = system_instruction::transfer(&sender, &recipient, too_much);
    let result = mollusk.process_instruction(&instruction, &accounts);

    let fixture = mollusk_svm::fuzz::firedancer::build_fixture_from_mollusk_test(
        &mollusk,
        &instruction,
        &accounts,
        &result,
        &[],
    );

    Mollusk::process_and_validate_firedancer_fixture(&fixture);
}
