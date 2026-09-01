use std::ffi::c_char;
use std::ptr::NonNull;

pub type BmResult = i32;

pub const BM_OK: BmResult = 0;
pub const BM_ERR_INVALID_URL: BmResult = 3;
pub const BM_ERR_INVALID_BUFFER: BmResult = 6;
pub const BM_ERR_BUFFER_TOO_SMALL: BmResult = 7;

#[no_mangle]
pub unsafe extern "C" fn bm_normalize_url_rust(
    url: Option<NonNull<c_char>>,
    out: Option<NonNull<c_char>>,
    out_len: usize,
) -> BmResult {
    todo!("Replace this stub with your solution for bm_normalize_url in Chapter 1, exercise 07");
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
