use eterea_app::{AppServices, BookmarkQuery};
use std::collections::{BTreeMap, BTreeSet};
use tempfile::NamedTempFile;

const SAMPLE_JSON: &str = include_str!("../../../fixtures/import/bookmarks.json");
const SAMPLE_NEW_CSV: &str = include_str!("../../../fixtures/import/new_bookmarks.csv");
const SAMPLE_LEGACY_CSV: &str = include_str!("../../../fixtures/import/legacy_bookmarks.csv");

fn imported_services(filename: &str, content: &str) -> AppServices {
    let services = AppServices::open_memory().expect("in-memory services should open");
    let imported = services
        .import_content(filename, content)
        .expect("fixture import should succeed");
    assert!(imported > 0, "fixture should import bookmarks");
    services
}

fn handles_for(page: &eterea_app::BookmarkPage) -> BTreeSet<String> {
    page.items
        .iter()
        .map(|bookmark| bookmark.author_handle.clone())
        .collect()
}

#[test]
fn imports_supported_export_formats_into_queryable_archive() {
    for (filename, content) in [
        ("bookmarks.json", SAMPLE_JSON),
        ("new-bookmarks.csv", SAMPLE_NEW_CSV),
        ("legacy-bookmarks.csv", SAMPLE_LEGACY_CSV),
    ] {
        let services = imported_services(filename, content);
        let page = services
            .list_bookmarks(0, 10)
            .expect("imported library page should load");
        assert!(
            !page.items.is_empty(),
            "{filename} should yield visible bookmarks"
        );
        assert!(page.total >= page.items.len() as i64);

        let stats = services.stats().expect("stats should load after import");
        assert!(stats.total_bookmarks >= page.total);
        assert!(
            stats.unique_authors > 0,
            "{filename} should include authors"
        );
    }
}

#[test]
fn import_preview_is_dry_run_and_import_failures_do_not_mutate_archive() {
    let services = AppServices::open_memory().expect("in-memory services should open");

    let preview = services
        .preview_import_content("bookmarks.json", SAMPLE_JSON)
        .expect("json preview should parse");
    assert!(preview.bookmark_count > 0);
    assert!(!preview.sample.is_empty());
    assert_eq!(
        services
            .list_bookmarks(0, 10)
            .expect("library should still load")
            .total,
        0,
        "preview must not mutate the archive"
    );

    let imported = services
        .import_content_with_preview("bookmarks.json", SAMPLE_JSON)
        .expect("preview-backed import should succeed");
    assert_eq!(imported.preview.bookmark_count, imported.imported_count);

    let total_after_import = services
        .list_bookmarks(0, 10)
        .expect("library should load after import")
        .total;
    assert_eq!(total_after_import, imported.imported_count as i64);

    assert!(
        services
            .import_content("broken.json", "{not valid json")
            .is_err(),
        "broken imports should fail before writes"
    );
    assert_eq!(
        services
            .list_bookmarks(0, 10)
            .expect("library should still load after failed import")
            .total,
        total_after_import,
        "failed imports must leave existing archive intact"
    );
}

#[test]
fn query_favorite_delete_and_persistence_guardrails_hold() {
    let file = NamedTempFile::new().expect("temp db file should exist");
    let path = file.path().to_path_buf();
    drop(file);

    let target_id = {
        let services = AppServices::open(&path).expect("disk-backed services should open");
        services
            .import_content("bookmarks.json", SAMPLE_JSON)
            .expect("json import should succeed");

        let first_page = services
            .query_bookmarks(&BookmarkQuery {
                query: Some("rust".to_string()),
                limit: 20,
                ..BookmarkQuery::default()
            })
            .expect("query should succeed");
        assert!(!first_page.items.is_empty(), "expected rust query results");

        let target = first_page.items[0].clone();
        let toggled = services
            .toggle_favorite(&target.id)
            .expect("favorite toggle should succeed");
        assert_eq!(
            toggled, !target.is_favorite,
            "toggle should invert favorite state"
        );

        let favorites = services
            .query_bookmarks(&BookmarkQuery {
                favorites_only: true,
                limit: 200,
                ..BookmarkQuery::default()
            })
            .expect("favorites query should succeed");
        assert!(
            favorites
                .items
                .iter()
                .any(|bookmark| bookmark.id == target.id),
            "toggled bookmark should be visible in favorites"
        );

        target.id
    };

    let reopened = AppServices::open(&path).expect("reopened services should open");
    let favorites_after_reopen = reopened
        .query_bookmarks(&BookmarkQuery {
            favorites_only: true,
            limit: 200,
            ..BookmarkQuery::default()
        })
        .expect("favorites should load after reopen");
    assert!(
        favorites_after_reopen
            .items
            .iter()
            .any(|bookmark| bookmark.id == target_id),
        "favorite state should persist across reopen"
    );

    assert!(
        reopened
            .delete_bookmark(&target_id)
            .expect("delete should succeed"),
        "delete should report a removed bookmark"
    );
    let after_delete = reopened
        .list_bookmarks(0, 500)
        .expect("library should load after delete");
    assert!(
        after_delete
            .items
            .iter()
            .all(|bookmark| bookmark.id != target_id),
        "deleted bookmark should not appear after delete"
    );
}

