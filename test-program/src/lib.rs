use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    incinerator,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction, system_program,
};

solana_program::declare_id!("239vxAL9Q7e3uLoinJpJ873r3bvT9sPFxH7yekwPppNF");

solana_program::entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    match input.split_first() {
        Some((0, _)) => {
            // No-op.
        }
        Some((1, rest)) => {
            // Simply write the remaining data to the first account.
            let account_info = next_account_info(accounts_iter)?;

            if !account_info.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            if rest.len() > account_info.data_len() {
                return Err(ProgramError::AccountDataTooSmall);
            }

            account_info.try_borrow_mut_data()?[..].copy_from_slice(rest);
        }
        Some((2, rest)) if rest.len() == 8 => {
            // Transfer from the first account to the second.
            let payer_info = next_account_info(accounts_iter)?;
            let recipient_info = next_account_info(accounts_iter)?;
            let _system_program = next_account_info(accounts_iter)?;

            if !payer_info.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            let lamports = u64::from_le_bytes(rest.try_into().unwrap());

            invoke(
                &system_instruction::transfer(payer_info.key, recipient_info.key, lamports),
                &[payer_info.clone(), recipient_info.clone()],
            )?;
        }
        Some((3, _)) => {
            // Close the first account and burn its lamports.
            let account_info = next_account_info(accounts_iter)?;
            let incinerator_info = next_account_info(accounts_iter)?;
            let _system_program = next_account_info(accounts_iter)?;

            if !account_info.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            account_info.realloc(0, true)?;
            account_info.assign(&system_program::id());

            let lamports = account_info.lamports();

            invoke(
                &system_instruction::transfer(account_info.key, &incinerator::id(), lamports),
                &[account_info.clone(), incinerator_info.clone()],
            )?;
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}
