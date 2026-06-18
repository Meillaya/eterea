use eterea_core::models::{Media, MediaType};

pub(super) const PREVIEW_LIMIT: usize = 4;

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
    let mut image_previews = Vec::new();
    let mut external_links = Vec::new();
    let mut safe_image_count = 0usize;
    let mut blocked_count = 0usize;

    for item in media {
        let url = item.url.trim();
        if !is_safe_https_url(url) {
            blocked_count += 1;
            continue;
        }

        match item.media_type {
            MediaType::Image => {
                safe_image_count += 1;
                if remote_images_enabled && image_previews.len() < PREVIEW_LIMIT {
                    image_previews.push(MediaPreview {
                        url: url.to_string(),
                        alt_text: media_alt_text(context_label, &MediaType::Image),
                        media_type: MediaType::Image,
                    });
                }
            }
            MediaType::Video | MediaType::Gif | MediaType::Unknown => {
                let media_type = item.media_type.clone();
                external_links.push(MediaPreview {
                    url: url.to_string(),
                    alt_text: media_alt_text(context_label, &media_type),
                    media_type,
                });
            }
        }
    }

    let hidden_image_count = if remote_images_enabled {
        0
    } else {
        safe_image_count
    };
    let overflow_count = safe_image_count.saturating_sub(PREVIEW_LIMIT);

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

fn media_alt_text(context_label: &str, media_type: &MediaType) -> String {
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
