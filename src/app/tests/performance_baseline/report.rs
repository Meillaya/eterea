use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::report_environment::{environment_json, hardware_json};
use crate::{invalid_data, TestResult, SAMPLE_COUNT};

#[derive(Clone, Copy)]
pub(crate) enum MetricKey {
    ColdStart,
    Open,
    Import,
    List,
    Search,
    AuthorIndex,
    TopicIndex,
    MediaHydration,
    Stats,
}

impl MetricKey {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ColdStart => "cold_start_ms",
            Self::Open => "open_ms",
            Self::Import => "import_ms",
            Self::List => "list_ms",
            Self::Search => "search_ms",
            Self::AuthorIndex => "author_index_ms",
            Self::TopicIndex => "topic_index_ms",
            Self::MediaHydration => "media_hydration_ms",
            Self::Stats => "stats_ms",
        }
    }
}

pub(crate) struct MetricSamples {
    pub(crate) key: MetricKey,
    pub(crate) budget_ms: u64,
    pub(crate) samples: Vec<Duration>,
}

pub(crate) struct ReportSpec<'a> {
    pub(crate) name: &'a str,
    pub(crate) generated_count: usize,
    pub(crate) input_generation: &'a str,
    pub(crate) classification: &'a str,
    pub(crate) release_evidence: bool,
    pub(crate) storage_mode: &'a str,
    pub(crate) extra_dataset_json: &'a str,
}

pub(crate) struct DurationStats {
    pub(crate) min: u128,
    pub(crate) median: u128,
    pub(crate) p95: u128,
    pub(crate) max: u128,
}

struct ReportSections {
    path_json: String,
    budget_json: String,
    min_json: String,
    median_json: String,
    p95_json: String,
    max_json: String,
    all_pass: bool,
}

pub(crate) fn repo_root() -> TestResult<PathBuf> {
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = app_dir
        .parent()
        .ok_or_else(|| invalid_data("app crate manifest directory has no parent"))?;
    let root = src_dir
        .parent()
        .ok_or_else(|| invalid_data("src directory has no repository parent"))?;
    Ok(root.to_path_buf())
}

pub(crate) fn duration_stats(samples: &[Duration]) -> TestResult<DurationStats> {
    if samples.is_empty() {
        return Err(invalid_data("metric requires at least one sample").into());
    }
    let mut sorted = samples.iter().map(Duration::as_millis).collect::<Vec<_>>();
    sorted.sort_unstable();
    let p95_index = (sorted.len() * 95).div_ceil(100) - 1;
    Ok(DurationStats {
        min: sorted[0],
        median: sorted[sorted.len() / 2],
        p95: sorted[p95_index],
        max: sorted[sorted.len() - 1],
    })
}

pub(crate) fn assert_bounded(label: &str, elapsed: Duration, max: Duration) {
    assert!(
        elapsed <= max,
        "{label} took {:?}, budget <= {:?}",
        elapsed,
        max
    );
}

pub(crate) fn write_report(
    spec: &ReportSpec<'_>,
    metrics: &[MetricSamples],
) -> TestResult<PathBuf> {
    let mut path_json = String::new();
    let mut budget_json = String::new();
    let mut min_json = String::new();
    let mut median_json = String::new();
    let mut p95_json = String::new();
    let mut max_json = String::new();
    let mut all_pass = true;

    for (index, metric) in metrics.iter().enumerate() {
        let comma = if index == 0 { "" } else { "," };
        let stats = duration_stats(&metric.samples)?;
        let (metric_block, pass) = metric_json(metric, &stats);
        let key = metric.key.as_str();
        path_json.push_str(&format!("{comma}\n    \"{key}\": {metric_block}"));
        budget_json.push_str(&format!("{comma}\"{key}\":{}", metric.budget_ms));
        min_json.push_str(&format!("{comma}\"{key}\":{}", stats.min));
        median_json.push_str(&format!("{comma}\"{key}\":{}", stats.median));
        p95_json.push_str(&format!("{comma}\"{key}\":{}", stats.p95));
        max_json.push_str(&format!("{comma}\"{key}\":{}", stats.max));
        all_pass &= pass;
    }

    let report = report_json(
        spec,
        &ReportSections {
            path_json,
            budget_json,
            min_json,
            median_json,
            p95_json,
            max_json,
            all_pass,
        },
    );
    let report_path = repo_root()?.join(format!("target/eterea/perf/{}.json", spec.name));
    create_parent_dir(&report_path)?;
    fs::write(&report_path, report)?;
    Ok(report_path)
}

fn create_parent_dir(path: &Path) -> TestResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid_data(format!("report path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)?;
    Ok(())
}

fn metric_json(metric: &MetricSamples, stats: &DurationStats) -> (String, bool) {
    let sample_values = metric
        .samples
        .iter()
        .map(|sample| sample.as_millis().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let pass = stats.p95 <= u128::from(metric.budget_ms);
    let json = format!(
        concat!(
            "{{\"sample_count\":{},\"samples_ms\":[{}],\"min\":{},\"median\":{},",
            "\"p95\":{},\"max\":{},\"budget\":{},\"pass\":{},\"status\":\"{}\"}}"
        ),
        metric.samples.len(),
        sample_values,
        stats.min,
        stats.median,
        stats.p95,
        stats.max,
        metric.budget_ms,
        pass,
        if pass { "pass" } else { "fail" }
    );
    (json, pass)
}

fn report_json(spec: &ReportSpec<'_>, sections: &ReportSections) -> String {
    format!(
        concat!(
            "{{\n  \"report_name\": \"{}\",\n  \"sample_count\": {},\n  ",
            "\"generated_count\": {},\n  \"dataset\": {{\"generator\": \"{}\",",
            " \"generated_count\": {}, \"sample_count\": {}{} }},\n  ",
            "\"classification\": \"{}\",\n  \"release_evidence\": {},\n  ",
            "\"storage_mode\": \"{}\",\n  \"environment\": {},\n  \"hardware\": {},\n  ",
            "\"budget\": {{{}}},\n  \"min\": {{{}}},\n  \"median\": {{{}}},\n  ",
            "\"p95\": {{{}}},\n  \"max\": {{{}}},\n  \"pass\": {},\n  ",
            "\"status\": \"{}\",\n  \"paths\": {{\n{}\n  }}\n}}\n"
        ),
        json_escape(spec.name),
        SAMPLE_COUNT,
        spec.generated_count,
        json_escape(spec.input_generation),
        spec.generated_count,
        SAMPLE_COUNT,
        spec.extra_dataset_json,
        json_escape(spec.classification),
        spec.release_evidence,
        json_escape(spec.storage_mode),
        environment_json(spec.storage_mode, spec.release_evidence),
        hardware_json(),
        sections.budget_json,
        sections.min_json,
        sections.median_json,
        sections.p95_json,
        sections.max_json,
        sections.all_pass,
        if sections.all_pass { "pass" } else { "fail" },
        sections.path_json
    )
}

pub(crate) fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
