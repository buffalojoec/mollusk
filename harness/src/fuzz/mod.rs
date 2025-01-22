#[cfg(any(feature = "fuzz", feature = "fuzz-fd"))]
pub mod check;
#[cfg(feature = "fuzz-fd")]
pub mod firedancer;
#[cfg(feature = "fuzz")]
pub mod mollusk;

use {
    crate::{result::InstructionResult, Mollusk},
    solana_sdk::{account::Account, instruction::Instruction, pubkey::Pubkey},
    solana_svm_fuzz_harness_fixture_fs::FsHandler,
};

pub fn generate_fixtures_from_mollusk_test(
    mollusk: &Mollusk,
    instruction: &Instruction,
    accounts: &[(Pubkey, Account)],
    result: &InstructionResult,
) {
    #[cfg(feature = "fuzz")]
    {
        if std::env::var("EJECT_FUZZ_FIXTURES").is_ok()
            || std::env::var("EJECT_FUZZ_FIXTURES_JSON").is_ok()
        {
            let fixture =
                mollusk::build_fixture_from_mollusk_test(mollusk, instruction, accounts, result);
            let handler = FsHandler::new(fixture);
            if let Ok(blob_dir) = std::env::var("EJECT_FUZZ_FIXTURES") {
                handler.dump_to_blob_file(&blob_dir).unwrap();
            }

            if let Ok(json_dir) = std::env::var("EJECT_FUZZ_FIXTURES_JSON") {
                handler.dump_to_json_file(&json_dir).unwrap();
            }
        }
    }
    #[cfg(feature = "fuzz-fd")]
    {
        if std::env::var("EJECT_FUZZ_FIXTURES_FD").is_ok()
            || std::env::var("EJECT_FUZZ_FIXTURES_JSON_FD").is_ok()
        {
            let fixture =
                firedancer::build_fixture_from_mollusk_test(mollusk, instruction, accounts, result);
            let handler = FsHandler::new(fixture);
            if let Ok(blob_dir) = std::env::var("EJECT_FUZZ_FIXTURES_FD") {
                handler.dump_to_blob_file(&blob_dir).unwrap();
            }

            if let Ok(json_dir) = std::env::var("EJECT_FUZZ_FIXTURES_JSON_FD") {
                handler.dump_to_json_file(&json_dir).unwrap();
            }
        }
    }
}
