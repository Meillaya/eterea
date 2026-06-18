//! Metadata-only media extraction for JSON/X-like payloads.

use crate::models::{Media, MediaType};
use serde_json::Value;
use std::collections::HashSet;

pub(crate) fn extract_media(raw: &Value) -> Vec<Media> {
    let mut media = Vec::new();
    let mut seen = HashSet::new();

    for item in direct_media_items(raw) {
        push_deduped(&mut media, &mut seen, media_from_value(item));
    }

    for item in attached_included_media(raw) {
        push_deduped(&mut media, &mut seen, media_from_value(item));
    }

    media
}

fn direct_media_items(raw: &Value) -> Vec<&Value> {
    let mut items = Vec::new();
    for path in [
        &["media"][..],
        &["extended_media"][..],
        &["entities", "media"][..],
        &["extended_entities", "media"][..],
        &["legacy", "entities", "media"][..],
        &["legacy", "extended_entities", "media"][..],
    ] {
        if let Some(Value::Array(candidates)) = value_at_path(raw, path) {
            items.extend(candidates);
        }
    }
    items
}

fn attached_included_media(raw: &Value) -> Vec<&Value> {
    let Some(Value::Array(included)) = value_at_path(raw, &["includes", "media"]) else {
        return Vec::new();
    };

    let keys = attachment_media_keys(raw);
    if keys.is_empty() {
        return included.iter().collect();
    }

    included
        .iter()
        .filter(|item| {
            string_at(item, &["media_key"])
                .as_deref()
                .is_some_and(|key| keys.contains(key))
        })
        .collect()
}

fn attachment_media_keys(raw: &Value) -> HashSet<String> {
    let mut keys = HashSet::new();
    for path in [
        &["attachments", "media_keys"][..],
        &["legacy", "attachments", "media_keys"][..],
    ] {
        let Some(Value::Array(values)) = value_at_path(raw, path) else {
            continue;
        };
        for value in values {
            if let Some(key) = value_to_string(value) {
                keys.insert(key);
            }
        }
    }
    keys
}

pub(crate) fn media_from_value(raw: &Value) -> Option<Media> {
    let source_type = string_at(raw, &["type"])
        .or_else(|| string_at(raw, &["media_type"]))
        .or_else(|| string_at(raw, &["source_type"]));
    let url = media_url(raw)?;
    let media_type = source_type
        .as_deref()
        .map(MediaType::from_source_type)
        .filter(|media_type| *media_type != MediaType::Unknown)
        .unwrap_or_else(|| MediaType::from_url(&url));

    Some(
        Media::new(url, media_type)
            .with_alt_text(
                string_at(raw, &["alt_text"])
                    .or_else(|| string_at(raw, &["ext_alt_text"]))
                    .or_else(|| string_at(raw, &["accessibility_label"])),
            )
            .with_dimensions(number_at(raw, &["width"]), number_at(raw, &["height"]))
            .with_source(string_at(raw, &["media_key"]), source_type)
            .with_preview_urls(string_at(raw, &["preview_image_url"]), variant_url(raw))
            .with_variants_json(variants_json(raw)),
    )
}

fn media_url(raw: &Value) -> Option<String> {
    string_at(raw, &["media_url_https"])
        .or_else(|| string_at(raw, &["media_url"]))
        .or_else(|| string_at(raw, &["url"]))
        .or_else(|| string_at(raw, &["preview_image_url"]))
}

fn variant_url(raw: &Value) -> Option<String> {
    variant_items(raw)?
        .iter()
        .find_map(|variant| string_at(variant, &["url"]))
}

fn variants_json(raw: &Value) -> Option<String> {
    let variants = variant_items(raw)?;
    if variants.is_empty() {
        return None;
    }
    serde_json::to_string(variants).ok()
}

fn variant_items(raw: &Value) -> Option<&Vec<Value>> {
    [&["variants"][..], &["video_info", "variants"][..]]
        .into_iter()
        .find_map(|path| match value_at_path(raw, path) {
            Some(Value::Array(variants)) => Some(variants),
            _ => None,
        })
}

fn push_deduped(media: &mut Vec<Media>, seen: &mut HashSet<String>, candidate: Option<Media>) {
    let Some(candidate) = candidate else {
        return;
    };
    let key = candidate
        .source_media_key
        .clone()
        .unwrap_or_else(|| candidate.url.clone());
    if seen.insert(key) {
        media.push(candidate);
    }
}

fn string_at(raw: &Value, path: &[&str]) -> Option<String> {
    value_at_path(raw, path)
        .and_then(value_to_string)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn number_at(raw: &Value, path: &[&str]) -> Option<i64> {
    value_at_path(raw, path).and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn value_at_path<'a>(raw: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = raw;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_direct_media_metadata_and_prefers_declared_type() {
        let raw = json!({
            "extended_entities": {
                "media": [{
                    "media_key": "3_111",
                    "type": "photo",
                    "media_url_https": "https://pbs.twimg.com/media/full.jpg",
                    "alt_text": "Screenshot of a dashboard",
                    "width": 1200,
                    "height": "900",
                    "preview_image_url": "https://pbs.twimg.com/media/preview.jpg"
                }]
            }
        });

        let media = extract_media(&raw);

        assert_eq!(media.len(), 1);
        assert_eq!(media[0].media_type, MediaType::Image);
        assert_eq!(media[0].source_media_key.as_deref(), Some("3_111"));
        assert_eq!(media[0].source_type.as_deref(), Some("photo"));
        assert_eq!(
            media[0].alt_text.as_deref(),
            Some("Screenshot of a dashboard")
        );
        assert_eq!(media[0].width, Some(1200));
        assert_eq!(media[0].height, Some(900));
    }

    #[test]
    fn filters_included_media_by_attachment_keys_and_deduplicates() {
        let raw = json!({
            "attachments": { "media_keys": ["7_video"] },
            "includes": {
                "media": [
                    {
                        "media_key": "7_video",
                        "type": "video",
                        "preview_image_url": "https://pbs.twimg.com/ext_tw_video/thumb.jpg",
                        "variants": [{"url": "https://video.twimg.com/ext_tw_video/720.mp4"}]
                    },
                    {
                        "media_key": "7_video",
                        "type": "video",
                        "preview_image_url": "https://pbs.twimg.com/ext_tw_video/duplicate.jpg"
                    },
                    {
                        "media_key": "3_other",
                        "type": "photo",
                        "url": "https://pbs.twimg.com/media/other.jpg"
                    }
                ]
            }
        });

        let media = extract_media(&raw);

        assert_eq!(media.len(), 1);
        assert_eq!(media[0].media_type, MediaType::Video);
        assert_eq!(
            media[0].preview_url.as_deref(),
            Some("https://pbs.twimg.com/ext_tw_video/thumb.jpg")
        );
        assert_eq!(
            media[0].variant_url.as_deref(),
            Some("https://video.twimg.com/ext_tw_video/720.mp4")
        );
        let variants_json = media[0].variants_json.as_deref().unwrap_or_default();
        assert!(variants_json.contains("720.mp4"));
    }
}
