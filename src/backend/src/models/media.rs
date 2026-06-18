//! Media attachment data model.

use serde::{Deserialize, Serialize};

/// Media attachment metadata stored by Eterea.
///
/// This intentionally stores only source metadata and remote URLs. It does not
/// cache, download, or persist media bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Media {
    pub url: String,
    pub media_type: MediaType,
    pub alt_text: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub source_media_key: Option<String>,
    pub source_type: Option<String>,
    pub preview_url: Option<String>,
    pub variant_url: Option<String>,
    pub variants_json: Option<String>,
}

impl Media {
    pub fn new(url: impl Into<String>, media_type: MediaType) -> Self {
        Self {
            url: url.into(),
            media_type,
            alt_text: None,
            width: None,
            height: None,
            source_media_key: None,
            source_type: None,
            preview_url: None,
            variant_url: None,
            variants_json: None,
        }
    }

    pub fn with_alt_text(mut self, alt_text: Option<String>) -> Self {
        self.alt_text = non_empty(alt_text);
        self
    }

    pub fn with_dimensions(mut self, width: Option<i64>, height: Option<i64>) -> Self {
        self.width = positive_dimension(width);
        self.height = positive_dimension(height);
        self
    }

    pub fn with_source(mut self, media_key: Option<String>, source_type: Option<String>) -> Self {
        self.source_media_key = non_empty(media_key);
        self.source_type = non_empty(source_type);
        self
    }

    pub fn with_preview_urls(
        mut self,
        preview_url: Option<String>,
        variant_url: Option<String>,
    ) -> Self {
        self.preview_url = non_empty(preview_url);
        self.variant_url = non_empty(variant_url);
        self
    }

    pub fn with_variants_json(mut self, variants_json: Option<String>) -> Self {
        self.variants_json = non_empty(variants_json);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
    Gif,
    Unknown,
}

impl MediaType {
    pub fn as_storage_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Gif => "gif",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_storage_str(value: &str) -> Self {
        match value {
            "image" => Self::Image,
            "video" => Self::Video,
            "gif" => Self::Gif,
            _ => Self::Unknown,
        }
    }

    pub fn from_source_type(value: &str) -> Self {
        match value {
            "photo" | "image" => Self::Image,
            "video" => Self::Video,
            "animated_gif" | "gif" => Self::Gif,
            _ => Self::Unknown,
        }
    }

    pub fn from_url(url: &str) -> Self {
        let lower = url.to_lowercase();
        if lower.contains(".gif") || lower.contains("gif") {
            Self::Gif
        } else if lower.contains(".mp4") || lower.contains("video") {
            Self::Video
        } else if lower.contains(".jpg")
            || lower.contains(".jpeg")
            || lower.contains(".png")
            || lower.contains(".webp")
            || lower.contains("pbs.twimg.com")
        {
            Self::Image
        } else {
            Self::Unknown
        }
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn positive_dimension(value: Option<i64>) -> Option<i64> {
    value.filter(|dimension| *dimension > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_constructor_keeps_metadata_empty_by_default() {
        let media = Media::new("https://pbs.twimg.com/media/a.jpg", MediaType::Image);

        assert_eq!(media.url, "https://pbs.twimg.com/media/a.jpg");
        assert_eq!(media.media_type, MediaType::Image);
        assert_eq!(media.alt_text, None);
        assert_eq!(media.preview_url, None);
        assert_eq!(media.variants_json, None);
    }

    #[test]
    fn metadata_builders_drop_empty_text_and_invalid_dimensions() {
        let media = Media::new("https://video.twimg.com/a.mp4", MediaType::Video)
            .with_alt_text(Some("  demo clip  ".to_string()))
            .with_dimensions(Some(1280), Some(0))
            .with_source(Some(" 3_123 ".to_string()), Some("".to_string()));

        assert_eq!(media.alt_text.as_deref(), Some("demo clip"));
        assert_eq!(media.width, Some(1280));
        assert_eq!(media.height, None);
        assert_eq!(media.source_media_key.as_deref(), Some("3_123"));
        assert_eq!(media.source_type, None);
    }
}
