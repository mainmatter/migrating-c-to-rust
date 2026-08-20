//! Provided FFI plumbing for the allocator exercise.
//!
//! Learners do not need to change this file.

use std::ffi::{CStr, CString, c_char};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A Rust-owned type that C may hold only through an opaque pointer.
///
/// Its fields deliberately use Rust-owned containers. C must never call
/// `free` on this value; it must return the pointer to `rust_bookmark_free`.
///
/// Note: in production code you would likely not use `CString` and `Vec<CString>` for the
/// fields, but this is a simple example to illustrate the FFI boundary and ownership transfer.
pub struct RustBookmark {
    url: CString,
    tags: Vec<CString>,
}

static RUST_BOOKMARKS_ALIVE: AtomicUsize = AtomicUsize::new(0);

impl RustBookmark {
    /// Validate and copy the FFI inputs into Rust-owned values.
    ///
    /// # Safety
    ///
    /// A non-null `url` must point to a valid C string. When `n_tags > 0`,
    /// `tags` must point to `n_tags` pointers to valid C strings.
    pub(crate) unsafe fn copy_from_c(
        url: *const c_char,
        tags: *const *const c_char,
        n_tags: usize,
    ) -> Option<Self> {
        if url.is_null() || (n_tags > 0 && tags.is_null()) {
            return None;
        }

        // SAFETY: the caller guarantees that a non-null URL is a valid C
        // string.
        let url = unsafe { CStr::from_ptr(url) }.to_owned();

        let tag_pointers = if n_tags == 0 {
            &[]
        } else {
            // SAFETY: checked non-null above; the caller guarantees the array
            // contains `n_tags` elements.
            unsafe { std::slice::from_raw_parts(tags, n_tags) }
        };

        let mut owned_tags = Vec::with_capacity(n_tags);
        for &tag in tag_pointers {
            if tag.is_null() {
                return None;
            }
            // SAFETY: the caller guarantees that each non-null tag is a valid
            // C string.
            owned_tags.push(unsafe { CStr::from_ptr(tag) }.to_owned());
        }

        RUST_BOOKMARKS_ALIVE.fetch_add(1, Ordering::Relaxed);
        Some(Self {
            url,
            tags: owned_tags,
        })
    }
}

impl Drop for RustBookmark {
    fn drop(&mut self) {
        RUST_BOOKMARKS_ALIVE.fetch_sub(1, Ordering::Relaxed);
    }
}

// These accessors let the C verification harness inspect the opaque Rust value.

/// Borrow the bookmark's URL, or return null for a null bookmark.
///
/// # Safety
///
/// A non-null `bookmark` must point to a live `RustBookmark`. The returned
/// pointer is valid only while that bookmark remains alive.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_bookmark_url(
    bookmark: Option<NonNull<RustBookmark>>,
) -> *const c_char {
    bookmark
        .map(|bookmark| unsafe { bookmark.as_ref() }.url.as_ptr())
        .unwrap_or(ptr::null())
}

/// Return the number of tags, or zero for a null bookmark.
///
/// # Safety
///
/// A non-null `bookmark` must point to a live `RustBookmark`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_bookmark_tag_count(bookmark: Option<NonNull<RustBookmark>>) -> usize {
    bookmark
        .map(|bookmark| unsafe { bookmark.as_ref() }.tags.len())
        .unwrap_or(0)
}

/// Borrow the tag at `index`, or return null if unavailable.
///
/// # Safety
///
/// A non-null `bookmark` must point to a live `RustBookmark`. The returned
/// pointer is valid only while that bookmark remains alive.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_bookmark_tag(
    bookmark: Option<NonNull<RustBookmark>>,
    index: usize,
) -> *const c_char {
    bookmark
        .and_then(|bookmark| unsafe { bookmark.as_ref() }.tags.get(index))
        .map(|tag| tag.as_ptr())
        .unwrap_or(ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_bookmark_live_count() -> usize {
    RUST_BOOKMARKS_ALIVE.load(Ordering::Relaxed)
}

/// The FFI-safe layout of the legacy C `Bookmark`.
#[repr(C)]
pub(crate) struct CBookmark {
    url: *mut c_char,
    tags: *mut *mut c_char,
    n_tags: usize,
}

#[allow(dead_code)]
unsafe extern "C" {
    pub(crate) fn bookmark_new(
        url: *const c_char,
        tags: *const *const c_char,
        n_tags: usize,
    ) -> *mut CBookmark;
    pub(crate) fn bookmark_free(bookmark: *mut CBookmark);
    pub(crate) fn bookmark_live_count() -> usize;
}

/// An owning Rust wrapper around a C-allocated `Bookmark`.
///
/// Its private field prevents safe callers from extracting the pointer and
/// accidentally freeing it through Rust's allocator.
#[cfg(test)]
pub(crate) struct OwnedCBookmark(pub(crate) NonNull<CBookmark>);

#[cfg(test)]
impl OwnedCBookmark {
    pub fn url(&self) -> &CStr {
        // SAFETY: `self.0` points to a live C Bookmark. Its URL is a valid C
        // string owned by that Bookmark and lives for at least as long as self.
        unsafe { CStr::from_ptr(self.0.as_ref().url) }
    }
}
