use super::actions::{
    format_timestamp, load_bookmark_remote_images, load_more, reload_library,
    set_remote_images_enabled, toggle_expanded_bookmark,
};
use super::components::{MediaGallery, MediaGalleryMode};
use super::route::ScreenRoute;
use super::state::{LayoutMode, LibraryState, Services};
use chrono::{Timelike, Utc};
use dioxus::prelude::*;
use eterea_core::Bookmark;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn archive_feed_or_empty(
    mut state: Signal<LibraryState>,
    services: Services,
) -> Element {
    let payload = feed_payload(&state);
    if let Some(error) = payload.error_message.clone() {
        return rsx! {
            div { class: "error-card", strong { "Couldn’t load the archive." } p { "{error}" } }
        };
    }

    if payload.bookmarks.is_empty() {
        return rsx! {
            div {
                class: "empty-card",
                p { class: "eyebrow", "Nothing here yet" }
                h4 { "The archive is quiet." }
                p { class: "muted-copy", "Import a bookmark export to fill the library back in." }
                button {
                    class: "accent-button",
                    onclick: move |_| state.write().import.open = true,
                    "Import bookmarks"
                }
            }
        };
    }

    let load_more_services = services.clone();
    rsx! {
        div { class: "library-view {payload.layout.class_name()}",
            {view_switcher(state)}
            {match payload.layout {
                LayoutMode::Table => table_view(state, services.clone(), payload.clone()),
                LayoutMode::Tree => tree_view(state, services.clone(), payload.clone()),
                LayoutMode::Dashboard => dashboard_view(payload.clone()),
                LayoutMode::Graph => graph_view(payload.clone()),
                LayoutMode::Calendar => calendar_view(payload.clone()),
            }}
        }
        if payload.has_more && matches!(payload.layout, LayoutMode::Table | LayoutMode::Tree) {
            button {
                class: "ghost-button wide load-more",
                onclick: move |_| load_more(&load_more_services, &mut state),
                "Load more"
            }
        }
    }
}

#[derive(Clone)]
struct FeedPayload {
    bookmarks: Vec<Bookmark>,
    layout: LayoutMode,
    has_more: bool,
    error_message: Option<String>,
    expanded_bookmark_id: Option<String>,
    selected_ids: BTreeSet<String>,
    total: i64,
    top_tags: Vec<(String, i64)>,
    unique_authors: i64,
    favorite_bookmarks: i64,
    remote_images_enabled: bool,
    loaded_media_bookmark_ids: BTreeSet<String>,
}

#[derive(Clone)]
struct FeedRowContext {
    expanded_bookmark_id: Option<String>,
    selected_ids: BTreeSet<String>,
    remote_images_enabled: bool,
    loaded_media_bookmark_ids: BTreeSet<String>,
}

impl From<&FeedPayload> for FeedRowContext {
    fn from(payload: &FeedPayload) -> Self {
        Self {
            expanded_bookmark_id: payload.expanded_bookmark_id.clone(),
            selected_ids: payload.selected_ids.clone(),
            remote_images_enabled: payload.remote_images_enabled,
            loaded_media_bookmark_ids: payload.loaded_media_bookmark_ids.clone(),
        }
    }
}

fn feed_payload(state: &Signal<LibraryState>) -> FeedPayload {
    let snapshot = state.read();
    let stats = snapshot.stats.clone();
    FeedPayload {
        bookmarks: snapshot.bookmarks.clone(),
        layout: snapshot.layout.clone(),
        has_more: snapshot.has_more,
        error_message: snapshot.error.clone(),
        expanded_bookmark_id: snapshot.expanded_bookmark_id.clone(),
        selected_ids: snapshot.shell.selected_bookmark_ids.clone(),
        total: snapshot.total,
        top_tags: snapshot.top_tags.clone(),
        unique_authors: stats.as_ref().map_or(0, |stats| stats.unique_authors),
        favorite_bookmarks: stats.as_ref().map_or(0, |stats| stats.favorite_bookmarks),
        remote_images_enabled: snapshot.remote_images_enabled,
        loaded_media_bookmark_ids: snapshot.loaded_media_bookmark_ids.clone(),
    }
}

fn view_switcher(mut state: Signal<LibraryState>) -> Element {
    let current = state.read().layout.clone();
    rsx! {
        div { class: "view-switcher",
            for candidate in LayoutMode::ALL {
                button {
                    class: if current == candidate { "view-tab active" } else { "view-tab" },
                    title: "{candidate.description()}",
                    onclick: move |_| state.write().layout = candidate.clone(),
                    "{candidate.as_str()}"
                }
            }
        }
    }
}

