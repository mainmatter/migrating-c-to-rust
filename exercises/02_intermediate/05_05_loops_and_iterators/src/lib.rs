//! Express what each traversal computes with an iterator adapter.
//!
//! The starter functions are correct, so their tests pass. `wr` also runs
//! Clippy with warnings denied; replace every index-based loop with the
//! iterator operation named in the function's TODO.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bookmark {
    pub url: String,
    pub tags: Vec<String>,
}

pub fn any_tag_contains(bookmark: &Bookmark, query: &str) -> bool {
    // TODO: Express the early boolean return with `any`.
    for index in 0..bookmark.tags.len() {
        if bookmark.tags[index].contains(query) {
            return true;
        }
    }
    false
}

pub fn position_of_url(bookmarks: &[Bookmark], url: &str) -> Option<usize> {
    // TODO: Express the search for an index with `position`.
    for index in 0..bookmarks.len() {
        if bookmarks[index].url == url {
            return Some(index);
        }
    }
    None
}

pub fn count_matching_tags(bookmark: &Bookmark, query: &str) -> usize {
    // TODO: Express the condition with `filter` and consume it with `count`.
    let mut matches = 0;
    for index in 0..bookmark.tags.len() {
        if bookmark.tags[index].contains(query) {
            matches += 1;
        }
    }
    matches
}

pub fn urls_with_tag<'a>(bookmarks: &'a [Bookmark], tag: &str) -> Vec<&'a str> {
    // TODO: Use `filter`, `map`, and `collect`, with `any` for the nested tag
    // check. Preserve input order and return each bookmark at most once.
    let mut urls = Vec::new();
    for index in 0..bookmarks.len() {
        if bookmarks[index].tags.iter().any(|item| item == tag) {
            urls.push(bookmarks[index].url.as_str());
        }
    }
    urls
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
