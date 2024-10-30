use {
    mollusk_svm::Mollusk,
    solana_sdk::{account::AccountSharedData, pubkey::Pubkey},
};

pub const ID: Pubkey = solana_sdk::pubkey!("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo");

pub fn add_program(mollusk: &mut Mollusk) {
    //BPFLoader1111111111111111111111111111111111
    mollusk.add_program_with_elf_and_loader(
        &ID,
        include_bytes!("elf/memo-v1.so"),
        &mollusk_svm::program::loader_keys::LOADER_V1,
    );
}

/// Get the key and account for the system program.
pub fn keyed_account() -> (Pubkey, AccountSharedData) {
    (
        ID,
        mollusk_svm::program::create_program_account_loader_v3(&ID),
    )
}
