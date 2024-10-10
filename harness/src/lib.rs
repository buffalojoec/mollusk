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
#[cfg(feature = "fuzz")]
pub mod fuzz;
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
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_program_runtime::{
        invoke_context::{EnvironmentConfig, InvokeContext},
        sysvar_cache::SysvarCache,
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
    pub program_cache: ProgramCache,
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
        mollusk.add_program(program_id, program_name, &DEFAULT_LOADER_KEY);
        mollusk
    }

    /// Add a program to the test environment.
    ///
    /// If you intend to CPI to a program, this is likely what you want to use.
    pub fn add_program(&mut self, program_id: &Pubkey, program_name: &str, loader_key: &Pubkey) {
        let elf = file::load_program_elf(program_name);
        self.add_program_with_elf_and_loader(program_id, &elf, loader_key);
    }

    /// Add a program to the test environment using a provided ELF under a
    /// specific loader.
    ///
    /// If you intend to CPI to a program, this is likely what you want to use.
    pub fn add_program_with_elf_and_loader(
        &mut self,
        program_id: &Pubkey,
        elf: &[u8],
        loader_key: &Pubkey,
    ) {
        self.program_cache.add_program(
            program_id,
            loader_key,
            elf,
            &self.compute_budget,
            &self.feature_set,
        );
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

        let loader_key = self
            .program_cache
            .load_program(&instruction.program_id)
            .or_panic_with(MolluskError::ProgramNotCached(&instruction.program_id))
            .account_owner();

        let CompiledAccounts {
            program_id_index,
            instruction_accounts,
            transaction_accounts,
        } = crate::keys::compile_accounts(instruction, accounts, loader_key);

        let mut transaction_context = TransactionContext::new(
            transaction_accounts,
            self.sysvars.rent.clone(),
            self.compute_budget.max_instruction_stack_depth,
            self.compute_budget.max_instruction_trace_length,
        );

        let invoke_result = {
            let mut cache = self.program_cache.cache().write().unwrap();
            InvokeContext::new(
                &mut transaction_context,
                &mut cache,
                EnvironmentConfig::new(
                    Hash::default(),
                    None,
                    None,
                    Arc::new(self.feature_set.clone()),
                    self.fee_structure.lamports_per_signature,
                    &SysvarCache::from(&self.sysvars),
                ),
                None,
                self.compute_budget,
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

    /// The secondary Mollusk API method.
    ///
    /// Process an instruction using the minified Solana Virtual Machine (SVM)
    /// environment, then perform checks on the result. Panics if any checks
    /// fail.
    ///
    /// For `fuzz` feature only:
    ///
    /// If the `EJECT_FUZZ_FIXTURES` environment variable is set, this function
    /// will convert the provided test to a fuzz fixture and write it to the
    /// provided directory.
    ///
    /// ```ignore
    /// EJECT_FUZZ_FIXTURES="./fuzz-fixtures" cargo test-sbf ...
    /// ```
    pub fn process_and_validate_instruction(
        &self,
        instruction: &Instruction,
        accounts: &[(Pubkey, AccountSharedData)],
        checks: &[Check],
    ) -> InstructionResult {
        let result = self.process_instruction(instruction, accounts);

        #[cfg(feature = "fuzz")]
        if let Ok(dir_path) = std::env::var("EJECT_FUZZ_FIXTURES") {
            fuzz::build_fixture_from_mollusk_test(self, instruction, accounts, &result, checks)
                .dump_to_blob_file(&dir_path);
        }

        result.run_checks(checks);
        result
    }
}
