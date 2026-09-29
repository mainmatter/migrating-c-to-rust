//! Convert a traditional C string to Rust text and return a borrowed pointer.

use std::ffi::c_char;

/// Return a tag without its optional `#` prefix.
pub fn tag_name(tag: &str) -> &str {
    // TODO 1 OF 2: Strip one leading `#`, or return `tag` unchanged.
    tag
}

/// Convert a C string to UTF-8 and return its tag name as a borrowed pointer.
///
/// A null pointer or invalid UTF-8 produces a null pointer and length zero. A
/// valid result remains valid only while the memory behind `tag` remains alive.
///
/// # Safety
///
/// A non-null `tag` must point to a live, NUL-terminated byte string. `out_len`
/// must be a valid, writable pointer.
#[unsafe(export_name = "bm_tag_name")]
pub unsafe extern "C" fn bm_tag_name_ffi(tag: *const c_char, out_len: *mut usize) -> *const u8 {
    // TODO 2 OF 2: Reject null, borrow the input as `CStr`, convert it to
    // `&str`, call `tag_name`, write its byte length, and return its pointer.
    let _ = (tag, out_len);
    std::ptr::null()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn safe_api_returns_the_tag_name() {
        assert_eq!(tag_name("#rust"), "rust");
        assert_eq!(tag_name("ffi"), "ffi");
    }

    #[test]
    fn ffi_adapter_converts_between_both_representations() {
        let tag = CString::new("#rust").unwrap();
        let mut len = usize::MAX;

        // SAFETY: `tag` is a live C string and `len` is writable.
        let pointer = unsafe { bm_tag_name_ffi(tag.as_ptr(), &mut len) };
        assert!(!pointer.is_null());
        // SAFETY: The returned pointer borrows `tag`, which is still alive.
        let bytes = unsafe { std::slice::from_raw_parts(pointer, len) };
        assert_eq!(std::str::from_utf8(bytes), Ok("rust"));

        // SAFETY: Null is handled and `len` remains writable.
        let pointer = unsafe { bm_tag_name_ffi(std::ptr::null(), &mut len) };
        assert!(pointer.is_null());
        assert_eq!(len, 0);

        let invalid_utf8 = CString::new([0xff]).unwrap();
        // SAFETY: The input is a live C string and `len` is writable.
        let pointer = unsafe { bm_tag_name_ffi(invalid_utf8.as_ptr(), &mut len) };
        assert!(pointer.is_null());
        assert_eq!(len, 0);
    }
}
