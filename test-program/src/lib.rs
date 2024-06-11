use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::set_return_data,
    program_error::ProgramError, pubkey::Pubkey,
};

solana_program::declare_id!("239vxAL9Q7e3uLoinJpJ873r3bvT9sPFxH7yekwPppNF");

solana_program::entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    if input.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    set_return_data(input);

    Ok(())
}
