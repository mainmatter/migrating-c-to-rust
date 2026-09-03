use std::ffi::{CStr, c_char};
use std::ptr::NonNull;

pub type BmResult = i32;

pub const BM_OK: BmResult = 0;
pub const BM_ERR_INVALID_URL: BmResult = 3;
pub const BM_ERR_INVALID_BUFFER: BmResult = 6;
pub const BM_ERR_BUFFER_TOO_SMALL: BmResult = 7;

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
pub unsafe extern "C" fn bm_normalize_url_rust(
    url: Option<NonNull<c_char>>,
    out: Option<NonNull<c_char>>,
    out_len: usize,
) -> BmResult {
    let Some(url) = url else {
        return BM_ERR_INVALID_URL;
    };
    // SAFETY: clause (1) makes `url` sound to view as a `&CStr`: the scan for
    // the terminator stays in bounds, the region lies in one allocation, and
    // nothing writes to it while the borrow is live.
    let url = unsafe { CStr::from_ptr(url.as_ptr()) }.to_bytes();

    let Some(out) = out else {
        return BM_ERR_INVALID_BUFFER;
    };

    // Validate the *whole* input before touching `out`. The C version
    // lowercases into the caller's buffer and only then looks for a tab, so it
    // scribbles over `out` on inputs it rejects; narrowing up front means a
    // rejected URL leaves the caller's buffer exactly as it was.
    if !url.iter().copied().all(is_url_safe) {
        return BM_ERR_INVALID_URL;
    }

    // The normalized URL occupies `url.len()` bytes plus a NUL terminator, so
    // `out_len` has to be strictly greater than `url.len()`.
    if url.len() >= out_len {
        return BM_ERR_BUFFER_TOO_SMALL;
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

    BM_OK
}

#[cfg(test)]
mod tests {
    use std::ffi::c_char;
    use std::ptr::NonNull;

    use super::{BM_ERR_INVALID_URL, BM_OK, BmResult, bm_normalize_url_rust};

    unsafe extern "C" {
        #[link_name = "bm_normalize_url"]
        fn bm_normalize_url_c(raw: *const c_char, out: *mut c_char, out_len: usize) -> BmResult;
    }

    /// Runs one input through both implementations and hands back what each
    /// returned, along with the buffer each wrote into.
    fn both(raw: &[u8], out_len: usize) -> (BmResult, [c_char; 64], BmResult, [c_char; 64]) {
        let mut c_out = [0 as c_char; 64];
        let mut rust_out = [0 as c_char; 64];

        // SAFETY: `raw` is NUL-terminated and both buffers hold 64 bytes, which
        // is what we pass as `out_len`.
        let c_result =
            unsafe { bm_normalize_url_c(raw.as_ptr().cast(), c_out.as_mut_ptr(), out_len) };
        // SAFETY: as above.
        let rust_result = unsafe {
            bm_normalize_url_rust(
                NonNull::new(raw.as_ptr() as *mut c_char).into(),
                NonNull::new(rust_out.as_mut_ptr()).into(),
                out_len,
            )
        };

        (c_result, c_out, rust_result, rust_out)
    }

    #[test]
    fn c_and_rust_have_the_same_observable_behavior() {
        for (raw, out_len) in [
            (b"HTTPS://EXAMPLE.COM/Rust\0".as_slice(), 64),
            (b"https://example.com/has\ta-tab\0".as_slice(), 64),
        ] {
            let (c_result, c_out, rust_result, rust_out) = both(raw, out_len);

            assert_eq!(rust_result, c_result, "return codes differ for {raw:?}");

            // On the error path the two deliberately disagree about `out`, see
            // `rejected_input_leaves_the_rust_buffer_untouched` below.
            if c_result == BM_OK {
                assert_eq!(rust_out, c_out, "output differs for {raw:?}");
            }
        }
    }

    /// Differential testing turns up real differences, and not every difference
    /// is a bug. This one is an improvement, so we pin it down instead of
    /// papering over it.
    ///
    /// The C version lowercases the whole URL into the caller's buffer and only
    /// then goes looking for a tab, so it scribbles into `out` on inputs it
    /// rejects. Your port validates first, the way chapter 1 argues for, and so
    /// leaves the caller's buffer alone. Both still return the same code, which
    /// is the part a C caller can actually observe.
    #[test]
    fn rejected_input_leaves_the_rust_buffer_untouched() {
        let (c_result, c_out, rust_result, rust_out) =
            both(b"https://example.com/has\ta-tab\0", 64);

        assert_eq!(c_result, BM_ERR_INVALID_URL);
        assert_eq!(rust_result, BM_ERR_INVALID_URL);

        let untouched = [0 as c_char; 64];
        assert_ne!(c_out, untouched, "the C version writes before it validates");
        assert_eq!(
            rust_out, untouched,
            "the Rust port validates before it writes"
        );
    }
}
