//! Pins the Rust-side API surface of this crate.
//!
//! Every item in `src/lib.rs` is there because it trips a lint, and fixing a
//! lint is not the same thing as making it go away: deleting an item silences
//! its warning just as effectively. Naming the items here keeps that honest --
//! the crate has to still export them for this file to compile.
//!
//! The coverage is deliberately narrow. It names only the items the exercise
//! does not ask you to rename, so it never dictates what `bm_db` and
//! `bm_default_db` should become. And it only *imports* them rather than
//! calling them, because whether `first_byte` and `pointer_casts` end up
//! `unsafe` or keep a narrowly scoped `allow` is a choice left to you.
//!
//! The C-visible names are pinned separately, by the harness in `c_test/`.

#[test]
fn the_public_api_is_still_there() {
    // The import is the assertion: this file does not compile if an item is
    // missing.
    #[allow(unused_imports)]
    use bm_clippy_lints::{copy_bytes, first_byte, pointer_casts, second_byte, status};
}
