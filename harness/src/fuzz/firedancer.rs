//! Module for converting to and from Mollusk SVM Firedancer fuzz fixtures and
//! Mollusk types. These conversions allow Mollusk to eject Firedancer fuzzing
//! fixtures from tests, amongst other things.
//!
//! Only available when the `fuzz-fd` feature is enabled.

use {
    crate::{
        accounts::{compile_accounts, CompiledAccounts},
        result::InstructionResult,
        Mollusk, DEFAULT_LOADER_KEY,
    },
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_sdk::{
        account::Account,
        feature_set::FeatureSet,
        instruction::{AccountMeta, Instruction, InstructionError},
        pubkey::Pubkey,
        transaction_context::InstructionAccount,
    },
    solana_svm_fuzz_harness_fixture::{
        context::{
            epoch_context::EpochContext as FuzzEpochContext,
            slot_context::SlotContext as FuzzSlotContext,
        },
        invoke::{
            context::InstrContext as FuzzContext, effects::InstrEffects as FuzzEffects,
            instr_account::InstrAccount as FuzzInstrAccount,
            metadata::FixtureMetadata as FuzzMetadata, InstrFixture as FuzzFixture,
        },
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

fn build_fixture_context(
    accounts: &[(Pubkey, Account)],
    compute_budget: &ComputeBudget,
    feature_set: &FeatureSet,
    instruction: &Instruction,
    slot: u64,
) -> FuzzContext {
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

    let instruction_accounts = instruction_accounts
        .into_iter()
        .map(
            |InstructionAccount {
                 index_in_transaction,
                 is_signer,
                 is_writable,
                 ..
             }| FuzzInstrAccount {
                index: index_in_transaction as u32,
                is_signer,
                is_writable,
            },
        )
        .collect();

    FuzzContext {
        program_id: instruction.program_id,
        accounts: transaction_accounts,
        instruction_accounts,
        instruction_data: instruction.data.clone(),
        compute_units_available: compute_budget.compute_unit_limit,
        slot_context: FuzzSlotContext { slot },
        epoch_context: FuzzEpochContext {
            feature_set: feature_set.clone(),
        },
    }
}

pub struct ParsedFixtureContext {
    pub accounts: Vec<(Pubkey, Account)>,
    pub compute_budget: ComputeBudget,
    pub feature_set: FeatureSet,
    pub instruction: Instruction,
    pub slot: u64,
}

pub(crate) fn parse_fixture_context(context: &FuzzContext) -> ParsedFixtureContext {
    let FuzzContext {
        program_id,
        accounts,
        instruction_accounts,
        instruction_data,
        compute_units_available,
        slot_context,
        epoch_context,
    } = context;

    let compute_budget = ComputeBudget {
        compute_unit_limit: *compute_units_available,
        ..Default::default()
    };

    let metas = instruction_accounts
        .iter()
        .map(|ia| {
            let pubkey = accounts
                .get(ia.index as usize)
                .expect("Index out of bounds")
                .0;
            AccountMeta {
                pubkey,
                is_signer: ia.is_signer,
                is_writable: ia.is_writable,
            }
        })
        .collect::<Vec<_>>();

    let instruction = Instruction::new_with_bytes(*program_id, instruction_data, metas);

    ParsedFixtureContext {
        accounts: accounts.clone(),
        compute_budget,
        feature_set: epoch_context.feature_set.clone(),
        instruction,
        slot: slot_context.slot,
    }
}

fn build_fixture_effects(context: &FuzzContext, result: &InstructionResult) -> FuzzEffects {
    let program_custom_code = if let Err(InstructionError::Custom(code)) = &result.raw_result {
        Some(*code)
    } else {
        None
    };

    let modified_accounts = context
        .accounts
        .iter()
        .filter_map(|(key, account)| {
            if let Some((_, resulting_account)) =
                result.resulting_accounts.iter().find(|(k, _)| k == key)
            {
                if account != resulting_account {
                    return Some((*key, resulting_account.clone()));
                }
            }
            None
        })
        .collect();

    FuzzEffects {
        program_result: result.raw_result.clone().err(),
        program_custom_code,
        modified_accounts,
        compute_units_available: context
            .compute_units_available
            .saturating_sub(result.compute_units_consumed),
        return_data: result.return_data.clone(),
    }
}

pub(crate) fn parse_fixture_effects(
    accounts: &[(Pubkey, Account)],
    compute_unit_limit: u64,
    effects: &FuzzEffects,
) -> InstructionResult {
    let raw_result = effects.program_result.clone().map_or(Ok(()), Err);

    let program_result = raw_result.clone().into();
    let return_data = effects.return_data.clone();

    let resulting_accounts = accounts
        .iter()
        .map(|(key, acct)| {
            let resulting_account = effects
                .modified_accounts
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, acct)| acct.clone())
                .unwrap_or_else(|| acct.clone());
            (*key, resulting_account)
        })
        .collect();

    InstructionResult {
        program_result,
        raw_result,
        execution_time: 0, // TODO: Omitted for now.
        compute_units_consumed: compute_unit_limit.saturating_sub(effects.compute_units_available),
        return_data,
        resulting_accounts,
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
    accounts: &[(Pubkey, Account)],
    result: &InstructionResult,
) -> FuzzFixture {
    let input = build_fixture_context(
        accounts,
        &mollusk.compute_budget,
        &mollusk.feature_set,
        instruction,
        mollusk.slot, // FD-fuzz feature only.
    );
    // This should probably be built from the checks, but there's currently no
    // mechanism to enforce full check coverage on a result.
    let output = build_fixture_effects(&input, result);
    FuzzFixture {
        metadata: Some(instruction_metadata()),
        input,
        output,
    }
}

pub fn load_firedancer_fixture(fixture: &FuzzFixture) -> (ParsedFixtureContext, InstructionResult) {
    let parsed = parse_fixture_context(&fixture.input);
    let result = parse_fixture_effects(
        &parsed.accounts,
        parsed.compute_budget.compute_unit_limit,
        &fixture.output,
    );
    (parsed, result)
}
