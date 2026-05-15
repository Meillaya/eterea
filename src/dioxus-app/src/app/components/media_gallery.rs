use crate::app::actions::open_external_url;
use dioxus::prelude::*;
use eterea_core::models::{Media, MediaType};

const PREVIEW_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MediaGalleryMode {
    Compact,
    Detail,
}

impl MediaGalleryMode {
    fn class_name(self) -> &'static str {
        match self {
            Self::Compact => "media-gallery compact",
            Self::Detail => "media-gallery detail",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MediaPreview {
    pub(crate) url: String,
    pub(crate) alt_text: String,
    pub(crate) media_type: MediaType,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MediaPresentation {
    pub(crate) image_previews: Vec<MediaPreview>,
    pub(crate) external_links: Vec<MediaPreview>,
    pub(crate) hidden_image_count: usize,
    pub(crate) overflow_count: usize,
    pub(crate) blocked_count: usize,
    pub(crate) total_count: usize,
    pub(crate) remote_images_enabled: bool,
}

impl MediaPresentation {
    pub(crate) fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    pub(crate) fn has_hidden_images(&self) -> bool {
        !self.remote_images_enabled && self.hidden_image_count > 0
    }
}

pub(crate) fn media_presentation(
    media: &[Media],
    remote_images_enabled: bool,
    context_label: &str,
) -> MediaPresentation {
    let mut safe_images = Vec::new();
    let mut external_links = Vec::new();
    let mut blocked_count = 0;

    for item in media {
        let url = item.url.trim();
        if !is_safe_https_url(url) {
            blocked_count += 1;
            continue;
        }

        let preview = MediaPreview {
            url: url.to_string(),
            alt_text: media_alt_text(context_label, &item.media_type),
            media_type: item.media_type.clone(),
        };

        if item.media_type == MediaType::Image {
            safe_images.push(preview);
        } else {
            external_links.push(preview);
        }
    }

    let hidden_image_count = if remote_images_enabled {
        0
    } else {
        safe_images.len()
    };
    let overflow_count = safe_images.len().saturating_sub(PREVIEW_LIMIT);
    let image_previews = if remote_images_enabled {
        safe_images.into_iter().take(PREVIEW_LIMIT).collect()
    } else {
        Vec::new()
    };

    MediaPresentation {
        image_previews,
        external_links,
        hidden_image_count,
        overflow_count,
        blocked_count,
        total_count: media.len(),
        remote_images_enabled,
    }
}

pub(crate) fn is_safe_https_url(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.len() > "https://".len()
        && trimmed
            .get(.."https://".len())
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
}

pub(crate) fn tile_class(failed: bool) -> &'static str {
    if failed {
        "media-tile failed"
    } else {
        "media-tile"
    }
}

fn media_alt_text(context_label: &str, media_type: &MediaType) -> String {
    match media_type {
        MediaType::Image => format!("Image from {context_label}"),
        MediaType::Video => format!("Video media from {context_label}"),
        MediaType::Gif => format!("GIF media from {context_label}"),
        MediaType::Unknown => format!("Media from {context_label}"),
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

#[component]
pub(crate) fn MediaGallery(
    media: Vec<Media>,
    remote_images_enabled: bool,
    on_enable_remote_images: EventHandler<()>,
    context_label: String,
    mode: MediaGalleryMode,
) -> Element {
    let presentation = media_presentation(&media, remote_images_enabled, &context_label);
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
                            on_enable_remote_images.call(());
                        },
                        "Load remote images"
                    }
                }
            }

