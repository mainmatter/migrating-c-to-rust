fn main() {
    cc::Build::new()
        .file("c_src/bm_legacy.c")
        .compile("bm_legacy");

    // TODO: bindgen::Builder — generate bindings from c_src/bm_legacy.h
    //   into $OUT_DIR/bindings.rs. Use `std::env::var("OUT_DIR")` for the path.
    todo!();

    println!("cargo::rerun-if-changed=c_src/bm_legacy.c");
    println!("cargo::rerun-if-changed=c_src/bm_legacy.h");
}
