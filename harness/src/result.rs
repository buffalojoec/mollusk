//! Results of Mollusk program execution.

use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    instruction::InstructionError,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// The result code of the program's execution.
#[derive(Debug, PartialEq, Eq)]
pub enum ProgramResult {
    /// The program executed successfully.
    Success,
    /// The program returned an error.
    Failure(ProgramError),
    /// Mollusk encountered an error while executing the program.
    UnknownError(InstructionError),
}

impl From<Result<(), InstructionError>> for ProgramResult {
    fn from(result: Result<(), InstructionError>) -> Self {
        match result {
            Ok(()) => ProgramResult::Success,
            Err(err) => {
                if let Ok(program_error) = ProgramError::try_from(err.clone()) {
                    ProgramResult::Failure(program_error)
                } else {
                    ProgramResult::UnknownError(err)
                }
            }
        }
    }
}

/// The overall result of the instruction.
#[derive(Debug, PartialEq, Eq)]
pub struct InstructionResult {
    /// The number of compute units consumed by the instruction.
    pub compute_units_consumed: u64,
    /// The time taken to execute the instruction.
    pub execution_time: u64,
    /// The result code of the program's execution.
    pub program_result: ProgramResult,
    /// The resulting accounts after executing the instruction.
    ///
    /// This includes all accounts provided to the processor, in the order
    /// they were provided. Any accounts that were modified will maintain
    /// their original position in this list, but with updated state.
    pub resulting_accounts: Vec<(Pubkey, AccountSharedData)>,
}

impl InstructionResult {
    /// Get an account from the resulting accounts by its pubkey.
    pub fn get_account(&self, pubkey: &Pubkey) -> Option<&AccountSharedData> {
        self.resulting_accounts
            .iter()
            .find(|(k, _)| k == pubkey)
            .map(|(_, a)| a)
    }

    /// Perform checks on the instruction result, panicking if any checks fail.
    pub(crate) fn run_checks(&self, checks: &[Check]) {
        for check in checks {
            match &check.check {
                CheckType::ComputeUnitsConsumed(units) => {
                    let check_units = *units;
                    let actual_units = self.compute_units_consumed;
                    assert_eq!(
                        check_units, actual_units,
                        "Checking compute units consumed: expected {}, got {}",
                        check_units, actual_units
                    );
                }
                CheckType::ExecutionTime(time) => {
                    let check_time = *time;
                    let actual_time = self.execution_time;
                    assert_eq!(
                        check_time, actual_time,
                        "Checking execution time: expected {}, got {}",
                        check_time, actual_time
                    );
                }
                CheckType::ProgramResult(result) => {
                    let check_result = result;
                    let actual_result = &self.program_result;
                    assert_eq!(
                        check_result, actual_result,
                        "Checking program result: expected {:?}, got {:?}",
                        check_result, actual_result
                    );
                }
                CheckType::ResultingAccount(account) => {
                    let pubkey = account.pubkey;
                    let resulting_account = self
                        .resulting_accounts
                        .iter()
                        .find(|(k, _)| k == &pubkey)
                        .map(|(_, a)| a)
                        .unwrap_or_else(|| {
                            panic!("Account not found in resulting accounts: {}", pubkey)
                        });
                    if let Some(check_data) = account.check_data {
                        let actual_data = resulting_account.data();
                        assert_eq!(
                            check_data, actual_data,
                            "Checking account data: expected {:?}, got {:?}",
                            check_data, actual_data
                        );
                    }
                    if let Some(check_lamports) = account.check_lamports {
                        let actual_lamports = resulting_account.lamports();
                        assert_eq!(
                            check_lamports, actual_lamports,
                            "Checking account lamports: expected {}, got {}",
                            check_lamports, actual_lamports
                        );
                    }
                    if let Some(check_owner) = &account.check_owner {
                        let actual_owner = resulting_account.owner();
                        assert_eq!(
                            check_owner, actual_owner,
                            "Checking account owner: expected {}, got {}",
                            check_owner, actual_owner
                        );
                    }
                    if let Some(check_state) = &account.check_state {
                        match check_state {
                            AccountStateCheck::Closed => {
                                assert_eq!(
                                    &AccountSharedData::default(),
                                    resulting_account,
                                    "Checking account closed: expected true, got false"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

enum CheckType<'a> {
    /// Check the number of compute units consumed by the instruction.
    ComputeUnitsConsumed(u64),
    /// Check the time taken to execute the instruction.
    ExecutionTime(u64),
    /// Check the result code of the program's execution.
    ProgramResult(ProgramResult),
    /// Check a resulting account after executing the instruction.
    ResultingAccount(AccountCheck<'a>),
}

pub struct Check<'a> {
    check: CheckType<'a>,
}

impl<'a> Check<'a> {
    fn new(check: CheckType<'a>) -> Self {
        Self { check }
    }

    /// Check the number of compute units consumed by the instruction.
    pub fn compute_units(units: u64) -> Self {
        Check::new(CheckType::ComputeUnitsConsumed(units))
    }

    /// Check the time taken to execute the instruction.
    pub fn time(time: u64) -> Self {
        Check::new(CheckType::ExecutionTime(time))
    }

    /// Assert that the program executed successfully.
    pub fn success() -> Self {
        Check::new(CheckType::ProgramResult(ProgramResult::Success))
    }

    /// Assert that the program returned an error.
    pub fn err(error: ProgramError) -> Self {
        Check::new(CheckType::ProgramResult(ProgramResult::Failure(error)))
    }

    /// Assert that the instruction returned an error.
    pub fn instruction_err(error: InstructionError) -> Self {
        Check::new(CheckType::ProgramResult(ProgramResult::UnknownError(error)))
    }

    /// Check a resulting account after executing the instruction.
    pub fn account(pubkey: &Pubkey) -> AccountCheckBuilder {
        AccountCheckBuilder::new(pubkey)
    }
}

enum AccountStateCheck {
    Closed,
}

struct AccountCheck<'a> {
    pubkey: Pubkey,
    check_data: Option<&'a [u8]>,
    check_lamports: Option<u64>,
    check_owner: Option<Pubkey>,
    check_state: Option<AccountStateCheck>,
}

impl AccountCheck<'_> {
    fn new(pubkey: &Pubkey) -> Self {
        Self {
            pubkey: *pubkey,
            check_data: None,
            check_lamports: None,
            check_owner: None,
            check_state: None,
        }
    }
}

pub struct AccountCheckBuilder<'a> {
    check: AccountCheck<'a>,
}

impl<'a> AccountCheckBuilder<'a> {
    fn new(pubkey: &Pubkey) -> Self {
        Self {
            check: AccountCheck::new(pubkey),
        }
    }

    pub fn closed(mut self) -> Self {
        self.check.check_state = Some(AccountStateCheck::Closed);
        self
    }

    pub fn data(mut self, data: &'a [u8]) -> Self {
        self.check.check_data = Some(data);
        self
    }

    pub fn lamports(mut self, lamports: u64) -> Self {
        self.check.check_lamports = Some(lamports);
        self
    }

    pub fn owner(mut self, owner: Pubkey) -> Self {
        self.check.check_owner = Some(owner);
        self
    }

    pub fn build(self) -> Check<'a> {
        Check::new(CheckType::ResultingAccount(self.check))
    }
}
