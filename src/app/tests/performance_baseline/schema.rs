use crate::report::MetricKey;
use crate::{invalid_data, TestResult};

pub(crate) fn assert_report_schema(report: &str, metrics: &[MetricKey]) -> TestResult<()> {
    for fragment in [
        "\"sample_count\": 7",
        "\"dataset\":",
        "\"generator\":",
        "\"generated_count\":",
        "\"environment\":",
        "\"hardware\":",
        "\"storage_mode\":",
        "\"release_evidence\":",
        "\"budget\":",
        "\"min\":",
        "\"median\":",
        "\"p95\":",
        "\"max\":",
        "\"pass\":",
        "\"status\":",
    ] {
        require_fragment(report, fragment)?;
    }

    let spaced_sample_count = report.matches("\"sample_count\": 7").count();
    let compact_sample_count = report.matches("\"sample_count\":7").count();
    if spaced_sample_count + compact_sample_count < metrics.len() + 2 {
        return Err(invalid_data(
            "schema report does not include top-level, dataset, and per-metric sample counts",
        )
        .into());
    }

    for metric in metrics {
        let key = metric.as_str();
        require_fragment(report, &format!("\"{key}\": {{\"sample_count\":7"))?;
    }
    Ok(())
}

fn require_fragment(report: &str, fragment: &str) -> TestResult<()> {
    if report.contains(fragment) {
        Ok(())
    } else {
        Err(invalid_data(format!("schema report missing fragment: {fragment}")).into())
    }
}
