use std::ffi::c_char;
use std::ptr::NonNull;

use bm_benchmarking::{BmResult, bm_normalize_url_rust};

const RAW: &[u8] = b"HTTPS://EXAMPLE.COM/Rust\0";
const OUT_LEN: usize = 2048; // BM_MAX_URL_LEN in c_src/normalize.h

unsafe extern "C" {
    #[link_name = "bm_normalize_url"]
    fn bm_normalize_url_c(raw: *const c_char, out: *mut c_char, out_len: usize) -> BmResult;
}

fn main() {
    divan::main();
}

/// The C baseline. Use it as a template for `rust` below.
///
/// Note what is and is not inside the timed closure: the output buffer is
/// created once, up front, so its cost is not part of the measurement.
///
/// Do not be surprised if this comes out slower than your Rust port, and do
/// not read that as "Rust beats C" (or the other way around, depending on your implementation from Chapter 1, exercise 07).
/// `bm_normalize_url` calls libc's `tolower`, which is locale-aware, so every byte costs a real function call and a table  lookup, and none of it inlines. `strchr` then walks the string a second time. That is a property of the two library calls this C happened to reach
/// for, not of the language.
#[divan::bench(sample_size = 4096)]
fn c(bencher: divan::Bencher) {
    let mut out = [0 as c_char; OUT_LEN];

    bencher.bench_local(|| unsafe {
        let result = bm_normalize_url_c(
            divan::black_box(RAW.as_ptr().cast()),
            divan::black_box(out.as_mut_ptr()),
            out.len(),
        );
        divan::black_box(result)
    });
}

/// The Rust port, measured the same way as `c` above: same input, same
/// `out_len`, and the output buffer allocated once outside the timed closure so
/// only the normalization itself is measured.
#[divan::bench(sample_size = 4096)]
fn rust(bencher: divan::Bencher) {
    let mut out = [0 as c_char; OUT_LEN];

    bencher.bench_local(|| unsafe {
        let result = bm_normalize_url_rust(
            NonNull::new(divan::black_box(RAW.as_ptr().cast_mut().cast())),
            NonNull::new(divan::black_box(out.as_mut_ptr())),
            out.len(),
        );
        divan::black_box(result)
    });
}
