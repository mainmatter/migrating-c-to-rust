// Compiles the C side of this exercise and links it into the crate.

fn main() {
    println!("cargo:rerun-if-changed=c_src/chars.c");
    println!("cargo:rerun-if-changed=c_src/chars.h");

    // Compiling C for another target needs a cross compiler, which this
    // exercise doesn't ask you to install. `cargo check --target ...` only
    // type-checks, so skipping the C build keeps that command working.
    if std::env::var("TARGET") != std::env::var("HOST") {
        return;
    }

    cc::Build::new()
        .file("c_src/chars.c")
        .include("c_src")
        .compile("chars");
}
