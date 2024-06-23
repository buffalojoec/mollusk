use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

solana_program::declare_id!("MD24T7azhc2q9ZXaeskbLpmVA41k7StzTGgcfvGcpHj");

solana_program::entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    // Simply write the input data to the first account.
    let accounts_iter = &mut accounts.iter();

    let account_info = next_account_info(accounts_iter)?;

    if !account_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if account_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if input.len() > account_info.data_len() {
        return Err(ProgramError::AccountDataTooSmall);
    }

    account_info.try_borrow_mut_data()?[..].copy_from_slice(input);

    Ok(())
}
