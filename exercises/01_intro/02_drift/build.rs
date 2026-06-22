fn main() {
    cc::Build::new()
        .file("c_src/bm_legacy.c")
        .include("c_src")
        .compile("bm_legacy");

    println!("cargo:rerun-if-changed=c_src/bm_legacy.c");
    println!("cargo:rerun-if-changed=c_src/bm_legacy.h");
}
