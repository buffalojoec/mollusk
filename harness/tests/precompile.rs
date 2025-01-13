use {
    mollusk_svm::{result::Check, Mollusk},
    rand0_7::thread_rng,
    solana_sdk::{
        account::{Account, WritableAccount},
        ed25519_instruction::new_ed25519_instruction,
        ed25519_program, native_loader,
        pubkey::Pubkey,
        secp256k1_instruction::new_secp256k1_instruction,
        secp256k1_program,
    },
};

fn precompile_account() -> Account {
    let mut account = Account::new(1, 0, &native_loader::id());
    account.set_executable(true);
    account
}

#[test]
fn test_secp256k1() {
    let mollusk = Mollusk::default();
    let secret_key = libsecp256k1::SecretKey::random(&mut thread_rng());

    mollusk.process_and_validate_instruction(
        &new_secp256k1_instruction(&secret_key, b"hello"),
        &[
            (Pubkey::new_unique(), Account::default()),
            (secp256k1_program::id(), precompile_account()),
        ],
        &[Check::success()],
    );
}

#[test]
fn test_ed25519() {
    let mollusk = Mollusk::default();
    let secret_key = ed25519_dalek::Keypair::generate(&mut thread_rng());

    mollusk.process_and_validate_instruction(
        &new_ed25519_instruction(&secret_key, b"hello"),
        &[
            (Pubkey::new_unique(), Account::default()),
            (ed25519_program::id(), precompile_account()),
        ],
        &[Check::success()],
    );
}

#[test]
fn test_secp256r1() {
    // Add me when patch version for 2.1 is advanced!
}
