//! Effects of a single instruction.

use {
    super::{error::FixtureError, proto},
    serde::{Deserialize, Serialize},
    solana_sdk::{account::AccountSharedData, pubkey::Pubkey},
};

/// Represents the effects of a single instruction.
#[derive(Debug, Deserialize, Serialize)]
pub struct FixtureEffects {
    /// Compute units consumed by the instruction.
    pub compute_units_consumed: u64,
    /// Execution time for instruction.
    pub execution_time: u64,
    // Program return code. Zero is success, errors are non-zero.
    pub program_result: u32,
    /// Resulting accounts with state, to be checked post-simulation.
    pub resulting_accounts: Vec<(Pubkey, AccountSharedData)>,
}

impl TryFrom<proto::InstrEffects> for FixtureEffects {
    type Error = FixtureError;

    fn try_from(input: proto::InstrEffects) -> Result<Self, Self::Error> {
        let proto::InstrEffects {
            compute_units_consumed,
            execution_time,
            program_result,
            resulting_accounts,
        } = input;

        let resulting_accounts = resulting_accounts
            .into_iter()
            .map(|acct_state| acct_state.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            compute_units_consumed,
            execution_time,
            program_result,
            resulting_accounts,
        })
    }
}

impl From<&FixtureEffects> for proto::InstrEffects {
    fn from(input: &FixtureEffects) -> Self {
        let FixtureEffects {
            compute_units_consumed,
            execution_time,
            program_result,
            resulting_accounts,
        } = input;

        let resulting_accounts = resulting_accounts
            .iter()
            .map(|a| a.into())
            .collect::<Vec<_>>();

        proto::InstrEffects {
            compute_units_consumed: *compute_units_consumed,
            execution_time: *execution_time,
            program_result: *program_result,
            resulting_accounts,
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, solana_sdk::account::Account};

    #[test]
    fn test_try_from_proto_instr_effects() {
        let address1 = Pubkey::new_unique();
        let owner1 = Pubkey::new_unique();
        let address2 = Pubkey::new_unique();
        let owner2 = Pubkey::new_unique();

        let compute_units_consumed = 50_000;
        let execution_time = 100;
        let program_result = 0;
        let resulting_accounts = vec![
            proto::AcctState {
                address: address1.to_bytes().to_vec(),
                owner: owner1.to_bytes().to_vec(),
                lamports: 42,
                data: vec![1, 2, 3],
                executable: false,
                rent_epoch: 0,
            },
            proto::AcctState {
                address: address2.to_bytes().to_vec(),
                owner: owner2.to_bytes().to_vec(),
                lamports: 42,
                data: vec![5, 4, 3],
                executable: true,
                rent_epoch: 0,
            },
        ];

        let input = proto::InstrEffects {
            compute_units_consumed,
            execution_time,
            program_result,
            resulting_accounts,
        };

        let effects = FixtureEffects::try_from(input).unwrap();
        assert_eq!(effects.compute_units_consumed, compute_units_consumed);
        assert_eq!(effects.execution_time, execution_time);
        assert_eq!(effects.program_result, program_result);
        assert_eq!(effects.resulting_accounts.len(), 2);

        let (pubkey, account) = &effects.resulting_accounts[0];
        assert_eq!(*pubkey, address1);
        assert_eq!(
            *account,
            AccountSharedData::from(Account {
                lamports: 42,
                data: vec![1, 2, 3],
                owner: owner1,
                executable: false,
                rent_epoch: 0,
            })
        );

        let (pubkey, account) = &effects.resulting_accounts[1];
        assert_eq!(*pubkey, address2);
        assert_eq!(
            *account,
            AccountSharedData::from(Account {
                lamports: 42,
                data: vec![5, 4, 3],
                owner: owner2,
                executable: true,
                rent_epoch: 0,
            })
        );
    }
}
