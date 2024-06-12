//! Compute unit benchmarking for Solana programs.

use {
    mollusk::{InstructionResult, Mollusk, ProgramResult},
    num_format::{Locale, ToFormattedString},
    serde::{Deserialize, Serialize},
    solana_sdk::{account::AccountSharedData, instruction::Instruction, pubkey::Pubkey},
    std::path::{Path, PathBuf},
};

pub type Bench = (String, Instruction, Vec<(Pubkey, AccountSharedData)>);

pub struct MolluskComputeUnitBencher {
    benches: Vec<Bench>,
    benchmark: Option<u64>,
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
            benchmark: None,
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

    pub fn benchmark(mut self, benchmark: u64) -> Self {
        self.benchmark = Some(benchmark);
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
                let account_keys = accounts.len();
                let instruction_data_size = instruction.data.len();

                let mut results = vec![];
                for _ in 0..self.iterations {
                    let result = self
                        .mollusk
                        .process_instruction(&instruction, accounts.clone());

                    match result.result {
                        ProgramResult::Success => (),
                        _ => {
                            if self.must_pass {
                                panic!(
                                    "Program execution failed, but `must_pass` was set. Error: \
                                     {:?}",
                                    result.result
                                );
                            }
                        }
                    }

                    results.push(result);
                }

                MolluskComputeUnitBenchResult::new(
                    name,
                    account_keys,
                    instruction_data_size,
                    self.iterations,
                    self.benchmark,
                    results,
                )
            })
            .collect::<Vec<_>>();
        write_results(&self.out_dir, bench_results);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MolluskComputeUnitBenchResult {
    name: String,
    account_keys: usize,
    instruction_data_size: usize,
    iterations: u64,
    // The benchmark value to compare the median against.
    #[serde(skip_serializing_if = "Option::is_none")]
    mark: Option<u64>,
    // The number of compute units the median is above or below the benchmark.
    #[serde(skip_serializing_if = "Option::is_none")]
    mark_delta: Option<i64>,
    #[serde(skip)]
    mark_delta_str: Option<String>,
    max: u64,
    mean: f64,
    median: u64,
    min: u64,
}

impl MolluskComputeUnitBenchResult {
    pub fn new(
        name: String,
        account_keys: usize,
        instruction_data_size: usize,
        iterations: u64,
        mark: Option<u64>,
        results: Vec<InstructionResult>,
    ) -> Self {
        let mut runs = results
            .iter()
            .map(|result| result.compute_units_consumed)
            .collect::<Vec<_>>();
        runs.sort_unstable();

        let len = runs.len();
        let max = *runs.last().unwrap();
        let mean = runs.iter().sum::<u64>() as f64 / len as f64;
        let median = runs[len / 2];
        let min = *runs.first().unwrap();

        let mark_delta = mark.map(|benchmark| {
            let median = median as i64;
            let benchmark = benchmark as i64;
            median - benchmark
        });
        let mark_delta_str = mark_delta.map(|delta| {
            if delta > 0 {
                format!("+{}", delta.to_formatted_string(&Locale::en))
            } else {
                delta.to_string()
            }
        });

        Self {
            name,
            account_keys,
            instruction_data_size,
            iterations,
            mark,
            mark_delta,
            mark_delta_str,
            max,
            mean,
            median,
            min,
        }
    }
}

fn write_results(out_dir: &Path, results: Vec<MolluskComputeUnitBenchResult>) {
    write_to_json(out_dir, &results);
    write_to_markdown(out_dir, &results);
}

fn write_to_json(out_dir: &Path, results: &[MolluskComputeUnitBenchResult]) {
    let json_results = serde_json::to_string(&results).unwrap();
    let path = out_dir.join("compute_units.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, json_results).unwrap();
}

fn write_to_markdown(out_dir: &Path, results: &[MolluskComputeUnitBenchResult]) {
    let mut md_table = String::new();
    md_table.push_str("| Name | Median | Mark Delta |\n");
    md_table.push_str("|------|--------|------------|\n");
    for result in results {
        let median_str = result.median.to_formatted_string(&Locale::en);
        let mark_delta_str = match &result.mark_delta_str {
            Some(delta) => delta.clone(),
            None => String::from("N/A"),
        };
        md_table.push_str(&format!(
            "| {} | {} | {} |\n",
            result.name, median_str, mark_delta_str
        ));
    }

    let path = out_dir.join("compute_units.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, md_table).unwrap();
}
