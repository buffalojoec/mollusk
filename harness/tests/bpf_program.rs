use {
    mollusk::{program::system_program_account, result::Check, Mollusk},
    solana_sdk::{
        account::AccountSharedData,
        incinerator,
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction::SystemError,
        system_program,
    },
};

#[test]
fn test_write_data() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");

    let program_id = Pubkey::new_unique();

    let mollusk = Mollusk::new(&program_id, "test_program");

    let data = &[1, 2, 3, 4, 5];
    let space = data.len();
    let lamports = mollusk.get_rent().minimum_balance(space);

    let key = Pubkey::new_unique();
    let account = AccountSharedData::new(lamports, space, &program_id);

    let instruction = {
        let mut instruction_data = vec![1];
        instruction_data.extend_from_slice(data);
        Instruction::new_with_bytes(
            program_id,
            &instruction_data,
            vec![AccountMeta::new(key, true)],
        )
    };

    // Fail account not signer.
    {
        let mut account_not_signer_ix = instruction.clone();
        account_not_signer_ix.accounts[0].is_signer = false;

        mollusk.process_and_validate_instruction(
            &account_not_signer_ix,
            &[(key, account.clone())],
            &[
                Check::err(ProgramError::MissingRequiredSignature),
                Check::compute_units(272),
            ],
        );
    }

    // Fail data too large.
    {
        let mut data_too_large_ix = instruction.clone();
        data_too_large_ix.data = vec![1; space + 2];

        mollusk.process_and_validate_instruction(
            &data_too_large_ix,
            &[(key, account.clone())],
            &[
                Check::err(ProgramError::AccountDataTooSmall),
                Check::compute_units(281),
            ],
        );
    }

    // Success.
    mollusk.process_and_validate_instruction(
        &instruction,
        &[(key, account.clone())],
        &[
            Check::success(),
            Check::compute_units(350),
            Check::account(&key)
                .data(data)
                .lamports(lamports)
                .owner(program_id)
                .build(),
        ],
    );
}

#[test]
fn test_transfer() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");

    let program_id = Pubkey::new_unique();

    let mollusk = Mollusk::new(&program_id, "test_program");

    let payer = Pubkey::new_unique();
    let payer_lamports = 100_000_000;
    let payer_account = AccountSharedData::new(payer_lamports, 0, &system_program::id());

    let recipient = Pubkey::new_unique();
    let recipient_lamports = 0;
    let recipient_account = AccountSharedData::new(recipient_lamports, 0, &system_program::id());

    let transfer_amount = 2_000_000_u64;

    let instruction = {
        let mut instruction_data = vec![2];
        instruction_data.extend_from_slice(&transfer_amount.to_le_bytes());
        Instruction::new_with_bytes(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(payer, true),
                AccountMeta::new(recipient, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    };

    // Fail payer not signer.
    {
        let mut payer_not_signer_ix = instruction.clone();
        payer_not_signer_ix.accounts[0].is_signer = false;

        mollusk.process_and_validate_instruction(
            &payer_not_signer_ix,
            &[
                (payer, payer_account.clone()),
                (recipient, recipient_account.clone()),
                (system_program::id(), system_program_account()),
            ],
            &[
                Check::err(ProgramError::MissingRequiredSignature),
                Check::compute_units(598),
            ],
        );
    }

    // Fail insufficient lamports.
    {
        mollusk.process_and_validate_instruction(
            &instruction,
            &[
                (payer, AccountSharedData::default()),
                (recipient, recipient_account.clone()),
                (system_program::id(), system_program_account()),
            ],
            &[
                Check::err(ProgramError::Custom(
                    SystemError::ResultWithNegativeLamports as u32,
                )),
                Check::compute_units(2256),
            ],
        );
    }

    // Success.
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (payer, payer_account.clone()),
            (recipient, recipient_account.clone()),
            (system_program::id(), system_program_account()),
        ],
        &[
            Check::success(),
            Check::compute_units(2366),
            Check::account(&payer)
                .lamports(payer_lamports - transfer_amount)
                .build(),
            Check::account(&recipient)
                .lamports(recipient_lamports + transfer_amount)
                .build(),
        ],
    );
}

#[test]
fn test_close_account() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");

    let program_id = Pubkey::new_unique();

    let mollusk = Mollusk::new(&program_id, "test_program");

    let key = Pubkey::new_unique();
    let account = AccountSharedData::new(50_000_000, 50, &program_id);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &[3],
        vec![
            AccountMeta::new(key, true),
            AccountMeta::new(incinerator::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // Fail account not signer.
    {
        let mut account_not_signer_ix = instruction.clone();
        account_not_signer_ix.accounts[0].is_signer = false;

        mollusk.process_and_validate_instruction(
            &account_not_signer_ix,
            &[
                (key, account.clone()),
                (incinerator::id(), AccountSharedData::default()),
                (system_program::id(), system_program_account()),
            ],
            &[
                Check::err(ProgramError::MissingRequiredSignature),
                Check::compute_units(598),
            ],
        );
    }

    // Success.
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (key, account.clone()),
            (incinerator::id(), AccountSharedData::default()),
            (system_program::id(), system_program_account()),
        ],
        &[
            Check::success(),
            Check::compute_units(2558),
            Check::account(&key)
                .data(&[])
                .lamports(0)
                .owner(system_program::id())
                .closed()
                .build(),
        ],
    );
}
