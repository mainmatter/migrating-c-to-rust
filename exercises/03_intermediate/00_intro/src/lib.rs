//! Take `bm` for a spin.
//!
//! Build and run `bm` as described in this chapter's introduction, then answer
//! the two questions below by replacing each `todo!()` with what `bm` told you.
//!
//! The tests build `bm` themselves and compare your answers with its output, so
//! the only way to pass is to run it.

/// How many bookmarks does `BM_DB=fixtures/sample.bm ./bm find blog` print?
pub fn blog_matches() -> usize {
    todo!()
}

/// According to `BM_DB=fixtures/sample.bm ./bm list`, how many bookmarks are
/// tagged `rust`?
pub fn rust_tagged() -> usize {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::OnceLock;

    fn bm_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../_bm")
    }

    /// Builds `bm` with its Makefile, once, even though the tests run in
    /// parallel.
    fn build_bm() {
        static BUILT: OnceLock<()> = OnceLock::new();
        BUILT.get_or_init(|| {
            let status = Command::new("make")
                .current_dir(bm_dir())
                .status()
                .expect("couldn't run `make`");
            assert!(status.success(), "building `bm` with `make` failed");
        });
    }

    /// Runs `bm` against `fixtures/sample.bm` and returns what it printed.
    fn run_bm(args: &[&str]) -> String {
        build_bm();
        let dir = bm_dir();

        let output = Command::new(dir.join("bm"))
            .args(args)
            .current_dir(&dir)
            .env("BM_DB", "fixtures/sample.bm")
            .output()
            .expect("couldn't run `bm`");
        assert!(output.status.success(), "`bm {}` failed", args.join(" "));
        String::from_utf8(output.stdout).expect("`bm` printed invalid UTF-8")
    }

    #[test]
    fn blog_matches_is_what_bm_prints() {
        let printed = run_bm(&["find", "blog"]).lines().count();
        assert_eq!(super::blog_matches(), printed);
    }

    #[test]
    fn rust_tagged_is_what_bm_prints() {
        let listing = run_bm(&["list"]);
        let count = listing
            .lines()
            .find_map(|line| line.strip_prefix("rust\t"))
            .expect("`bm list` has no `rust` line")
            .trim()
            .parse::<usize>()
            .expect("`bm list` printed a non-numeric count for `rust`");
        assert_eq!(super::rust_tagged(), count);
    }
}
