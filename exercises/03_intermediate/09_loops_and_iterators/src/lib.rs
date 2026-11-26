//! Express what each traversal computes with an iterator adapter.
//!
//! The starter functions are correct, so their tests pass. `wr` also runs
//! Clippy with warnings denied; replace every index-based loop with the
//! iterator operation named in the comment of each function.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bookmark {
    pub url: String,
    pub tags: Vec<String>,
}

pub fn any_tag_contains(bookmark: &Bookmark, query: &str) -> bool {
    // The early boolean return, expressed with `any`.
    bookmark.tags.iter().any(|tag| tag.contains(query))
}

pub fn position_of_url(bookmarks: &[Bookmark], url: &str) -> Option<usize> {
    // The search for an index, expressed with `position`.
    bookmarks.iter().position(|bookmark| bookmark.url == url)
}

pub fn count_matching_tags(bookmark: &Bookmark, query: &str) -> usize {
    // The condition, expressed with `filter` and consumed with `count`.
    bookmark
        .tags
        .iter()
        .filter(|tag| tag.contains(query))
        .count()
}

pub fn urls_with_tag<'a>(bookmarks: &'a [Bookmark], tag: &str) -> Vec<&'a str> {
    // `filter`, `map`, and `collect`, with `any` for the nested tag check. This
    // preserves input order and returns each bookmark at most once.
    bookmarks
        .iter()
        .filter(|bookmark| bookmark.tags.iter().any(|item| item == tag))
        .map(|bookmark| bookmark.url.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bookmark(url: &str, tags: &[&str]) -> Bookmark {
        Bookmark {
            url: url.into(),
            tags: tags.iter().map(|tag| (*tag).into()).collect(),
        }
    }

    #[test]
    fn detects_any_matching_tag() {
        let item = bookmark("a", &["rust", "ffi", "migration"]);
        assert!(any_tag_contains(&item, "fi"));
        assert!(!any_tag_contains(&item, "java"));
        assert!(!any_tag_contains(&bookmark("empty", &[]), "rust"));
    }

    #[test]
    fn finds_the_first_url_position() {
        let items = vec![bookmark("a", &[]), bookmark("b", &[]), bookmark("b", &[])];
        assert_eq!(position_of_url(&items, "b"), Some(1));
        assert_eq!(position_of_url(&items, "missing"), None);
        assert_eq!(position_of_url(&[], "missing"), None);
    }

    #[test]
    fn counts_every_matching_tag() {
        let item = bookmark("a", &["rust", "trust", "ffi", "rust"]);
        assert_eq!(count_matching_tags(&item, "rust"), 3);
        assert_eq!(count_matching_tags(&item, "none"), 0);
    }

    #[test]
    fn collects_urls_once_in_input_order() {
        let items = vec![
            bookmark("first", &["rust", "rust"]),
            bookmark("second", &["c"]),
            bookmark("third", &["rust"]),
        ];
        assert_eq!(urls_with_tag(&items, "rust"), vec!["first", "third"]);
        assert!(urls_with_tag(&items, "missing").is_empty());
    }
}
