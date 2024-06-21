//! Compute unit benchmarking for Solana programs.

mod result;

use {
    mollusk::{result::ProgramResult, Mollusk},
    result::{write_results, MolluskComputeUnitBenchResult},
    solana_sdk::{account::AccountSharedData, instruction::Instruction, pubkey::Pubkey},
    std::path::PathBuf,
};

pub type Bench = (String, Instruction, Vec<(Pubkey, AccountSharedData)>);

pub struct MolluskComputeUnitBencher {
    benches: Vec<Bench>,
    iterations: u64,
    mollusk: Mollusk,
    must_pass: bool,
    out_dir: PathBuf,
}

impl MolluskComputeUnitBencher {
    pub fn new(mollusk: Mollusk) -> Self {
        let mut out_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        out_dir.push("benches");
        Self {
            benches: Vec::new(),
            iterations: 25, // Default to 25 iterations.
            mollusk,
            must_pass: false,
            out_dir,
        }
    }

    pub fn bench(mut self, bench: Bench) -> Self {
        self.benches.push(bench);
        self
    }

    pub fn iterations(mut self, iterations: u64) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn must_pass(mut self, must_pass: bool) -> Self {
        self.must_pass = must_pass;
        self
    }

    pub fn out_dir(mut self, out_dir: &str) -> Self {
        self.out_dir = PathBuf::from(out_dir);
        self
    }

    pub fn execute(&mut self) {
        let bench_results = std::mem::take(&mut self.benches)
            .into_iter()
            .map(|(name, instruction, accounts)| {
                let mut results = vec![];

                for _ in 0..self.iterations {
                    let result = self
                        .mollusk
                        .process_instruction(&instruction, accounts.clone());

                    match result.program_result {
                        ProgramResult::Success => (),
                        _ => {
                            if self.must_pass {
                                panic!(
                                    "Program execution failed, but `must_pass` was set. Error: \
                                     {:?}",
                                    result.program_result
                                );
                            }
                        }
                    }

                    results.push(result);
                }

                MolluskComputeUnitBenchResult::new(name, results)
            })
            .collect::<Vec<_>>();
        write_results(&self.out_dir, bench_results);
    }
}
