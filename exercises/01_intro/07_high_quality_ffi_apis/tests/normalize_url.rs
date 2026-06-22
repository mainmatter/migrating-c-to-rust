extern crate bm_high_quality_ffi_apis;

use core::ptr;
use std::ffi::{CStr, CString, c_char};

const BM_OK: i32 = 0;
const BM_ERR_INVALID_URL: i32 = 3;
const BM_ERR_INVALID_BUFFER: i32 = 3;
const BM_ERR_BUFFER_TOO_SMALL: i32 = 6;

extern "C" {
    fn bm_normalize_url(url: *const c_char, out: *mut c_char, out_len: usize) -> i32;
}

#[test]
fn lowercases_ascii() {
    let input = CString::new("https://Rust-Lang.ORG").unwrap();
    let mut output = vec![0u8; 64];

    let ret = unsafe {
        bm_normalize_url(
            input.as_ptr(),
            output.as_mut_ptr().cast::<c_char>(),
            output.len(),
        )
    };

    let output = CStr::from_bytes_until_nul(output.as_slice()).unwrap();

    assert_eq!(ret, BM_OK);
    assert_eq!(output, c"https://rust-lang.org");
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

    let ret = unsafe {
        bm_normalize_url(
            input.as_ptr(),
            output.as_mut_ptr().cast::<c_char>(),
            output.len(),
        )
    };
    assert_eq!(ret, BM_ERR_BUFFER_TOO_SMALL);
}

#[test]
fn embedded_tab_is_rejected() {
    let input = CString::new("https://example.com/a\tb").unwrap();
    let mut output = vec![0u8; 64];

    let ret = unsafe {
        bm_normalize_url(
            input.as_ptr(),
            output.as_mut_ptr().cast::<c_char>(),
            output.len(),
        )
    };
    assert_eq!(ret, BM_ERR_INVALID_URL);
}

#[test]
fn non_utf8_is_rejected() {
    let input: Vec<u8> = vec![b'h', b't', b't', b':', 0x80];
    let mut output = vec![0u8; 64];

    let ret = unsafe {
        bm_normalize_url(
            input.as_ptr().cast::<c_char>(),
            output.as_mut_ptr().cast::<c_char>(),
            output.len(),
        )
    };
    assert_eq!(ret, BM_ERR_INVALID_URL);
}
