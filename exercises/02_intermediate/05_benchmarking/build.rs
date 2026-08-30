fn main() {
    cc::Build::new()
        .file("c_src/normalize.c")
        .include("c_src")
        .compile("bm_normalize_c");

    println!("cargo::rerun-if-changed=c_src/normalize.c");
    println!("cargo::rerun-if-changed=c_src/normalize.h");
    println!("cargo::rerun-if-changed=c_src/result.h");
}