fn table_view(state: Signal<LibraryState>, services: Services, payload: FeedPayload) -> Element {
    let row_context = FeedRowContext::from(&payload);
    let bookmarks = payload.bookmarks;
    rsx! {
        div { class: "terminal-table",
            div { class: "table-head",
                span {}
                span { "idx" }
                span { "author" }
                span { "when" }
                span { "content" }
                span { "tags" }
                span { "★" }
            }
            div { class: "table-body",
                for (index, bookmark) in bookmarks.into_iter().enumerate() {
                    {table_row(state, services.clone(), index, bookmark, &row_context)}
                }
            }
        }
    }
}

fn table_row(
    mut state: Signal<LibraryState>,
    services: Services,
    index: usize,
    bookmark: Bookmark,
    context: &FeedRowContext,
) -> Element {
    let id = bookmark.id.clone();
    let detail_id = bookmark.id.clone();
    let favorite_id = bookmark.id.clone();
    let media_id = bookmark.id.clone();
    let author_services = services.clone();
    let favorite_services = services.clone();
    let is_active = context.expanded_bookmark_id.as_deref() == Some(bookmark.id.as_str());
    let is_selected = context.selected_ids.contains(&bookmark.id);
    let bookmark_images_enabled = context.loaded_media_bookmark_ids.contains(&bookmark.id);
    let remote_images_enabled = context.remote_images_enabled;
    let fav = bookmark.is_favorite;
    let tags = bookmark
        .tags
        .iter()
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<_>>()
        .join(" ");
    let when = format_timestamp(&bookmark.tweeted_at);
    let imported = format_timestamp(&bookmark.imported_at);
    let author = bookmark.author_handle.clone();
    let content = bookmark.content.clone();
    let media_count = bookmark.media.len();
    let media = bookmark.media.clone();
    let context_label = format!("@{author} tweet");

    rsx! {
        div {
            class: if is_active { "table-row active-row" } else if is_selected { "table-row multi-row" } else { "table-row" },
            onclick: move |_| {
                let mut next = state.write();
                toggle_expanded_bookmark(&mut next, id.clone());
            },
            span { class: "multi-dot", if is_selected { "●" } else { "" } }
            span { class: "row-index", "{index_plus(index)}" }
            button {
                class: "row-author",
                onclick: move |event| {
                    event.stop_propagation();
                    {
                        let mut next = state.write();
                        next.filters.author_query = author.clone();
                        next.route = ScreenRoute::Author(author.clone());
                        next.error = None;
                    }
                    reload_library(&author_services, &mut state);
                },
                "@{author}"
            }
            span { class: "row-when", "{when}" }
            span { class: "row-content", "{content}" }
            span { class: "row-tags", "{tags}" }
            button {
                class: if fav { "favorite-cell is-favorite" } else { "favorite-cell" },
                onclick: move |event| {
                    event.stop_propagation();
                    match favorite_services.borrow().toggle_favorite(&favorite_id) {
                        Ok(_) => reload_library(&favorite_services, &mut state),
                        Err(error) => state.write().error = Some(error.to_string()),
                    }
                },
                if fav { "★" } else { "·" }
            }
            if is_active {
                div { class: "row-detail",
                    div { class: "row-detail-actions",
                        button {
                            class: "ghost-button small",
                            onclick: move |event| {
                                event.stop_propagation();
                                state.write().route = ScreenRoute::Entry(detail_id.clone());
                        },
                        "open detail"
                    }
                        span { "media {media_count} · imported {imported}" }
                    }
                    MediaGallery {
                        media,
                        remote_images_enabled,
                        bookmark_images_enabled,
                        on_enable_remote_images: move |_| set_remote_images_enabled(&mut state, true),
                        on_enable_bookmark_images: move |_| load_bookmark_remote_images(&mut state, &media_id),
                        context_label,
                        mode: MediaGalleryMode::Feed,
                    }
                }
            }
        }
    }
}

fn tree_view(state: Signal<LibraryState>, services: Services, payload: FeedPayload) -> Element {
    let mut by_author: BTreeMap<String, Vec<Bookmark>> = BTreeMap::new();
    for bookmark in payload.bookmarks {
        by_author
            .entry(bookmark.author_handle.clone())
            .or_default()
            .push(bookmark);
    }

    rsx! {
        div { class: "tree-view",
            div { class: "tree-root", "~/eterea/library/by-author/" }
            for (author, bookmarks) in by_author {
                div { class: "tree-group",
                    div { class: "tree-author", "▾ " span { "@{author}" } " ({bookmarks.len()})" }
                    for bookmark in bookmarks {
                        {tree_row(state, services.clone(), bookmark, payload.expanded_bookmark_id.as_deref())}
                    }
                }
            }
        }
    }
}