#[test]
fn directory_and_detail_service_apis_are_data_backed() {
    let services = imported_services("bookmarks.json", SAMPLE_JSON);

    let authors = services.author_index().expect("author index should load");
    assert!(!authors.is_empty(), "expected author summaries");
    assert!(authors.iter().all(|author| author.bookmark_count > 0));

    let topics = services.topic_index().expect("topic index should load");
    assert!(!topics.is_empty(), "expected topic summaries");
    assert!(topics.iter().all(|topic| topic.bookmark_count > 0));

    let author_page = services
        .bookmarks_by_author(&authors[0].handle, 0, 20)
        .expect("author bookmark page should load");
    assert!(!author_page.items.is_empty());
    assert!(author_page
        .items
        .iter()
        .all(|bookmark| bookmark.author_handle == authors[0].handle));

    let topic_page = services
        .bookmarks_by_tag(&topics[0].tag, 0, 20)
        .expect("topic bookmark page should load");
    assert!(!topic_page.items.is_empty());
    assert!(topic_page
        .items
        .iter()
        .all(|bookmark| bookmark.tags.contains(&topics[0].tag)));

    let detail = services
        .bookmark_detail(&author_page.items[0].id)
        .expect("detail lookup should succeed")
        .expect("bookmark should exist");
    assert_eq!(detail.id, author_page.items[0].id);
}
#[test]
fn media_hydrates_across_list_search_filter_and_reopen() {
    let file = NamedTempFile::new().expect("temp db file should exist");
    let path = file.path().to_path_buf();
    drop(file);

    {
        let services = AppServices::open(&path).expect("disk-backed services should open");
        services
            .import_content("bookmarks.json", SAMPLE_JSON)
            .expect("json import should succeed");

        let media_page = services
            .query_bookmarks(&BookmarkQuery {
                has_media: Some(true),
                limit: 200,
                ..BookmarkQuery::default()
            })
            .expect("has-media query should load");
        assert!(
            media_page.total > 0,
            "fixture should include media bookmarks"
        );
        assert!(media_page
            .items
            .iter()
            .all(|bookmark| !bookmark.media.is_empty()));

        let target = media_page.items[0].clone();
        let search_page = services
            .query_bookmarks(&BookmarkQuery {
                query: Some(target.author_handle.clone()),
                has_media: Some(true),
                limit: 200,
                ..BookmarkQuery::default()
            })
            .expect("search with media filter should load");
        assert!(search_page
            .items
            .iter()
            .any(|bookmark| bookmark.id == target.id && !bookmark.media.is_empty()));
    }

    let reopened = AppServices::open(&path).expect("reopened services should open");
    let reopened_media = reopened
        .query_bookmarks(&BookmarkQuery {
            has_media: Some(true),
            limit: 200,
            ..BookmarkQuery::default()
        })
        .expect("reopened has-media query should load");
    assert!(reopened_media.total > 0);
    assert!(reopened_media
        .items
        .iter()
        .all(|bookmark| !bookmark.media.is_empty()));
}

