use std::ffi::{CStr, CString, c_char};
use std::ptr;

use bm_rewrite_a_module::{BM_ERR_INVALID_URL, BM_OK, bm_normalize_url, tag_normalize};

fn output(buffer: &[c_char]) -> &CStr {
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
}

#[test]
fn normalizes_urls_in_the_callers_buffer() {
    let input = CString::new("HTTPS://Rust-Lang.ORG/Book").unwrap();
    let mut buffer = [0; 64];

    let result = unsafe { bm_normalize_url(input.as_ptr(), buffer.as_mut_ptr(), buffer.len()) };

    assert_eq!(result, BM_OK);
    assert_eq!(output(&buffer).to_bytes(), b"https://rust-lang.org/book");
}

#[test]
fn rejects_invalid_url_inputs() {
    let input = CString::new("https://example.com").unwrap();
    let with_tab = CString::new("https://example.com/a\tb").unwrap();
    let mut buffer = [0; 8];

    assert_eq!(
        unsafe { bm_normalize_url(ptr::null(), buffer.as_mut_ptr(), buffer.len()) },
        BM_ERR_INVALID_URL
    );
    assert_eq!(
        unsafe { bm_normalize_url(input.as_ptr(), ptr::null_mut(), buffer.len()) },
        BM_ERR_INVALID_URL
    );
    assert_eq!(
        unsafe { bm_normalize_url(input.as_ptr(), buffer.as_mut_ptr(), buffer.len()) },
        BM_ERR_INVALID_URL
    );
    assert_eq!(
        unsafe { bm_normalize_url(with_tab.as_ptr(), buffer.as_mut_ptr(), buffer.len()) },
        BM_ERR_INVALID_URL
    );
}

#[test]
fn trims_and_normalizes_tags_in_the_callers_buffer() {
    let input = CString::new("  RuSt ignored").unwrap();
    let mut buffer = [0; 16];

    let result = unsafe { tag_normalize(input.as_ptr(), buffer.as_mut_ptr(), buffer.len()) };

    assert_eq!(result, BM_OK);
    assert_eq!(output(&buffer).to_bytes(), b"rust");
}

#[test]
fn rejects_invalid_tag_inputs() {
    let empty = CString::new("   ").unwrap();
    let too_long = CString::new("rustacean").unwrap();
    let mut buffer = [0; 5];

    assert_eq!(
        unsafe { tag_normalize(ptr::null(), buffer.as_mut_ptr(), buffer.len()) },
        BM_ERR_INVALID_URL
    );
    assert_eq!(
        unsafe { tag_normalize(empty.as_ptr(), buffer.as_mut_ptr(), buffer.len()) },
        BM_ERR_INVALID_URL
    );
    assert_eq!(
        unsafe { tag_normalize(too_long.as_ptr(), buffer.as_mut_ptr(), buffer.len()) },
        BM_ERR_INVALID_URL
    );
}
