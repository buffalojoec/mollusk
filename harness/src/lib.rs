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
//! * `process_instruction_chain`: Process a chain of instructions and return
//!   the result.
//! * `process_and_validate_instruction_chain`: Process a chain of instructions
//!   and perform a series of checks on each result, panicking if any checks
//!   fail.

mod accounts;
pub mod file;
#[cfg(any(feature = "fuzz", feature = "fuzz-fd"))]
pub mod fuzz;
pub mod program;
pub mod result;
pub mod sysvar;

use {
    crate::{
        program::ProgramCache,
        result::{Check, InstructionResult},
        sysvar::Sysvars,
    },
    accounts::CompiledAccounts,
    mollusk_svm_error::error::{MolluskError, MolluskPanic},
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_program_runtime::{
        invoke_context::{EnvironmentConfig, InvokeContext},
        timings::ExecuteTimings,
    },
    solana_sdk::{
        account::Account, bpf_loader_upgradeable, feature_set::FeatureSet, fee::FeeStructure,
        hash::Hash, instruction::Instruction, pubkey::Pubkey,
        transaction_context::TransactionContext,
    },
    std::sync::Arc,
};

pub(crate) const DEFAULT_LOADER_KEY: Pubkey = bpf_loader_upgradeable::id();

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
    #[cfg(feature = "fuzz-fd")]
    pub slot: u64,
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
            #[cfg(feature = "fuzz-fd")]
            slot: 0,
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

    /// Process an instruction using the minified Solana Virtual Machine (SVM)
    /// environment. Simply returns the result.
    pub fn process_instruction(
        &self,
        instruction: &Instruction,
        accounts: &[(Pubkey, Account)],
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
        } = crate::accounts::compile_accounts(instruction, accounts, loader_key);

        let mut transaction_context = TransactionContext::new(
            transaction_accounts,
            self.sysvars.rent.clone(),
            self.compute_budget.max_instruction_stack_depth,
            self.compute_budget.max_instruction_trace_length,
        );

        let invoke_result = {
            let mut program_cache = self.program_cache.cache().write().unwrap();
            let sysvar_cache = self.sysvars.setup_sysvar_cache(accounts);
            InvokeContext::new(
                &mut transaction_context,
                &mut program_cache,
                EnvironmentConfig::new(
                    Hash::default(),
                    None,
                    None,
                    Arc::new(self.feature_set.clone()),
                    self.fee_structure.lamports_per_signature,
                    &sysvar_cache,
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

        let return_data = transaction_context.get_return_data().1.to_vec();

        let resulting_accounts: Vec<(Pubkey, Account)> = if invoke_result.is_ok() {
            accounts
                .iter()
                .map(|(pubkey, account)| {
                    transaction_context
                        .find_index_of_account(pubkey)
                        .map(|index| {
                            let resulting_account = transaction_context
                                .get_account_at_index(index)
                                .unwrap()
                                .borrow()
                                .clone()
                                .into();
                            (*pubkey, resulting_account)
                        })
                        .unwrap_or((*pubkey, account.clone()))
                })
                .collect()
        } else {
            accounts.to_vec()
        };

        InstructionResult {
            compute_units_consumed,
            execution_time: timings.details.execute_us,
            program_result: invoke_result.clone().into(),
            raw_result: invoke_result,
            return_data,
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
        accounts: &[(Pubkey, Account)],
    ) -> InstructionResult {
        let mut result = InstructionResult {
            resulting_accounts: accounts.to_vec(),
            ..Default::default()
        };

        for instruction in instructions {
            let this_result = self.process_instruction(instruction, &result.resulting_accounts);

            result.absorb(this_result);

            if result.program_result.is_err() {
                break;
            }
        }

        result
    }

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
    ///
    /// You can also provide `EJECT_FUZZ_FIXTURES_JSON` to write the fixture in
    /// JSON format.
    ///
    /// The `fuzz-fd` feature works the same way, but the variables require
    /// the `_FD` suffix, in case both features are active together
    /// (ie. `EJECT_FUZZ_FIXTURES_FD`). This will generate Firedancer fuzzing
    /// fixtures, which are structured a bit differently than Mollusk's own
    /// protobuf layouts.
    pub fn process_and_validate_instruction(
        &self,
        instruction: &Instruction,
        accounts: &[(Pubkey, Account)],
        checks: &[Check],
    ) -> InstructionResult {
        let result = self.process_instruction(instruction, accounts);

        #[cfg(any(feature = "fuzz", feature = "fuzz-fd"))]
        fuzz::generate_fixtures_from_mollusk_test(self, instruction, accounts, &result);

        result.run_checks(checks);
        result
    }

    /// Process a chain of instructions using the minified Solana Virtual
    /// Machine (SVM) environment, then perform checks on the result.
    /// Panics if any checks fail.
    ///
    /// For `fuzz` feature only:
    ///
    /// Similar to `process_and_validate_instruction`, if the
    /// `EJECT_FUZZ_FIXTURES` environment variable is set, this function will
    /// convert the provided test to a set of fuzz fixtures - each of which
    /// corresponds to a single instruction in the chain - and write them to
    /// the provided directory.
    ///
    /// ```ignore
    /// EJECT_FUZZ_FIXTURES="./fuzz-fixtures" cargo test-sbf ...
    /// ```
    ///
    /// You can also provide `EJECT_FUZZ_FIXTURES_JSON` to write the fixture in
    /// JSON format.
    ///
    /// The `fuzz-fd` feature works the same way, but the variables require
    /// the `_FD` suffix, in case both features are active together
    /// (ie. `EJECT_FUZZ_FIXTURES_FD`). This will generate Firedancer fuzzing
    /// fixtures, which are structured a bit differently than Mollusk's own
    /// protobuf layouts.
    pub fn process_and_validate_instruction_chain(
        &self,
        instructions: &[(&Instruction, &[Check])],
        accounts: &[(Pubkey, Account)],
    ) -> InstructionResult {
        let mut result = InstructionResult {
            resulting_accounts: accounts.to_vec(),
            ..Default::default()
        };

        for (instruction, checks) in instructions.iter() {
            let this_result = self.process_and_validate_instruction(
                instruction,
                &result.resulting_accounts,
                checks,
            );

            result.absorb(this_result);

            if result.program_result.is_err() {
                break;
            }
        }

        result
    }

    #[cfg(feature = "fuzz")]
    /// Process a fuzz fixture using the minified Solana Virtual Machine (SVM)
    /// environment.
    ///
    /// Fixtures provide an API to `decode` a raw blob, as well as read
    /// fixtures from files. Those fixtures can then be provided to this
    /// function to process them and get a Mollusk result.
    ///
    /// Note: This is a mutable method on `Mollusk`, since loading a fixture
    /// into the test environment will alter `Mollusk` values, such as compute
    /// budget and sysvars. However, the program cache remains unchanged.
    ///
    /// Therefore, developers can provision a `Mollusk` instance, set up their
    /// desired program cache, and then run a series of fixtures against that
    /// `Mollusk` instance (and cache).
    pub fn process_fixture(
        &mut self,
        fixture: &mollusk_svm_fuzz_fixture::Fixture,
    ) -> InstructionResult {
        let fuzz::mollusk::ParsedFixtureContext {
            accounts,
            compute_budget,
            feature_set,
            instruction,
            sysvars,
        } = fuzz::mollusk::parse_fixture_context(&fixture.input);
        self.compute_budget = compute_budget;
        self.feature_set = feature_set;
        self.sysvars = sysvars;
        self.process_instruction(&instruction, &accounts)
    }

    #[cfg(feature = "fuzz")]
    /// Process a fuzz fixture using the minified Solana Virtual Machine (SVM)
    /// environment and compare the result against the fixture's effects.
    ///
    /// Fixtures provide an API to `decode` a raw blob, as well as read
    /// fixtures from files. Those fixtures can then be provided to this
    /// function to process them and get a Mollusk result.
    ///
    ///
    /// Note: This is a mutable method on `Mollusk`, since loading a fixture
    /// into the test environment will alter `Mollusk` values, such as compute
    /// budget and sysvars. However, the program cache remains unchanged.
    ///
    /// Therefore, developers can provision a `Mollusk` instance, set up their
    /// desired program cache, and then run a series of fixtures against that
    /// `Mollusk` instance (and cache).
    ///
    /// Note: To compare the result against the entire fixture effects, pass
    /// `&[FixtureCheck::All]` for `checks`.
    pub fn process_and_validate_fixture(
        &mut self,
        fixture: &mollusk_svm_fuzz_fixture::Fixture,
    ) -> InstructionResult {
        let result = self.process_fixture(fixture);
        InstructionResult::from(&fixture.output).compare(&result);
        result
    }

    #[cfg(feature = "fuzz")]
    /// a specific set of checks.
    ///
    /// This is useful for when you may not want to compare the entire effects,
    /// such as omitting comparisons of compute units consumed.
    /// Process a fuzz fixture using the minified Solana Virtual Machine (SVM)
    /// environment and compare the result against the fixture's effects using
    /// a specific set of checks.
    ///
    /// This is useful for when you may not want to compare the entire effects,
    /// such as omitting comparisons of compute units consumed.
    ///
    /// Fixtures provide an API to `decode` a raw blob, as well as read
    /// fixtures from files. Those fixtures can then be provided to this
    /// function to process them and get a Mollusk result.
    ///
    ///
    /// Note: This is a mutable method on `Mollusk`, since loading a fixture
    /// into the test environment will alter `Mollusk` values, such as compute
    /// budget and sysvars. However, the program cache remains unchanged.
    ///
    /// Therefore, developers can provision a `Mollusk` instance, set up their
    /// desired program cache, and then run a series of fixtures against that
    /// `Mollusk` instance (and cache).
    ///
    /// Note: To compare the result against the entire fixture effects, pass
    /// `&[FixtureCheck::All]` for `checks`.
    pub fn process_and_partially_validate_fixture(
        &mut self,
        fixture: &mollusk_svm_fuzz_fixture::Fixture,
        checks: &[fuzz::check::FixtureCheck],
    ) -> InstructionResult {
        let result = self.process_fixture(fixture);
        let expected = InstructionResult::from(&fixture.output);
        fuzz::check::evaluate_results_with_fixture_checks(&expected, &result, checks);
        result
    }

    #[cfg(feature = "fuzz-fd")]
    /// Process a Firedancer fuzz fixture using the minified Solana Virtual
    /// Machine (SVM) environment.
    ///
    /// Fixtures provide an API to `decode` a raw blob, as well as read
    /// fixtures from files. Those fixtures can then be provided to this
    /// function to process them and get a Mollusk result.
    ///
    /// Note: This is a mutable method on `Mollusk`, since loading a fixture
    /// into the test environment will alter `Mollusk` values, such as compute
    /// budget and sysvars. However, the program cache remains unchanged.
    ///
    /// Therefore, developers can provision a `Mollusk` instance, set up their
    /// desired program cache, and then run a series of fixtures against that
    /// `Mollusk` instance (and cache).
    pub fn process_firedancer_fixture(
        &mut self,
        fixture: &mollusk_svm_fuzz_fixture_firedancer::Fixture,
    ) -> InstructionResult {
        let fuzz::firedancer::ParsedFixtureContext {
            accounts,
            compute_budget,
            feature_set,
            instruction,
            slot,
        } = fuzz::firedancer::parse_fixture_context(&fixture.input);
        self.compute_budget = compute_budget;
        self.feature_set = feature_set;
        self.slot = slot;
        self.process_instruction(&instruction, &accounts)
    }

    #[cfg(feature = "fuzz-fd")]
    /// Process a Firedancer fuzz fixture using the minified Solana Virtual
    /// Machine (SVM) environment and compare the result against the
    /// fixture's effects.
    ///
    /// Fixtures provide an API to `decode` a raw blob, as well as read
    /// fixtures from files. Those fixtures can then be provided to this
    /// function to process them and get a Mollusk result.
    ///
    ///
    /// Note: This is a mutable method on `Mollusk`, since loading a fixture
    /// into the test environment will alter `Mollusk` values, such as compute
    /// budget and sysvars. However, the program cache remains unchanged.
    ///
    /// Therefore, developers can provision a `Mollusk` instance, set up their
    /// desired program cache, and then run a series of fixtures against that
    /// `Mollusk` instance (and cache).
    ///
    /// Note: To compare the result against the entire fixture effects, pass
    /// `&[FixtureCheck::All]` for `checks`.
    pub fn process_and_validate_firedancer_fixture(
        &mut self,
        fixture: &mollusk_svm_fuzz_fixture_firedancer::Fixture,
    ) -> InstructionResult {
        let fuzz::firedancer::ParsedFixtureContext {
            accounts,
            compute_budget,
            feature_set,
            instruction,
            slot,
        } = fuzz::firedancer::parse_fixture_context(&fixture.input);
        self.compute_budget = compute_budget;
        self.feature_set = feature_set;
        self.slot = slot;

        let result = self.process_instruction(&instruction, &accounts);
        let expected_result = fuzz::firedancer::parse_fixture_effects(
            &accounts,
            self.compute_budget.compute_unit_limit,
            &fixture.output,
        );

        expected_result.compare(&result);
        result
    }

    #[cfg(feature = "fuzz-fd")]
    /// Process a Firedancer fuzz fixture using the minified Solana Virtual
    /// Machine (SVM) environment and compare the result against the
    /// fixture's effects using a specific set of checks.
    ///
    /// This is useful for when you may not want to compare the entire effects,
    /// such as omitting comparisons of compute units consumed.
    ///
    /// Fixtures provide an API to `decode` a raw blob, as well as read
    /// fixtures from files. Those fixtures can then be provided to this
    /// function to process them and get a Mollusk result.
    ///
    ///
    /// Note: This is a mutable method on `Mollusk`, since loading a fixture
    /// into the test environment will alter `Mollusk` values, such as compute
    /// budget and sysvars. However, the program cache remains unchanged.
    ///
    /// Therefore, developers can provision a `Mollusk` instance, set up their
    /// desired program cache, and then run a series of fixtures against that
    /// `Mollusk` instance (and cache).
    ///
    /// Note: To compare the result against the entire fixture effects, pass
    /// `&[FixtureCheck::All]` for `checks`.
    pub fn process_and_partially_validate_firedancer_fixture(
        &mut self,
        fixture: &mollusk_svm_fuzz_fixture_firedancer::Fixture,
        checks: &[fuzz::check::FixtureCheck],
    ) -> InstructionResult {
        let fuzz::firedancer::ParsedFixtureContext {
            accounts,
            compute_budget,
            feature_set,
            instruction,
            slot,
        } = fuzz::firedancer::parse_fixture_context(&fixture.input);
        self.compute_budget = compute_budget;
        self.feature_set = feature_set;
        self.slot = slot;

        let result = self.process_instruction(&instruction, &accounts);
        let expected = fuzz::firedancer::parse_fixture_effects(
            &accounts,
            self.compute_budget.compute_unit_limit,
            &fixture.output,
        );

        fuzz::check::evaluate_results_with_fixture_checks(&expected, &result, checks);
        result
    }
}
