use eterea_app::{AppServices, BookmarkQuery};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const GENERATED_COUNT: usize = 500;
const LARGE_GENERATED_COUNT: usize = 10_000;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app crate should have repo parent")
        .parent()
        .expect("src directory should have repo parent")
        .to_path_buf()
}

fn generated_archive(count: usize) -> String {
    let mut out = String::from("[");
    for i in 0..count {
        if i > 0 {
            out.push(',');
        }
        let handle = format!("author{}", i % 25);
        let tag = match i % 5 {
            0 => "rust",
            1 => "design",
            2 => "systems",
            3 => "performance",
            _ => "ai",
        };
        let media = if i % 7 == 0 {
            format!(
                r#","extended_media":[{{"media_url_https":"https://pbs.twimg.com/media/{i}.jpg"}}]"#
            )
        } else {
            String::new()
        };
        out.push_str(&format!(
            r#"{{"screen_name":"{handle}","name":"Author {handle}","full_text":"Generated bookmark {i} about #{tag} and local-first archives","tweeted_at":"2024-05-{day:02}T12:00:00Z","tweet_url":"https://x.com/{handle}/status/{i}"{media}}}"#,
            day = (i % 28) + 1,
        ));
    }
    out.push(']');
    out
}

fn assert_bounded(label: &str, elapsed: Duration, max: Duration) {
    assert!(
        elapsed <= max,
        "{label} took {:?}, expected <= {:?}",
        elapsed,
        max
    );
}

#[test]
fn performance_baseline_for_current_service_paths() {
    let archive = generated_archive(GENERATED_COUNT);
    let services = AppServices::open_memory().expect("in-memory services should open");

    let import_started = Instant::now();
    let imported = services
        .import_content("generated.json", &archive)
        .expect("generated archive should import");
    let import_elapsed = import_started.elapsed();
    assert_eq!(imported, GENERATED_COUNT);

    let list_started = Instant::now();
    let first_page = services
        .list_bookmarks(0, 48)
        .expect("first library page should load");
    let list_elapsed = list_started.elapsed();
    assert_eq!(first_page.items.len(), 48);

    let search_started = Instant::now();
    let search_page = services
        .query_bookmarks(&BookmarkQuery {
            query: Some("rust".to_string()),
            limit: 48,
            ..BookmarkQuery::default()
        })
        .expect("search query should load");
    let search_elapsed = search_started.elapsed();
    assert!(!search_page.items.is_empty());

    let stats_started = Instant::now();
    let stats = services.stats().expect("stats should load");
    let stats_elapsed = stats_started.elapsed();
    assert_eq!(stats.total_bookmarks, GENERATED_COUNT as i64);
    assert!(stats.unique_authors > 1);
    assert!(!stats.top_tags.is_empty());

    // Loose guardrails catch accidental pathological regressions without making
    // normal development machines fail on small timing variance. Release budgets
    // are stricter and live in docs/design-system.md.
    assert_bounded(
        "import 500 generated bookmarks",
        import_elapsed,
        Duration::from_secs(5),
    );
    assert_bounded("list first page", list_elapsed, Duration::from_secs(1));
    assert_bounded("search first page", search_elapsed, Duration::from_secs(1));
    assert_bounded("stats", stats_elapsed, Duration::from_secs(1));

    let report_dir = repo_root().join("target/eterea/perf");
    fs::create_dir_all(&report_dir).expect("performance report directory should be created");
    let report_path = report_dir.join("performance_baseline.json");
    fs::write(
        &report_path,
        format!(
            concat!(
                "{{\n",
                "  \"generated_count\": {count},\n",
                "  \"import_ms\": {import_ms},\n",
                "  \"list_ms\": {list_ms},\n",
                "  \"search_ms\": {search_ms},\n",
                "  \"stats_ms\": {stats_ms}\n",
                "}}\n"
            ),
            count = GENERATED_COUNT,
            import_ms = import_elapsed.as_millis(),
            list_ms = list_elapsed.as_millis(),
            search_ms = search_elapsed.as_millis(),
            stats_ms = stats_elapsed.as_millis(),
        ),
    )
    .expect("performance report should be written");

    eprintln!(
        "performance baseline: count={GENERATED_COUNT} import={}ms list={}ms search={}ms stats={}ms report={}",
        import_elapsed.as_millis(),
        list_elapsed.as_millis(),
        search_elapsed.as_millis(),
        stats_elapsed.as_millis(),
        report_path.display()
    );
}

