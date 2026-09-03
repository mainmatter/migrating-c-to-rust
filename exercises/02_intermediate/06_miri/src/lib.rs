//! Rust-side wrapper around the not-yet-ported `tag_normalize` from the legacy
//! C code (`c_src/tag.c`).
//!
//! The tests below pass with a plain `cargo test` but fail under Miri:
//!
//! ```text
//! cargo miri test -p bm_miri
//! ```
//!
//! Your job, in two steps:
//!
//! 1. Write the Miri stub. define `tag_normalize` *in Rust*, with
//!    the exact same signature as the extern declaration below. Marking it
//!    `#[unsafe(no_mangle)]` makes it *the* `tag_normalize` symbol, so Miri's
//!    call into the extern block lands in your function. `#[cfg(miri)]` keeps it
//!    out of the real build. Remember the stub does not need to be a faithful port.
//!    It needs to honour the contract in `c_src/tag.h` as far as the tests exercise it.
//!
//! 2. With the stub in place Miri gets past the call — and reports a bug in the
//!    "first draft" wrapper below that the native tests never noticed. Read the
//!    report (the backtrace points at the allocation) and fix `normalize`.

use std::ffi::{c_char, CStr, CString};

/// Mirrors `BmResult` in `c_src/tag.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BmResult {
    Ok = 0,
    ErrNotFound,
    ErrDuplicate,
    ErrInvalidUrl,
    ErrIo,
    ErrCorrupt,
}

/// Mirrors `BM_MAX_TAG_LEN` in `c_src/tag.h`.
pub const BM_MAX_TAG_LEN: usize = 64;

unsafe extern "C" {
    fn tag_normalize(raw: *const c_char, out: *mut c_char, out_len: usize) -> BmResult;
}

/// Normalize a tag through the legacy C implementation.
pub fn normalize(raw: &str) -> Result<String, BmResult> {
    // Hand the string over to C. `into_raw` gives us a stable `*mut c_char`
    // that C is free to read for as long as it likes.
    let raw = CString::new(raw).map_err(|_| BmResult::ErrInvalidUrl)?;

    let mut out = [0 as c_char; BM_MAX_TAG_LEN];

    // SAFETY: `raw` is a NUL-terminated C string that outlives the call, and
    // `out` is valid for writes of `out.len()` bytes.
    let result = unsafe { tag_normalize(raw.as_ptr(), out.as_mut_ptr(), out.len()) };
    if result != BmResult::Ok {
        return Err(result);
    }

    // SAFETY: on success C wrote a NUL-terminated string into `out`.
    let normalized = unsafe { CStr::from_ptr(out.as_ptr()) };
    normalized
        .to_str()
        .map(str::to_owned)
        .map_err(|_| BmResult::ErrInvalidUrl)
}

#[cfg(miri)]
mod stubs {
    use super::*;

    /// A Rust stand-in for `tag_normalize` from `c_src/tag.c`.
    ///
    /// # Safety
    ///
    /// Same contract as `tag_normalize`: `raw` must be NUL-terminated and `out`
    /// must be valid for writes of `out_len` bytes.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn tag_normalize(
        raw: *const c_char,
        out: *mut c_char,
        out_len: usize,
    ) -> BmResult {
        if raw.is_null() || out.is_null() || out_len == 0 {
            return BmResult::ErrInvalidUrl;
        }

        // SAFETY: the caller guarantees `raw` points at a NUL-terminated string.
        let raw = unsafe { CStr::from_ptr(raw) };

        // Skip leading whitespace, then take everything up to the next
        // whitespace or ',', lowercased.
        let tag: Vec<c_char> = raw
            .to_bytes()
            .iter()
            .copied()
            .skip_while(u8::is_ascii_whitespace)
            .take_while(|b| !b.is_ascii_whitespace() && *b != b',')
            .map(|b| b.to_ascii_lowercase() as c_char)
            .collect();

        // `out_len` includes the NUL, so a tag of exactly `out_len` bytes does
        // not fit. An empty result is an error too.
        if tag.is_empty() || tag.len() >= out_len {
            return BmResult::ErrInvalidUrl;
        }

        // SAFETY: `tag.len() < out_len` was just checked, and the caller
        // guarantees `out` is valid for writes of `out_len` bytes.
        unsafe { std::ptr::copy_nonoverlapping(tag.as_ptr(), out, tag.len()) };
        // SAFETY: `tag.len() < out_len`, so this index is still inside `out`.
        unsafe { out.add(tag.len()).write(0) };

        BmResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercases() {
        assert_eq!(normalize("Rust").as_deref(), Ok("rust"));
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(normalize(""), Err(BmResult::ErrInvalidUrl));
    }

    #[test]
    fn rejects_too_long() {
        let too_long = "a".repeat(BM_MAX_TAG_LEN);
        assert_eq!(normalize(&too_long), Err(BmResult::ErrInvalidUrl));
    }
}
