# Performance benchmarking

You have ported a C function to Rust. The tests pass and the output is
identical. Great! But what about execution speed?

It is tempting to run the program once, look at the wall clock, and call that a
benchmark. The problem is that one number tells you almost nothing. Your laptop
may boost one run and throttle the next. The filesystem cache may already be
warm, and your browser may wake up and steal a core.

A _benchmark_ repeatedly measures a controlled workload. It can report
_latency_, how long one operation takes, or _throughput_, how many operations
finish in a given time. Our benchmark will measure the latency of one call to
`bm_normalize_url`.

A _profile_ answers a different question: where did a program spend CPU time,
allocate memory, or perform I/O? We focus on benchmarking here because it tells
us whether the C-to-Rust port regressed. Profiling appears briefly at the end,
as a way to investigate an unexpected result.

## Benchmarking tools

The main tool for running benchmarks is [`cargo bench`][cargo-bench]. By itself
it does nothing and requires a benchmark harness. At the time of writing, Rust's
built-in benchmark harness is nightly-only: `#[bench]`, the `test` crate, and
`test::Bencher` are all unstable.[^1] In stable Rust, you need a third-party
harness such as [Criterion][criterion] or [Divan][divan]. Criterion stores
baselines and reports changes between runs. Divan provides a smaller,
attribute-based harness and reports the fastest, slowest, median, and mean
times. For simplicity, we will use Divan.

Add Divan to the crate that owns the Rust port. The exercise also uses the `cc`
crate to compile the C baseline:

```toml
# Cargo.toml
[build-dependencies]
cc.workspace = true

[dev-dependencies]
divan = "0.1"

[[bench]]
name = "normalize"
harness = false
```

`harness = false` tells Cargo not to start its built-in test harness. The
benchmark binary supplies its own `main` function and starts Divan instead:

```rust,no_run
fn main() {
    divan::main();
}
```

The build script compiles the original C implementation:

```rust,no_run
// Sample build.rs
fn main() {
    cc::Build::new()
        .file("c_src/normalize.c")
        .compile("bm_normalize_c");
}
```

This keeps both implementations in the same Cargo build. When `cargo bench`
selects the optimized `bench` profile, `cc` reads Cargo's `OPT_LEVEL` and
applies the corresponding optimization level to the C code too.[^cc-opt]

If you build the C code manually instead of using `cc`, pass the appropriate
`-O` flag to the C compiler yourself. Otherwise you may end up comparing
optimized Rust with unoptimized C, which makes the benchmark meaningless.

## Benchmark the same work

Now we need to compare the C implementation with the Rust port. A C/Rust
comparison is only useful when both benchmarks include the same work.

Start from the C signature. The caller owns the output buffer, and nothing is
allocated:

```c
BmResult bm_normalize_url(const char *raw, char *out, size_t out_len);
```

Porting it gives you a choice, and this decides what you are able to measure:

```rust,compile_fail
// Safe and idiomatic. Not comparable: it allocates, and reaching it from
// C's `const char *` costs a UTF-8 check on the way in.
pub fn normalize_url(raw: &str) -> Result<String, InvalidUrl>;

// Comparable: the same contract as C, so the two benchmarks do the same work.
pub unsafe extern "C" fn bm_normalize_url_rust(
    raw: Option<NonNull<c_char>>,
    out: Option<NonNull<c_char>>,
    out_len: usize,
) -> BmResult;
```

If you benchmark the first one against C, the Rust number carries a validation
and an allocation that the C number does not: you have measured two different
APIs, not two implementations of the same operation.

This does not mean the safe API is wrong. The allocation may well be the right
design. It does mean you need to state what the benchmark includes.

For a direct port comparison, take the second signature, and give both
implementations the same input and equivalent reusable output buffers:

```rust,no_run
# use std::ffi::c_char;
# type BmResult = i32;
unsafe extern "C" {
    #[link_name = "bm_normalize_url"] // bind the C symbol under a second name
    fn bm_normalize_url_c(raw: *const c_char, out: *mut c_char, out_len: usize) -> BmResult;
}

#[divan::bench(sample_size = 4096)]
fn c(bencher: divan::Bencher) {
    let raw = b"HTTPS://EXAMPLE.COM/Rust\0";
    let mut out = [0 as c_char; 2048]; // created once, so it is not timed

    bencher.bench_local(|| unsafe {
        let result = bm_normalize_url_c(
            divan::black_box(raw.as_ptr().cast()),
            divan::black_box(out.as_mut_ptr()),
            out.len(),
        );
        divan::black_box(result)
    });
}
```

The benchmark setup happens before `bench_local`, so creating the output buffer
is not timed. The closure mutably captures that buffer and reuses it for every
iteration. `bench_local` runs single-threaded.

These calls take only a few dozen nanoseconds. `sample_size = 4096` runs each
benchmark that many times in every timing sample; Divan then divides the total
time by 4096. This keeps each sample well above timer precision and makes the
result less sensitive to an interruption during Divan's automatic calibration.
Use the same sample size for both implementations; adjust it for slower or
faster workloads.

> **`black_box` is not decorative.** Without it, the compiler can notice that
> the result goes nowhere and optimize away some or all of the work. You would
> then be wowed by the speed of a function that did not run, making the whole
> benchmark meaningless.

