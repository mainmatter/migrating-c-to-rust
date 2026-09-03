//! This tiny FFI module has been copied from C and needs a lint pass.
//!
//! Run `wr` from the repository root to see the complete lint set. Resolve
//! every warning, either by fixing the code or, where the code is right and the
//! lint cannot know that, with an `allow` scoped as tightly as possible and a
//! comment explaining why the operation is sound. Keep the public API's intent:
//! raw-pointer functions must document their safety contract, and FFI functions
//! must use FFI-safe types.

pub struct BMDb {
    pub value: u32,
}

pub static BM_DEFAULT_DB: BMDb = BMDb { value: 0 };

#[unsafe(export_name = "BMNormalizeURL")]
pub extern "C" fn bm_normalize_url() {}

unsafe extern "C" {
    pub fn upstream_bytes(bytes: *mut u8);
}

#[unsafe(no_mangle)]
pub extern "C" fn copy_bytes(bytes: *mut u8) -> *mut u8 {
    bytes
}

/// # Safety
///
/// `bytes` must point to at least two consecutive initialized `u8` values,
/// all within the same allocation.
pub unsafe fn second_byte(bytes: *const u8) -> u8 {
    // SAFETY: the caller guarantees that `bytes` has at least two elements, so
    // offsetting by one stays inside the same allocation.
    let second = unsafe { bytes.add(1) };
    // SAFETY: `second` is in bounds and initialized, as guaranteed by the
    // caller's contract.
    unsafe { *second }
}

/// # Safety
///
/// `bytes` must be non-null and point to an initialized `u8`.
pub unsafe fn first_byte(bytes: *const u8) -> u8 {
    // SAFETY: the caller guarantees that `bytes` points to an initialized `u8`.
    unsafe { *bytes }
}

pub fn status() {}

/// # Safety
///
/// `bytes` must be aligned for `u32`, i.e. its address must be a multiple of
/// four.
pub unsafe fn pointer_casts(bytes: *const u8) {
    // SAFETY: the caller guarantees that `bytes` is aligned for `u32`, which is
    // the requirement `cast_ptr_alignment` cannot verify on its own.
    #[allow(clippy::cast_ptr_alignment)]
    let _word = bytes.cast::<u32>();
    // SAFETY: same as above. `cast` is the idiomatic replacement for the
    // pointer transmute the C code used; it cannot change constness.
    #[allow(clippy::cast_ptr_alignment)]
    let _word = bytes.cast::<u32>();
}

#[cfg(test)]
mod tests {
    use super::second_byte;

    #[test]
    fn reads_the_second_byte() {
        let bytes = b"ab";

        // SAFETY: `bytes` has two initialized elements.
        assert_eq!(unsafe { second_byte(bytes.as_ptr()) }, b'b');
    }
}
