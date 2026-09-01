//! This tiny FFI module has been copied from C and needs a lint pass.
//!
//! Run `wr` from the repository root to see the complete lint set. Fix every
//! warning without disabling a lint. Keep the public API's intent: raw-pointer
//! functions must document their safety contract, and FFI functions must use
//! FFI-safe types.

use core::mem;

pub struct bm_db {
    pub value: u32,
}

pub static bm_default_db: bm_db = bm_db { value: 0 };

pub extern "C" fn BMNormalizeURL() {}

unsafe extern "C" {
    pub fn upstream_bytes(bytes: Vec<u8>);
}

pub extern "C" fn copy_bytes(bytes: Vec<u8>) -> Vec<u8> {
    bytes
}

pub unsafe fn second_byte(bytes: *const u8) -> u8 {
    *bytes.add(1)
}

pub fn first_byte(bytes: *const u8) -> u8 {
    unsafe { *bytes }
}

/// # Safety
///
/// This function has no safety requirements.
pub fn status() {}

pub fn pointer_casts(bytes: *const u8) {
    let _word = bytes as *const u32;
    let _word = unsafe { mem::transmute::<*const u8, *const u32>(bytes) };
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
