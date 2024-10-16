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
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_program_runtime::{
        invoke_context::{EnvironmentConfig, InvokeContext},
        sysvar_cache::SysvarCache,
        timings::ExecuteTimings,
    },
    solana_sdk::{
        account::AccountSharedData,
        bpf_loader_upgradeable,
        feature_set::FeatureSet,
        fee::FeeStructure,
        hash::Hash,
        instruction::Instruction,
        pubkey::Pubkey,
        transaction_context::{InstructionAccount, TransactionContext},
    },
    std::{collections::HashMap, sync::Arc},
};

const DEFAULT_LOADER_KEY: Pubkey = bpf_loader_upgradeable::id();

const PROGRAM_ACCOUNTS_LEN: usize = 1;
const PROGRAM_INDICES: &[u16] = &[0];

/// The Mollusk API, providing a simple interface for testing Solana programs.
///
/// All fields can be manipulated through a handful of helper methods, but
/// users can also directly access and modify them if they desire more control.
pub struct Mollusk {
    pub compute_budget: ComputeBudget,
    pub feature_set: FeatureSet,
    pub fee_structure: FeeStructure,
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
        let (program_id, program_account) = program::keyed_account_for_system_program();
        Self {
            compute_budget: ComputeBudget::default(),
            feature_set: FeatureSet::all_enabled(),
            fee_structure: FeeStructure::default(),
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
    pub fn new(program_id: &Pubkey, program_name: &str) -> Self {
        let mut mollusk = Self {
            program_id: *program_id,
            program_account: program::create_program_account_loader_v3(program_id),
            ..Default::default()
        };
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

    /// Add a program to the test environment under a specific loader.
    ///
    /// If you intend to CPI to a program, this is likely what you want to use.
    pub fn add_program_with_loader(
        &mut self,
        program_id: &Pubkey,
        program_name: &str,
        loader_key: &Pubkey,
    ) {
        let elf = file::load_program_elf(program_name);
        self.add_program_with_elf_and_loader(program_id, &elf, loader_key);
    }

    /// Add a program to the test environment using a provided ELF.
    ///
    /// If you intend to CPI to a program, this is likely what you want to use.
    pub fn add_program_with_elf(&mut self, program_id: &Pubkey, elf: &[u8]) {
        self.add_program_with_elf_and_loader(program_id, elf, &DEFAULT_LOADER_KEY);
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

    /// Switch the target program to a different program.
    ///
    /// Note: The program must already be contained in the program cache.
    pub fn switch_target_program(&mut self, program_id: &Pubkey) {
        let loader_key: Pubkey = self
            .program_cache
            .cache()
            .read()
            .unwrap()
            .find(program_id)
            .expect("Program not found in cache")
            .account_owner
            .into();
        if loader_key != DEFAULT_LOADER_KEY {
            panic!("Loader not supported for target program: {:?}", loader_key);
        }
        self.program_id = *program_id;
        self.program_account = program::create_program_account_loader_v3(program_id);
    }

    /// Add a program to the cache and make it the target program.
    pub fn add_and_switch_target_program(&mut self, program_id: &Pubkey, program_name: &str) {
        self.add_program(program_id, program_name);
        self.switch_target_program(program_id);
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

        let instruction_accounts = {
            // For ensuring each account has the proper privilege level (dedupe).
            //  <pubkey, (is_signer, is_writable)>
            let mut privileges: HashMap<Pubkey, (bool, bool)> = HashMap::new();

            for meta in instruction.accounts.iter() {
                let entry = privileges.entry(meta.pubkey).or_default();
                entry.0 |= meta.is_signer;
                entry.1 |= meta.is_writable;
            }

            instruction
                .accounts
                .iter()
                .enumerate()
                .map(|(i, meta)| {
                    // Guaranteed by the last iteration.
                    let (is_signer, is_writable) = privileges.get(&meta.pubkey).unwrap();
                    InstructionAccount {
                        index_in_callee: i as u16,
                        index_in_caller: i as u16,
                        index_in_transaction: (i + PROGRAM_ACCOUNTS_LEN) as u16,
                        is_signer: *is_signer,
                        is_writable: *is_writable,
                    }
                })
                .collect::<Vec<_>>()
        };

        let transaction_accounts = [(self.program_id, self.program_account.clone())]
            .iter()
            .chain(accounts)
            .cloned()
            .collect::<Vec<_>>();

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
