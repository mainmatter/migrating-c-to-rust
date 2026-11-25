//! Builds `bm` twice, once from all of its C sources and once with this crate
//! in place of `normalize.c`, then runs both on the same fixtures and checks
//! that they behave the same.
//!
//! `find` and `list` reach `bm_normalize_url`, because loading a database
//! normalizes every URL. Only `add --tag` and `tag` reach `tag_normalize`, so
//! those run too, on a copy of the fixture, and the tests also compare the
//! database each binary leaves behind.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// `normalize.c` is the only file left out of the Rust build.
const C_SOURCES: [&str; 6] = [
    "bookmark.c",
    "cli.c",
    "index.c",
    "storage.c",
    "tag.c",
    "util.c",
];

fn bm_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../_bm")
}

/// Cargo's scratch directory for integration tests, inside `target/`.
fn tmp_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
}

fn run(cmd: &mut Command) {
    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("couldn't run {cmd:?}: {e}"));
    assert!(
        output.status.success(),
        "{cmd:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Compiles and links `bm` with the warning flags of `exercises/_bm/Makefile`.
///
/// The Makefile's `-O0 -g` are left out: Nix's `cc` wrapper on Linux defines
/// `_FORTIFY_SOURCE`, which glibc rejects with a warning at `-O0`, and `-Werror`
/// would turn that into an error.
fn cc(extra_inputs: &[PathBuf], out: &Path) {
    let src = bm_dir().join("src");
    let mut cmd = Command::new("cc");
    cmd.args(["-Wall", "-Wextra", "-Werror", "-std=c11"])
        .arg("-I")
        .arg(&src)
        .args(C_SOURCES.iter().map(|file| src.join(file)))
        .args(extra_inputs)
        .arg("-o")
        .arg(out);
    run(&mut cmd);
}

/// Builds both binaries once, even though the tests run in parallel, and
/// returns their paths: all-C first, Rust second.
fn binaries() -> &'static (PathBuf, PathBuf) {
    static BUILT: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();
    BUILT.get_or_init(|| {
        let tmp = tmp_dir();
        let target = tmp.parent().expect("CARGO_TARGET_TMPDIR is inside target/");

        // `cargo test` doesn't build the static library, so ask for it. Cargo
        // releases its lock on `target/` before it runs tests. Remove an old
        // archive first, so that a stale one can't stand in for a missing one.
        let archive = target.join("debug").join("libbm_normalize.a");
        let _ = std::fs::remove_file(&archive);
        run(Command::new(env!("CARGO"))
            .args(["build", "-p", "bm_normalize", "--target-dir"])
            .arg(target));
        assert!(
            archive.exists(),
            "{} is missing: is the crate a `staticlib`?",
            archive.display()
        );

        let all_c = tmp.join("bm-c");
        let with_rust = tmp.join("bm-rust");
        cc(&[bm_dir().join("src/normalize.c")], &all_c);
        cc(&[archive], &with_rust);
        (all_c, with_rust)
    })
}

/// What one run of `bm` printed and left behind.
struct Run {
    stdout: String,
    stderr: String,
    code: Option<i32>,
    database: String,
}

/// Runs `bm` on a fresh copy of `fixture`, so that nothing can change the
/// original, and so that each run starts from the same database.
fn bm(binary: &Path, test: &str, fixture: &str, args: &[&str]) -> Run {
    let name = binary.file_name().unwrap().to_string_lossy();
    let dir = tmp_dir().join(test).join(&*name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(bm_dir().join("fixtures").join(fixture), dir.join(fixture)).unwrap();

    let output = Command::new(binary)
        .args(args)
        .current_dir(&dir)
        .env("BM_DB", fixture)
        .output()
        .unwrap_or_else(|e| panic!("couldn't run {}: {e}", binary.display()));
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    Run {
        stdout: text(&output.stdout),
        stderr: text(&output.stderr),
        code: output.status.code(),
        database: text(&std::fs::read(dir.join(fixture)).unwrap()),
    }
}

/// Runs both binaries with the same arguments, one test-named scratch directory
/// each, and compares everything they print and write.
fn assert_same(test: &str, fixture: &str, args: &[&str]) {
    let (all_c, with_rust) = binaries();
    let expected = bm(all_c, test, fixture, args);
    let actual = bm(with_rust, test, fixture, args);

    let what = format!("`bm {args:?}` on {fixture}");
    assert_eq!(actual.stdout, expected.stdout, "{what}: stdout differs");
    assert_eq!(actual.stderr, expected.stderr, "{what}: stderr differs");
    assert_eq!(actual.code, expected.code, "{what}: exit code differs");
    assert_eq!(
        actual.database, expected.database,
        "{what}: the database differs afterwards"
    );
}

#[test]
fn list_sample() {
    assert_same("list_sample", "sample.bm", &["list"]);
}

#[test]
fn find_sample() {
    for query in ["blog", "rust", "github.com", "nothing-matches-this"] {
        assert_same("find_sample", "sample.bm", &["find", query]);
    }
}

#[test]
fn list_corrupt() {
    assert_same("list_corrupt", "corrupt.bm", &["list"]);
}

#[test]
fn find_corrupt() {
    assert_same("find_corrupt", "corrupt.bm", &["find", "example"]);
}

#[test]
fn list_large() {
    assert_same("list_large", "large.bm", &["list"]);
}

#[test]
fn add_with_tags() {
    let url = "https://example.com/new";
    let cases: [&[&str]; 5] = [
        &["--tag", "  RuSt", "--tag", "c,lang"],
        &[
            "--tag",
            "\x0bVt\x0bignored",
            "--tag",
            "\x0cFf",
            "--tag",
            "\rCr\n",
        ],
        &["--tag", "a".repeat(63).leak()],
        &["--tag", "a".repeat(64).leak()],
        &["--tag", " "],
    ];
    for tags in cases {
        let mut args = vec!["add", url];
        args.extend_from_slice(tags);
        assert_same("add_with_tags", "sample.bm", &args);
    }
    assert_same("add_with_tags", "sample.bm", &["add", url, "--tag", ","]);
}

#[test]
fn retag() {
    let url = "https://github.com/rust-lang/rust";
    assert_same(
        "retag",
        "sample.bm",
        &["tag", url, "+NEW", "-Rust", "+ \tlang"],
    );
    assert_same("retag", "sample.bm", &["tag", url, "+"]);
}
