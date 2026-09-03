use std::ffi::{CStr, c_char};
use std::ptr::NonNull;

#[repr(i32)]
pub enum NormalizeUrlOutcome {
    Success = 0,
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
///
/// # Safety
///
/// The caller must guarantee all of the following:
///
/// 1. If `url` is `Some`, it points to a NUL-terminated C string; every byte
///    from `url` up to and including that terminator is valid for reads and
///    contained in a single allocated object, and nothing writes to those bytes
///    while the call runs.
/// 2. If `out` is `Some`, it points to `out_len` consecutive **initialized**
///    bytes that are valid for writes and contained in a single allocated
///    object, and nothing else reads or writes them while the call runs.
/// 3. The two regions described by (1) and (2) do not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bm_normalize_url(
    url: Option<NonNull<c_char>>,
    out: Option<NonNull<c_char>>,
    out_len: usize,
) -> NormalizeUrlOutcome {
    use NormalizeUrlOutcome::*;

    let Some(url) = url else {
        return InvalidUrl;
    };
    // SAFETY: clause (1) makes `url` sound to view as a `&CStr`: the scan for
    // the terminator stays in bounds, the region lies in one allocation, and
    // nothing writes to it while the borrow is live.
    let url = unsafe { CStr::from_ptr(url.as_ptr()) }.to_bytes();

    let Some(out) = out else {
        return InvalidOutBuffer;
    };

    // Validate the *whole* input before touching `out`. The C version
    // lowercases into the caller's buffer and only then looks for a tab, so it
    // scribbles over `out` on inputs it rejects; narrowing up front means a
    // rejected URL leaves the caller's buffer exactly as it was.
    if !url.iter().copied().all(is_url_safe) {
        return InvalidUrl;
    }

    // The normalized URL occupies `url.len()` bytes plus a NUL terminator, so
    // `out_len` has to be strictly greater than `url.len()`.
    if url.len() >= out_len {
        return OutBufferIsTooShort;
    }

    // SAFETY: clause (2) makes `out` sound to view as a `&mut [u8]` of
    // `out_len` bytes: valid for writes, initialized, within one allocation,
    // and untouched by anything else while the borrow is live. Clause (3) keeps
    // it disjoint from the `&CStr` above, so this `&mut` does not alias that
    // shared borrow.
    let out: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(out.as_ptr().cast(), out_len) };

    for (slot, b) in out.iter_mut().zip(url) {
        *slot = b.to_ascii_lowercase();
    }
    // In bounds: `url.len() < out_len == out.len()`, checked above.
    out[url.len()] = 0;

    Success
}
