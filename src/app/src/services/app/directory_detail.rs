use super::AppServices;
use crate::types::{AuthorSummary, BookmarkPage, BookmarkStats, TopicSummary};
use anyhow::{Context, Result};

impl AppServices {
    pub fn author_index(&self) -> Result<Vec<AuthorSummary>> {
        self.db
            .get_all_authors()
            .context("failed to load author index")
            .map(|authors| {
                authors
                    .into_iter()
                    .map(|author| AuthorSummary {
                        handle: author.handle,
                        name: author.name,
                        profile_image: author.profile_image,
                        bookmark_count: author.bookmark_count,
                        favorite_count: author.favorite_count,
                    })
                    .collect()
            })
    }

    pub fn topic_index(&self) -> Result<Vec<TopicSummary>> {
        self.db
            .get_all_tags()
            .context("failed to load topic index")
            .map(|topics| {
                topics
                    .into_iter()
                    .map(|(tag, bookmark_count)| TopicSummary {
                        tag,
                        bookmark_count,
                    })
                    .collect()
            })
    }

    pub fn bookmark_detail(&self, id: &str) -> Result<Option<eterea_core::Bookmark>> {
        self.db
            .get_bookmark(id)
            .with_context(|| format!("failed to load bookmark detail for {id}"))
    }

    pub fn bookmarks_by_author(
        &self,
        handle: &str,
        offset: usize,
        limit: usize,
    ) -> Result<BookmarkPage> {
        let (items, total) = self
            .db
            .search_with_filters_page(
                None,
                None,
                Some(handle),
                None,
                None,
                false,
                None,
                offset,
                limit,
            )
            .with_context(|| format!("failed to load bookmarks for author {handle}"))?;
        Ok(BookmarkPage::new(items, total, offset, limit))
    }

    pub fn bookmarks_by_tag(&self, tag: &str, offset: usize, limit: usize) -> Result<BookmarkPage> {
        let (items, total) = self
            .db
            .search_with_filters_page(
                None,
                Some(tag),
                None,
                None,
                None,
                false,
                None,
                offset,
                limit,
            )
            .with_context(|| format!("failed to load bookmarks for tag {tag}"))?;
        Ok(BookmarkPage::new(items, total, offset, limit))
    }

    pub fn stats(&self) -> Result<BookmarkStats> {
        self.db.get_stats().context("failed to load bookmark stats")
    }
}
