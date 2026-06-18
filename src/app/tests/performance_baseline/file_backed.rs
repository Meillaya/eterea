use eterea_app::{AppServices, BookmarkQuery};
use eterea_core::Database;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

use crate::data::generated_archive;
use crate::measure::{archive_metrics, run_file_backed_archive_paths};
use crate::report::{
    assert_bounded, duration_stats, json_escape, write_report, MetricKey, ReportSpec,
};
use crate::{invalid_data, TestResult, SAMPLE_COUNT};

const FILE_BACKED_GENERATED_COUNT: usize = 750;

#[test]
fn file_backed_sqlite_wal_report_for_release_path() -> TestResult<()> {
    let temp_dir = TempDir::new()?;
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut wal_checks = Vec::with_capacity(SAMPLE_COUNT);

    for sample_index in 0..SAMPLE_COUNT {
        let db_path = temp_dir.path().join(format!("sample-{sample_index}.db"));
        let (sample, wal_artifact_present) = run_file_backed_archive_paths(
            FILE_BACKED_GENERATED_COUNT,
            "generated-file-backed.json",
            true,
            &db_path,
        )?;
        samples.push(sample);
        wal_checks.push(sqlite_report_check(&db_path, wal_artifact_present)?);
    }

    let metrics = archive_metrics(
        &samples,
        &[
            (MetricKey::ColdStart, 1_000),
            (MetricKey::Open, 1_000),
            (MetricKey::Import, 7_500),
            (MetricKey::List, 250),
            (MetricKey::Search, 350),
            (MetricKey::AuthorIndex, 250),
            (MetricKey::TopicIndex, 250),
            (MetricKey::MediaHydration, 350),
            (MetricKey::Stats, 250),
        ],
    )?;
    for metric in &metrics {
        let stats = duration_stats(&metric.samples)?;
        assert_bounded(
            metric.key.as_str(),
            Duration::from_millis(u64::try_from(stats.p95)?),
            Duration::from_millis(metric.budget_ms),
        );
    }

    let report_path = write_report(
        &ReportSpec {
            name: "performance_file_backed",
            generated_count: FILE_BACKED_GENERATED_COUNT,
            input_generation: "deterministic-json-string-file-backed-tempdb",
            classification: "release-evidence",
            release_evidence: true,
            storage_mode: "file-backed-sqlite-wal",
            extra_dataset_json: &format!(
                concat!(
                    ", \"cold_start_operation\": \"AppServices::open(file-backed db)\", ",
                    "\"wal\": {{\"journal_mode\": \"{}\", \"wal_artifact_present\": {}, ",
                    "\"samples_checked\": {}}}, \"sqlite_pragmas\": {}"
                ),
                json_escape(&first_pragma_value(&wal_checks, "journal_mode")?),
                wal_checks.iter().all(|check| check.wal_artifact_present),
                wal_checks.len(),
                sqlite_pragmas_json(&wal_checks)?
            ),
        },
        &metrics,
    )?;

    let report = std::fs::read_to_string(&report_path)?;
    for fragment in [
        "\"storage_mode\": \"file-backed-sqlite-wal\"",
        "\"release_evidence\": true",
        "\"classification\": \"release-evidence\"",
        "\"journal_mode\": \"wal\"",
        "\"cold_start_ms\":",
        "\"open_ms\":",
        "\"media_hydration_ms\":",
        "\"sqlite_pragmas\":",
        "\"synchronous\": \"1\"",
        "\"cache_size\": \"-64000\"",
        "\"temp_store\": \"2\"",
        "\"mmap_size\": \"268435456\"",
        "\"foreign_keys\": \"1\"",
        "\"wal_artifact_present\": true",
        "\"hardware\":",
        "\"budget\":",
        "\"pass\": true",
    ] {
        if !report.contains(fragment) {
            return Err(
                invalid_data(format!("file-backed report missing fragment: {fragment}")).into(),
            );
        }
    }

    eprintln!(
        "file-backed SQLite WAL performance report: count={FILE_BACKED_GENERATED_COUNT} sample_count={SAMPLE_COUNT} report={}",
        report_path.display()
    );
    Ok(())
}

