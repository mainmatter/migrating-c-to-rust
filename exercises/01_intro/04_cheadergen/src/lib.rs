// Port `bm_strlower` from `exercises/_bm/src/util.c` to Rust.
//
// Requirements:
//   - The symbol must be callable from C (see the chapter for the two
//     annotations you need).
//   - Signature must match the C version: `void bm_strlower(char *s)`.
//   - NULL input must be a no-op (don't crash).
//   - Lowercase ASCII in place.
//
// hint: look up `std::slice::from_raw_parts_mut` and `std::str::from_utf8_mut`
// and `make_ascii_lowercase`!
//
// Once you're done, run `cheadergen generate --lang c --output-dir c_test -p cheadergen` to generate
// the header!

// TODO convert this C function to Rust
// void bm_strlower(char *s) {
//   if (!s)
//     return;
//   for (; *s; s++)
//     *s = (char)tolower((unsigned char)*s);
// }
