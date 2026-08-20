fn main() {
    cc::Build::new()
        .file("c_src/bookmark.c")
        .include("c_src")
        .warnings(true)
        .extra_warnings(true)
        .compile("bookmark");

    println!("cargo::rerun-if-changed=c_src/bookmark.c");
    println!("cargo::rerun-if-changed=c_src/bookmark.h");
}
