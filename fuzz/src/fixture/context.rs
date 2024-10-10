//! Instruction context fixture for invoking programs in a simulated program
//! runtime environment.

use {
    super::{
        error::FixtureError, feature_set::FixtureFeatureSet, proto, sysvars::FixtureSysvarContext,
    },
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_sdk::{account::AccountSharedData, instruction::AccountMeta, pubkey::Pubkey},
};

/// Instruction context fixture.
#[derive(Debug)]
pub struct FixtureContext {
    /// The compute budget to use for the simulation.
    pub compute_budget: ComputeBudget,
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
