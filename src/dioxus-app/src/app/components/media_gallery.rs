mod policy;

use policy::{media_presentation, media_presentation_with_limit, PREVIEW_LIMIT};

use crate::app::actions::open_external_url;
use dioxus::prelude::*;
use eterea_core::models::{Media, MediaType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MediaGalleryMode {
    Feed,
    Detail,
}

impl MediaGalleryMode {
    fn class_name(self) -> &'static str {
        match self {
            Self::Feed => "media-gallery feed",
            Self::Detail => "media-gallery detail",
        }
    }

    const fn preview_limit(self) -> usize {
        match self {
            Self::Feed => PREVIEW_LIMIT,
            Self::Detail => usize::MAX,
        }
    }
}

pub(crate) fn tile_class(failed: bool) -> &'static str {
    if failed {
        "media-tile failed"
    } else {
        "media-tile"
    }
}

pub(crate) fn record_failed_url(failed_urls: &mut Vec<String>, failed_url: &str) {
    if !failed_urls.iter().any(|url| url == failed_url) {
        failed_urls.push(failed_url.to_string());
    }
}

fn media_label(media_type: &MediaType) -> &'static str {
    match media_type {
        MediaType::Image => "Open image",
        MediaType::Video => "Open video",
        MediaType::Gif => "Open GIF",
        MediaType::Unknown => "Open media",
    }
}

pub(crate) fn aspect_ratio_style(width: Option<i64>, height: Option<i64>) -> String {
    match (width, height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => {
            format!("aspect-ratio: {width} / {height};")
        }
        _ => String::new(),
    }
}

#[component]
pub(crate) fn MediaGallery(
    media: Vec<Media>,
    remote_images_enabled: bool,
    bookmark_images_enabled: bool,
    on_enable_remote_images: EventHandler<()>,
    on_enable_bookmark_images: EventHandler<()>,
    context_label: String,
    mode: MediaGalleryMode,
) -> Element {
    let images_enabled = remote_images_enabled || bookmark_images_enabled;
    let presentation = if mode.preview_limit() == PREVIEW_LIMIT {
        media_presentation(&media, images_enabled, &context_label)
    } else {
        media_presentation_with_limit(&media, images_enabled, &context_label, mode.preview_limit())
    };
    let mut failed_urls = use_signal(Vec::<String>::new);

    if presentation.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "{mode.class_name()}",
            if presentation.has_hidden_images() {
                div { class: "media-hidden-card",
                    div {
                        span { class: "eyebrow", "Remote images hidden" }
                        p { "{presentation.hidden_image_count} image attachment(s) available for this tweet." }
                        small { "Load for this session to fetch stored HTTPS image URLs from the network." }
                    }
                    button {
                        class: "ghost-button small",
                        onclick: move |event| {
                            event.stop_propagation();
                            on_enable_bookmark_images.call(());
                        },
                        "Load this tweet"
                    }
                    button {
                        class: "ghost-button small",
                        onclick: move |event| {
                            event.stop_propagation();
                            on_enable_remote_images.call(());
                        },
                        "Load all this session"
                    }
                }
            }

            if !presentation.image_previews.is_empty() {
                div {
                    class: "media-grid count-{presentation.image_previews.len()}",
                    for preview in &presentation.image_previews {
                        {
                            let failed = failed_urls.read().contains(&preview.url);
                            let failed_url = preview.url.clone();
                            let aspect_ratio = aspect_ratio_style(preview.width, preview.height);
                            rsx! {
                                figure { class: "{tile_class(failed)}", style: "{aspect_ratio}",
                                    if failed {
                                        div { class: "media-fallback", role: "img", aria_label: "{preview.alt_text}",
                                            span { "Image unavailable" }
                                            small { "The remote thumbnail could not load; layout space is reserved." }
                                        }
                                    } else {
                                        img {
                                            src: "{preview.url}",
                                            alt: "{preview.alt_text}",
                                            loading: "lazy",
                                            onclick: move |event| event.stop_propagation(),
                                            onerror: move |_| {
                                                let mut failed = failed_urls.write();
                                                record_failed_url(&mut failed, &failed_url);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if presentation.overflow_count > 0 {
                        div { class: "media-overflow", "+{presentation.overflow_count}" }
                    }
                }
            }

            if !presentation.external_links.is_empty() || presentation.blocked_count > 0 {
                div { class: "media-chip-row",
                    for link in &presentation.external_links {
                        {
                            let url = link.url.clone();
                            rsx! {
                                button {
                                    class: "media-chip",
                                    title: "{link.alt_text}",
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        if let Err(error) = open_external_url(&url) {
                                            eprintln!("failed to open media URL: {error}");
                                        }
                                    },
                                    "{media_label(&link.media_type)}"
                                }
                            }
                        }
                    }
                    if presentation.blocked_count > 0 {
                        span { class: "media-chip blocked", "{presentation.blocked_count} unavailable" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_image_state_uses_reserved_fallback_tile_class() {
        assert_eq!(tile_class(false), "media-tile");
        assert_eq!(tile_class(true), "media-tile failed");
    }

    #[test]
    fn failed_image_state_records_each_url_once() {
        let mut failed_urls = vec!["https://example.com/old.png".to_string()];

        record_failed_url(&mut failed_urls, "https://example.com/new.png");
        record_failed_url(&mut failed_urls, "https://example.com/new.png");

        assert_eq!(
            failed_urls,
            vec![
                "https://example.com/old.png".to_string(),
                "https://example.com/new.png".to_string()
            ]
        );
    }

    #[test]
    fn aspect_ratio_style_uses_positive_metadata_dimensions() {
        assert_eq!(
            aspect_ratio_style(Some(1600), Some(900)),
            "aspect-ratio: 1600 / 900;"
        );
        assert_eq!(aspect_ratio_style(Some(1600), Some(0)), "");
        assert_eq!(aspect_ratio_style(None, Some(900)), "");
    }

    #[test]
    fn feed_mode_caps_previews_but_detail_mode_can_show_all_images() {
        assert_eq!(
            MediaGalleryMode::Feed.preview_limit(),
            policy::PREVIEW_LIMIT
        );
        assert_eq!(MediaGalleryMode::Detail.preview_limit(), usize::MAX);
    }
}
