//! Replace `exercises/_bm/src/normalize.c` with Rust.
//!
//! `normalize.c` defines two functions, so this crate has to export two
//! symbols before `bm` links without it. Three steps get you there:
//!
//! 1. configure this crate as a `staticlib` in `Cargo.toml`;
//! 2. bring in `bm_normalize_url` from your solution to exercise 2.11, which
//!    `Cargo.toml` already lists as a dependency;
//! 3. export `tag_normalize` with the ABI and symbol name that `normalize.h`
//!    declares, and port its body from `normalize.c`.
//!
//! The tests in `tests/bm.rs` build `bm` twice, once all-C and once with this
//! crate in place of `normalize.c`, and check that both print the same thing on
//! the fixtures.

use std::ffi::c_char;

// TODO 2 OF 3: Bring in `bm_normalize_url` from your exercise 2.11 crate.

/// Mirrors `BM_OK` in `exercises/_bm/src/result.h`.
pub const BM_OK: i32 = 0;
/// Mirrors `BM_ERR_INVALID_URL` in `exercises/_bm/src/result.h`.
pub const BM_ERR_INVALID_URL: i32 = 3;

/// Trim, lowercase, and reject empty tags, as `tag_normalize` in
/// `normalize.c` does.
// TODO 3 OF 3: Export this function with the ABI and symbol name that
// `normalize.h` declares, then port its body from `normalize.c`.
pub unsafe fn tag_normalize(_raw: *const c_char, _out: *mut c_char, _out_len: usize) -> i32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, c_char};
    use std::ptr;

    use super::{BM_ERR_INVALID_URL, BM_OK, tag_normalize};

    /// Runs `tag_normalize` on `raw` with a buffer of `out_len` bytes.
    fn normalize(raw: &CStr, out_len: usize) -> (i32, Vec<u8>) {
        let mut out = vec![0xAA_u8; out_len];
        // SAFETY: `raw` is NUL-terminated and `out` holds `out_len` bytes.
        let result = unsafe { tag_normalize(raw.as_ptr(), out.as_mut_ptr().cast(), out_len) };
        (result, out)
    }

    fn normalized(raw: &CStr) -> Vec<u8> {
        let (result, out) = normalize(raw, 64);
        assert_eq!(result, BM_OK, "{raw:?} should be accepted");
        CStr::from_bytes_until_nul(&out)
            .unwrap()
            .to_bytes()
            .to_vec()
    }

    #[test]
    fn lowercases() {
        assert_eq!(normalized(c"RuSt"), b"rust");
    }

    #[test]
    fn skips_leading_whitespace_and_stops_at_the_next() {
        assert_eq!(normalized(c" \t rust lang"), b"rust");
    }

    #[test]
    fn stops_at_a_comma() {
        assert_eq!(normalized(c"rust,lang"), b"rust");
    }

    #[test]
    fn treats_a_vertical_tab_as_whitespace() {
        // C's `isspace` counts `\v`. `u8::is_ascii_whitespace` does not.
        assert_eq!(normalized(c"\x0brust\x0blang"), b"rust");
    }

    #[test]
    fn treats_every_c_whitespace_byte_as_whitespace() {
        // ' ', '\t', '\n', '\v', '\f' and '\r', as C's `isspace` does.
        assert_eq!(normalized(c" \t\n\x0b\x0c\rRust\x0clang"), b"rust");
        assert_eq!(normalized(c"rust\rlang"), b"rust");
    }

    #[test]
    fn uses_the_c_abi() {
        // Fails to compile unless `tag_normalize` is an `extern "C"` function.
        let _: unsafe extern "C" fn(*const c_char, *mut c_char, usize) -> i32 = tag_normalize;
    }

    #[test]
    fn rejects_an_empty_tag() {
        assert_eq!(normalize(c"", 64).0, BM_ERR_INVALID_URL);
        assert_eq!(normalize(c"  ", 64).0, BM_ERR_INVALID_URL);
        assert_eq!(normalize(c",rust", 64).0, BM_ERR_INVALID_URL);
    }

    #[test]
    fn needs_room_for_the_terminator() {
        assert_eq!(normalize(c"rust", 5).0, BM_OK);
        assert_eq!(normalize(c"rust", 4).0, BM_ERR_INVALID_URL);
        assert_eq!(normalize(c"rust", 0).0, BM_ERR_INVALID_URL);
    }

    #[test]
    fn rejects_null_pointers() {
        let mut out = [0 as c_char; 8];
        // SAFETY: null pointers are documented inputs.
        unsafe {
            assert_eq!(
                tag_normalize(ptr::null(), out.as_mut_ptr(), out.len()),
                BM_ERR_INVALID_URL
            );
            assert_eq!(
                tag_normalize(c"rust".as_ptr(), ptr::null_mut(), 8),
                BM_ERR_INVALID_URL
            );
        }
    }
}
