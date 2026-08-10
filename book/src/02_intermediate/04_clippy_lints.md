# Clippy lints for unsafe Rust & static verification

A debugger tells you about a bug after it happened: you hit the crash, then work
backward to find the cause. A lint catches the same class of mistake before the
code runs, at the exact line that would have caused it. That's most valuable in
exactly the code this chapter is about: `unsafe` blocks, where the compiler
stops checking most of what you do and trusts whatever you tell it instead. It
still can't prove a pointer is valid, but it can refuse to compile code that
hides what you're assuming about that pointer.

## Make unsafe operations explicit

Start by enabling these lints in the crate root:[^1]

```rust
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::unnecessary_safety_doc)]
#![warn(clippy::ptr_as_ptr)]
#![warn(clippy::cast_ptr_alignment)]
#![warn(clippy::transmute_ptr_to_ptr)]
```

This chapter focuses on Clippy, but `unsafe_op_in_unsafe_fn` is a compiler lint,
not a Clippy lint. It's worth mentioning here because it makes unsafe operations
explicit, which is the foundation for the Clippy lints that follow. An
`unsafe
fn` means its caller has extra obligations. It doesn't mean every
operation in the body is automatically justified. With this lint enabled, each
dereference, FFI call, or other unsafe operation needs its own `unsafe {}`
block. It's allow-by-default in edition 2021 and warn-by-default in edition
2024, so it's worth denying explicitly either way.

The two documentation lints take that idea a step further.
`clippy::undocumented_unsafe_blocks` asks for a `// SAFETY:` comment before each
unsafe block. `clippy::multiple_unsafe_ops_per_block` keeps one block from
collecting several unrelated operations, each with a different set of
invariants. Together they turn a large, vague unsafe region into small claims
that can be checked one by one.

The remaining lints are especially useful when you're porting C habits
mechanically. Pointer casts and pointer transmutes often compile without
complaint, even if they discard an alignment guarantee or hide a more direct
API. A warning isn't proof that the code is wrong, but it's a good reason to
stop and check what the cast is claiming.

## The lints you already have

Three more fire without you enabling anything, and they're the ones that bite
hardest at the boundary.

`improper_ctypes` and `improper_ctypes_definitions` are rustc lints, warn by
default, and you've met them already: they're what complained about the
non-FFI-safe types back in chapter 1. `improper_ctypes` covers the types you
declare in an `extern` block, `improper_ctypes_definitions` the ones you define
with `extern "C"`. Between them they catch most type-level mistakes long before
a linker gets involved.

`clippy::not_unsafe_ptr_arg_deref` is a correctness lint, which means it's deny
by default. It fires when a public safe function dereferences a raw pointer it
received as an argument: the signature promises a safety the body can't deliver.
Silencing it is essentially never the right fix.

`clippy::missing_safety_doc` flags a public `unsafe fn` whose docs have no
`# Safety` section. It only looks at exported items unless you set
`check-private-items = true` in `clippy.toml`, which is worth doing in an FFI
crate, where plenty of the interesting unsafe code isn't public.

`clippy::unnecessary_safety_doc` is the converse: it flags `# Safety` sections
on public safe functions and traits. A safe API should not have safety
preconditions for its callers or implementors to uphold. This is a restriction
lint and allow-by-default, so enable it explicitly.

## When the lint is wrong

Porting from C also produces the opposite problem. C naming conventions don't
survive contact with rustc's style lints: a mirrored `bm_db_t` typedef trips
`non_camel_case_types`, an extern static trips `non_upper_case_globals`, and a
function or variable carrying its original name trips `non_snake_case`. The
group covering all three is `nonstandard_style`.

Sometimes the name genuinely does have to match, and then an `allow` is the
right answer. Scope it as tightly as you can:

```rust
#[allow(non_camel_case_types)]
pub struct bm_db_t {/* … */}
```

A crate-root `#![allow(non_snake_case)]` also silences the one name you
genuinely got wrong, which is the same reason we narrow `unsafe` blocks instead
of wrapping whole functions in them. Generated code you don't control is the
exception that earns a module-wide allow, and `bindgen` already handles its own:
it puts `#[allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]`
on the module it generates.

For exported functions you often don't need the allow at all. The pressure to
name a Rust function `BMNormalizeURL` comes from `#[unsafe(no_mangle)]`, which
forces the symbol to be the Rust name. If your header generator understands it,
`#[unsafe(export_name = "BMNormalizeURL")]` decouples the two: C gets the name
it expects, and the Rust function keeps the name you'd have given it anyway.

## Where to put them

Crate-root attributes are fine for a single crate. Across a workspace, the lint
tables in `Cargo.toml` are less work to keep in sync. They've been available
since Rust 1.74:

```toml
# Cargo.toml at the workspace root
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"

[workspace.lints.clippy]
undocumented_unsafe_blocks = "warn"
multiple_unsafe_ops_per_block = "warn"
```

Each member crate then opts in with:

```toml
[lints]
workspace = true
```

## `// SAFETY:` comments are part of the code

Consider a pointer received through an FFI function:

```rust
// SAFETY: `raw` is non-null (checked above) and points to a valid
// NUL-terminated byte sequence, as required by `bm_normalize_url`'s contract.
let raw = unsafe { CStr::from_ptr(raw) };
```

The null check happens in this function; the valid NUL-terminated sequence is
part of the caller's documented contract. A reviewer can check both claims
without re-deriving them from the surrounding code.

Compare that with this:

```rust
// SAFETY: this is fine
let raw = unsafe { CStr::from_ptr(raw) };
```

This moves no information into the code. The next reader still has to establish
whether `raw` is non-null, initialized, and terminated. Write the actual
reasoning down.

It's the same discipline as the `# Safety` documentation from chapter 1. The
function's `# Safety` section describes what its caller must guarantee. The
local `// SAFETY:` comment describes why one operation can rely on those
guarantees at this exact point in the implementation.

## Lints are a starting point, not a proof

Run the lints with:

```sh
cargo clippy --workspace --all-targets
```

`--all-targets` matters here. Without it `cargo clippy` skips tests, and in
these exercises the tests are where most of the FFI calls live.

`cargo clippy --fix` can apply some mechanical fixes. It wants a clean Git tree
and refuses to run on a dirty one unless you pass `--allow-dirty`. Review the
diff: replacing a pointer cast is easy; deciding whether the pointer is
correctly aligned, valid, and live is the real work.

Other mistakes won't trigger a lint at all. No compiler warning can tell you
that a C caller lies about a buffer length, or that two libraries expect
different allocators. Keep writing tests, use a debugger when they fail, and
treat every FFI contract as something to validate at the boundary.

## Head to the exercise

Head to the exercise, where you'll enable these lints on the crates from the
earlier exercises and fix everything they flag. Some warnings are mechanical;
others point at a real bug. The point isn't to make `cargo clippy` quiet. It's
to understand and document every unsafe operation that remains.

[^1]: The complete list of Clippy lints is available at
    <https://rust-lang.github.io/rust-clippy/master/index.html>.
