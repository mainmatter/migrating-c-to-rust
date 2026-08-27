// Same build.rs as in chapter 1: compile the legacy C slice into a static
// library and link it into this crate. Nothing to change here.
//
// Note for later: under `cargo miri` this script still runs natively (build
// scripts always do), but the resulting static library is never linked —
// Miri interprets Rust and has no way to execute `tag.o`.

fn main() {
    cc::Build::new()
        .file("c_src/tag.c")
        .include("c_src")
        .compile("tag");

    println!("cargo::rerun-if-changed=c_src/tag.c");
    println!("cargo::rerun-if-changed=c_src/tag.h");
}
