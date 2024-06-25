//! Mollusk SVM program test harness with fuzz fixture support.
//!
//! Offers a similar API to `mollusk-svm`, but with the ability to eject fuzz
//! fixtures from defined tests.

pub use mollusk_svm::*;
use {
    crate::result::{Check, InstructionResult},
    mollusk_svm_fuzz::fixture::{context::FixtureContext, effects::FixtureEffects, Fixture},
    solana_sdk::{account::AccountSharedData, instruction::Instruction, pubkey::Pubkey},
};

pub struct Oyster;

impl Oyster {
    /// Process an instruction using the minified Solana Virtual Machine (SVM)
    /// environment, then perform checks on the result. Panics if any checks
    /// fail.
    ///
    /// If the `EJECT_FUZZ_FIXTURES` environment variable is set, this function
    /// will convert the provided test to a fuzz fixture and write it to the
    /// provided directory.
    ///
    /// ```ignore
    /// EJECT_FUZZ_FIXTURES="./fuzz-fixtures" cargo test-sbf ...
    /// ```
    pub fn process_and_validate_instruction(
        mollusk: &Mollusk,
        instruction: &Instruction,
        accounts: &[(Pubkey, AccountSharedData)],
        checks: &[Check],
    ) -> InstructionResult {
        let result = mollusk.process_and_validate_instruction(instruction, accounts, checks);

        if let Ok(dir_path) = std::env::var("EJECT_FUZZ_FIXTURES") {
            build_fixture_from_mollusk_test(mollusk, instruction, accounts, &result, checks)
                .dump(&dir_path);
        }

        result
    }
}

fn build_fixture_from_mollusk_test(
    mollusk: &Mollusk,
    instruction: &Instruction,
    accounts: &[(Pubkey, AccountSharedData)],
    result: &InstructionResult,
    _checks: &[Check],
) -> Fixture {
    let input = FixtureContext::from_mollusk_test(mollusk, instruction, accounts);
    // This should probably be built from the checks, but there's currently no
    // mechanism to enforce full check coverage on a result.
    let output = FixtureEffects::from_mollusk_result(result);
    Fixture { input, output }
}
