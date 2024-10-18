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
    solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        instruction::Instruction,
        pubkey::Pubkey,
        transaction_context::{IndexOfAccount, InstructionAccount, TransactionAccount},
    },
    std::collections::HashMap,
};

struct KeyMap(HashMap<Pubkey, (bool, bool)>);

impl KeyMap {
    fn compile(instruction: &Instruction) -> Self {
        let mut map: HashMap<Pubkey, (bool, bool)> = HashMap::new();
        map.entry(instruction.program_id).or_default();
        for meta in instruction.accounts.iter() {
            let entry = map.entry(meta.pubkey).or_default();
            entry.0 |= meta.is_signer;
            entry.1 |= meta.is_writable;
        }
        Self(map)
    }

    fn is_signer(&self, key: &Pubkey) -> bool {
        self.0.get(key).map(|(s, _)| *s).unwrap_or(false)
    }

    fn is_writable(&self, key: &Pubkey) -> bool {
        self.0.get(key).map(|(_, w)| *w).unwrap_or(false)
    }
}

struct Keys<'a> {
    keys: Vec<&'a Pubkey>,
    key_map: &'a KeyMap,
}

impl<'a> Keys<'a> {
    fn new(key_map: &'a KeyMap) -> Self {
        Self {
            keys: key_map.0.keys().collect(),
            key_map,
        }
    }

    fn position(&self, key: &Pubkey) -> u8 {
        self.keys.iter().position(|k| *k == key).unwrap() as u8
    }

    fn is_signer(&self, index: usize) -> bool {
        self.key_map.is_signer(self.keys[index])
    }

    fn is_writable(&self, index: usize) -> bool {
        self.key_map.is_writable(self.keys[index])
    }
}

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
    let key_map = KeyMap::compile(instruction);
    let keys = Keys::new(&key_map);

    let compiled_instruction = CompiledInstructionWithoutData {
        program_id_index: keys.position(&instruction.program_id),
        accounts: instruction
            .accounts
            .iter()
            .map(|account_meta| keys.position(&account_meta.pubkey))
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
                is_signer: keys.is_signer(index_in_transaction),
                is_writable: keys.is_writable(index_in_transaction),
            }
        })
        .collect();

    let transaction_accounts: Vec<TransactionAccount> = keys
        .keys
        .iter()
        .map(|key| {
            if *key == &instruction.program_id {
                (**key, stub_out_program_account(loader_key))
            } else {
                let account = accounts
                    .iter()
                    .find(|(k, _)| k == *key)
                    .map(|(_, account)| account.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "    [mollusk]: An account required by the instruction was not \
                             provided: {:?}",
                            key,
                        )
                    });
                (**key, account)
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
