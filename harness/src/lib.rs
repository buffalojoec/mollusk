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

mod error;
pub mod file;
mod keys;
pub mod program;
pub mod result;
pub mod sysvar;

use {
    crate::{
        error::{MolluskError, MolluskPanic},
        program::ProgramCache,
        result::{Check, InstructionResult},
        sysvar::Sysvars,
    },
    keys::CompiledAccounts,
    solana_program_runtime::{
        compute_budget::ComputeBudget, invoke_context::InvokeContext, sysvar_cache::SysvarCache,
        timings::ExecuteTimings,
    },
    solana_sdk::{
        account::AccountSharedData, bpf_loader_upgradeable, feature_set::FeatureSet,
        fee::FeeStructure, hash::Hash, instruction::Instruction, pubkey::Pubkey,
        transaction_context::TransactionContext,
    },
    std::sync::Arc,
};

const DEFAULT_LOADER_KEY: Pubkey = bpf_loader_upgradeable::id();

/// The Mollusk API, providing a simple interface for testing Solana programs.
///
/// All fields can be manipulated through a handful of helper methods, but
/// users can also directly access and modify them if they desire more control.
pub struct Mollusk {
    pub compute_budget: ComputeBudget,
    pub feature_set: FeatureSet,
    pub fee_structure: FeeStructure,
    program_cache: ProgramCache,
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
        Self {
            compute_budget: ComputeBudget::default(),
            feature_set: FeatureSet::all_enabled(),
            fee_structure: FeeStructure::default(),
            program_cache: ProgramCache::default(),
            sysvars: Sysvars::default(),
        }
    }
}

impl Mollusk {
    /// Create a new Mollusk instance containing the provided program.
    ///
    /// Attempts the load the program's ELF file from the default search paths.
    /// Once loaded, adds the program to the program cache and returns the
    /// newly created Mollusk instance.
    pub fn new(program_id: &Pubkey, program_name: &str) -> Self {
        let mut mollusk = Self::default();
        mollusk.add_program(program_id, program_name);
        mollusk
    }

    /// Add a program to the test environment.
    ///
    /// If you intend to CPI to a program, this is likely what you want to use.
    pub fn add_program(&mut self, program_id: &Pubkey, program_name: &str) {
        let elf = file::load_program_elf(program_name);
        self.add_program_with_elf(program_id, &elf);
    }

    /// Add a program to the test environment using a provided ELF under a
    /// specific loader.
    ///
    /// If you intend to CPI to a program, this is likely what you want to use.
    pub fn add_program_with_elf(&mut self, program_id: &Pubkey, elf: &[u8]) {
        self.program_cache.add_program(
            program_id,
            &DEFAULT_LOADER_KEY,
            elf,
            &self.compute_budget,
            &self.feature_set,
        );
    }

    /// Warp the test environment to a slot by updating sysvars.
    pub fn warp_to_slot(&mut self, slot: u64) {
        self.sysvars.warp_to_slot(slot)
    }

    /// Process an instruction using the minified Solana Virtual Machine (SVM)
    /// environment. Simply returns the result.
    pub fn process_instruction(
        &self,
        instruction: &Instruction,
        accounts: &[(Pubkey, AccountSharedData)],
    ) -> InstructionResult {
        let mut compute_units_consumed = 0;
        let mut timings = ExecuteTimings::default();

        self.program_cache
            .load_program(&instruction.program_id)
            .or_panic_with(MolluskError::ProgramNotCached(&instruction.program_id));

        let loader_key = if crate::program::is_builtin(&instruction.program_id) {
            solana_sdk::native_loader::id()
        } else {
            DEFAULT_LOADER_KEY
        };

        let CompiledAccounts {
            program_id_index,
            instruction_accounts,
            transaction_accounts,
        } = crate::keys::compile_accounts(instruction, accounts, loader_key);

        let mut transaction_context = TransactionContext::new(
            transaction_accounts,
            self.sysvars.rent,
            self.compute_budget.max_invoke_stack_height,
            self.compute_budget.max_instruction_trace_length,
        );

        let invoke_result = {
            let readonly_cache = self.program_cache.cache().read().unwrap().clone();
            let mut cache = self.program_cache.cache().write().unwrap();
            InvokeContext::new(
                &mut transaction_context,
                &SysvarCache::from(&self.sysvars),
                None,
                self.compute_budget,
                &readonly_cache,
                &mut cache,
                Arc::new(self.feature_set.clone()),
                Hash::default(),
                self.fee_structure.lamports_per_signature,
            )
            .process_instruction(
                &instruction.data,
                &instruction_accounts,
                &[program_id_index],
                &mut compute_units_consumed,
                &mut timings,
            )
        };

        let resulting_accounts: Vec<(Pubkey, AccountSharedData)> = (0..transaction_context
            .get_number_of_accounts())
            .filter_map(|index| {
                let key = transaction_context
                    .get_key_of_account_at_index(index)
                    .unwrap();
                let account = transaction_context.get_account_at_index(index).unwrap();
                if *key != instruction.program_id {
                    Some((*key, account.take()))
                } else {
                    None
                }
            })
            .collect();

        InstructionResult {
            compute_units_consumed,
            execution_time: timings.details.execute_us,
            program_result: invoke_result.into(),
            resulting_accounts,
        }
    }

    /// Process a chain of instructions using the minified Solana Virtual
    /// Machine (SVM) environment. The returned result is an
    /// `InstructionResult`, containing:
    ///
    /// * `compute_units_consumed`: The total compute units consumed across all
    ///   instructions.
    /// * `execution_time`: The total execution time across all instructions.
    /// * `program_result`: The program result of the _last_ instruction.
    /// * `resulting_accounts`: The resulting accounts after the _last_
    ///   instruction.
    pub fn process_instruction_chain(
        &self,
        instructions: &[Instruction],
        accounts: &[(Pubkey, AccountSharedData)],
    ) -> InstructionResult {
        let mut result = InstructionResult {
            resulting_accounts: accounts.to_vec(),
            ..Default::default()
        };

        for instruction in instructions {
            result.absorb(self.process_instruction(instruction, &result.resulting_accounts));
            if result.program_result.is_err() {
                break;
            }
        }

        result
    }

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

    /// Process a chain of instructions using the minified Solana Virtual
    /// Machine (SVM) environment, then perform checks on the result.
    /// Panics if any checks fail.
    pub fn process_and_validate_instruction_chain(
        &self,
        instructions: &[Instruction],
        accounts: &[(Pubkey, AccountSharedData)],
        checks: &[Check],
    ) -> InstructionResult {
        let result = self.process_instruction_chain(instructions, accounts);
        result.run_checks(checks);
        result
    }
}
