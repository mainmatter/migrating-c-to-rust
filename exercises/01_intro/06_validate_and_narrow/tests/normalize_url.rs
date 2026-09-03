extern crate bm_validate_and_narrow;

use core::ptr;
use std::ffi::{CStr, CString, c_char};

const BM_OK: i32 = 0;
const BM_ERR_INVALID_URL: i32 = 3;
const BM_ERR_INVALID_BUFFER: i32 = 6;
const BM_ERR_BUFFER_TOO_SMALL: i32 = 7;

extern "C" {
    fn bm_normalize_url(url: *const c_char, out: *mut c_char, out_len: usize) -> i32;
}

/// Calls `bm_normalize_url`, using `out`'s length as the reported capacity.
///
/// The null-argument cases can't go through here -- they have no `&CStr` or no
/// buffer to hand over -- so they call the function directly.
fn normalize(url: &CStr, out: &mut [u8]) -> i32 {
    unsafe { bm_normalize_url(url.as_ptr(), out.as_mut_ptr().cast::<c_char>(), out.len()) }
}

#[test]
fn lowercases_ascii() {
    let input = CString::new("https://Rust-Lang.ORG").unwrap();
    let mut output = vec![0u8; 64];

    let ret = normalize(&input, &mut output);

    assert_eq!(ret, BM_OK);
    assert_eq!(
        CStr::from_bytes_until_nul(&output).unwrap(),
        c"https://rust-lang.org"
    );
}

#[test]
fn null_url_is_rejected() {
    let mut output = vec![0u8; 64];

    let ret = unsafe {
        bm_normalize_url(
            ptr::null(),
            output.as_mut_ptr().cast::<c_char>(),
            output.len(),
        )
    };

    assert_eq!(ret, BM_ERR_INVALID_URL);
}

#[test]
fn null_out_is_rejected() {
    let input = CString::new("https://example.com").unwrap();
    let ret = unsafe { bm_normalize_url(input.as_ptr(), ptr::null_mut(), 64) };

    assert_eq!(ret, BM_ERR_INVALID_BUFFER);
}

#[test]
fn buffer_too_small_is_rejected() {
    let input = CString::new("https://example.com").unwrap();
    let mut output = vec![0u8; 4];

    assert_eq!(normalize(&input, &mut output), BM_ERR_BUFFER_TOO_SMALL);
}

#[test]
fn embedded_tab_is_rejected() {
    let input = CString::new("https://example.com/a\tb").unwrap();
    let mut output = vec![0u8; 64];

    assert_eq!(normalize(&input, &mut output), BM_ERR_INVALID_URL);
}

#[test]
fn non_utf8_is_rejected() {
    // NUL-terminated, so a valid C string, even though it is not valid UTF-8.
    let raw = [b'h', b't', b't', b':', 0x80, 0];
    let input = CStr::from_bytes_with_nul(&raw).unwrap();
    let mut output = vec![0u8; 64];

    assert_eq!(normalize(input, &mut output), BM_ERR_INVALID_URL);
}

#[test]
fn a_buffer_that_fits_exactly_is_accepted() {
    let input = CString::new("https://example.com").unwrap();
    // Room for every byte plus the NUL terminator, and not one byte more.
    let mut output = vec![0u8; input.as_bytes().len() + 1];

    let ret = normalize(&input, &mut output);

    assert_eq!(ret, BM_OK);
    assert_eq!(
        CStr::from_bytes_until_nul(&output).unwrap(),
        c"https://example.com"
    );
}

#[test]
fn a_buffer_one_byte_short_is_rejected() {
    let input = CString::new("https://example.com").unwrap();
    // Room for the bytes but not for the NUL terminator, which still has to
    // fit: forgetting it is an off-by-one that writes past the end.
    let mut output = vec![0u8; input.as_bytes().len()];

    assert_eq!(normalize(&input, &mut output), BM_ERR_BUFFER_TOO_SMALL);
}

#[test]
fn a_zero_length_buffer_is_rejected() {
    let input = CString::new("https://example.com").unwrap();
    let mut output: Vec<u8> = Vec::new();

    assert_eq!(normalize(&input, &mut output), BM_ERR_BUFFER_TOO_SMALL);
}

#[test]
fn a_null_url_is_reported_before_a_null_buffer() {
    let ret = unsafe { bm_normalize_url(ptr::null(), ptr::null_mut(), 64) };

    assert_eq!(ret, BM_ERR_INVALID_URL);
}

#[test]
fn url_safe_punctuation_is_accepted() {
    // Every one of `@ _ - ? = & # [ ] %` is legal in a URL, so an allowlist
    // that only knows about letters, digits, `:`, `/`, `.` and `-` is too
    // strict.
    let input = CString::new("HTTPS://USER@EXAMPLE.COM/P_a-TH?Q=1&R=2#FRAG[0]%20").unwrap();
    let mut output = vec![0u8; 128];

    let ret = normalize(&input, &mut output);

    assert_eq!(ret, BM_OK);
    assert_eq!(
        CStr::from_bytes_until_nul(&output).unwrap(),
        c"https://user@example.com/p_a-th?q=1&r=2#frag[0]%20"
    );
}

#[test]
fn characters_outside_the_allowlist_are_rejected() {
    // None of these may appear in a URL unescaped, so an allowlist as loose as
    // `u8::is_ascii_graphic` is not enough either.
    for bad in [b' ', b'"', b'<', b'>', b'\\', b'\n'] {
        let mut raw = b"https://example.com/".to_vec();
        raw.push(bad);
        raw.push(b'b');
        let input = CString::new(raw).unwrap();
        let mut output = vec![0u8; 64];

        assert_eq!(
            normalize(&input, &mut output),
            BM_ERR_INVALID_URL,
            "byte {bad:#04x} should not be accepted in a URL"
        );
    }
}
