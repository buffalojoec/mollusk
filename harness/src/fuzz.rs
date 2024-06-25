//! Module for converting from Mollusk SVM Fuzz fixtures to Mollusk types.
//!
//! These conversions allow Mollusk to eject fuzzing fixtures from tests.
//!
//! Only available when the `fuzz` feature is enabled.

pub use mollusk_svm_fuzz::*;
use {
    crate::{
        result::{Check, InstructionResult, ProgramResult},
        sysvar::Sysvars,
        Mollusk,
    },
    fixture::{
        context::FixtureContext, effects::FixtureEffects, sysvars::FixtureSysvarContext, Fixture,
    },
    solana_sdk::{
        account::AccountSharedData, instruction::Instruction, pubkey::Pubkey,
        slot_hashes::SlotHashes,
    },
};

impl From<&Sysvars> for FixtureSysvarContext {
    fn from(input: &Sysvars) -> Self {
        let slot_hashes = SlotHashes::new(&input.slot_hashes);
        Self {
            clock: input.clock.clone(),
            epoch_rewards: input.epoch_rewards,
            epoch_schedule: input.epoch_schedule,
            rent: input.rent,
            slot_hashes,
            stake_history: input.stake_history.clone(),
        }
    }
}

impl From<&InstructionResult> for FixtureEffects {
    fn from(input: &InstructionResult) -> Self {
        let compute_units_consumed = input.compute_units_consumed;
        let execution_time = input.execution_time;
        let program_result = match &input.program_result {
            ProgramResult::Success => 0,
            ProgramResult::Failure(e) => e.clone().into(),
            ProgramResult::UnknownError(_) => u64::MAX, //TODO
        } as u32; // Also TODO.

        let resulting_accounts = input
            .resulting_accounts
            .iter()
            .map(|(pubkey, account)| (*pubkey, account.clone()))
            .collect();

        Self {
            compute_units_consumed,
            execution_time,
            program_result,
            resulting_accounts,
        }
    }
}

fn build_foxture_context(
    mollusk: &Mollusk,
    instruction: &Instruction,
    accounts: &[(Pubkey, AccountSharedData)],
) -> FixtureContext {
    let Mollusk {
        compute_budget,
        feature_set,
        sysvars,
        program_id,
        ..
    } = mollusk;

    let instruction_accounts = instruction.accounts.clone();
    let instruction_data = instruction.data.clone();
    let accounts = accounts.to_vec();

    FixtureContext {
        compute_budget: *compute_budget,
        feature_set: feature_set.clone(),
        sysvar_context: sysvars.into(),
        program_id: *program_id,
        instruction_accounts,
        instruction_data,
        accounts,
    }
}

pub fn build_fixture_from_mollusk_test(
    mollusk: &Mollusk,
    instruction: &Instruction,
    accounts: &[(Pubkey, AccountSharedData)],
    result: &InstructionResult,
    _checks: &[Check],
) -> Fixture {
    let input = build_foxture_context(mollusk, instruction, accounts);
    // This should probably be built from the checks, but there's currently no
    // mechanism to enforce full check coverage on a result.
    let output = FixtureEffects::from(result);
    Fixture { input, output }
}
