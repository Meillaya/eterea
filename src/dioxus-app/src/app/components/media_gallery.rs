mod policy;

use policy::media_presentation;

use crate::app::actions::open_external_url;
use dioxus::prelude::*;
use eterea_core::models::{Media, MediaType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MediaGalleryMode {
    Detail,
}

impl MediaGalleryMode {
    fn class_name(self) -> &'static str {
        match self {
            Self::Detail => "media-gallery detail",
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
                    for preview in &presentation.image_previews {
                        {
                            let failed = failed_urls.read().contains(&preview.url);
                            let failed_url = preview.url.clone();
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
                                                if !failed.contains(&failed_url) {
                                                    failed.push(failed_url.clone());
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
}
