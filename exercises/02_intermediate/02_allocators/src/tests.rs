//! Verification for the C-allocated Rust wrapper.
//!
//! Learners do not need to change this file.

use crate::given::OwnedCBookmark;
use crate::given::bookmark_live_count;

#[test]
fn rust_wrapper_uses_the_c_allocation_api() {
    // SAFETY: the counter function takes no pointers and has no preconditions.
    let before = unsafe { bookmark_live_count() };

    {
        let bookmark = OwnedCBookmark::new(c"https://example.com").unwrap();

        assert_eq!(bookmark.url(), c"https://example.com");

        // SAFETY: the counter function takes no pointers and has no
        // preconditions.
        assert_eq!(unsafe { bookmark_live_count() }, before + 1);
    }

    // `Drop` must return the allocation to C.
    // SAFETY: the counter function takes no pointers and has no preconditions.
    assert_eq!(unsafe { bookmark_live_count() }, before);
}
