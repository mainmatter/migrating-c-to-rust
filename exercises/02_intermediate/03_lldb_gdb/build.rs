fn main() {
    cc::Build::new()
        .file("c_src/shout.c")
        .include("c_src")
        .flag_if_supported("-g")
        .compile("shout");

    println!("cargo:rerun-if-changed=c_src/shout.c");
    println!("cargo:rerun-if-changed=c_src/shout.h");
}
