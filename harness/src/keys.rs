//! Instruction <-> Transaction key deduplication and privilege handling.
//!
//! Solana instructions and transactions are designed to be intentionally
//! verbosely declarative, to provide the runtime with granular directives
//! for manipulating chain state.
//!
//! As a result, when a transaction is _compiled_, many steps occur:
//! * Ensuring there is a fee payer.
//! * Ensuring there is a signature.
//! * Deduplicating account keys.
//! * Configuring the highest role awarded to each account key.
//! * ...
//!
//! Since Mollusk does not use transactions or fee payers, the deduplication
//! of account keys and handling of roles are the only two steps necessary
//! to perform under the hood within the harness.
//!
//! This implementation closely follows the implementation in the Anza SDK
//! for `Message::new_with_blockhash`. For more information, see:
//! <https://github.com/anza-xyz/agave/blob/c6e8239843af8e6301cd198e39d0a44add427bef/sdk/program/src/message/legacy.rs#L357>.

use {
    mollusk_svm_error::error::{MolluskError, MolluskPanic},
    mollusk_svm_keys::keys::KeyMap,
    solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        instruction::Instruction,
        pubkey::Pubkey,
        transaction_context::{IndexOfAccount, InstructionAccount, TransactionAccount},
    },
};

// Helper struct so Mollusk doesn't have to clone instruction data.
struct CompiledInstructionWithoutData {
    program_id_index: u8,
    accounts: Vec<u8>,
}

pub struct CompiledAccounts {
    pub program_id_index: u16,
    pub instruction_accounts: Vec<InstructionAccount>,
    pub transaction_accounts: Vec<TransactionAccount>,
}

pub fn compile_accounts(
    instruction: &Instruction,
    accounts: &[(Pubkey, AccountSharedData)],
    loader_key: Pubkey,
) -> CompiledAccounts {
    let key_map = KeyMap::compile_from_instruction(instruction);

    let compiled_instruction = CompiledInstructionWithoutData {
        program_id_index: key_map.position(&instruction.program_id).unwrap() as u8,
        accounts: instruction
            .accounts
            .iter()
            .map(|account_meta| key_map.position(&account_meta.pubkey).unwrap() as u8)
            .collect(),
    };

    let instruction_accounts: Vec<InstructionAccount> = compiled_instruction
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
        .collect();

    let transaction_accounts: Vec<TransactionAccount> = key_map
        .keys()
        .map(|key| {
            if *key == instruction.program_id {
                (*key, stub_out_program_account(loader_key))
            } else {
                let account = accounts
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, account)| account.clone())
                    .or_panic_with(MolluskError::AccountMissing(key));
                (*key, account)
            }
        })
        .collect();

    CompiledAccounts {
        program_id_index: compiled_instruction.program_id_index as u16,
        instruction_accounts,
        transaction_accounts,
    }
}

fn stub_out_program_account(loader_key: Pubkey) -> AccountSharedData {
    let mut program_account = AccountSharedData::default();
    program_account.set_owner(loader_key);
    program_account.set_executable(true);
    program_account
}
