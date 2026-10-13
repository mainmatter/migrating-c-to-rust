// Compiles the C library this exercise binds to.

fn main() {
    cc::Build::new()
        .file("c_src/buffer.c")
        .include("c_src")
        .compile("buffer");

    println!("cargo:rerun-if-changed=c_src/buffer.c");
    println!("cargo:rerun-if-changed=c_src/buffer.h");
}
