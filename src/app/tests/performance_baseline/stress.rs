use std::env;

use crate::{invalid_data, TestResult};

const STRESS_TIERS: &str = include_str!("../../../../fixtures/perf/stress-tiers.txt");

pub(crate) fn selected_stress_count() -> TestResult<usize> {
    let supported_counts = supported_stress_counts()?;
    let count = match env::var("ETEREA_STRESS_COUNT") {
        Ok(value) => value.parse::<usize>()?,
        Err(env::VarError::NotPresent) => first_supported_count(&supported_counts)?,
        Err(error) => {
            return Err(
                invalid_data(format!("ETEREA_STRESS_COUNT is invalid unicode: {error}")).into(),
            )
        }
    };
    if supported_counts.contains(&count) {
        Ok(count)
    } else {
        Err(invalid_data(format!(
            "ETEREA_STRESS_COUNT must be one of {:?}",
            supported_counts
        ))
        .into())
    }
}

pub(crate) fn stress_dataset_extra_json() -> TestResult<String> {
    let supported_counts = supported_stress_counts()?;
    Ok(format!(
        ", \"supported_stress_counts\": [{}], \"memory_scope\": \"archive generation and in-memory SQLite are included in this stress-lab run; record host memory ceilings before using high-volume results as release evidence\"",
        stress_tier_json_array(&supported_counts)
    ))
}

fn supported_stress_counts() -> TestResult<Vec<usize>> {
    let mut counts = Vec::new();
    for line in STRESS_TIERS.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            counts.push(trimmed.parse::<usize>()?);
        }
    }
    if counts.is_empty() {
        Err(
            invalid_data("fixtures/perf/stress-tiers.txt must list at least one stress tier")
                .into(),
        )
    } else {
        Ok(counts)
    }
}

fn first_supported_count(counts: &[usize]) -> TestResult<usize> {
    if counts.is_empty() {
        Err(invalid_data("stress tier list is empty").into())
    } else {
        Ok(counts[0])
    }
}

fn stress_tier_json_array(counts: &[usize]) -> String {
    counts
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
