use std::{
    ffi::{CStr, c_char},
    ptr::NonNull,
};

#[repr(i32)]
pub enum NormalizeUrlOutcome {
    Success,
    InvalidUrl = 3,
    InvalidOutBuffer = 6,
    OutBufferIsTooShort = 7,
}

/// Returns whether `b` may appear in a URL unescaped (RFC 3986 unreserved,
/// reserved, and the `%` of a percent-escape).
const fn is_url_safe(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
        | b'-' | b'.' | b'_' | b'~'
        | b':' | b'/' | b'?' | b'#' | b'[' | b']' | b'@'
        | b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
        | b'%')
}

/// Normalize `url` (lowercase ASCII) into the caller-provided `out` buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bm_normalize_url(
    url: Option<NonNull<c_char>>,
    out: Option<NonNull<c_char>>,
    out_len: usize,
) -> NormalizeUrlOutcome {
    use NormalizeUrlOutcome::*;

    let Some(url) = url else { return InvalidUrl };
    let url = unsafe { CStr::from_ptr(url.as_ptr()) };
    let Some(out) = out else {
        return InvalidOutBuffer;
    };
    let output_slice: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(out.as_ptr().cast(), out_len) };

    let url = url.to_bytes();
    for (i, b) in url.iter().enumerate() {
        let Some(slot) = output_slice.get_mut(i) else {
            return OutBufferIsTooShort;
        };
        if !is_url_safe(*b) {
            return InvalidUrl;
        }
        *slot = b.to_ascii_lowercase();
    }

    // Null termination
    let Some(slot) = output_slice.get_mut(url.len()) else {
        return OutBufferIsTooShort;
    };
    *slot = 0;

    Success
}
