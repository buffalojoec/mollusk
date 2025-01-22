//! Instruction <-> Transaction account compilation.

use {
    crate::keys::KeyMap,
    mollusk_svm_error::error::{MolluskError, MolluskPanic},
    solana_sdk::{
        account::Account,
        instruction::Instruction,
        pubkey::Pubkey,
        transaction_context::{IndexOfAccount, InstructionAccount},
    },
};

// Helper struct to avoid cloning instruction data.
pub struct CompiledInstructionWithoutData {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
}

pub fn compile_instruction_without_data(
    key_map: &KeyMap,
    instruction: &Instruction,
) -> CompiledInstructionWithoutData {
    CompiledInstructionWithoutData {
        program_id_index: key_map.position(&instruction.program_id).unwrap() as u8,
        accounts: instruction
            .accounts
            .iter()
            .map(|account_meta| key_map.position(&account_meta.pubkey).unwrap() as u8)
            .collect(),
    }
}

pub fn compile_instruction_accounts(
    key_map: &KeyMap,
    compiled_instruction: &CompiledInstructionWithoutData,
) -> Vec<InstructionAccount> {
    compiled_instruction
        .accounts
        .iter()
        .enumerate()
        .map(|(ix_account_index, &index_in_transaction)| {
            let index_in_callee = compiled_instruction
                .accounts
                .get(0..ix_account_index)
                .unwrap()
                .iter()
                .position(|&account_index| account_index == index_in_transaction)
                .unwrap_or(ix_account_index) as IndexOfAccount;
            let index_in_transaction = index_in_transaction as usize;
            InstructionAccount {
                index_in_transaction: index_in_transaction as IndexOfAccount,
                index_in_caller: index_in_transaction as IndexOfAccount,
                index_in_callee,
                is_signer: key_map.is_signer_at_index(index_in_transaction),
                is_writable: key_map.is_writable_at_index(index_in_transaction),
            }
        })
        .collect()
}

pub fn compile_transaction_accounts_for_instruction(
    key_map: &KeyMap,
    instruction: &Instruction,
    accounts: &[(Pubkey, Account)],
    stub_out_program_account: Option<Box<dyn Fn() -> Account>>,
) -> Vec<(Pubkey, Account)> {
    key_map
        .keys()
        .map(|key| {
            if let Some(stub_out_program_account) = &stub_out_program_account {
                if instruction.program_id == *key {
                    return (*key, stub_out_program_account());
                }
            }
            let account = accounts
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, account)| account.clone())
                .or_panic_with(MolluskError::AccountMissing(key));
            (*key, account)
        })
        .collect()
}

pub fn compile_transaction_accounts(
    key_map: &KeyMap,
    instructions: &[Instruction],
    accounts: &[(Pubkey, Account)],
    stub_out_program_account: Option<Box<dyn Fn() -> Account>>,
) -> Vec<(Pubkey, Account)> {
    key_map
        .keys()
        .map(|key| {
            if let Some(stub_out_program_account) = &stub_out_program_account {
                if instructions.iter().any(|ix| ix.program_id == *key) {
                    return (*key, stub_out_program_account());
                }
            }
            let account = accounts
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, account)| account.clone())
                .or_panic_with(MolluskError::AccountMissing(key));
            (*key, account)
        })
        .collect()
}
