//! Instruction account.

use {
    super::proto::InstrAcct as ProtoInstrAccount,
    solana_sdk::transaction_context::InstructionAccount,
};

impl From<ProtoInstrAccount> for InstructionAccount {
    fn from(value: ProtoInstrAccount) -> Self {
        let ProtoInstrAccount {
            index,
            is_writable,
            is_signer,
        } = value;
        Self {
            index_in_transaction: index as u16,
            index_in_caller: index as u16,
            index_in_callee: index as u16,
            is_signer,
            is_writable,
        }
    }
}

impl From<InstructionAccount> for ProtoInstrAccount {
    fn from(value: InstructionAccount) -> Self {
        let InstructionAccount {
            index_in_transaction,
            is_signer,
            is_writable,
            ..
        } = value;
        Self {
            index: index_in_transaction as u32,
            is_signer,
            is_writable,
        }
    }
}
