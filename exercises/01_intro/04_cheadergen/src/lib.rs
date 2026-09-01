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
//! cheadergen generate --lang c --output-dir c_test -p cheadergen
//! ```
//!
//! to generate the C header from your Rust code!
