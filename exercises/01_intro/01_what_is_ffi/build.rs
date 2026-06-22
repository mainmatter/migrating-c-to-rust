// Compiles the legacy C source under c_src/ into a static library and links it
// into this crate. This is the Cargo-native equivalent of a Makefile rule that
// runs `cc -c` and then links the .a — except the cc crate handles all of it
// (compiler detection, flags, archive creation, link directives).
//
// You don't need to change anything here for this exercise; it's the same
// build.rs you'll see again in 03_bindgen. The interesting work is in lib.rs.

fn main() {
    cc::Build::new()
        .file("c_src/bm_legacy.c")
        .include("c_src")
        .compile("bm_legacy");

    println!("cargo:rerun-if-changed=c_src/bm_legacy.c");
    println!("cargo:rerun-if-changed=c_src/bm_legacy.h");
}
