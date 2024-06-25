//! Instruction context fixture for invoking programs in a simulated program
//! runtime environment.

use {
    super::{error::FixtureError, proto, sysvars::FixtureSysvarContext},
    solana_sdk::{
        account::AccountSharedData, feature_set::FeatureSet, instruction::AccountMeta,
        pubkey::Pubkey,
    },
};

/// Instruction context fixture.
#[derive(Debug)]
pub struct FixtureContext {
    /// The program ID of the program being invoked.
    pub program_id: Pubkey,
    /// The loader ID to use for the program.
    pub loader_id: Pubkey,
    /// The feature set to use for the simulation.
    pub feature_set: FeatureSet,
    /// The sysvar context to use for the simulation.
    pub sysvar_context: FixtureSysvarContext,
    /// Input accounts with state.
    pub accounts: Vec<(Pubkey, AccountSharedData)>,
    /// Accounts to pass to the instruction.
    pub instruction_accounts: Vec<AccountMeta>,
    /// The instruction data.
    pub instruction_data: Vec<u8>,
}

impl TryFrom<proto::InstrContext> for FixtureContext {
    type Error = FixtureError;

    fn try_from(input: proto::InstrContext) -> Result<Self, Self::Error> {
        let proto::InstrContext {
            program_id,
            loader_id,
            feature_set,
            sysvars,
            accounts,
            instr_accounts,
            data: instruction_data,
        } = input;

        let program_id = Pubkey::new_from_array(
            program_id
                .try_into()
                .map_err(|_| FixtureError::InvalidPubkeyBytes)?,
        );
        let loader_id = Pubkey::new_from_array(
            loader_id
                .try_into()
                .map_err(|_| FixtureError::InvalidPubkeyBytes)?,
        );

        let feature_set = feature_set.map(|fs| fs.into()).unwrap_or_default();

        let sysvar_context = sysvars
            .map(|sysvars| sysvars.try_into())
            .transpose()?
            .unwrap_or_default();

        let accounts = accounts
            .into_iter()
            .map(|acct_state| acct_state.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let instruction_accounts = instr_accounts
            .into_iter()
            .map(
                |proto::InstrAcct {
                     index,
                     is_signer,
                     is_writable,
                 }| {
                    accounts
                        .get(index as usize)
                        .ok_or(FixtureError::AccountMissing)
                        .map(|(pubkey, _)| AccountMeta {
                            pubkey: *pubkey,
                            is_signer,
                            is_writable,
                        })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            program_id,
            loader_id,
            feature_set,
            sysvar_context,
            accounts,
            instruction_accounts,
            instruction_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use {super::*, solana_sdk::account::Account};

    #[test]
    fn test_try_from_proto_instr_context() {
        let address1 = Pubkey::new_unique();
        let owner1 = Pubkey::new_unique();
        let address2 = Pubkey::new_unique();
        let owner2 = Pubkey::new_unique();

        let program_id = Pubkey::new_unique();
        let loader_id = Pubkey::new_unique();

        let accounts = vec![
            (
                address1,
                AccountSharedData::from(Account {
                    lamports: 42,
                    data: vec![1, 2, 3],
                    owner: owner1,
                    executable: false,
                    rent_epoch: 0,
                }),
            ),
            (
                address2,
                AccountSharedData::from(Account {
                    lamports: 42,
                    data: vec![5, 4, 3],
                    owner: owner2,
                    executable: true,
                    rent_epoch: 0,
                }),
            ),
        ];
        let instruction_accounts = vec![
            AccountMeta {
                pubkey: address1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: address2,
                is_signer: false,
                is_writable: false,
            },
        ];
        let instruction_data = vec![1, 2, 3];

        let input = proto::InstrContext {
            program_id: program_id.to_bytes().to_vec(),
            loader_id: loader_id.to_bytes().to_vec(),
            accounts: vec![
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
            ],
            instr_accounts: vec![
                proto::InstrAcct {
                    index: 0,
                    is_signer: false,
                    is_writable: true,
                },
                proto::InstrAcct {
                    index: 1,
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: instruction_data.clone(),
            // Feature set and sysvars have their own tests
            ..proto::InstrContext::default()
        };

        // Success
        let context = FixtureContext::try_from(input.clone()).unwrap();
        assert_eq!(context.program_id, program_id);
        assert_eq!(context.loader_id, loader_id);
        assert_eq!(context.accounts, accounts);
        assert_eq!(context.instruction_accounts, instruction_accounts);

        // Failures
        let too_many_bytes = vec![0; 33];
        let too_few_bytes = vec![0; 31];

        // Too many bytes for program_id
        assert_eq!(
            FixtureContext::try_from(proto::InstrContext {
                program_id: too_many_bytes.clone(),
                ..input.clone()
            })
            .unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );

        // Too few bytes for program_id
        assert_eq!(
            FixtureContext::try_from(proto::InstrContext {
                program_id: too_few_bytes.clone(),
                ..input.clone()
            })
            .unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );

        // Too many bytes for loader_id
        assert_eq!(
            FixtureContext::try_from(proto::InstrContext {
                loader_id: too_many_bytes.clone(),
                ..input.clone()
            })
            .unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );

        // Too few bytes for loader_id
        assert_eq!(
            FixtureContext::try_from(proto::InstrContext {
                loader_id: too_few_bytes.clone(),
                ..input.clone()
            })
            .unwrap_err(),
            FixtureError::InvalidPubkeyBytes
        );

        // Invalid account index
        assert_eq!(
            FixtureContext::try_from(proto::InstrContext {
                instr_accounts: vec![proto::InstrAcct {
                    index: 2,
                    is_signer: false,
                    is_writable: true,
                }],
                ..input.clone()
            })
            .unwrap_err(),
            FixtureError::AccountMissing
        );
    }
}
