use {
    mollusk::InstructionResult,
    num_format::{Locale, ToFormattedString},
    std::path::Path,
};

pub(crate) struct MolluskComputeUnitBenchResult {
    name: String,
    mean: u64,
}

impl MolluskComputeUnitBenchResult {
    pub fn new(name: String, results: Vec<InstructionResult>) -> Self {
        let mut runs = results
            .iter()
            .map(|result| result.compute_units_consumed)
            .collect::<Vec<_>>();
        runs.sort();

        let len = runs.len();
        let mean = runs.iter().sum::<u64>() / len as u64;

        Self { name, mean }
    }
}

pub(crate) fn write_results(out_dir: &Path, results: Vec<MolluskComputeUnitBenchResult>) {
    let mut md_table = String::new();

    md_table.push_str("| Name | Mean |\n");
    md_table.push_str("|------|--------|\n");

    for result in results {
        let mean_str = result.mean.to_formatted_string(&Locale::en);
        md_table.push_str(&format!("| {} | {} |\n", result.name, mean_str));
    }

    let path = out_dir.join("compute_units.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, md_table).unwrap();
}
