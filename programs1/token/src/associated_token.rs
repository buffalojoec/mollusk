use {
    mollusk_svm::Mollusk,
    solana_sdk::{account::AccountSharedData, pubkey::Pubkey},
};

pub const ID: Pubkey = solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub fn add_program(mollusk: &mut Mollusk) {
    mollusk.add_program_with_elf_and_loader(
        &ID,
        include_bytes!("elf/associated_token.so"),
        &mollusk_svm::program::loader_keys::LOADER_V2,
    );
}

/// Get the key and account for the system program.
pub fn keyed_account() -> (Pubkey, AccountSharedData) {
    (
        ID,
        mollusk_svm::program::create_program_account_loader_v3(&ID),
    )
}
