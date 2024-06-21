use {
    mollusk::Mollusk,
    mollusk_bencher::MolluskComputeUnitBencher,
    solana_sdk::{instruction::Instruction, pubkey::Pubkey},
};

#[test]
fn test_markdown() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");

    let program_id = Pubkey::new_unique();

    let bench = String::from("bench");
    let instruction = Instruction::new_with_bytes(program_id, &[1], vec![]);
    let accounts = vec![];

    let mollusk = Mollusk::new(&program_id, "test_program");

    MolluskComputeUnitBencher::new(mollusk)
        .bench((bench, instruction, accounts))
        .iterations(100)
        .must_pass(true)
        .out_dir("../target/benches")
        .execute();
}
