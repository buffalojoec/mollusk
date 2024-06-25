//! Mollusk is a lightweight test harness for Solana programs. It provides a
//! simple interface for testing Solana program executions in a minified
//! Solana Virtual Machine (SVM) environment.
//!
//! It does not create any semblance of a validator runtime, but instead
//! provisions a program execution pipeline directly from lower-level SVM
//! components.
//!
//! In summary, the main processor - `process_instruction` - creates minified
//! instances of Agave's program cache, transaction context, and invoke
//! context. It uses these components to directly execute the provided
//! program's ELF using the BPF Loader.
//!
//! Because it does not use AccountsDB, Bank, or any other large Agave
//! components, the harness is exceptionally fast. However, it does require
//! the user to provide an explicit list of accounts to use, since it has
//! nowhere to load them from.
//!
//! The test environment can be further configured by adjusting the compute
//! budget, feature set, or sysvars. These configurations are stored directly
//! on the test harness (the `Mollusk` struct), but can be manipulated through
//! a handful of helpers.
//!
//! Two main API methods are offered:
//!
//! * `process_instruction`: Process an instruction and return the result.
//! * `process_and_validate_instruction`: Process an instruction and perform a
//!   series of checks on the result, panicking if any checks fail.

pub mod file;
pub mod program;
pub mod result;
pub mod sysvar;

use {
    crate::{
        program::ProgramCache,
        result::{Check, InstructionResult},
        sysvar::Sysvars,
    },
    solana_program_runtime::{
        compute_budget::ComputeBudget, invoke_context::InvokeContext,
        loaded_programs::LoadedProgramsForTxBatch, sysvar_cache::SysvarCache,
        timings::ExecuteTimings,
    },
    solana_sdk::{
        account::AccountSharedData,
        feature_set::FeatureSet,
        hash::Hash,
        instruction::Instruction,
        pubkey::Pubkey,
        rent::Rent,
        transaction_context::{InstructionAccount, TransactionContext},
    },
    std::sync::Arc,
};

const PROGRAM_ACCOUNTS_LEN: usize = 1;
const PROGRAM_INDICES: &[u16] = &[0];

/// The Mollusk API, providing a simple interface for testing Solana programs.
///
/// All fields can be manipulated through a handful of helper methods, but
/// users can also directly access and modify them if they desire more control.
pub struct Mollusk {
    pub compute_budget: ComputeBudget,
    pub feature_set: FeatureSet,
    pub program_account: AccountSharedData,
    pub program_cache: ProgramCache,
    pub program_id: Pubkey,
    pub sysvars: Sysvars,
}

impl Default for Mollusk {
    fn default() -> Self {
        #[rustfmt::skip]
        solana_logger::setup_with_default(
            "solana_rbpf::vm=debug,\
             solana_runtime::message_processor=debug,\
             solana_runtime::system_instruction_processor=trace",
        );
        let (program_id, program_account) = program::system_program();
        Self {
            compute_budget: ComputeBudget::default(),
            feature_set: FeatureSet::all_enabled(),
            program_account,
            program_cache: ProgramCache::default(),
            program_id,
            sysvars: Sysvars::default(),
        }
    }
}

impl Mollusk {
    /// Create a new Mollusk instance for the provided program.
    ///
    /// Attempts the load the program's ELF file from the default search paths.
    /// Once loaded, adds the program to the program cache and updates the
    /// Mollusk instance with the program's ID and account.
    pub fn new(program_id: &Pubkey, program_name: &'static str) -> Self {
        let mut mollusk = Self {
            program_id: *program_id,
            program_account: program::program_account(program_id),
            ..Default::default()
        };
        mollusk.add_program(program_id, program_name);
        mollusk
    }

    /// Add a program to the test environment.
    ///
    /// If you intend to CPI to a program, this is likely what you want to use.
    pub fn add_program(&mut self, program_id: &Pubkey, program_name: &'static str) {
        let elf = file::load_program_elf(program_name);
        self.program_cache
            .add_program(program_id, &elf, &self.compute_budget, &self.feature_set);
    }

    /// Warp the test environment to a slot by updating sysvars.
    pub fn warp_to_slot(&mut self, slot: u64) {
        self.sysvars.warp_to_slot(slot)
    }

    /// The main Mollusk API method.
    ///
    /// Process an instruction using the minified Solana Virtual Machine (SVM)
    /// environment. Simply returns the result.
    pub fn process_instruction(
        &self,
        instruction: &Instruction,
        accounts: &[(Pubkey, AccountSharedData)],
    ) -> InstructionResult {
        let mut compute_units_consumed = 0;
        let mut timings = ExecuteTimings::default();

        let instruction_accounts = instruction
            .accounts
            .iter()
            .enumerate()
            .map(|(i, meta)| InstructionAccount {
                index_in_callee: i as u16,
                index_in_caller: i as u16,
                index_in_transaction: (i + PROGRAM_ACCOUNTS_LEN) as u16,
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect::<Vec<_>>();

        let transaction_accounts = [(self.program_id, self.program_account.clone())]
            .iter()
            .chain(accounts)
            .cloned()
            .collect::<Vec<_>>();

        let mut transaction_context = TransactionContext::new(
            transaction_accounts,
            Rent::default(),
            self.compute_budget.max_invoke_stack_height,
            self.compute_budget.max_instruction_trace_length,
        );

        let invoke_result = {
            let mut programs_modified_by_tx = LoadedProgramsForTxBatch::default();
            let sysvar_cache = SysvarCache::from(&self.sysvars);

            let mut invoke_context = InvokeContext::new(
                &mut transaction_context,
                &sysvar_cache,
                None,
                self.compute_budget,
                self.program_cache.cache(),
                &mut programs_modified_by_tx,
                Arc::new(self.feature_set.clone()),
                Hash::default(),
                0,
            );

            invoke_context.process_instruction(
                &instruction.data,
                &instruction_accounts,
                PROGRAM_INDICES,
                &mut compute_units_consumed,
                &mut timings,
            )
        };

        let resulting_accounts = transaction_context
            .deconstruct_without_keys()
            .unwrap()
            .into_iter()
            .skip(PROGRAM_ACCOUNTS_LEN)
            .zip(instruction.accounts.iter())
            .map(|(account, meta)| (meta.pubkey, account))
            .collect::<Vec<_>>();

        InstructionResult {
            compute_units_consumed,
            execution_time: timings.details.execute_us,
            program_result: invoke_result.into(),
            resulting_accounts,
        }
    }

    /// The secondary Mollusk API method.
    ///
    /// Process an instruction using the minified Solana Virtual Machine (SVM)
    /// environment, then perform checks on the result. Panics if any checks
    /// fail.
    pub fn process_and_validate_instruction(
        &self,
        instruction: &Instruction,
        accounts: &[(Pubkey, AccountSharedData)],
        checks: &[Check],
    ) -> InstructionResult {
        let result = self.process_instruction(instruction, accounts);
        result.run_checks(checks);
        result
    }
}
