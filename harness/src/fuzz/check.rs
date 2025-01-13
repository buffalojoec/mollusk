//! Checks to run against a fixture when validating.

use {
    crate::result::{Check, InstructionResult},
    solana_sdk::{
        account::{Account, ReadableAccount},
        pubkey::Pubkey,
    },
};

/// Checks to run against a fixture when validating.
///
/// Similar to Mollusk's `result::Check`, this allows a developer to dictate
/// the type of checks to run on the fixture's effects.
///
/// Keep in mind that validation of fixtures works slightly differently than
/// typical Mollusk unit tests. In a Mollusk test, you can provide the value to
/// compare a portion of the result against (ie. compute units). However, when
/// comparing the result of a Mollusk invocation against a fixture, the value
/// from the fixture itself is used.
///
/// For that reason, these are unary variants, and do not offer the developer a
/// way to provide values to check against.
pub enum FixtureCheck {
    /// Validate compute units consumed.
    ComputeUnits,
    /// Validate the program result.
    ProgramResult,
    /// Validate the return data.
    ReturnData,
    /// Validate all resulting accounts.
    AllResultingAccounts {
        /// Whether or not to validate each account's data.
        data: bool,
        /// Whether or not to validate each account's lamports.
        lamports: bool,
        /// Whether or not to validate each account's owner.
        owner: bool,
        /// Whether or not to validate each account's space.
        space: bool,
    },
    /// Validate the resulting accounts at certain addresses.
    OnlyResultingAccounts {
        /// The addresses on which to apply the validation.
        addresses: Vec<Pubkey>,
        /// Whether or not to validate each account's data.
        data: bool,
        /// Whether or not to validate each account's lamports.
        lamports: bool,
        /// Whether or not to validate each account's owner.
        owner: bool,
        /// Whether or not to validate each account's space.
        space: bool,
    },
    /// Validate all of the resulting accounts _except_ the provided addresses.
    AllResultingAccountsExcept {
        /// The addresses on which to _not_ apply the validation.
        ignore_addresses: Vec<Pubkey>,
        /// On non-ignored accounts, whether or not to validate each account's
        /// data.
        data: bool,
        /// On non-ignored accounts, whether or not to validate each account's
        /// lamports.
        lamports: bool,
        /// On non-ignored accounts, whether or not to validate each account's
        /// owner.
        owner: bool,
        /// On non-ignored accounts, whether or not to validate each account's
        /// space.
        space: bool,
    },
}

fn add_account_checks<'a>(
    checks: &mut Vec<Check<'a>>,
    accounts: impl Iterator<Item = &'a (Pubkey, Account)>,
    data: bool,
    lamports: bool,
    owner: bool,
    space: bool,
) {
    for (pubkey, account) in accounts {
        let mut builder = Check::account(pubkey);
        if data {
            builder = builder.data(account.data());
        }
        if lamports {
            builder = builder.lamports(account.lamports());
        }
        if owner {
            builder = builder.owner(account.owner());
        }
        if space {
            builder = builder.space(account.data().len());
        }
        checks.push(builder.build());
    }
}

pub(crate) fn evaluate_results_with_fixture_checks(
    expected: &InstructionResult,
    result: &InstructionResult,
    fixture_checks: &[FixtureCheck],
) {
    let mut checks = vec![];

    for fixture_check in fixture_checks {
        match fixture_check {
            FixtureCheck::ComputeUnits => {
                checks.push(Check::compute_units(expected.compute_units_consumed))
            }
            FixtureCheck::ProgramResult => {
                checks.push(Check::program_result(expected.program_result.clone()))
            }
            FixtureCheck::ReturnData => checks.push(Check::return_data(&expected.return_data)),
            FixtureCheck::AllResultingAccounts {
                data,
                lamports,
                owner,
                space,
            } => {
                add_account_checks(
                    &mut checks,
                    expected.resulting_accounts.iter(),
                    *data,
                    *lamports,
                    *owner,
                    *space,
                );
            }
            FixtureCheck::OnlyResultingAccounts {
                addresses,
                data,
                lamports,
                owner,
                space,
            } => {
                add_account_checks(
                    &mut checks,
                    expected
                        .resulting_accounts
                        .iter()
                        .filter(|(pubkey, _)| addresses.contains(pubkey)),
                    *data,
                    *lamports,
                    *owner,
                    *space,
                );
            }
            FixtureCheck::AllResultingAccountsExcept {
                ignore_addresses,
                data,
                lamports,
                owner,
                space,
            } => {
                add_account_checks(
                    &mut checks,
                    expected
                        .resulting_accounts
                        .iter()
                        .filter(|(pubkey, _)| !ignore_addresses.contains(pubkey)),
                    *data,
                    *lamports,
                    *owner,
                    *space,
                );
            }
        }
    }

    result.run_checks(&checks);
}
