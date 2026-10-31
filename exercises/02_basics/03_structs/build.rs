// Compiles the C side of this exercise and links it into the crate.

fn main() {
    println!("cargo:rerun-if-changed=c_src/layout.c");
    println!("cargo:rerun-if-changed=c_src/shapes.h");

    cc::Build::new()
        .file("c_src/layout.c")
        .include("c_src")
        // `alignof` comes from C11's <stdalign.h>; MSVC only enables it with /std:c11.
        .std("c11")
        .compile("layout");
}
