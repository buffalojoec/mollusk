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
    /// Perform checks on the instruction result, panicking if any checks fail.
    pub(crate) fn run_checks(&self, checks: &[InstructionCheck]) {
        for check in checks {
            match &check.check {
                Check::ComputeUnitsConsumed(units) => {
                    let check_units = *units;
                    let actual_units = self.compute_units_consumed;
                    assert_eq!(
                        check_units, actual_units,
                        "Checking compute units consumed: expected {}, got {}",
                        check_units, actual_units
                    );
                }
                Check::ExecutionTime(time) => {
                    let check_time = *time;
                    let actual_time = self.execution_time;
                    assert_eq!(
                        check_time, actual_time,
                        "Checking execution time: expected {}, got {}",
                        check_time, actual_time
                    );
                }
                Check::ProgramResult(result) => {
                    let check_result = result;
                    let actual_result = &self.program_result;
                    assert_eq!(
                        check_result, actual_result,
                        "Checking program result: expected {:?}, got {:?}",
                        check_result, actual_result
                    );
                }
                Check::ResultingAccount(account) => {
                    let pubkey = account.pubkey;
                    let resulting_account = self
                        .resulting_accounts
                        .iter()
                        .find(|(k, _)| k == &pubkey)
                        .map(|(_, a)| a)
                        .unwrap_or_else(|| {
                            panic!("Account not found in resulting accounts: {}", pubkey)
                        });
                    if let Some(check_data) = &account.check_data {
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
                            CheckState::Closed => {
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

enum Check {
    ComputeUnitsConsumed(u64),
    ExecutionTime(u64),
    ProgramResult(ProgramResult),
    ResultingAccount(CheckAccount),
}

pub struct InstructionCheck {
    check: Check,
}

impl InstructionCheck {
    fn new(check: Check) -> Self {
        Self { check }
    }

    /// Check the number of compute units consumed by the instruction.
    pub fn compute_units_consumed(units: u64) -> Self {
        Self::new(Check::ComputeUnitsConsumed(units))
    }

    /// Check the time taken to execute the instruction.
    pub fn execution_time(time: u64) -> Self {
        Self::new(Check::ExecutionTime(time))
    }

    /// Check the result code of the program's execution.
    pub fn program_result(result: ProgramResult) -> Self {
        Self::new(Check::ProgramResult(result))
    }

    /// Check a resulting account after executing the instruction.
    pub fn account(account: CheckAccount) -> Self {
        Self::new(Check::ResultingAccount(account))
    }
}

enum CheckState {
    Closed,
}

pub struct CheckAccount {
    pubkey: Pubkey,
    check_data: Option<Vec<u8>>,
    check_lamports: Option<u64>,
    check_owner: Option<Pubkey>,
    check_state: Option<CheckState>,
}

impl CheckAccount {
    /// Create a new check for a resulting account.
    pub fn new(pubkey: &Pubkey) -> Self {
        Self {
            pubkey: *pubkey,
            check_data: None,
            check_lamports: None,
            check_owner: None,
            check_state: None,
        }
    }

    /// Check that a resulting account was closed.
    pub fn closed(mut self) -> Self {
        self.check_state = Some(CheckState::Closed);
        self
    }

    /// Check a resulting account's data.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.check_data = Some(data);
        self
    }

    /// Check a resulting account's lamports.
    pub fn lamports(mut self, lamports: u64) -> Self {
        self.check_lamports = Some(lamports);
        self
    }

    /// Check a resulting account's owner.
    pub fn owner(mut self, owner: Pubkey) -> Self {
        self.check_owner = Some(owner);
        self
    }
}
