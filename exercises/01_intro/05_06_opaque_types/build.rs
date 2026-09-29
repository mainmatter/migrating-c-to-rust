// Compiles the C library this exercise binds to.

fn main() {
    cc::Build::new()
        .file("c_src/widget.c")
        .include("c_src")
        .compile("widget");

    println!("cargo:rerun-if-changed=c_src/widget.c");
    println!("cargo:rerun-if-changed=c_src/widget.h");
}
