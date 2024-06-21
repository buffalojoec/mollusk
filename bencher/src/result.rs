use {
    chrono::{DateTime, Utc},
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
    let path = out_dir.join("compute_units.md");

    let mut no_changes = true;
    let previous = parse_last_md_table(&path);

    let mut md_table = md_header();

    for result in results {
        let delta = match previous.as_ref().and_then(|prev_results| {
            prev_results
                .iter()
                .find(|prev_result| prev_result.name == result.name)
        }) {
            Some(prev) => {
                let delta = result.mean as i64 - prev.mean as i64;
                if delta == 0 {
                    "--".to_string()
                } else {
                    no_changes = false;
                    if delta > 0 {
                        format!("+{}", delta.to_formatted_string(&Locale::en))
                    } else {
                        delta.to_formatted_string(&Locale::en)
                    }
                }
            }
            None => {
                no_changes = false;
                "- new -".to_string()
            }
        };

        md_table.push_str(&format!(
            "| {} | {} | {} |\n",
            result.name, result.mean, delta
        ));
    }

    // Only create a new table if there were changes.
    if !no_changes {
        md_table.push('\n');
        prepend_to_md_file(&path, &md_table);
    }
}

fn md_header() -> String {
    let now: DateTime<Utc> = Utc::now();
    format!(
        r#"#### Compute Units: {}

| Name | Mean | Delta |
|------|------|-------|
"#,
        now
    )
}

fn parse_last_md_table(path: &Path) -> Option<Vec<MolluskComputeUnitBenchResult>> {
    if !path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(path).unwrap();
    let mut results = vec![];

    for line in contents.lines().skip(4) {
        if line.starts_with("####") || line.is_empty() {
            break;
        }

        let mut parts = line.split('|').skip(1).map(str::trim);
        let name = parts.next().unwrap().to_string();
        let mean = parts.next().unwrap().parse().unwrap();

        results.push(MolluskComputeUnitBenchResult { name, mean });
    }

    Some(results)
}

fn prepend_to_md_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let contents = if path.exists() {
        std::fs::read_to_string(path).unwrap()
    } else {
        String::new()
    };

    let mut new_contents = content.to_string();
    new_contents.push_str(&contents);

    std::fs::write(path, new_contents).unwrap();
}
