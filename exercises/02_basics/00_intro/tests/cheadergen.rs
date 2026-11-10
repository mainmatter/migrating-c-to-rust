use std::process::Command;

#[test]
fn cheadergen_generates_a_header() {
    let out_dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("basics_intro_header");

    let output = Command::new("cheadergen")
        .args([
            "generate",
            "--lang",
            "c",
            "-p",
            "basics_intro",
            "--output-dir",
        ])
        .arg(&out_dir)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("couldn't run `cheadergen`: is `cheadergen_cli` installed and on your PATH?");
    assert!(
        output.status.success(),
        "`cheadergen generate` failed. Is its pinned nightly toolchain installed?\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let header = std::fs::read_to_string(out_dir.join("basics_intro.h"))
        .expect("`cheadergen` didn't write `basics_intro.h`");
    assert!(
        header.contains("setup_check_double"),
        "the generated header doesn't declare `setup_check_double`:\n{header}"
    );
}
