use super::actions::reload_library;
use super::route::ScreenRoute;
use super::state::{LibraryState, Services};
use dioxus::prelude::*;
use eterea_app::{AuthorSummary, TopicSummary};

const TOPIC_CLOUD_RENDER_LIMIT: usize = 250;

pub(crate) fn visible_topic_cloud(topics: &[TopicSummary]) -> (&[TopicSummary], bool) {
    let visible_count = topics.len().min(TOPIC_CLOUD_RENDER_LIMIT);
    (
        &topics[..visible_count],
        topics.len() > TOPIC_CLOUD_RENDER_LIMIT,
    )
}

pub(crate) fn authors_directory(
    mut state: Signal<LibraryState>,
    services: Services,
    visible_authors: Vec<AuthorSummary>,
    author_status: String,
) -> Element {
    rsx! {
        p { class: "muted-copy tiny", "{author_status}" }
        div { class: "directory-list",
            for author in visible_authors {
                button {
                    class: "directory-row",
                    onclick: {
                        let directory_services = services.clone();
                        move |_| {
                            {
                                let mut next = state.write();
                                next.route = ScreenRoute::Author(author.handle.clone());
                                next.filters.author_query = author.handle.clone();
                                next.filters.selected_tag = None;
                                next.filters.favorites_only = false;
                                next.error = None;
                            }
                            reload_library(&directory_services, &mut state);
                        }
                    },
                    span { class: "directory-title", "{author.name}" }
                    small { "@{author.handle} · {author.bookmark_count} entries · {author.favorite_count} ★" }
                }
            }
        }
    }
}

pub(crate) fn topics_cloud(
    mut state: Signal<LibraryState>,
    services: Services,
    topics: Vec<TopicSummary>,
    total_topics: usize,
    topics_limited: bool,
) -> Element {
    rsx! {
        if topics_limited {
            p { class: "muted-copy tiny", "Showing top {topics.len()} of {total_topics} topics to keep the topic cloud responsive." }
        }
        div { class: "topic-cloud",
            for topic in topics {
                button {
                    class: "topic-token",
                    onclick: {
                        let directory_services = services.clone();
                        move |_| {
                            {
                                let mut next = state.write();
                                next.route = ScreenRoute::Topic(topic.tag.clone());
                                next.filters.selected_tag = Some(topic.tag.clone());
                                next.filters.author_query.clear();
                                next.filters.favorites_only = false;
                                next.error = None;
                            }
                            reload_library(&directory_services, &mut state);
                        }
                    },
                    "#{topic.tag}"
                    sup { "{topic.bookmark_count}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topic(index: usize) -> TopicSummary {
        TopicSummary {
            tag: format!("topic_{index:03}"),
            bookmark_count: 1,
        }
    }

    #[test]
    fn visible_topic_cloud_caps_initial_render() {
        let topics = (0..300).map(topic).collect::<Vec<_>>();

        let (visible, limited) = visible_topic_cloud(&topics);

        assert_eq!(visible.len(), TOPIC_CLOUD_RENDER_LIMIT);
        assert!(limited);
    }

    #[test]
    fn visible_topic_cloud_keeps_small_topic_sets_complete() {
        let topics = (0..12).map(topic).collect::<Vec<_>>();

        let (visible, limited) = visible_topic_cloud(&topics);

        assert_eq!(visible.len(), topics.len());
        assert!(!limited);
    }
}
