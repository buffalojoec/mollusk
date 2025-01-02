#[cfg(feature = "fuzz-fd")]
pub mod firedancer;
#[cfg(feature = "fuzz")]
pub mod mollusk;

use {
    crate::{
        result::{Check, InstructionResult},
        Mollusk,
    },
    mollusk_svm_fuzz_fs::FsHandler,
    solana_sdk::{account::AccountSharedData, instruction::Instruction, pubkey::Pubkey},
};

pub fn generate_fixtures_from_mollusk_test(
    mollusk: &Mollusk,
    instruction: &Instruction,
    accounts: &[(Pubkey, AccountSharedData)],
    result: &InstructionResult,
    checks: &[Check],
) {
    #[cfg(feature = "fuzz")]
    {
        if std::env::var("EJECT_FUZZ_FIXTURES").is_ok()
            || std::env::var("EJECT_FUZZ_FIXTURES_JSON").is_ok()
        {
            let fixture = mollusk::build_fixture_from_mollusk_test(
                mollusk,
                instruction,
                accounts,
                result,
                checks,
            );
            let handler = FsHandler::new(fixture);
            if let Ok(blob_dir) = std::env::var("EJECT_FUZZ_FIXTURES") {
                handler.dump_to_blob_file(&blob_dir);
            }

            if let Ok(json_dir) = std::env::var("EJECT_FUZZ_FIXTURES_JSON") {
                handler.dump_to_json_file(&json_dir);
            }
        }
    }
    #[cfg(feature = "fuzz-fd")]
    {
        if std::env::var("EJECT_FUZZ_FIXTURES_FD").is_ok()
            || std::env::var("EJECT_FUZZ_FIXTURES_JSON_FD").is_ok()
        {
            let fixture = firedancer::build_fixture_from_mollusk_test(
                mollusk,
                instruction,
                accounts,
                result,
                checks,
            );
            let handler = FsHandler::new(fixture);
            if let Ok(blob_dir) = std::env::var("EJECT_FUZZ_FIXTURES_FD") {
                handler.dump_to_blob_file(&blob_dir);
            }

            if let Ok(json_dir) = std::env::var("EJECT_FUZZ_FIXTURES_JSON_FD") {
                handler.dump_to_json_file(&json_dir);
            }
        }
    }
}