#[test]
fn large_archive_budget_for_release_paths() {
    let archive = generated_archive(LARGE_GENERATED_COUNT);
    let services = AppServices::open_memory().expect("in-memory services should open");

    let import_started = Instant::now();
    let imported = services
        .import_content("generated-large.json", &archive)
        .expect("large generated archive should import");
    let import_elapsed = import_started.elapsed();
    assert_eq!(imported, LARGE_GENERATED_COUNT);

    let list_started = Instant::now();
    let first_page = services
        .list_bookmarks(0, 48)
        .expect("first library page should load");
    let list_elapsed = list_started.elapsed();
    assert_eq!(first_page.items.len(), 48);

    let search_started = Instant::now();
    let search_page = services
        .query_bookmarks(&BookmarkQuery {
            query: Some("rust".to_string()),
            limit: 48,
            ..BookmarkQuery::default()
        })
        .expect("search query should load");
    let search_elapsed = search_started.elapsed();
    assert!(!search_page.items.is_empty());

    let author_started = Instant::now();
    let authors = services.author_index().expect("author index should load");
    let author_elapsed = author_started.elapsed();
    assert!(!authors.is_empty());

    let topic_started = Instant::now();
    let topics = services.topic_index().expect("topic index should load");
    let topic_elapsed = topic_started.elapsed();
    assert!(!topics.is_empty());

    assert_bounded(
        "import 10k generated bookmarks",
        import_elapsed,
        Duration::from_secs(10),
    );
    assert_bounded(
        "warm 10k library page",
        list_elapsed,
        Duration::from_millis(100),
    );
    assert_bounded("10k search", search_elapsed, Duration::from_millis(150));
    assert_bounded(
        "10k author index",
        author_elapsed,
        Duration::from_millis(100),
    );
    assert_bounded("10k topic index", topic_elapsed, Duration::from_millis(100));

    let report_dir = repo_root().join("target/eterea/perf");
    fs::create_dir_all(&report_dir).expect("performance report directory should be created");
    let report_path = report_dir.join("performance_large_archive.json");
    fs::write(
        &report_path,
        format!(
            concat!(
                "{{\n",
                "  \"generated_count\": {count},\n",
                "  \"import_ms\": {import_ms},\n",
                "  \"list_ms\": {list_ms},\n",
                "  \"search_ms\": {search_ms},\n",
                "  \"author_index_ms\": {author_ms},\n",
                "  \"topic_index_ms\": {topic_ms},\n",
                "  \"budgets\": {{\n",
                "    \"import_ms\": 10000,\n",
                "    \"list_ms\": 100,\n",
                "    \"search_ms\": 150,\n",
                "    \"author_index_ms\": 100,\n",
                "    \"topic_index_ms\": 100\n",
                "  }}\n",
                "}}\n"
            ),
            count = LARGE_GENERATED_COUNT,
            import_ms = import_elapsed.as_millis(),
            list_ms = list_elapsed.as_millis(),
            search_ms = search_elapsed.as_millis(),
            author_ms = author_elapsed.as_millis(),
            topic_ms = topic_elapsed.as_millis(),
        ),
    )
    .expect("large performance report should be written");

    eprintln!(
        "large performance baseline: count={LARGE_GENERATED_COUNT} import={}ms list={}ms search={}ms author_index={}ms topic_index={}ms report={}",
        import_elapsed.as_millis(),
        list_elapsed.as_millis(),
        search_elapsed.as_millis(),
        author_elapsed.as_millis(),
        topic_elapsed.as_millis(),
        report_path.display()
    );
}
