//! Effects of a single instruction.

use {
    super::{error::FixtureError, proto},
    solana_sdk::{account::AccountSharedData, pubkey::Pubkey},
};

/// Represents the effects of a single instruction.
pub struct FixtureEffects {
    /// The result of the instruction.
    pub result: i32,
    /// The custom error of the instruction, if any.
    pub custom_error: u64,
    /// Resulting accounts with state, to be checked post-simulation.
    pub modified_accounts: Vec<(Pubkey, AccountSharedData)>,
}

impl TryFrom<proto::InstrEffects> for FixtureEffects {
    type Error = FixtureError;

    fn try_from(input: proto::InstrEffects) -> Result<Self, Self::Error> {
        let proto::InstrEffects {
            result,
            custom_err: custom_error,
            modified_accounts,
        } = input;

        let modified_accounts = modified_accounts
            .into_iter()
            .map(|acct_state| acct_state.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            result,
            custom_error,
            modified_accounts,
        })
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

        let result = 0;
        let custom_error = 0;
        let modified_accounts = vec![
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
            result,
            custom_err: custom_error,
            modified_accounts,
        };

        let effects = FixtureEffects::try_from(input).unwrap();
        assert_eq!(effects.result, result);
        assert_eq!(effects.custom_error, custom_error);
        assert_eq!(effects.modified_accounts.len(), 2);

        let (pubkey, account) = &effects.modified_accounts[0];
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

        let (pubkey, account) = &effects.modified_accounts[1];
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
