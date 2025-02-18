use {
    mollusk_svm::{result::Check, Mollusk},
    solana_account::Account,
    solana_instruction::error::InstructionError,
    solana_pubkey::Pubkey,
    solana_system_program::system_processor::DEFAULT_COMPUTE_UNITS,
};

#[test]
fn test_transfer() {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let instruction =
        solana_system_interface::instruction::transfer(&sender, &recipient, transfer_amount);
    let accounts = [
        (
            sender,
            Account::new(base_lamports, 0, &solana_sdk_ids::system_program::id()),
        ),
        (
            recipient,
            Account::new(base_lamports, 0, &solana_sdk_ids::system_program::id()),
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
fn test_transfer_account_ordering() {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100_000_000u64;
    let transfer_amount = 42_000u64;

    let instruction =
        solana_system_interface::instruction::transfer(&sender, &recipient, transfer_amount);

    // Ordering of provided accounts doesn't matter.
    let accounts = [
        (
            recipient,
            Account::new(base_lamports, 0, &solana_sdk_ids::system_program::id()),
        ),
        (
            sender,
            Account::new(base_lamports, 0, &solana_sdk_ids::system_program::id()),
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

    let instruction =
        solana_system_interface::instruction::transfer(&sender, &recipient, transfer_amount);
    let accounts = [
        (
            sender,
            Account::new(base_lamports, 0, &Pubkey::new_unique()), // <-- Bad owner.
        ),
        (
            recipient,
            Account::new(base_lamports, 0, &solana_sdk_ids::system_program::id()),
        ),
    ];
    let checks = vec![
        Check::instruction_err(InstructionError::ExternalAccountLamportSpend),
        Check::compute_units(DEFAULT_COMPUTE_UNITS),
    ];

    Mollusk::default().process_and_validate_instruction(&instruction, &accounts, &checks);
}
