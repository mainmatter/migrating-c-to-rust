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

// Pull in exercise 2.11. Only crates you use get linked, so without this line
// `bm_normalize_url` would be missing from the static library.
pub use bm_high_quality_ffi_apis::bm_normalize_url;

/// Mirrors `BM_OK` in `exercises/_bm/src/result.h`.
pub const BM_OK: i32 = 0;
/// Mirrors `BM_ERR_INVALID_URL` in `exercises/_bm/src/result.h`.
pub const BM_ERR_INVALID_URL: i32 = 3;

/// The bytes C's `isspace` accepts in the "C" locale. `u8::is_ascii_whitespace`
/// leaves out `\v`, so it isn't a drop-in replacement.
const fn is_c_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// Trim, lowercase, and reject empty tags, as `tag_normalize` in
/// `normalize.c` does.
///
/// # Safety
///
/// If `raw` is non-null, it must point to a NUL-terminated C string. If `out`
/// is non-null, it must be valid for writes of `out_len` bytes and must not
/// overlap `raw`. `out` may be uninitialized: `bm` passes stack buffers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tag_normalize(
    raw: *const c_char,
    out: *mut c_char,
    out_len: usize,
) -> i32 {
    if raw.is_null() || out.is_null() || out_len == 0 {
        return BM_ERR_INVALID_URL;
    }

    // SAFETY: `raw` is non-null, and the caller guarantees it is NUL-terminated.
    let raw = unsafe { std::ffi::CStr::from_ptr(raw) }.to_bytes();

    let tag = raw
        .iter()
        .copied()
        .skip_while(|&b| is_c_space(b))
        .take_while(|&b| !is_c_space(b) && b != b',');

    let mut n = 0;
    for b in tag {
        // Keep room for the NUL terminator.
        if n + 1 >= out_len {
            return BM_ERR_INVALID_URL;
        }
        // SAFETY: `n < out_len`, and the caller guarantees `out` is valid for
        // writes of `out_len` bytes. Writing through the pointer, instead of
        // making a `&mut [u8]`, is fine for uninitialized memory.
        unsafe { out.add(n).write(b.to_ascii_lowercase() as c_char) };
        n += 1;
    }
    // SAFETY: `n < out_len`, by the check in the loop or because `out_len > 0`.
    unsafe { out.add(n).write(0) };

    if n == 0 { BM_ERR_INVALID_URL } else { BM_OK }
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
