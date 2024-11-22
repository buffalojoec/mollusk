//! Module for converting to and from Mollusk SVM Firedancer fuzz fixtures and
//! Mollusk types. These conversions allow Mollusk to eject Firedancer fuzzing
//! fixtures from tests, amongst other things.
//!
//! Only available when the `fuzz-fd` feature is enabled.

use {
    crate::{
        accounts::{compile_accounts, CompiledAccounts},
        result::{Check, InstructionResult},
        Mollusk, DEFAULT_LOADER_KEY,
    },
    mollusk_svm_fuzz_fixture_firedancer::{
        context::{
            Context as FuzzContext, EpochContext as FuzzEpochContext,
            SlotContext as FuzzSlotContext,
        },
        effects::Effects as FuzzEffects,
        metadata::Metadata as FuzzMetadata,
        Fixture as FuzzFixture,
    },
    solana_sdk::{
        account::AccountSharedData,
        instruction::{Instruction, InstructionError},
        pubkey::Pubkey,
    },
};

mod zk_token_proof_program {
    solana_sdk::declare_id!("ZkTokenProof1111111111111111111111111111111");
}

mod zk_elgamal_proof_program {
    solana_sdk::declare_id!("ZkE1Gama1Proof11111111111111111111111111111");
}

static BUILTIN_PROGRAM_IDS: &[Pubkey] = &[
    solana_sdk::system_program::id(),
    solana_sdk::vote::program::id(),
    solana_sdk::stake::program::id(),
    solana_sdk::config::program::id(),
    solana_sdk::bpf_loader_deprecated::id(),
    solana_sdk::bpf_loader::id(),
    solana_sdk::bpf_loader_upgradeable::id(),
    solana_sdk::compute_budget::id(),
    solana_sdk::address_lookup_table::program::id(),
    zk_token_proof_program::id(),
    solana_sdk::loader_v4::id(),
    zk_elgamal_proof_program::id(),
];

fn instr_err_to_num(error: &InstructionError) -> i32 {
    let serialized_err = bincode::serialize(error).unwrap();
    i32::from_le_bytes((&serialized_err[0..4]).try_into().unwrap()) + 1
}

fn build_fixture_context(
    mollusk: &Mollusk,
    instruction: &Instruction,
    accounts: &[(Pubkey, AccountSharedData)],
) -> FuzzContext {
    let Mollusk {
        compute_budget,
        feature_set,
        sysvars,
        ..
    } = mollusk;

    let loader_key = if BUILTIN_PROGRAM_IDS.contains(&instruction.program_id) {
        solana_sdk::native_loader::id()
    } else {
        DEFAULT_LOADER_KEY
    };

    let CompiledAccounts {
        instruction_accounts,
        transaction_accounts,
        ..
    } = compile_accounts(instruction, accounts, loader_key);

    let accounts = transaction_accounts
        .into_iter()
        .map(|(key, account)| (key, account, None))
        .collect::<Vec<_>>();

    FuzzContext {
        program_id: instruction.program_id,
        accounts,
        instruction_accounts,
        instruction_data: instruction.data.clone(),
        compute_units_available: compute_budget.compute_unit_limit,
        slot_context: FuzzSlotContext {
            slot: sysvars.clock.slot,
        },
        epoch_context: FuzzEpochContext {
            feature_set: feature_set.clone(),
        },
    }
}

fn build_fixture_effects(context: &FuzzContext, result: &InstructionResult) -> FuzzEffects {
    let mut program_custom_code = 0;
    let program_result = match &result.raw_result {
        Ok(()) => 0,
        Err(e) => {
            if let InstructionError::Custom(code) = e {
                program_custom_code = *code;
            }
            instr_err_to_num(e)
        }
    };

    let modified_accounts = context
        .accounts
        .iter()
        .filter_map(|(key, account, seed_addr)| {
            if let Some((_, resulting_account)) =
                result.resulting_accounts.iter().find(|(k, _)| k == key)
            {
                if account != resulting_account {
                    return Some((*key, resulting_account.clone(), seed_addr.clone()));
                }
            }
            None
        })
        .collect();

    FuzzEffects {
        program_result,
        program_custom_code,
        modified_accounts,
        compute_units_available: context
            .compute_units_available
            .saturating_sub(result.compute_units_consumed),
        return_data: Vec::new(), // TODO: Mollusk doesn't capture return data.
    }
}

fn instruction_metadata() -> FuzzMetadata {
    FuzzMetadata {
        // Mollusk is always an instruction harness.
        entrypoint: String::from("sol_compat_instr_execute_v1"),
    }
}

pub fn build_fixture_from_mollusk_test(
    mollusk: &Mollusk,
    instruction: &Instruction,
    accounts: &[(Pubkey, AccountSharedData)],
    result: &InstructionResult,
    _checks: &[Check],
) -> FuzzFixture {
    let input = build_fixture_context(mollusk, instruction, accounts);
    // This should probably be built from the checks, but there's currently no
    // mechanism to enforce full check coverage on a result.
    let output = build_fixture_effects(&input, result);
    FuzzFixture {
        metadata: Some(instruction_metadata()),
        input,
        output,
    }
}
