use {
    super::proto::{
        EpochContext as ProtoEpochContext, InstrContext as ProtoContext,
        SlotContext as ProtoSlotContext,
    },
    crate::account::SeedAddress,
    solana_sdk::{
        account::AccountSharedData, feature_set::FeatureSet, pubkey::Pubkey,
        transaction_context::InstructionAccount,
    },
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotContext {
    /// The slot to use for the simulation.
    pub slot: u64,
}

impl From<ProtoSlotContext> for SlotContext {
    fn from(value: ProtoSlotContext) -> Self {
        let ProtoSlotContext { slot } = value;
        Self { slot }
    }
}

impl From<SlotContext> for ProtoSlotContext {
    fn from(value: SlotContext) -> Self {
        let SlotContext { slot } = value;
        Self { slot }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EpochContext {
    /// The feature set to use for the simulation.
    pub feature_set: FeatureSet,
}

impl From<ProtoEpochContext> for EpochContext {
    fn from(value: ProtoEpochContext) -> Self {
        Self {
            feature_set: value.features.map(Into::into).unwrap_or_default(),
        }
    }
}

impl From<EpochContext> for ProtoEpochContext {
    fn from(value: EpochContext) -> Self {
        Self {
            features: Some(value.feature_set.into()),
        }
    }
}

/// Instruction context fixture.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Context {
    /// The program ID of the program being invoked.
    pub program_id: Pubkey,
    /// Input accounts with state.
    pub accounts: Vec<(Pubkey, AccountSharedData, Option<SeedAddress>)>,
    /// Accounts to pass to the instruction.
    pub instruction_accounts: Vec<InstructionAccount>,
    /// The instruction data.
    pub instruction_data: Vec<u8>,
    /// The compute units available to the program.
    pub compute_units_available: u64,
    /// Slot context.
    pub slot_context: SlotContext,
    /// Epoch context.
    pub epoch_context: EpochContext,
}

impl From<ProtoContext> for Context {
    fn from(value: ProtoContext) -> Self {
        let program_id_bytes: [u8; 32] = value
            .program_id
            .try_into()
            .expect("Invalid bytes for program ID");
        let program_id = Pubkey::new_from_array(program_id_bytes);

        let accounts: Vec<(Pubkey, AccountSharedData, Option<SeedAddress>)> =
            value.accounts.into_iter().map(Into::into).collect();

        let instruction_accounts: Vec<InstructionAccount> =
            value.instr_accounts.into_iter().map(Into::into).collect();

        Self {
            program_id,
            accounts,
            instruction_accounts,
            instruction_data: value.data,
            compute_units_available: value.cu_avail,
            slot_context: value.slot_context.map(Into::into).unwrap_or_default(),
            epoch_context: value.epoch_context.map(Into::into).unwrap_or_default(),
        }
    }
}

impl From<Context> for ProtoContext {
    fn from(value: Context) -> Self {
        let accounts = value.accounts.into_iter().map(Into::into).collect();

        let instr_accounts = value
            .instruction_accounts
            .into_iter()
            .map(Into::into)
            .collect();

        Self {
            program_id: value.program_id.to_bytes().to_vec(),
            accounts,
            instr_accounts,
            data: value.instruction_data,
            cu_avail: value.compute_units_available,
            slot_context: Some(value.slot_context.into()),
            epoch_context: Some(value.epoch_context.into()),
        }
    }
}
