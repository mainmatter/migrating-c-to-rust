// Compiles `c_src/setup_check.c` and generates Rust bindings for
// `c_src/setup_check.h` with `bindgen`. If bindgen fails here, it couldn't find
// libclang: see the course requirements in the welcome page.

use std::{env, path::PathBuf};

fn main() {
    cc::Build::new()
        .file("c_src/setup_check.c")
        .compile("setup_check");

    let bindings = bindgen::Builder::default()
        .header("c_src/setup_check.h")
        .generate()
        .expect("bindgen failed: is libclang installed?");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();

    println!("cargo::rerun-if-changed=c_src/setup_check.c");
    println!("cargo::rerun-if-changed=c_src/setup_check.h");
}