> **Rule of thumb:** Benchmark the semantics you want to compare, not the
> convenience API you happened to write first.

Check correctness first, then run both benchmarks, from the workspace root:

```bash
cargo test -p bm_benchmarking
cargo bench -p bm_benchmarking --bench normalize
```

You can pass a filter to Divan when you only want one implementation:

```bash
cargo bench -p bm_benchmarking --bench normalize -- rust
```

Divan invokes each function many times. It groups those invocations into samples
and reports a per-iteration time:

- **median** is the middle sample and usually the most useful value for a quick
  comparison;
- **mean** is the average and moves more when one sample is unusually slow;
- **fastest** and **slowest** show the extremes and help expose noisy runs;
- **samples** and **iters** show how many measurements and function calls
  contributed to the result.

The fastest-to-slowest range shows only the extremes within one run. A single
outlier can make it huge, so do not use it to decide whether two implementations
differ.

## Check for a regression

Suppose repeated runs put Rust around 220 ns and C around 200 ns. That is a 10%
difference.

It is a _performance regression_ only if the slowdown remains under the same
workload and conditions. Run the benchmark several times on the same
otherwise-idle machine and compare the medians. If the difference keeps changing
direction, it is too small to distinguish from noise under those conditions. If
Rust remains slower by roughly the same amount, investigate.

The instructions are deterministic; the conditions under which they run are not.
Caches, frequency scaling, and whatever else the machine is doing all land in
your measurement.

Treat a repeatable regression like any other bug:

1. Run the correctness tests first. A faster wrong port is not an optimization.
2. Record the benchmark command, input, machine, and baseline median.
3. Change one thing, then rerun the same benchmark under the same conditions.
4. If the regression remains, profile the slow path before changing more code.

Keep the benchmark after the fix. It is now the thing that will catch the
regression if it comes back.

> **CI tip:** A continuous-benchmarking tool such as [Bencher][bencher] can
> store results over time and fail a pull request when performance regresses.

## A brief note on profiling

If a benchmark finds a repeatable regression, a profiler can show where the
extra work happens.

[`cargo-samply`][samply] builds an optimized binary with debug information and
records CPU samples. See its documentation for installation and usage. Look for
functions that occupy the widest frames in the profile. Those are where the
sampled CPU time accumulated. Then record a profile for the C implementation, or
for the version before your change, and compare it with the new profile. This
shows which functions now take more CPU time and explains the slower benchmark.

[Hotpath][hotpath] is useful when you want timing or allocation measurements for
a small set of functions in an application. Its instrumentation is disabled
unless you enable the corresponding Cargo features:

```toml
# Cargo.toml
[dependencies]
hotpath = "0.24"

[features]
hotpath = ["hotpath/hotpath"]
hotpath-alloc = ["hotpath/hotpath-alloc"]
```

Mark the functions you want to measure and the application's entry point:

```rust,no_run
#[hotpath::measure] // time every call to this function
fn normalize_url(raw: &str) -> String {
    // ...
}

#[hotpath::main] // install the reporter, print the table on exit
fn main() {
    // ...
}
```

```bash
cargo run --release --features='hotpath,hotpath-alloc'
```

Hotpath can also write JSON reports for CI. Its `hotpath-utils` command compares
reports from the pull request and the base branch, including per-function timing
and allocation differences.[^hotpath-ci] This is useful for watching a few
representative application paths. Keep Divan for controlled microbenchmarks, and
disable Hotpath while collecting the final Divan numbers because instrumentation
changes the code being measured.

## Head to the exercise

The exercise is in `exercises/02_intermediate/07_benchmarking`. It ships with
the C benchmark. Replace the `bm_normalize_url_rust` stub and add the matching
Rust benchmark, then run both and compare their median latencies.

The parity test compares return codes on every input, but compares the output
buffers only when the call succeeded. The C version lowercases into the caller's
buffer before it looks for a tab, so it writes into `out` even on inputs it
rejects. Your port validates first, so it doesn't. That is a difference
differential testing is supposed to surface, and one worth keeping: the test
pins it down rather than asking you to reproduce it.

[^1]: See the [`test` library feature][unstable-test] in the Unstable Book for
    the current status of `#[bench]` and `test::Bencher`.

[^cc-opt]: The `cc` crate reads the optimization level from Cargo's `OPT_LEVEL`
    environment variable. See the
    [`cc::Build::opt_level` documentation][cc-opt].

[^hotpath-ci]: The [Hotpath CI guide][hotpath-ci] shows how to produce the head
    and base JSON reports and compare them in GitHub Actions.

[cargo-bench]: https://doc.rust-lang.org/cargo/commands/cargo-bench.html
[cc-opt]: https://docs.rs/cc/latest/cc/struct.Build.html#method.opt_level
[bencher]: https://bencher.dev/
[criterion]: https://github.com/criterion-rs/criterion.rs
[divan]: https://docs.rs/divan/latest/divan/
[hotpath]: https://docs.rs/hotpath/latest/hotpath/
[hotpath-ci]: https://hotpath.rs/github_ci
[samply]: https://docs.rs/cargo-samply/latest/cargo_samply/
[unstable-test]: https://doc.rust-lang.org/unstable-book/library-features/test.html
