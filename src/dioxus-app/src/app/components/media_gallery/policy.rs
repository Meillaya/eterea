use eterea_core::models::{Media, MediaType};

pub(super) const PREVIEW_LIMIT: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MediaPreview {
    pub(crate) url: String,
    pub(crate) alt_text: String,
    pub(crate) media_type: MediaType,
    pub(crate) width: Option<i64>,
    pub(crate) height: Option<i64>,
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
    media_presentation_with_limit(media, remote_images_enabled, context_label, PREVIEW_LIMIT)
}

pub(crate) fn media_presentation_with_limit(
    media: &[Media],
    remote_images_enabled: bool,
    context_label: &str,
    preview_limit: usize,
) -> MediaPresentation {
    let mut image_previews = Vec::new();
    let mut external_links = Vec::new();
    let mut safe_image_count = 0usize;
    let mut blocked_count = 0usize;

    for item in media {
        let primary_url = item.url.trim();
        let safe_primary = is_safe_https_url(primary_url);

        match item.media_type {
            MediaType::Image => {
                if !safe_primary {
                    blocked_count += 1;
                    continue;
                }
                safe_image_count += 1;
                if remote_images_enabled && image_previews.len() < preview_limit {
                    image_previews.push(media_preview(
                        primary_url,
                        item,
                        context_label,
                        MediaType::Image,
                    ));
                }
            }
            MediaType::Video | MediaType::Gif | MediaType::Unknown => {
                let Some(url) = safe_external_url(item, primary_url) else {
                    blocked_count += 1;
                    continue;
                };
                let media_type = item.media_type.clone();
                external_links.push(media_preview(&url, item, context_label, media_type));
            }
        }
    }

    let hidden_image_count = if remote_images_enabled {
        0
    } else {
        safe_image_count
    };
    let overflow_count = safe_image_count.saturating_sub(preview_limit);

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

fn media_preview(
    url: &str,
    item: &Media,
    context_label: &str,
    media_type: MediaType,
) -> MediaPreview {
    MediaPreview {
        url: url.to_string(),
        alt_text: media_alt_text(context_label, &media_type, item.alt_text.as_deref()),
        media_type,
        width: item.width.filter(|width| *width > 0),
        height: item.height.filter(|height| *height > 0),
    }
}

fn safe_external_url(item: &Media, primary_url: &str) -> Option<String> {
    item.variant_url
        .as_deref()
        .filter(|url| is_safe_https_url(url))
        .or_else(|| {
            item.preview_url
                .as_deref()
                .filter(|url| is_safe_https_url(url))
        })
        .or_else(|| is_safe_https_url(primary_url).then_some(primary_url))
        .map(|url| url.trim().to_string())
}

fn media_alt_text(context_label: &str, media_type: &MediaType, alt_text: Option<&str>) -> String {
    if let Some(alt_text) = alt_text.map(str::trim).filter(|text| !text.is_empty()) {
        return alt_text.to_string();
    }

    match media_type {
        MediaType::Image => format!("Image from {context_label}"),
        MediaType::Video => format!("Video media from {context_label}"),
        MediaType::Gif => format!("GIF media from {context_label}"),
        MediaType::Unknown => format!("Media from {context_label}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn media(url: &str, media_type: MediaType) -> Media {
        Media::new(url, media_type)
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
    fn hidden_remote_images_count_without_rendering_preview_urls() {
        let media = (0..6)
            .map(|index| {
                media(
                    &format!("https://pbs.twimg.com/media/hidden-{index}.jpg"),
                    MediaType::Image,
                )
            })
            .collect::<Vec<_>>();

        let state = media_presentation(&media, false, "@alice tweet");

        assert!(state.image_previews.is_empty());
        assert_eq!(state.hidden_image_count, 6);
        assert_eq!(state.overflow_count, 2);
    }

    #[test]
    fn custom_preview_limit_allows_detail_gallery_to_show_more_images() {
        let media = (0..6)
            .map(|index| {
                media(
                    &format!("https://pbs.twimg.com/media/detail-{index}.jpg"),
                    MediaType::Image,
                )
            })
            .collect::<Vec<_>>();

        let state = media_presentation_with_limit(&media, true, "@alice tweet", usize::MAX);

        assert_eq!(state.image_previews.len(), 6);
        assert_eq!(state.overflow_count, 0);
    }

    #[test]
    fn image_preview_uses_alt_text_and_dimensions_from_metadata() {
        let state = media_presentation(
            &[
                Media::new("https://pbs.twimg.com/media/a.jpg", MediaType::Image)
                    .with_alt_text(Some("Architecture diagram".to_string()))
                    .with_dimensions(Some(1600), Some(900)),
            ],
            true,
            "@alice tweet",
        );

        assert_eq!(state.image_previews[0].alt_text, "Architecture diagram");
        assert_eq!(state.image_previews[0].width, Some(1600));
        assert_eq!(state.image_previews[0].height, Some(900));
    }

    #[test]
    fn external_media_prefers_safe_variant_url_without_inline_previewing_video() {
        let state = media_presentation(
            &[Media::new(
                "https://pbs.twimg.com/ext_tw_video/thumb.jpg",
                MediaType::Video,
            )
            .with_preview_urls(
                Some("https://pbs.twimg.com/ext_tw_video/thumb.jpg".to_string()),
                Some("https://video.twimg.com/ext_tw_video/720.mp4".to_string()),
            )],
            true,
            "@alice tweet",
        );

        assert!(state.image_previews.is_empty());
        assert_eq!(state.external_links.len(), 1);
        assert_eq!(
            state.external_links[0].url,
            "https://video.twimg.com/ext_tw_video/720.mp4"
        );
    }

    #[test]
    fn hidden_mode_does_not_render_metadata_preview_urls() {
        let state = media_presentation(
            &[
                Media::new("https://pbs.twimg.com/media/a.jpg", MediaType::Image)
                    .with_preview_urls(
                        Some("https://pbs.twimg.com/media/preview.jpg".to_string()),
                        None,
                    ),
            ],
            false,
            "@alice tweet",
        );

        assert!(state.image_previews.is_empty());
        assert_eq!(state.hidden_image_count, 1);
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
}
