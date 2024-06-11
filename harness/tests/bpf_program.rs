use {
    mollusk::{Mollusk, ProgramResult},
    solana_sdk::{instruction::Instruction, program_error::ProgramError, pubkey::Pubkey},
};

#[test]
fn test_set_return_data() {
    let program_id = Pubkey::new_unique();

    let input = vec![1];

    let instruction = Instruction::new_with_bytes(program_id, &input, vec![]);

    let mollusk = Mollusk::new(&program_id, "test_program");

    let result = mollusk.process_instruction(&instruction, vec![]);

    assert_eq!(result.result, ProgramResult::Success);
    assert_eq!(result.compute_units_consumed, 143);
}

#[test]
fn test_fail_empty_input() {
    let program_id = Pubkey::new_unique();

    let instruction = Instruction::new_with_bytes(program_id, &[], vec![]);

    let mollusk = Mollusk::new(&program_id, "test_program");

    let result = mollusk.process_instruction(&instruction, vec![]);

    assert_eq!(
        result.result,
        ProgramResult::Failure(ProgramError::InvalidInstructionData)
    );
    assert_eq!(result.compute_units_consumed, 55);
}
