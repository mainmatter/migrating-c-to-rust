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

    use super::{BmResult, bm_normalize_url_rust};

    unsafe extern "C" {
        #[link_name = "bm_normalize_url"]
        fn bm_normalize_url_c(raw: *const c_char, out: *mut c_char, out_len: usize) -> BmResult;
    }

    #[test]
    fn c_and_rust_have_the_same_observable_behavior() {
        for (raw, out_len) in [
            (b"HTTPS://EXAMPLE.COM/Rust\0".as_slice(), 64),
            (b"https://example.com/has\ta-tab\0".as_slice(), 64),
        ] {
            let mut c_out = [0 as c_char; 64];
            let mut rust_out = [0 as c_char; 64];

            let c_result =
                unsafe { bm_normalize_url_c(raw.as_ptr().cast(), c_out.as_mut_ptr(), out_len) };
            let rust_result = unsafe {
                bm_normalize_url_rust(
                    NonNull::new(raw.as_ptr() as *mut c_char).into(),
                    NonNull::new(rust_out.as_mut_ptr()).into(),
                    out_len,
                )
            };

            assert_eq!(rust_result, c_result);
            assert_eq!(rust_out, c_out);
        }
    }
}