#[test]
fn local_fixture_parity_locks_import_search_filters_stats_and_reopen() {
    let file = NamedTempFile::new().expect("temp db file should exist");
    let path = file.path().to_path_buf();
    drop(file);

    let bea_id = {
        let services = AppServices::open(&path).expect("disk-backed services should open");
        let imported = services
            .import_content("bookmarks.json", SAMPLE_JSON)
            .expect("json import should succeed");
        assert_eq!(imported, 3, "fixture import count is the parity baseline");

        let duplicate_import = services
            .import_content("bookmarks.json", SAMPLE_JSON)
            .expect("duplicate import should be handled without mutation");
        assert_eq!(
            duplicate_import, 0,
            "duplicate import should skip already persisted tweet URLs"
        );

        let all = services
            .list_bookmarks(0, 10)
            .expect("all bookmarks should load after import");
        assert_eq!(all.total, 3);
        assert_eq!(
            handles_for(&all),
            ["ada_dev", "bea_design", "cy_ops"]
                .into_iter()
                .map(String::from)
                .collect(),
            "fixture authors must remain stable for UI route guardrails"
        );

        let stats = services.stats().expect("stats should load after import");
        assert_eq!(stats.total_bookmarks, 3);
        assert_eq!(stats.unique_authors, 3);
        assert_eq!(stats.unique_tags, 4);
        assert_eq!(stats.favorite_bookmarks, 0);

        let top_tags: BTreeMap<_, _> = stats.top_tags.into_iter().collect();
        assert_eq!(top_tags.get("rust"), Some(&1));
        assert_eq!(top_tags.get("architecture"), Some(&1));
        assert_eq!(top_tags.get("design"), Some(&1));
        assert_eq!(top_tags.get("ops"), Some(&1));

        let rust = services
            .query_bookmarks(&BookmarkQuery {
                query: Some("rust".to_string()),
                limit: 10,
                ..BookmarkQuery::default()
            })
            .expect("rust search should load");
        assert_eq!(handles_for(&rust), BTreeSet::from(["ada_dev".to_string()]));

        let design = services
            .query_bookmarks(&BookmarkQuery {
                tag: Some("design".to_string()),
                limit: 10,
                ..BookmarkQuery::default()
            })
            .expect("tag filter should load");
        assert_eq!(
            handles_for(&design),
            BTreeSet::from(["bea_design".to_string()])
        );

        let bea_day = services
            .query_bookmarks(&BookmarkQuery {
                from_date: Some("2026-05-02T00:00:00Z".to_string()),
                to_date: Some("2026-05-02T23:59:59Z".to_string()),
                limit: 10,
                ..BookmarkQuery::default()
            })
            .expect("date filter should load");
        assert_eq!(
            handles_for(&bea_day),
            BTreeSet::from(["bea_design".to_string()])
        );

        let with_media = services
            .query_bookmarks(&BookmarkQuery {
                has_media: Some(true),
                limit: 10,
                ..BookmarkQuery::default()
            })
            .expect("media filter should load");
        assert_eq!(
            handles_for(&with_media),
            ["ada_dev", "cy_ops"]
                .into_iter()
                .map(String::from)
                .collect()
        );

        let bea = services
            .query_bookmarks(&BookmarkQuery {
                author: Some("bea_design".to_string()),
                limit: 10,
                ..BookmarkQuery::default()
            })
            .expect("author filter should load")
            .items
            .into_iter()
            .next()
            .expect("bea fixture bookmark should exist");
        services
            .toggle_favorite(&bea.id)
            .expect("favorite toggle should succeed");

        let favorites = services
            .query_bookmarks(&BookmarkQuery {
                favorites_only: true,
                limit: 10,
                ..BookmarkQuery::default()
            })
            .expect("favorites filter should load");
        assert_eq!(
            handles_for(&favorites),
            BTreeSet::from(["bea_design".to_string()])
        );

        bea.id
    };

    let reopened = AppServices::open(&path).expect("reopened services should open");
    let reopened_stats = reopened.stats().expect("stats should reload from disk");
    assert_eq!(reopened_stats.total_bookmarks, 3);
    assert_eq!(
        reopened_stats.favorite_bookmarks, 1,
        "favorite mutation must persist across app/service restart"
    );

    let reopened_favorites = reopened
        .query_bookmarks(&BookmarkQuery {
            favorites_only: true,
            limit: 10,
            ..BookmarkQuery::default()
        })
        .expect("reopened favorites should load");
    assert!(reopened_favorites
        .items
        .iter()
        .any(|bookmark| bookmark.id == bea_id && bookmark.is_favorite));
}
