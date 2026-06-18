#[path = "performance_baseline/data.rs"]
mod data;
#[path = "performance_baseline/file_backed.rs"]
mod file_backed;
#[path = "performance_baseline/measure.rs"]
mod measure;
#[path = "performance_baseline/report.rs"]
mod report;
#[path = "performance_baseline/report_environment.rs"]
mod report_environment;
#[path = "performance_baseline/schema.rs"]
mod schema;
#[path = "performance_baseline/stress.rs"]
mod stress;

use std::error::Error;
use std::fs;
use std::io;
use std::time::Duration;

use measure::{archive_metrics, author_directory_metric, run_archive_paths};
use report::{assert_bounded, duration_stats, write_report, MetricKey, MetricSamples, ReportSpec};
use schema::assert_report_schema;

pub(crate) const SAMPLE_COUNT: usize = 7;
const GENERATED_COUNT: usize = 500;
const LARGE_GENERATED_COUNT: usize = 10_000;
const AUTHOR_HEAVY_GENERATED_COUNT: usize = 10_000;

type TestResult<T> = Result<T, Box<dyn Error>>;

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn sampled_archive_report(
    report_name: &str,
    filename: &str,
    generated_count: usize,
    include_stats: bool,
    classification: &str,
    extra_dataset_json: &str,
    specs: &[(MetricKey, u64)],
) -> TestResult<()> {
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        samples.push(run_archive_paths(generated_count, filename, include_stats)?);
    }
    let metrics = archive_metrics(&samples, specs)?;
    assert_metric_budgets(&metrics)?;
    let report_path = write_report(
        &ReportSpec {
            name: report_name,
            generated_count,
            input_generation: "deterministic-json-string",
            classification,
            release_evidence: false,
            storage_mode: "in-memory-sqlite",
            extra_dataset_json,
        },
        &metrics,
    )?;
    eprintln!(
        "statistical performance report: name={report_name} count={generated_count} sample_count={SAMPLE_COUNT} report={}",
        report_path.display()
    );
    Ok(())
}

fn assert_metric_budgets(metrics: &[MetricSamples]) -> TestResult<()> {
    for metric in metrics {
        let stats = duration_stats(&metric.samples)?;
        assert_bounded(
            metric.key.as_str(),
            Duration::from_millis(u64::try_from(stats.p95)?),
            Duration::from_millis(metric.budget_ms),
        );
    }
    Ok(())
}

#[test]
fn performance_baseline_for_current_service_paths() -> TestResult<()> {
    sampled_archive_report(
        "performance_baseline",
        "generated.json",
        GENERATED_COUNT,
        true,
        "dev-guardrail",
        "",
        &[
            (MetricKey::Import, 5_000),
            (MetricKey::List, 1_000),
            (MetricKey::Search, 1_000),
            (MetricKey::Stats, 1_000),
        ],
    )
}

#[test]
#[ignore = "stress-lab target; run via scripts/perf-baseline.sh --stress <count>"]
fn stress_archive_report_for_configured_count() -> TestResult<()> {
    let count = stress::selected_stress_count()?;
    let extra_dataset_json = stress::stress_dataset_extra_json()?;
    sampled_archive_report(
        &format!("stress-lab/performance_stress_lab_{count}"),
        "generated-stress.json",
        count,
        false,
        "stress-lab",
        &extra_dataset_json,
        &[
            (MetricKey::Import, 60_000),
            (MetricKey::List, 1_000),
            (MetricKey::Search, 1_000),
            (MetricKey::AuthorIndex, 1_000),
            (MetricKey::TopicIndex, 1_000),
        ],
    )
}

#[test]
fn large_archive_budget_for_release_paths() -> TestResult<()> {
    sampled_archive_report(
        "performance_large_archive",
        "generated-large.json",
        LARGE_GENERATED_COUNT,
        false,
        "dev-guardrail",
        "",
        &[
            (MetricKey::Import, 10_000),
            (MetricKey::List, 100),
            (MetricKey::Search, 150),
            (MetricKey::AuthorIndex, 100),
            (MetricKey::TopicIndex, 100),
        ],
    )
}

#[test]
fn high_cardinality_author_directory_budget() -> TestResult<()> {
    let metrics = vec![author_directory_metric(AUTHOR_HEAVY_GENERATED_COUNT, 100)?];
    assert_metric_budgets(&metrics)?;
    let report_path = write_report(
        &ReportSpec {
            name: "performance_author_directory",
            generated_count: AUTHOR_HEAVY_GENERATED_COUNT,
            input_generation: "deterministic-author-heavy-json-string",
            classification: "dev-guardrail",
            release_evidence: false,
            storage_mode: "in-memory-sqlite",
            extra_dataset_json: ", \"cardinality\": \"unique-author-per-bookmark\"",
        },
        &metrics,
    )?;
    eprintln!(
        "author directory statistical report: count={AUTHOR_HEAVY_GENERATED_COUNT} sample_count={SAMPLE_COUNT} report={}",
        report_path.display()
    );
    Ok(())
}

#[test]
fn statistical_report_schema_contract_contains_required_fields() -> TestResult<()> {
    let metrics = vec![MetricSamples {
        key: MetricKey::Import,
        budget_ms: 1_000,
        samples: vec![
            Duration::from_millis(1),
            Duration::from_millis(2),
            Duration::from_millis(3),
            Duration::from_millis(4),
            Duration::from_millis(5),
            Duration::from_millis(6),
            Duration::from_millis(7),
        ],
    }];
    let report_path = write_report(
        &ReportSpec {
            name: "schema_regression/performance_schema_regression",
            generated_count: SAMPLE_COUNT,
            input_generation: "deterministic-schema-regression",
            classification: "schema-regression",
            release_evidence: false,
            storage_mode: "in-memory-sqlite",
            extra_dataset_json: ", \"schema_case\": \"required-statistical-fields\"",
        },
        &metrics,
    )?;
    let report = fs::read_to_string(report_path)?;
    assert_report_schema(&report, &[MetricKey::Import])?;
    Ok(())
}
