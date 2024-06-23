//! Compute unit benchmarking for Solana programs.

mod result;

use {
    mollusk_svm::{result::ProgramResult, Mollusk},
    result::{write_results, MolluskComputeUnitBenchResult},
    solana_sdk::{account::AccountSharedData, instruction::Instruction, pubkey::Pubkey},
    std::path::PathBuf,
};

/// A bench is a tuple of a name, an instruction, and a list of accounts.
pub type Bench<'a> = (&'a str, &'a Instruction, &'a [(Pubkey, AccountSharedData)]);

/// Mollusk's compute unit bencher.
///
/// Allows developers to bench test compute unit usage on their programs.
pub struct MolluskComputeUnitBencher<'a> {
    benches: Vec<Bench<'a>>,
    mollusk: Mollusk,
    must_pass: bool,
    out_dir: PathBuf,
}

impl<'a> MolluskComputeUnitBencher<'a> {
    /// Create a new bencher, to which benches and configurations can be added.
    pub fn new(mollusk: Mollusk) -> Self {
        let mut out_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        out_dir.push("benches");
        Self {
            benches: Vec::new(),
            mollusk,
            must_pass: false,
            out_dir,
        }
    }

    /// Add a bench to the bencher.
    pub fn bench(mut self, bench: Bench<'a>) -> Self {
        self.benches.push(bench);
        self
    }

    /// Set whether the bencher should panic if a program execution fails.
    pub fn must_pass(mut self, must_pass: bool) -> Self {
        self.must_pass = must_pass;
        self
    }

    /// Set the output directory for the results.
    pub fn out_dir(mut self, out_dir: &str) -> Self {
        self.out_dir = PathBuf::from(out_dir);
        self
    }

    /// Execute the benches.
    pub fn execute(&mut self) {
        let bench_results = std::mem::take(&mut self.benches)
            .into_iter()
            .map(|(name, instruction, accounts)| {
                let result = self.mollusk.process_instruction(instruction, accounts);
                match result.program_result {
                    ProgramResult::Success => (),
                    _ => {
                        if self.must_pass {
                            panic!(
                                "Program execution failed, but `must_pass` was set. Error: {:?}",
                                result.program_result
                            );
                        }
                    }
                }
                MolluskComputeUnitBenchResult::new(name, result)
            })
            .collect::<Vec<_>>();
        write_results(&self.out_dir, bench_results);
    }
}
