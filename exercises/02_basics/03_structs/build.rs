// Compiles the C side of this exercise and links it into the crate.

fn main() {
    println!("cargo:rerun-if-changed=c_src/layout.c");
    println!("cargo:rerun-if-changed=c_src/shapes.h");

    cc::Build::new()
        .file("c_src/layout.c")
        .include("c_src")
        .compile("layout");
}
