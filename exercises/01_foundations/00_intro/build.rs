// Compiles `c_src/setup_check.c` with your system's C compiler and links it
// into this crate. If this step fails, the C toolchain from the course
// requirements isn't installed or isn't on your `PATH`.

fn main() {
    cc::Build::new()
        .file("c_src/setup_check.c")
        .compile("setup_check");

    println!("cargo::rerun-if-changed=c_src/setup_check.c");
}
