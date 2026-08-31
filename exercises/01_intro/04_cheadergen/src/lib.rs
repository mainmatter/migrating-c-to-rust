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
//!   - Signature must equivalent the original C signature: `void bm_strlower(char *s)`.
//!   - NULL input must be a no-op (don't crash).
//!   - Lowercase ASCII in place.
//!
//! Hint: look up [`std::slice::from_raw_parts_mut`], [`std::str::from_utf8_mut`]
//! and `make_ascii_lowercase`!
//!
//! Once you're done, run:
//!
//! ```bash
//! cheadergen generate --lang c --output-dir c_test -p cheadergen
//! ```
//!
//! to generate the C header from your Rust code!
