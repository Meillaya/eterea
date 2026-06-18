use eterea_app::{AppServices, BookmarkQuery};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::data::{generated_archive, generated_author_heavy_archive};
use crate::report::{MetricKey, MetricSamples};
use crate::{invalid_data, TestResult};

#[derive(Clone, Copy)]
pub(crate) struct ArchiveSample {
    cold_start: Option<Duration>,
    import: Duration,
    list: Duration,
    search: Duration,
    author: Duration,
    topic: Duration,
    media_hydration: Duration,
    stats: Option<Duration>,
}

pub(crate) fn run_archive_paths(
    count: usize,
    filename: &str,
    include_stats: bool,
) -> TestResult<ArchiveSample> {
    let archive = generated_archive(count);
    let services = AppServices::open_memory()?;
    run_archive_paths_with_services(None, count, filename, include_stats, &archive, &services)
}

pub(crate) fn run_file_backed_archive_paths(
    count: usize,
    filename: &str,
    include_stats: bool,
    db_path: &Path,
) -> TestResult<(ArchiveSample, bool)> {
    let archive = generated_archive(count);
    let open_started = Instant::now();
    let services = AppServices::open(db_path)?;
    let cold_start = open_started.elapsed();
    let sample = run_archive_paths_with_services(
        Some(cold_start),
        count,
        filename,
        include_stats,
        &archive,
        &services,
    )?;
    Ok((sample, wal_path(db_path).exists()))
}

fn run_archive_paths_with_services(
    cold_start: Option<Duration>,
    count: usize,
    filename: &str,
    include_stats: bool,
    archive: &str,
    services: &AppServices,
) -> TestResult<ArchiveSample> {
    let import_started = Instant::now();
    let imported = services.import_content(filename, archive)?;
    let import = import_started.elapsed();
    assert_eq!(imported, count);

    let list_started = Instant::now();
    let first_page = services.list_bookmarks(0, 48)?;
    let list = list_started.elapsed();
    assert_eq!(first_page.items.len(), count.min(48));

    let search_started = Instant::now();
    let search_page = services.query_bookmarks(&BookmarkQuery {
        query: Some("rust".to_string()),
        limit: 48,
        ..BookmarkQuery::default()
    })?;
    let search = search_started.elapsed();
    assert!(!search_page.items.is_empty());

    let media_started = Instant::now();
    let media_page = services.query_bookmarks(&BookmarkQuery {
        has_media: Some(true),
        limit: 48,
        ..BookmarkQuery::default()
    })?;
    let media_hydration = media_started.elapsed();
    assert!(!media_page.items.is_empty());
    assert!(media_page
        .items
        .iter()
        .all(|bookmark| !bookmark.media.is_empty()));

    let author_started = Instant::now();
    let authors = services.author_index()?;
    let author = author_started.elapsed();
    assert!(!authors.is_empty());

    let topic_started = Instant::now();
    let topics = services.topic_index()?;
    let topic = topic_started.elapsed();
    assert!(!topics.is_empty());

    let stats = if include_stats {
        Some(measure_stats(services, count)?)
    } else {
        None
    };

    Ok(ArchiveSample {
        cold_start,
        import,
        list,
        search,
        author,
        topic,
        media_hydration,
        stats,
    })
}

pub(crate) fn archive_metrics(
    samples: &[ArchiveSample],
    specs: &[(MetricKey, u64)],
) -> TestResult<Vec<MetricSamples>> {
    let mut metrics = Vec::with_capacity(specs.len());
    for (key, budget_ms) in specs {
        let mut metric_samples = Vec::with_capacity(samples.len());
        for sample in samples {
            metric_samples.push(duration_for_key(sample, *key)?);
        }
        metrics.push(MetricSamples {
            key: *key,
            budget_ms: *budget_ms,
            samples: metric_samples,
        });
    }
    Ok(metrics)
}

pub(crate) fn author_directory_metric(count: usize, budget_ms: u64) -> TestResult<MetricSamples> {
    let mut samples = Vec::new();
    for _ in 0..crate::SAMPLE_COUNT {
        let archive = generated_author_heavy_archive(count);
        let services = AppServices::open_memory()?;
        let imported = services.import_content("generated-author-heavy.json", &archive)?;
        assert_eq!(imported, count);
        let author_started = Instant::now();
        let authors = services.author_index()?;
        samples.push(author_started.elapsed());
        assert_eq!(authors.len(), count);
    }
    Ok(MetricSamples {
        key: MetricKey::AuthorIndex,
        budget_ms,
        samples,
    })
}

fn measure_stats(services: &AppServices, count: usize) -> TestResult<Duration> {
    let stats_started = Instant::now();
    let stats = services.stats()?;
    let elapsed = stats_started.elapsed();
    assert_eq!(stats.total_bookmarks, i64::try_from(count)?);
    assert!(stats.unique_authors > 1);
    assert!(!stats.top_tags.is_empty());
    Ok(elapsed)
}

fn duration_for_key(sample: &ArchiveSample, key: MetricKey) -> TestResult<Duration> {
    match key {
        MetricKey::ColdStart => sample.cold_start.ok_or_else(|| {
            invalid_data("cold start metric requested without file-backed open timing").into()
        }),
        MetricKey::Open => sample
            .cold_start
            .ok_or_else(|| invalid_data("open metric requested without file-backed timing").into()),
        MetricKey::Import => Ok(sample.import),
        MetricKey::List => Ok(sample.list),
        MetricKey::Search => Ok(sample.search),
        MetricKey::AuthorIndex => Ok(sample.author),
        MetricKey::TopicIndex => Ok(sample.topic),
        MetricKey::MediaHydration => Ok(sample.media_hydration),
        MetricKey::Stats => sample
            .stats
            .ok_or_else(|| invalid_data("stats metric requested without stats sampling").into()),
    }
}

fn wal_path(db_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}-wal", db_path.display()))
}
