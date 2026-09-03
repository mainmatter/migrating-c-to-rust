//! Port the following C function over to Rust:
//!
//! ```c
//! void bm_strlower(char *s) {
//!   if (!s)
//!     return;
//!   for (; *s; s++)
//!     *s = (char)tolower((unsigned char)*s);
//! }
//! ```
//!
//! Requirements:
//!   - The new Rust function must be callable from C, using the same symbol name, `bm_strlower`.
//!   - Signature must match the original C signature: `void bm_strlower(char *s)`.
//!   - NULL input must be a no-op (don't crash).
//!   - Lowercase ASCII in place.
//!
//! Hint: [`std::ffi::CStr::from_ptr`] gets you the length,
//! [`std::slice::from_raw_parts_mut`] turns the pointer into a mutable slice,
//! and `make_ascii_lowercase` on that byte slice does the rest. No UTF-8
//! validation needed: the C version happily lowercases bytes that aren't valid
//! UTF-8, and so should yours.
//!
//! Once you're done, run:
//!
//! ```bash
//! cheadergen generate --lang c --output-dir c_test -p bm_cheadergen
//! ```
//!
//! to generate the C header from your Rust code!

use core::ffi::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn bm_strlower(c: *mut c_char) {
    if c.is_null() {
        return;
    }
    let len = {
        let c = unsafe { std::ffi::CStr::from_ptr(c) };
        c.count_bytes()
    };

    let slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(c.cast(), len) };
    slice.make_ascii_lowercase();
}
