use {
    mollusk::InstructionResult,
    num_format::{Locale, ToFormattedString},
    std::path::Path,
};

pub(crate) struct MolluskComputeUnitBenchResult {
    name: String,
    iterations: u64,
    // The benchmark value to compare the mean against.
    mark: Option<u64>,
    // The number of compute units the mean is above or below the benchmark.
    mark_delta: Option<i64>,
    mark_delta_str: Option<String>,
    mean: u64,
}

impl MolluskComputeUnitBenchResult {
    pub fn new(
        name: String,
        iterations: u64,
        mark: Option<u64>,
        results: Vec<InstructionResult>,
    ) -> Self {
        let mut runs = results
            .iter()
            .map(|result| result.compute_units_consumed)
            .collect::<Vec<_>>();
        runs.sort();

        let len = runs.len();
        let mean = runs.iter().sum::<u64>() / len as u64;

        let mark_delta = mark.map(|benchmark| {
            let mean = mean as i64;
            let benchmark = benchmark as i64;
            mean - benchmark
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
            iterations,
            mark,
            mark_delta,
            mark_delta_str,
            mean,
        }
    }
}

pub(crate) fn write_results(out_dir: &Path, results: Vec<MolluskComputeUnitBenchResult>) {
    let mut md_table = String::new();
    md_table.push_str("| Name | Mean | Mark Delta |\n");
    md_table.push_str("|------|--------|------------|\n");
    for result in results {
        let mean_str = result.mean.to_formatted_string(&Locale::en);
        let mark_delta_str = match &result.mark_delta_str {
            Some(delta) => delta.clone(),
            None => String::from("N/A"),
        };
        md_table.push_str(&format!(
            "| {} | {} | {} |\n",
            result.name, mean_str, mark_delta_str
        ));
    }

    let path = out_dir.join("compute_units.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, md_table).unwrap();
}