#[test]
fn file_backed_duplicate_and_failure_paths_do_not_mutate_archive() -> TestResult<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("non-mutation.db");
    let services = AppServices::open(&db_path)?;
    let archive = generated_archive(24);

    let imported = services.import_content("generated-file-backed.json", &archive)?;
    assert_eq!(imported, 24);
    let total_after_first_import = services.list_bookmarks(0, 100)?.total;
    assert_eq!(total_after_first_import, 24);

    let duplicate_imported = services.import_content("generated-file-backed.json", &archive)?;
    assert_eq!(duplicate_imported, 0);
    assert_eq!(
        services.list_bookmarks(0, 100)?.total,
        total_after_first_import
    );

    assert!(services
        .import_content("broken.json", "{not valid json")
        .is_err());
    assert_eq!(
        services.list_bookmarks(0, 100)?.total,
        total_after_first_import
    );

    let filtered = services.query_bookmarks(&BookmarkQuery {
        query: Some("rust".to_string()),
        has_media: Some(false),
        limit: 100,
        ..BookmarkQuery::default()
    })?;
    assert!(filtered.total > 0);
    Ok(())
}

struct SqliteReportCheck {
    pragmas: Vec<(String, String)>,
    wal_artifact_present: bool,
}

fn sqlite_report_check(
    db_path: &Path,
    wal_artifact_present: bool,
) -> TestResult<SqliteReportCheck> {
    if !wal_artifact_present {
        return Err(invalid_data("expected SQLite WAL artifact while connection is active").into());
    }
    let db = Database::open(db_path)?;
    let pragmas = db.observed_pragma_settings()?;
    validate_pragmas(&pragmas)?;
    Ok(SqliteReportCheck {
        pragmas,
        wal_artifact_present,
    })
}

fn validate_pragmas(pragmas: &[(String, String)]) -> TestResult<()> {
    for (name, expected) in [
        ("journal_mode", "wal"),
        ("synchronous", "1"),
        ("cache_size", "-64000"),
        ("temp_store", "2"),
        ("mmap_size", "268435456"),
        ("foreign_keys", "1"),
    ] {
        let observed = pragma_value(pragmas, name)?;
        if observed != expected {
            return Err(invalid_data(format!(
                "unexpected SQLite PRAGMA {name}: observed={observed} expected={expected}"
            ))
            .into());
        }
    }
    Ok(())
}

fn sqlite_pragmas_json(checks: &[SqliteReportCheck]) -> TestResult<String> {
    let first = checks
        .first()
        .ok_or_else(|| invalid_data("SQLite PRAGMA report requires at least one sample"))?;
    let all_samples_match = checks.iter().all(|check| check.pragmas == first.pragmas);
    let entries = first
        .pragmas
        .iter()
        .map(|(name, value)| format!("\"{}\": \"{}\"", json_escape(name), json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "{{\"samples_checked\": {}, \"all_samples_match\": {}, {entries}}}",
        checks.len(),
        all_samples_match
    ))
}

fn first_pragma_value(checks: &[SqliteReportCheck], name: &str) -> TestResult<String> {
    let first = checks
        .first()
        .ok_or_else(|| invalid_data("SQLite PRAGMA report requires at least one sample"))?;
    Ok(pragma_value(&first.pragmas, name)?.to_string())
}

fn pragma_value<'a>(pragmas: &'a [(String, String)], name: &str) -> TestResult<&'a str> {
    pragmas
        .iter()
        .find_map(|(current_name, value)| (current_name == name).then_some(value.as_str()))
        .ok_or_else(|| invalid_data(format!("missing observed SQLite PRAGMA {name}")).into())
}
