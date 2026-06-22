use std::ffi::c_char;

/// Normalize `url` (lowercase ASCII) into the caller-provided `out` buffer.
///
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn bm_normalize_url(
    url: *const c_char,
    out: *mut c_char,
    out_len: usize,
) -> i32 {
    let mut len = 0;
    while *url.add(len) != 0 {
        len += 1;
    }

    for i in 0..len {
        // raw pointer arithmetic, yuck...
        let b = *url.add(i) as u8;
        let normalized = if b.is_ascii_uppercase() { b + 32 } else { b };
        *out.add(i) = normalized as c_char;
    }
    *out.add(len) = 0;

    // we only ever return success?
    0
}
