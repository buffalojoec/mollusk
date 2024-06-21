use {
    mollusk::{
        result::{CheckAccount, InstructionCheck, ProgramResult},
        Mollusk,
    },
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
        InstructionCheck::program_result(ProgramResult::Success),
        InstructionCheck::compute_units_consumed(DEFAULT_COMPUTE_UNITS),
        InstructionCheck::account(
            CheckAccount::new(&sender).lamports(base_lamports - transfer_amount),
        ),
        InstructionCheck::account(
            CheckAccount::new(&recipient).lamports(base_lamports + transfer_amount),
        ),
    ];

    Mollusk::default().process_and_validate_instruction(&instruction, accounts, &checks);
}

#[test]
fn test_transfer_bad_owner() {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);
    let accounts = vec![
        (
            sender,
            AccountSharedData::new(base_lamports, 0, &Pubkey::new_unique()), // <-- Bad owner.
        ),
        (
            recipient,
            AccountSharedData::new(base_lamports, 0, &system_program::id()),
        ),
    ];
    let checks = vec![
        InstructionCheck::program_result(ProgramResult::UnknownError(
            InstructionError::ExternalAccountLamportSpend,
        )),
        InstructionCheck::compute_units_consumed(DEFAULT_COMPUTE_UNITS),
    ];

    Mollusk::default().process_and_validate_instruction(&instruction, accounts, &checks);
}
