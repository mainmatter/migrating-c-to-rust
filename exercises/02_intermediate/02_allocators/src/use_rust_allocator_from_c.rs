//! Part 2: use Rust's allocator from C.
//!
//! C receives an opaque pointer to a Rust-owned `RustBookmark`. It must return
//! that pointer to Rust for deallocation instead of passing it to C's `free`.
//!
//! Replace the two numbered `todo!()` calls in this file. Everything else is
//! provided.

use crate::given::RustBookmark;
use std::ffi::c_char;
use std::ptr::{self, NonNull};

/// Allocate a Rust-owned bookmark and transfer ownership to the C caller.
///
/// # Safety
///
/// `url` must point to a valid NUL-terminated string. When `n_tags > 0`, `tags`
/// must point to an array of `n_tags` pointers to valid NUL-terminated strings.
/// The returned pointer must be released exactly once with
/// [`rust_bookmark_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_bookmark_new(
    url: *const c_char,
    tags: *const *const c_char,
    n_tags: usize,
) -> *mut RustBookmark {
    // GIVEN: validate and copy the C inputs into a fully owned Rust value.
    let Some(bookmark) = (unsafe { RustBookmark::copy_from_c(url, tags, n_tags) }) else {
        return ptr::null_mut();
    };

    // TODO 1 OF 2:
    // Allocate `bookmark` with `Box`, then transfer ownership to C by
    // returning a raw pointer.

    todo!()
}

/// Release a bookmark previously returned by [`rust_bookmark_new`].
///
/// Like C's `free`, this function accepts null.
///
/// # Safety
///
/// A non-null `bookmark` must have been returned by [`rust_bookmark_new`] and
/// must not have been freed already. It must not be used after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_bookmark_free(bookmark: Option<NonNull<RustBookmark>>) {
    // GIVEN: match C's `free(NULL)` behavior.
    let Some(bookmark) = bookmark else {
        return;
    };

    // TODO 2 OF 2:
    // Take ownership of the allocation back from C and let Rust drop it.

    todo!()
}