            if !presentation.image_previews.is_empty() {
                div {
                    class: "media-grid count-{presentation.image_previews.len()}",
                    for preview in presentation.image_previews.clone() {
                        {
                            let image_url = preview.url.clone();
                            let image_url_for_error = image_url.clone();
                            let failed = failed_urls.read().contains(&image_url);
                            rsx! {
                                figure { class: "{tile_class(failed)}",
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
                                                if !failed.contains(&image_url_for_error) {
                                                    failed.push(image_url_for_error.clone());
                                                }
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
                    for link in presentation.external_links.clone() {
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

    fn media(url: &str, media_type: MediaType) -> Media {
        Media {
            url: url.to_string(),
            media_type,
        }
    }

    #[test]
    fn filters_inline_images_to_safe_https_images_only() {
        let state = media_presentation(
            &[
                media("https://pbs.twimg.com/media/a.jpg", MediaType::Image),
                media("http://pbs.twimg.com/media/b.jpg", MediaType::Image),
                media("https://video.twimg.com/a.mp4", MediaType::Video),
                media("javascript:alert(1)", MediaType::Image),
            ],
            true,
            "@alice tweet",
        );

        assert_eq!(state.image_previews.len(), 1);
        assert_eq!(
            state.image_previews[0].url,
            "https://pbs.twimg.com/media/a.jpg"
        );
        assert_eq!(state.external_links.len(), 1);
        assert_eq!(state.blocked_count, 2);
    }

    #[test]
    fn excludes_gifs_videos_and_unknown_media_from_inline_images() {
        let state = media_presentation(
            &[
                media("https://pbs.twimg.com/media/a.gif", MediaType::Gif),
                media("https://video.twimg.com/a.mp4", MediaType::Video),
                media("https://example.com/file", MediaType::Unknown),
            ],
            true,
            "@alice tweet",
        );

        assert!(state.image_previews.is_empty());
        assert_eq!(state.external_links.len(), 3);
    }

    #[test]
    fn caps_previews_and_reports_overflow() {
        let media = (0..6)
            .map(|index| {
                media(
                    &format!("https://pbs.twimg.com/media/{index}.jpg"),
                    MediaType::Image,
                )
            })
            .collect::<Vec<_>>();

        let state = media_presentation(&media, true, "@alice tweet");

        assert_eq!(state.image_previews.len(), PREVIEW_LIMIT);
        assert_eq!(state.overflow_count, 2);
    }

    #[test]
    fn default_hidden_mode_returns_summary_and_no_image_urls() {
        let state = media_presentation(
            &[
                media("https://pbs.twimg.com/media/a.jpg", MediaType::Image),
                media("https://pbs.twimg.com/media/b.jpg", MediaType::Image),
            ],
            false,
            "@alice tweet",
        );

        assert!(state.image_previews.is_empty());
        assert_eq!(state.hidden_image_count, 2);
        assert!(state.has_hidden_images());
    }

    #[test]
    fn empty_media_returns_empty_gallery_state() {
        let state = media_presentation(&[], true, "@alice tweet");

        assert!(state.is_empty());
        assert!(state.image_previews.is_empty());
        assert!(state.external_links.is_empty());
    }

    #[test]
    fn safe_url_policy_rejects_non_https_schemes() {
        assert!(is_safe_https_url("https://pbs.twimg.com/media/a.jpg"));
        assert!(is_safe_https_url(" HTTPS://pbs.twimg.com/media/a.jpg "));
        assert!(!is_safe_https_url("http://pbs.twimg.com/media/a.jpg"));
        assert!(!is_safe_https_url("file:///tmp/a.jpg"));
        assert!(!is_safe_https_url("javascript:alert(1)"));
        assert!(!is_safe_https_url(""));
    }

    #[test]
    fn trims_safe_urls_before_rendering_or_opening() {
        let state = media_presentation(
            &[media(
                " https://pbs.twimg.com/media/spaced.jpg ",
                MediaType::Image,
            )],
            true,
            "@alice tweet",
        );

        assert_eq!(
            state.image_previews[0].url,
            "https://pbs.twimg.com/media/spaced.jpg"
        );
    }

    #[test]
    fn broken_image_state_uses_reserved_fallback_tile_class() {
        assert_eq!(tile_class(false), "media-tile");
        assert_eq!(tile_class(true), "media-tile failed");
    }
}
