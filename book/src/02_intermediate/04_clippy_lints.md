# Clippy lints for unsafe Rust and static verification

In the previous section we learned how to debug an application, but debugging
only gets you so far: it starts from a crash you already have, and works
backward to the root cause.

Lints, on the other hand, help prevent certain issues in the first place. They
catch a narrower set of problems and suspicious patterns before the code runs.
Running them can be fully automated, which helps enforce coding standards across
an entire team.

They are especially useful in `unsafe` code. They can require each unsafe
operation to be explicit, flag signatures that don't state the caller's
obligations, and surface undocumented assumptions. They can't prove that a
pointer is valid or that a C caller is honest, but they can make unchecked
assumptions visible for review.

## Built-in lints

You already have several useful lints enabled by default.

`improper_ctypes` and `improper_ctypes_definitions` are rustc lints,
warn-by-default, and you've encountered them already: they're what complained
about the non-FFI-safe types back in Chapter 1. `improper_ctypes` covers the
types you declare in an `extern` block; `improper_ctypes_definitions` covers the
ones you define with `extern "C"`. Between them they catch most type-level
mistakes long before a linker gets involved.

`clippy::not_unsafe_ptr_arg_deref` is a correctness lint, so Clippy already
marks it as deny rather than warn. It fires when a public safe function
dereferences a raw-pointer argument even though its signature places no
requirements on the caller. Either make the function unsafe and document its
pointer contract, or keep it safe and accept a type that can be dereferenced
safely. Don't silence the lint.

`clippy::missing_safety_doc` flags a public `unsafe fn` whose docs have no
`# Safety` section. Set `check-private-items = true` in `clippy.toml` to apply
it to private items as well. This is worth doing in an FFI crate, where plenty
of the interesting unsafe code isn't public.

## More lints to turn on

You can configure a lint in code:

```rust
#![warn(clippy::undocumented_unsafe_blocks)]
```

Or you can use lint tables in `Cargo.toml`. We'll use `Cargo.toml` here to keep
this crate-wide policy in one place:[^1]

```toml
[lints.rust]
unsafe_op_in_unsafe_fn = "deny"

[lints.clippy]
undocumented_unsafe_blocks = "warn"
multiple_unsafe_ops_per_block = "warn"
unnecessary_safety_doc = "warn"
ptr_as_ptr = "warn"
cast_ptr_alignment = "warn"
transmute_ptr_to_ptr = "warn"
```

This section focuses on Clippy, but `unsafe_op_in_unsafe_fn` is a compiler lint,
not a Clippy lint. It's worth mentioning here because it makes unsafe operations
explicit, which is the foundation for the Clippy lints that follow. An
`unsafe fn` means its caller has extra obligations. It doesn't mean every unsafe
operation in the body is automatically justified. With this lint enabled, each
dereference, FFI call, or other unsafe operation needs its own `unsafe {}`
block. It's allow-by-default in edition 2021 and warn-by-default in edition
2024, so it's worth denying explicitly either way.

Two lints take that idea a step further. `clippy::undocumented_unsafe_blocks`
asks for a `// SAFETY:` comment before each unsafe block.
`clippy::multiple_unsafe_ops_per_block` encourages each block to contain a
single unsafe operation, keeping its safety argument focused. Together they turn
a large, vague unsafe region into small claims that can be checked one by one.

`clippy::unnecessary_safety_doc` catches the opposite documentation mistake: a
`# Safety` section on a public safe function or trait. A safe API should not
have safety preconditions for its callers or implementors to uphold. This is a
restriction lint and allow-by-default, so enable it explicitly.

The pointer lints earn their keep when you're porting C habits mechanically.
Pointer casts and pointer transmutes often compile without complaint, even when
they introduce a stricter alignment requirement or hide a more direct API. A
warning doesn't necessarily mean that the code is wrong, but it's a good reason
to double-check what the cast is claiming.

## When the lint is wrong

Porting from C can also produce the opposite problem. Some C naming conventions
conflict directly with rustc's style lints: a mirrored `bm_db_t` typedef
triggers `non_camel_case_types`, an extern static trips
`non_upper_case_globals`, and a function or variable that keeps its original
name trips `non_snake_case`. The group covering all three is
`nonstandard_style`.

Sometimes the name genuinely does have to match, and then an `allow` is the
right solution. Scope it as tightly as you can:

```rust
#[allow(non_camel_case_types)]
pub struct bm_db_t {/* … */}
```

A crate-root `#![allow(non_snake_case)]` also silences names you genuinely got
wrong. Prefer applying allows to individual items instead of whole modules or
entire crates. Generated code you don't control is the exception that can
justify a module-wide allow. Put it on the module containing the generated
bindings rather than on the entire crate.

You can often avoid an `allow` for functions exposed to C. `no_mangle` uses the
Rust function's name for the exported symbol, but `export_name` lets you choose
a different name for the C API:

```rust
#[unsafe(export_name = "BMNormalizeURL")]
pub extern "C" fn bm_normalize_url() {
    // ...
}
```

**Note:** `cheadergen` recognizes `export_name`, so it emits `BMNormalizeURL` in
the C header while the Rust function keeps its idiomatic name.

## Share lint settings across a workspace

The configuration above applies to one crate. To use it across a workspace, put
the tables in the root `Cargo.toml` instead. They've been available since Rust
1.74:

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

Consider a pointer that arrives across the FFI boundary:

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

That comment carries no information. The next reader still has to establish
whether `raw` is non-null, initialized, and terminated. Write the reasoning
down.

It's the same discipline as the `# Safety` documentation from Chapter 1. The
function's `# Safety` section describes what its caller must guarantee. The
local `// SAFETY:` comment describes why one operation can rely on those
guarantees at this exact point in the implementation.

## Lints are a starting point, not a proof

Run the lints with:

```sh
cargo clippy --workspace --all-targets
```

`--all-targets` matters here. Without it, `cargo clippy` skips tests, and in
these exercises the tests are where most of the FFI calls live.

`cargo clippy --fix` can apply some mechanical fixes. It wants a clean Git tree
and refuses to run on a dirty one unless you pass `--allow-dirty`. Review the
diff: replacing a pointer cast is easy; deciding whether the pointer is
correctly aligned, valid, and live is the more difficult work.

Other mistakes won't trigger a lint at all. No compiler warning can tell you
that a C caller lies about a buffer length, or that two libraries expect
different allocators. Keep writing tests, use a debugger when they fail, and
treat every FFI contract as something to validate at the boundary.

## Head to the exercise

You'll enable these lints on the crates from the earlier exercises and fix
everything they flag. Some warnings are mechanical; others point to a real bug.
The point isn't to make `cargo clippy` quiet. It's to understand and document
every unsafe operation that remains.

[^1]: The complete list of Clippy lints is available at
    <https://rust-lang.github.io/rust-clippy/master/index.html>.
