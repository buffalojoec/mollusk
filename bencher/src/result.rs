use {
    mollusk::InstructionResult,
    num_format::{Locale, ToFormattedString},
    serde::{Deserialize, Serialize},
    std::path::Path,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MolluskComputeUnitBenchResult {
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

pub(crate) fn write_results(out_dir: &Path, results: Vec<MolluskComputeUnitBenchResult>) {
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