fn tree_row(
    mut state: Signal<LibraryState>,
    services: Services,
    bookmark: Bookmark,
    expanded_id: Option<&str>,
) -> Element {
    let id = bookmark.id.clone();
    let tag_services = services.clone();
    let tag = bookmark
        .tags
        .first()
        .cloned()
        .unwrap_or_else(|| "untagged".to_string());
    let is_active = expanded_id == Some(bookmark.id.as_str());
    let when = format_timestamp(&bookmark.tweeted_at);
    rsx! {
        div {
            class: if is_active { "tree-row active-row" } else { "tree-row" },
            onclick: move |_| {
                let mut next = state.write();
                toggle_expanded_bookmark(&mut next, id.clone());
            },
            span { class: "tree-glyph", "├─" }
            span { class: "row-when", "{when}" }
            span { class: "row-content", "{bookmark.content}" }
            button {
                class: "row-tags",
                onclick: move |event| {
                    event.stop_propagation();
                    {
                        let mut next = state.write();
                        next.filters.selected_tag = Some(tag.clone());
                        next.route = ScreenRoute::Topic(tag.clone());
                        next.error = None;
                    }
                    reload_library(&tag_services, &mut state);
                },
                "#{tag}"
            }
        }
    }
}

fn dashboard_view(payload: FeedPayload) -> Element {
    let scope_note = analytics_scope_note(&payload);
    let recent = payload
        .bookmarks
        .iter()
        .take(5)
        .cloned()
        .collect::<Vec<_>>();
    let max_tag = payload
        .top_tags
        .iter()
        .map(|(_, count)| *count)
        .max()
        .unwrap_or(1);
    rsx! {
        div { class: "dashboard-view",
            section { class: "dashboard-panel span-2",
                h4 { "── analytics scope ──" }
                p { class: "muted-copy tiny", "{scope_note}" }
            }
            {metric_card("current matches", payload.total.to_string(), "blue")}
            {metric_card("archive authors", payload.unique_authors.to_string(), "green")}
            {metric_card("archive ★ favorites", payload.favorite_bookmarks.to_string(), "yellow")}
            {metric_card("loaded rows", payload.bookmarks.len().to_string(), "peach")}
            section { class: "dashboard-panel span-2",
                h4 { "── archive-wide top tags ──" }
                for (tag, count) in payload.top_tags.iter().take(8) {
                    div { class: "tag-bar-row",
                        span { class: "mini-tag", "#{tag}" }
                        div { class: "tag-bar-track", div { class: "tag-bar-fill", style: "width: {(count * 100 / max_tag).max(4)}%;" } }
                        span { class: "tag-count", "{count}" }
                    }
                }
            }
            section { class: "dashboard-panel span-2",
                h4 { "── loaded-row recent saves ──" }
                for bookmark in recent {
                    div { class: "recent-row",
                        span { class: "row-author", "@{bookmark.author_handle}" }
                        span { class: "row-when", "{format_timestamp(&bookmark.imported_at)}" }
                        span { class: "row-content", "{bookmark.content}" }
                    }
                }
            }
        }
    }
}

fn metric_card(label: &str, value: String, tone: &str) -> Element {
    rsx! {
        section { class: "dashboard-panel metric-card {tone}",
            h4 { "── {label} ──" }
            div { class: "metric-line",
                strong { "{value}" }
                span { class: "metric-source", "service-derived" }
            }
        }
    }
}

