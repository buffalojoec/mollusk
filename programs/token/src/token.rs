use {mollusk_svm::Mollusk, solana_account::Account, solana_pubkey::Pubkey};

pub const ID: Pubkey = solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub const ELF: &[u8] = include_bytes!("elf/token.so");

pub fn add_program(mollusk: &mut Mollusk) {
    // Loader v2
    mollusk.add_program_with_elf_and_loader(
        &ID,
        ELF,
        &mollusk_svm::program::loader_keys::LOADER_V2,
    );
}

pub fn account() -> Account {
    // Loader v2
    mollusk_svm::program::create_program_account_loader_v2(ELF)
}

/// Get the key and account for the SPL Token program.
pub fn keyed_account() -> (Pubkey, Account) {
    (ID, account())
}
