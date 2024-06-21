use {
    mollusk::{result::ProgramResult, Mollusk},
    solana_sdk::{
        account::AccountSharedData, instruction::InstructionError, pubkey::Pubkey,
        system_instruction, system_program,
    },
    solana_system_program::system_processor::DEFAULT_COMPUTE_UNITS,
};

#[test]
fn test_transfer() {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

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

    let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);

    let mollusk = Mollusk::default();

    let result = mollusk.process_instruction(&instruction, accounts);

    assert_eq!(result.program_result, ProgramResult::Success);
    assert_eq!(result.compute_units_consumed, DEFAULT_COMPUTE_UNITS);
}

#[test]
fn test_transfer_bad_owner() {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let accounts = vec![
        (
            sender,
            AccountSharedData::new(base_lamports, 0, &Pubkey::new_unique()),
        ),
        (
            recipient,
            AccountSharedData::new(base_lamports, 0, &system_program::id()),
        ),
    ];

    let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);

    let mollusk = Mollusk::default();

    let result = mollusk.process_instruction(&instruction, accounts);

    assert_eq!(
        result.program_result,
        ProgramResult::UnknownError(InstructionError::ExternalAccountLamportSpend)
    );
    assert_eq!(result.compute_units_consumed, DEFAULT_COMPUTE_UNITS);
}
