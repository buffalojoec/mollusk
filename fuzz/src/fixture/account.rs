//! An account with an address, in the form of a `(Pubkey, AccountSharedData)`
//! tuple from the Solana SDK.

use {
    super::{error::FixtureError, proto},
    solana_sdk::{
        account::{Account, AccountSharedData},
        pubkey::Pubkey,
    },
};

impl TryFrom<proto::AcctState> for (Pubkey, AccountSharedData) {
    type Error = FixtureError;

    fn try_from(input: proto::AcctState) -> Result<Self, Self::Error> {
        let proto::AcctState {
            address,
            owner,
            lamports,
            data,
            executable,
            rent_epoch,
        } = input;

        let pubkey = Pubkey::new_from_array(
            address
                .try_into()
                .map_err(|_| FixtureError::InvalidPubkeyBytes)?,
        );
        let owner = Pubkey::new_from_array(
            owner
                .try_into()
                .map_err(|_| FixtureError::InvalidPubkeyBytes)?,
        );

        Ok((
            pubkey,
            AccountSharedData::from(Account {
                lamports,
                data,
                owner,
                executable,
                rent_epoch,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_proto_acct_state() {
        let try_conversion = |address, owner| {
            let input = proto::AcctState {
                address,
                owner,
                lamports: 42,
                data: vec![1, 2, 3],
                executable: true,
                rent_epoch: 0,
            };
            TryInto::<(Pubkey, AccountSharedData)>::try_into(input)
        };

        let pubkey = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        // Success
        let (result_pubkey, result_account) =
            try_conversion(pubkey.to_bytes().to_vec(), owner.to_bytes().to_vec()).unwrap();
        assert_eq!(result_pubkey, pubkey);
        assert_eq!(
            result_account,
            AccountSharedData::from(Account {
                lamports: 42,
                data: vec![1, 2, 3],
                owner,
                executable: true,
                rent_epoch: 0,
            })
        );

        // Failures
        let too_many_bytes = vec![0; 33];
        let too_few_bytes = vec![0; 31];

        // Too many bytes for address
        assert_eq!(
            try_conversion(too_many_bytes.clone(), owner.to_bytes().to_vec()).unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );

        // Too few bytes for address
        assert_eq!(
            try_conversion(too_few_bytes.clone(), owner.to_bytes().to_vec()).unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );

        // Too many bytes for owner
        assert_eq!(
            try_conversion(pubkey.to_bytes().to_vec(), too_many_bytes).unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );

        // Too few bytes for owner
        assert_eq!(
            try_conversion(pubkey.to_bytes().to_vec(), too_few_bytes).unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );
    }
}
