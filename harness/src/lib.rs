//! Solana program runtime test harness for programs.

pub mod programs;

use {
    solana_program_runtime::{
        compute_budget::ComputeBudget, invoke_context::InvokeContext,
        loaded_programs::LoadedProgramsForTxBatch, sysvar_cache::SysvarCache,
        timings::ExecuteTimings,
    },
    solana_sdk::{
        account::AccountSharedData,
        feature_set::FeatureSet,
        hash::Hash,
        instruction::{AccountMeta, Instruction, InstructionError},
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        system_program,
        transaction_context::{InstructionAccount, TransactionContext},
    },
    std::sync::Arc,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramResult {
    Success,
    Failure(ProgramError),
    UnknownError(InstructionError),
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstructionResult {
    pub compute_units_consumed: u64,
    pub execution_time: u64,
    pub result: ProgramResult,
}

pub struct Mollusk {
    pub compute_budget: ComputeBudget,
    pub feature_set: FeatureSet,
    pub program_account: AccountSharedData,
    pub program_cache: LoadedProgramsForTxBatch,
    pub program_id: Pubkey,
    pub rent: Rent,
    pub sysvar_cache: SysvarCache,
}

impl Default for Mollusk {
    fn default() -> Self {
        let compute_budget = ComputeBudget::default();
        let feature_set = FeatureSet::all_enabled();
        let rent = Rent::default();

        let program_account = programs::system_program_account(&rent);
        let program_cache = programs::build_program_cache();
        let program_id = system_program::id();

        let mut sysvar_cache = SysvarCache::default();
        sysvar_cache.set_clock(solana_sdk::clock::Clock::default());
        sysvar_cache.set_epoch_schedule(solana_sdk::epoch_schedule::EpochSchedule::default());
        sysvar_cache.set_rent(rent.clone());
        sysvar_cache.set_slot_hashes(solana_sdk::slot_hashes::SlotHashes::default());

        Self {
            compute_budget,
            feature_set,
            program_account,
            program_cache,
            program_id,
            rent,
            sysvar_cache,
        }
    }
}

impl Mollusk {
    pub fn new(program_id: &Pubkey, program_name: &'static str) -> Self {
        let mut mollusk = Self::default();
        mollusk.program_id = *program_id;
        mollusk.program_account = programs::program_account(program_id, &mollusk.rent);
        programs::add_program_to_cache(
            &mut mollusk.program_cache,
            program_id,
            program_name,
            &mollusk.compute_budget,
            &mollusk.feature_set,
        );
        mollusk
    }

    /// Process an instruction using the simulated Solana program runtime.
    pub fn process_instruction(
        &self,
        instruction: &Instruction,
        accounts: Vec<(Pubkey, AccountSharedData)>,
    ) -> InstructionResult {
        let mut compute_units_consumed = 0;
        let mut timings = ExecuteTimings::default();

        let mut programs_modified_by_tx = LoadedProgramsForTxBatch::default();

        let (program_accounts_len, program_indices) = (1, &[0]);

        let instruction_accounts = instruction
            .accounts
            .iter()
            .enumerate()
            .map(
                |(
                    i,
                    AccountMeta {
                        pubkey: _,
                        is_signer,
                        is_writable,
                    },
                )| InstructionAccount {
                    index_in_callee: i as u16,
                    index_in_caller: i as u16,
                    index_in_transaction: (i + program_accounts_len) as u16,
                    is_signer: *is_signer,
                    is_writable: *is_writable,
                },
            )
            .collect::<Vec<_>>();

        let transaction_accounts = [(self.program_id, self.program_account.clone())]
            .into_iter()
            .chain(accounts)
            .collect::<Vec<_>>();

        let mut transaction_context = TransactionContext::new(
            transaction_accounts,
            self.rent.clone(),
            self.compute_budget.max_invoke_stack_height,
            self.compute_budget.max_instruction_trace_length,
        );

        let mut invoke_context = InvokeContext::new(
            &mut transaction_context,
            &self.sysvar_cache,
            None,
            self.compute_budget,
            &self.program_cache,
            &mut programs_modified_by_tx,
            Arc::new(self.feature_set.clone()),
            Hash::default(),
            0,
        );

        let invoke_result = invoke_context.process_instruction(
            &instruction.data,
            &instruction_accounts,
            program_indices,
            &mut compute_units_consumed,
            &mut timings,
        );

        let result = match invoke_result {
            Ok(()) => ProgramResult::Success,
            Err(err) => {
                if let Ok(program_error) = ProgramError::try_from(err.clone()) {
                    ProgramResult::Failure(program_error)
                } else {
                    ProgramResult::UnknownError(err)
                }
            }
        };

        let execution_time = timings.details.execute_us;

        InstructionResult {
            compute_units_consumed,
            execution_time,
            result,
        }
    }
}
