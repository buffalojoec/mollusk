use {
    mollusk::Mollusk,
    mollusk_bencher::MolluskComputeUnitBencher,
    solana_sdk::{instruction::Instruction, pubkey::Pubkey},
};

#[test]
fn test_markdown() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");
    solana_logger::setup_with("");

    let program_id = Pubkey::new_unique();

    let instruction = Instruction::new_with_bytes(program_id, &[0], vec![]);
    let accounts = vec![];

    let mollusk = Mollusk::new(&program_id, "test_program");

    MolluskComputeUnitBencher::new(mollusk)
        .bench((String::from("bench0"), instruction.clone(), &accounts))
        .bench((String::from("bench1"), instruction.clone(), &accounts))
        .bench((String::from("bench2"), instruction.clone(), &accounts))
        .bench((String::from("bench3"), instruction.clone(), &accounts))
        .bench((String::from("bench4"), instruction.clone(), &accounts))
        .bench((String::from("bench5"), instruction.clone(), &accounts))
        .bench((String::from("bench6"), instruction, &accounts))
        .iterations(100)
        .must_pass(true)
        .out_dir("../target/benches")
        .execute();
}
