//! Part 1: use C's allocator from Rust.
//!
//! The `bookmark_new` function allocates a bookmark in C. Rust wraps its
//! pointer in `OwnedCBookmark`, which must return it to `bookmark_free` when the
//! wrapper is dropped.
//!
//! Replace the two numbered `todo!()` calls in this file. The FFI declarations,
//! data layout, accessors, C implementation, and tests are provided elsewhere.

#![cfg(test)] // This module is only used by the test harness.

use crate::given::{OwnedCBookmark, bookmark_free, bookmark_new};
use std::ffi::CStr;
use std::ptr::{self, NonNull};

impl OwnedCBookmark {
    /// Ask the legacy C API to allocate a bookmark.
    pub fn new(url: &CStr) -> Option<Self> {
        // TODO 1 OF 2:
        // Call the C `bookmark_new` constructor and turn the returned raw pointer into an
        // `Option<OwnedCBookmark>`.
        //
        // Hint 1: `OwnedCBookmark` takes a `NonNull<CBookmark>`
        // Hint 2: You don't need to worry about the tags for this exercise. You can pass a null pointer and zero for the tag array and count.
        todo!()
    }
}

impl Drop for OwnedCBookmark {
    fn drop(&mut self) {
        let bookmark = self.0;

        // TODO 2 OF 2:
        // Return the pointer to the C function that owns the deallocation API.
        //
        // Hint: take the pointer out of the `bookmark` `NonNull` wrapper and free it with a C function
        todo!()
    }
}
