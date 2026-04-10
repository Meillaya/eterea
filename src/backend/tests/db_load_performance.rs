use chrono::{TimeZone, Utc};
use eterea_core::models::BookmarkBuilder;
use eterea_core::Database;

fn sample_bookmark(
    tweet_id: &str,
    handle: &str,
    date: chrono::DateTime<Utc>,
    tags: &[&str],
    media_urls: &[&str],
) -> eterea_core::Bookmark {
    let mut builder = BookmarkBuilder::new()
        .tweet_url(format!("https://x.com/{handle}/status/{tweet_id}"))
        .content(format!("Bookmark {tweet_id} from @{handle}"))
        .tweeted_at(date)
        .author_handle(handle)
        .author_name(format!("{handle} name"));

    for tag in tags {
        builder = builder.add_tag(*tag);
    }

    for media_url in media_urls {
        builder = builder.add_media(*media_url);
    }

    builder.build().expect("sample bookmark should build")
}

#[test]
fn paginated_reads_preserve_order_and_hydrate_related_data() {
    let db = Database::open_memory().expect("in-memory db should open");
    let first = sample_bookmark(
        "1",
        "alice",
        Utc.with_ymd_and_hms(2024, 5, 1, 8, 0, 0).unwrap(),
        &["rust", "perf"],
        &["https://pbs.twimg.com/media/a.jpg"],
    );
    let second = sample_bookmark(
        "2",
        "bob",
        Utc.with_ymd_and_hms(2024, 5, 2, 8, 0, 0).unwrap(),
        &["svelte"],
        &[],
    );
    let third = sample_bookmark(
        "3",
        "carol",
        Utc.with_ymd_and_hms(2024, 5, 3, 8, 0, 0).unwrap(),
        &["rust"],
        &[
            "https://pbs.twimg.com/media/c.jpg",
            "https://video.twimg.com/ext_tw_video/c.mp4",
        ],
    );

    db.insert_bookmarks(&[first.clone(), second.clone(), third.clone()])
        .expect("bookmarks should insert");

    let page = db
        .get_bookmarks(0, 2)
        .expect("paginated bookmarks should load");

    assert_eq!(page.len(), 2);
    assert_eq!(page[0].tweet_url, third.tweet_url);
    assert_eq!(page[1].tweet_url, second.tweet_url);
    assert_eq!(page[0].tags, vec!["rust"]);
    assert_eq!(page[0].media.len(), 2);
    assert_eq!(page[1].tags, vec!["svelte"]);
    assert!(page[1].media.is_empty());
}

#[test]
fn stats_return_zeroed_values_for_an_empty_archive() {
    let db = Database::open_memory().expect("in-memory db should open");

    let stats = db.get_stats().expect("stats should load for empty archive");

    assert_eq!(stats.total_bookmarks, 0);
    assert_eq!(stats.unique_authors, 0);
    assert_eq!(stats.unique_tags, 0);
    assert_eq!(stats.favorite_bookmarks, 0);
    assert!(stats.earliest_date.is_none());
    assert!(stats.latest_date.is_none());
    assert!(stats.top_tags.is_empty());
}

#[test]
fn stats_cover_counts_timeline_and_top_tags_for_populated_archives() {
    let db = Database::open_memory().expect("in-memory db should open");
    let first = sample_bookmark(
        "11",
        "alice",
        Utc.with_ymd_and_hms(2024, 4, 30, 22, 0, 0).unwrap(),
        &["rust", "database"],
        &[],
    );
    let second = sample_bookmark(
        "12",
        "alice",
        Utc.with_ymd_and_hms(2024, 5, 1, 22, 0, 0).unwrap(),
        &["rust"],
        &["https://pbs.twimg.com/media/rust.jpg"],
    );
    let third = sample_bookmark(
        "13",
        "bob",
        Utc.with_ymd_and_hms(2024, 5, 2, 22, 0, 0).unwrap(),
        &["svelte"],
        &[],
    );

    db.insert_bookmarks(&[first.clone(), second.clone(), third.clone()])
        .expect("bookmarks should insert");
    db.set_favorite(&second.id, true)
        .expect("favorite flag should update");

    let stats = db
        .get_stats()
        .expect("stats should load for populated archive");

    assert_eq!(stats.total_bookmarks, 3);
    assert_eq!(stats.unique_authors, 2);
    assert_eq!(stats.unique_tags, 3);
    assert_eq!(stats.favorite_bookmarks, 1);
    assert_eq!(stats.earliest_date, Some(first.tweeted_at));
    assert_eq!(stats.latest_date, Some(third.tweeted_at));
    assert_eq!(stats.top_tags.first(), Some(&("rust".to_string(), 2)));
}

#[test]
fn favorites_and_tag_filters_stay_correct_after_primary_reads() {
    let db = Database::open_memory().expect("in-memory db should open");
    let first = sample_bookmark(
        "21",
        "alice",
        Utc.with_ymd_and_hms(2024, 5, 1, 9, 0, 0).unwrap(),
        &["rust"],
        &[],
    );
    let second = sample_bookmark(
        "22",
        "bob",
        Utc.with_ymd_and_hms(2024, 5, 2, 9, 0, 0).unwrap(),
        &["rust", "tauri"],
        &["https://pbs.twimg.com/media/tauri.jpg"],
    );
    let third = sample_bookmark(
        "23",
        "carol",
        Utc.with_ymd_and_hms(2024, 5, 3, 9, 0, 0).unwrap(),
        &["svelte"],
        &[],
    );

    db.insert_bookmarks(&[first.clone(), second.clone(), third.clone()])
        .expect("bookmarks should insert");
    db.set_favorite(&second.id, true)
        .expect("favorite flag should update");

    let _ = db
        .get_bookmarks(0, 2)
        .expect("primary page should load successfully");

    let favorites = db
        .get_favorites(0, 10)
        .expect("favorites should load successfully");
    let rust_bookmarks = db
        .get_bookmarks_by_tag("rust", 0, 10)
        .expect("tag-filtered bookmarks should load successfully");

    assert_eq!(favorites.len(), 1);
    assert_eq!(favorites[0].id, second.id);
    assert_eq!(favorites[0].tags, vec!["rust", "tauri"]);
    assert_eq!(rust_bookmarks.len(), 2);
    assert_eq!(rust_bookmarks[0].id, second.id);
    assert_eq!(rust_bookmarks[1].id, first.id);
}
