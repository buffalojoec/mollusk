//! Instruction context fixture for invoking programs in a simulated program
//! runtime environment.

use {
    super::{
        compute_budget::FixtureComputeBudget, error::FixtureError, feature_set::FixtureFeatureSet,
        proto, sysvars::FixtureSysvarContext,
    },
    serde::{Deserialize, Serialize},
    solana_sdk::{
        account::{AccountSharedData, ReadableAccount},
        instruction::AccountMeta,
        pubkey::Pubkey,
    },
};

/// Instruction context fixture.
#[derive(Debug, Deserialize, Serialize)]
pub struct FixtureContext {
    /// The compute budget to use for the simulation.
    pub compute_budget: FixtureComputeBudget,
    /// The feature set to use for the simulation.
    pub feature_set: FixtureFeatureSet,
    /// The sysvar context to use for the simulation.
    pub sysvar_context: FixtureSysvarContext,
    /// The program ID of the program being invoked.
    pub program_id: Pubkey,
    /// Accounts to pass to the instruction.
    pub instruction_accounts: Vec<AccountMeta>,
    /// The instruction data.
    pub instruction_data: Vec<u8>,
    /// Input accounts with state.
    pub accounts: Vec<(Pubkey, AccountSharedData)>,
}

impl TryFrom<proto::InstrContext> for FixtureContext {
    type Error = FixtureError;

    fn try_from(input: proto::InstrContext) -> Result<Self, Self::Error> {
        let proto::InstrContext {
            compute_budget,
            feature_set,
            sysvars,
            program_id,
            instr_accounts,
            data: instruction_data,
            accounts,
        } = input;

        let compute_budget = compute_budget.map(|cb| cb.into()).unwrap_or_default();

        let feature_set = feature_set.map(|fs| fs.into()).unwrap_or_default();

        let sysvar_context = sysvars
            .map(|sysvars| sysvars.try_into())
            .transpose()?
            .unwrap_or_default();

        let program_id = Pubkey::new_from_array(
            program_id
                .try_into()
                .map_err(|_| FixtureError::InvalidPubkeyBytes)?,
        );

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
            compute_budget,
            feature_set,
            sysvar_context,
            program_id,
            instruction_accounts,
            instruction_data,
            accounts,
        })
    }
}

impl From<&FixtureContext> for proto::InstrContext {
    fn from(input: &FixtureContext) -> Self {
        let FixtureContext {
            compute_budget,
            feature_set,
            sysvar_context,
            program_id,
            instruction_accounts,
            instruction_data,
            accounts,
        } = input;

        let compute_budget = Some(compute_budget.into());
        let feature_set = Some(feature_set.into());
        let sysvars = Some(sysvar_context.into());
        let program_id = program_id.to_bytes().to_vec();

        let instr_accounts = instruction_accounts
            .iter()
            .map(|acct| proto::InstrAcct {
                index: accounts
                    .iter()
                    .position(|(pubkey, _)| *pubkey == acct.pubkey)
                    .unwrap() as u32,
                is_signer: acct.is_signer,
                is_writable: acct.is_writable,
            })
            .collect::<Vec<_>>();

        let accounts = accounts
            .iter()
            .map(|(pubkey, account)| proto::AcctState {
                address: pubkey.to_bytes().to_vec(),
                owner: account.owner().to_bytes().to_vec(),
                lamports: account.lamports(),
                data: account.data().to_vec(),
                executable: account.executable(),
                rent_epoch: account.rent_epoch(),
            })
            .collect::<Vec<_>>();

        proto::InstrContext {
            compute_budget,
            feature_set,
            sysvars,
            program_id,
            instr_accounts,
            data: instruction_data.clone(),
            accounts,
        }
    }
}