fn graph_view(payload: FeedPayload) -> Element {
    let scope_note = analytics_scope_note(&payload);
    let loaded_tags = loaded_tag_counts(&payload.bookmarks);
    let nodes = loaded_tags
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, (tag, count))| {
            let x = 80 + (i as i32 % 5) * 150;
            let y = 90 + (i as i32 / 5) * 170;
            let r = 14 + (*count as i32).min(24);
            (tag.clone(), x, y, r)
        })
        .collect::<Vec<_>>();
    let node_lookup = nodes
        .iter()
        .map(|(tag, x, y, _)| (tag.clone(), (*x, *y)))
        .collect::<BTreeMap<_, _>>();
    let mut edge_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for bookmark in &payload.bookmarks {
        let mut tags = bookmark.tags.clone();
        tags.sort();
        tags.dedup();
        for (left_index, left) in tags.iter().enumerate() {
            for right in tags.iter().skip(left_index + 1) {
                if node_lookup.contains_key(left) && node_lookup.contains_key(right) {
                    *edge_counts
                        .entry((left.clone(), right.clone()))
                        .or_default() += 1;
                }
            }
        }
    }
    let edges = edge_counts
        .into_iter()
        .filter_map(|((left, right), count)| {
            let (x1, y1) = *node_lookup.get(&left)?;
            let (x2, y2) = *node_lookup.get(&right)?;
            Some((x1, y1, x2, y2, count))
        })
        .collect::<Vec<_>>();

    rsx! {
        div { class: "graph-view",
            h4 { "── tag graph · loaded-row co-occurrence ──" }
            p { class: "muted-copy tiny", "{scope_note}" }
            svg { class: "tag-graph", view_box: "0 0 800 480",
                for (x1, y1, x2, y2, count) in edges {
                    line { x1: "{x1}", y1: "{y1}", x2: "{x2}", y2: "{y2}", stroke_width: "{count.min(6)}" }
                }
                for (tag, x, y, r) in nodes {
                    g {
                        circle { cx: "{x}", cy: "{y}", r: "{r}" }
                        text { x: "{x}", y: "{y + 4}", text_anchor: "middle", "#{tag}" }
                    }
                }
            }
        }
    }
}

fn calendar_view(payload: FeedPayload) -> Element {
    let scope_note = analytics_scope_note(&payload);
    let mut by_day = [0usize; 98];
    let mut by_hour = [0usize; 24];
    let today = Utc::now().date_naive();
    for bookmark in &payload.bookmarks {
        let days_ago = today
            .signed_duration_since(bookmark.imported_at.date_naive())
            .num_days();
        if (0..by_day.len() as i64).contains(&days_ago) {
            by_day[(by_day.len() - 1) - days_ago as usize] += 1;
        }
        by_hour[bookmark.imported_at.hour() as usize] += 1;
    }
    let max_hour = by_hour.iter().copied().max().unwrap_or(1).max(1);

    rsx! {
        div { class: "calendar-view",
            h4 { "── saves heatmap · loaded rows · last 14 weeks ──" }
            p { class: "muted-copy tiny", "{scope_note}" }
            div { class: "heatmap-grid",
                for week in 0..14 {
                    div { class: "heatmap-week",
                        for day in 0..7 {
                            {heat_cell(by_day[week * 7 + day])}
                        }
                    }
                }
            }
            div { class: "heat-legend", "less" {heat_cell(0)} {heat_cell(1)} {heat_cell(2)} {heat_cell(3)} "more" }
            h4 { "── by hour ──" }
            div { class: "hour-bars",
                for (hour, count) in by_hour.into_iter().enumerate() {
                    div { class: "hour-bar", style: "height: {((count * 100) / max_hour).max(4)}%;", title: "{hour}:00 · {count}" }
                }
            }
            div { class: "hour-labels", span { "00:00" } span { "06:00" } span { "12:00" } span { "18:00" } span { "23:00" } }
        }
    }
}

fn heat_cell(value: usize) -> Element {
    let class_name = match value {
        0 => "heat-cell level-0",
        1 => "heat-cell level-1",
        2 => "heat-cell level-2",
        _ => "heat-cell level-3",
    };
    rsx! { span { class: "{class_name}" } }
}

fn analytics_scope_note(payload: &FeedPayload) -> String {
    if payload.has_more {
        format!(
            "Loaded-row charts use the {} rows currently loaded out of {} matching bookmarks. Archive-wide metrics are labeled separately.",
            payload.bookmarks.len(),
            payload.total
        )
    } else {
        format!(
            "Loaded-row charts use all {} rows in the current result set. Archive-wide metrics are labeled separately.",
            payload.total
        )
    }
}

fn loaded_tag_counts(bookmarks: &[Bookmark]) -> Vec<(String, i64)> {
    let mut counts = BTreeMap::<String, i64>::new();
    for bookmark in bookmarks {
        for tag in &bookmark.tags {
            *counts.entry(tag.clone()).or_default() += 1;
        }
    }

    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|(left_tag, left_count), (right_tag, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_tag.cmp(right_tag))
    });
    counts
}

fn index_plus(index: usize) -> String {
    format!("{:03}", index + 1)
}
