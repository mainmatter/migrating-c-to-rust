# Using `lldb`/`gdb`

At some point in a migration you'll hit a crash at the FFI boundary with little
to go on. A C caller passes a string without the NUL terminator the API
requires, or Rust hands C a pointer whose lifetime has ended. The process takes
a `SIGSEGV` and dies before Rust can give you a tidy panic message or backtrace.

`gdb` and `lldb` can show the mixed C/Rust stack. Set a breakpoint in Rust, step
into C, and inspect both sides of the boundary in one session.

## Build with debug information

Build both sides of the boundary with debug information. Cargo's `dev` profile
already sets `debug = true`; for a release build, add this to `Cargo.toml`:

```toml
[profile.release]
debug = true
```

If `build.rs` compiles the C code with `cc`, pass `-g` there too:

```rust
cc::Build::new()
    .file("c_src/shout.c")
    .include("c_src")
    .flag_if_supported("-g")
    .compile("shout");
```

Forget it and you get Rust frames with line numbers and C frames without:
precisely the half of the stack you opened the debugger for.

## Start the debugger

Use `rust-gdb` or `rust-lldb` rather than plain `gdb` and `lldb`. They're thin
wrappers that load Rust's pretty-printers, which is what makes a `String`,
`Vec`, or `Option` readable instead of a pile of pointers and capacities.

On macOS, that's usually `rust-lldb`:

```sh
rust-lldb target/debug/bm_lldb_gdb
```

On Linux, `rust-gdb`:

```sh
rust-gdb target/debug/bm_lldb_gdb
```

Program arguments go after `--` for `rust-lldb` and after `--args` for
`rust-gdb`.

If you prefer a graphical frontend, `rust-gdbgui` starts the browser-based
[`gdbgui`](https://gdbgui.com/) with the same pretty-printers loaded. Install
`gdbgui` separately.

```sh
rust-gdbgui target/debug/bm_lldb_gdb
```

## Walk across the boundary

If you already know one debugger and need the equivalent command in the other,
keep the official [GDB to LLDB command map](https://lldb.llvm.org/use/map.html)
open. It covers both directions in one place.

Say you're investigating `bm_shout`, the C helper called by the Rust exercise.
Set a breakpoint on it, run the program, and look at the stack when execution
stops (output abbreviated):

```text
(lldb) b bm_shout
Breakpoint 1: where = bm`bm_shout at shout.c:6
(lldb) run
Process 51234 stopped
* thread #1, stop reason = breakpoint 1.1
  * frame #0: bm`bm_shout at shout.c:6
    frame #1: bm`bm_lldb_gdb::shout at lib.rs:17
    frame #2: bm`bm_lldb_gdb::demo at lib.rs:30
    frame #3: bm`main at main.rs:2
```

C frames and Rust frames are interleaved, and the numbering tells you which way
to walk: frame #0 is where execution stopped, and every higher number is one of
its callers. `up` moves toward callers, `down` back toward the stop. So the code
that handed over the bad value is always _up_, whichever language crashed. If
Rust crashed, `up` gets you to the C caller and the arguments it passed. If C
crashed, `up` gets you to the Rust frame that supplied them.

From there the usual commands work in both debuggers: `s` steps into a call, `n`
steps over one, `finish` runs the current frame to completion, `bt` prints the
stack, and `p` prints a value.

For a `char *out`, inspect the bytes directly: `x/s out` in `gdb`, and either
`x/s out` or `memory read --format c-string out` in `lldb` (the format name is
`c-string`, with the hyphen; `cstring` is rejected). For a raw buffer, use
`x/16xb out` in `gdb` or `memory read --size 1 --count 16 --format x out` in
`lldb`. This quickly tells you whether the pointer, length, and terminator agree
with the FFI contract.

When the process dies outright, you don't need a breakpoint at all. Run it, let
it take the signal, and print the stack:

```text
(lldb) run
Process 51234 stopped
* thread #1, stop reason = EXC_BAD_ACCESS (code=1, address=0x0)
(lldb) bt
```

The frame you land in is where the bad pointer was dereferenced, which is not
necessarily where the bug is. Walk up until the values stop making sense.

## A note on Rust symbol names

Everything above assumes your debugger can read the names in that stack. Since
Rust 1.97, `rustc` uses the v0 symbol-mangling scheme by default. Nightly has
done so since November 2025, so the toolchain this book pins already emits v0
symbols.

Symbol mangling is how the compiler turns a function into a linker-safe name.
For a C function, that's usually just the function name. Rust also has to
distinguish generic instantiations: `Vec<u8>::push` and `Vec<String>::push` are
different generated functions and therefore need different symbols.

The old scheme used a hash for much of that distinction. It worked, but a
backtrace through generic code often left you with a readable function name and
an opaque suffix. The v0 scheme preserves the concrete generic arguments in a
form tools can decode, so when you're debugging or profiling a monomorphized
call chain, the backtrace can show which instantiation actually ran rather than
a hash.

This doesn't affect functions exported to C with `#[unsafe(no_mangle)]`, but it
does require every tool that reads Rust symbols to understand v0. Recent `gdb`,
`lldb`, and Rust tooling do. Older debuggers, profilers, or crash-symbolication
pipelines may show raw names such as `_RNvNtNtCs...` instead. Check your whole
debugging path before upgrading a pinned CI or production toolchain.

A quick test is enough: write a small generic function that panics, run it with
`RUST_BACKTRACE=1`, and read the backtrace.

See the [stabilization PR][v0-mangling] and the
[v0 symbol-format documentation][v0-format] for details.

## Head to the exercise

Head to the exercise, where a small C helper and its Rust binding disagree about
something that only shows up once you call across the boundary. Build it with
`cargo build -p bm_lldb_gdb`, then debug `target/debug/bm_lldb_gdb`. No
`println!` allowed.

[v0-mangling]: https://github.com/rust-lang/rust/pull/151994
[v0-format]: https://doc.rust-lang.org/stable/rustc/symbol-mangling/v0.html
