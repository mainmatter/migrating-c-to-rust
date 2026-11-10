use std::{env, path::PathBuf};

fn main() {
    cc::Build::new()
        .file("c_src/bm_legacy.c")
        .compile("bm_legacy");

    // TODO: bindgen::Builder — generate bindings from c_src/bm_legacy.h
    //   into $OUT_DIR/bindings.rs. Use `std::env::var("OUT_DIR")` for the path.
    let bindings = bindgen::Builder::default()
        .header("c_src/bm_legacy.h")
        // tell Cargo to re-run when headers change
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();

    println!("cargo::rerun-if-changed=c_src/bm_legacy.c");
    println!("cargo::rerun-if-changed=c_src/bm_legacy.h");
}
