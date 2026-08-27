use std::ffi::c_char;

pub type BmResult = i32;

pub const BM_OK: BmResult = 0;
pub const BM_ERR_INVALID_URL: BmResult = 3;

/// Lowercase a URL into the caller-owned output buffer.
///
/// Match the contract in `c_src/normalize.h`.
#[no_mangle]
pub unsafe extern "C" fn bm_normalize_url(
    raw: *const c_char,
    out: *mut c_char,
    out_len: usize,
) -> BmResult {
    let _ = (raw, out, out_len);
    // Replace `normalize.c` with a Rust implementation.
    BM_ERR_INVALID_URL
}

/// Normalize a tag into the caller-owned output buffer.
///
/// Match the contract in `c_src/normalize.h`.
#[no_mangle]
pub unsafe extern "C" fn tag_normalize(
    raw: *const c_char,
    out: *mut c_char,
    out_len: usize,
) -> BmResult {
    let _ = (raw, out, out_len);
    // Replace `normalize.c` with a Rust implementation.
    BM_ERR_INVALID_URL
}
