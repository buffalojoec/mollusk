use {
    mollusk_svm::{result::Check, Mollusk},
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
        Check::compute_units(DEFAULT_COMPUTE_UNITS),
        Check::account(&sender)
            .lamports(base_lamports - transfer_amount)
            .build(),
        Check::account(&recipient)
            .lamports(base_lamports + transfer_amount)
            .build(),
    ];

    Mollusk::default().process_and_validate_instruction(&instruction, &accounts, &checks);
}

#[test]
fn test_transfer_bad_owner() {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);
    let accounts = [
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
        Check::instruction_err(InstructionError::ExternalAccountLamportSpend),
        Check::compute_units(DEFAULT_COMPUTE_UNITS),
    ];

    Mollusk::default().process_and_validate_instruction(&instruction, &accounts, &checks);
}
