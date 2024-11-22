use {
    mollusk_svm::Mollusk,
    solana_sdk::{account::AccountSharedData, pubkey::Pubkey},
};

pub const ID: Pubkey = solana_sdk::pubkey!("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo");

pub const MEMO_ELF: &'static [u8] = include_bytes!("elf/memo-v1.so");

pub fn add_program(mollusk: &mut Mollusk) {
    //BPFLoader1111111111111111111111111111111111
    mollusk.add_program_with_elf_and_loader(
        &ID,
        MEMO_ELF,
        &mollusk_svm::program::loader_keys::LOADER_V1,
    );
}

pub fn account() -> AccountSharedData {
    mollusk_svm::program::create_program_account_loader_v3(&ID)
}

/// Get the key and account for the SPL Memo program V1.
pub fn keyed_account() -> (Pubkey, AccountSharedData) {
    (ID, account())
}
