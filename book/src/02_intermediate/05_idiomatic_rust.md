# Replacing C-isms with Rust-isms

One of the hardest things when translating one language into another is
_idioms_: common patterns, practices, and expressions. These are shaped by the
affordances of the language and by the authors' cultures, norms, and
backgrounds.

Translating between programming languages runs into the same problem as
translating between human ones. You can do a _literal_ translation of a function
or module, but it won't be right: it won't sit well with the code around it, it
will cause confusion, and it will destroy confidence in the code that the
rewrite was supposed to build.

The good news is that these translations follow fairly regular patterns and are
easy to wrap your head around. This section walks through several of them,
starting with the easy ones:

## Out parameters

C functions make frequent use of so-called "out parameters", pointers that the
function will write its outputs into. In a lot of situations this is great,
because the caller can allocate the output however it sees fit. But that
flexibility brings a lot of potential for ambiguity:

```c
int add(int a, int b, int *out);
```

At a glance: what does this function return, and how does it return it? `out` is
a mutable pointer, so `add` could write the output there, but it also returns
`int`, so maybe that's where the output goes?

We have no way of knowing for sure. Here is the equivalent Rust function:

```rust,compile_fail
fn add(a: i32, b: i32) -> Option<i32>;
```

It becomes clearer that `add` is some kind of "checked" addition function that
returns an output only sometimes, and otherwise fails.

This is fundamentally a limitation of what's possible with C's type system. A
return code plus an out parameter is what you will often find in the wild, and
nothing in the signature tells you which is which. In Rust's more expressive
type system we can write `Option<T>` to signal the intent of our code to callers
from the signature alone.

## Error codes

Likewise, in C code you will often see the following:

```c
#define ERR_OK 0
#define ERR_FOO 1
#define ERR_BAR 2

int do_something();
```

You may also see it written as a `typedef enum`, which is common in more modern
code and is what `bm` does.

Much like with out parameters, correct usage relies on conventions, good
documentation, and proper code review. The compiler will not catch you
accidentally using the wrong constant: C converts these to and from `int`
freely.

In Rust we would express this using an `enum` (actually a combination of enums):

```rust,compile_fail
enum Error {
    Foo,
    Bar,
}

fn do_something() -> Result<(), Error>;
```

Here is a neat trick: an enum whose variants carry no fields (a so-called
_field-less_ enum) is nothing but an integer tag, and you can cast it to an
integer with `as`:

```rust,no_run
enum Error {
    Foo,      // variants are numbered starting at zero
    Bar = 45, // explicit tag
}

println!("{}", Error::Foo as usize); // prints 0
println!("{}", Error::Bar as usize); // print 45
```

If you want to make sure tags match your old C version exactly (for interop
purposes, for example), you might do this:

```rust,compile_fail
#[repr(C)] // use the target's C ABI representation for enums
enum Error {
    Foo = 0,
    Bar = 1,
}

fn do_something() -> Result<(), Error>;
```

`#[repr(C)]` gives `Error` the size and signedness a C compiler would pick for
the equivalent `typedef enum` on the same target, so the two agree on the values
they exchange. Reach for `#[repr(i32)]` when you want to pin the width yourself,
which is what the `#define` version above needs, since it passes plain `int`s
around. Don't reach for `#[repr(u32)]`: C's `int` is signed, and the difference
surfaces the first time someone adds a negative error code.

## Strings

Strings deserve extra care. The obvious translation would be `char *` => `&str`
(or `String`):

```c
void greet(const char *name);
```

```rust,compile_fail
fn greet(name: &str);
```

But C and Rust don't fully agree on what a string _is_. A C string is a pointer
to some bytes that end at the first NUL byte. A `&str` carries its length with
it, has no terminator, and must be valid UTF-8. Handing a `&str` to C directly,
or casting a `char*` to a `&str` in Rust, will cause immediate problems. C will
read past the end of a `&str` looking for a NUL that isn't there, and a `char*`
isn't necessarily valid UTF-8 at all.

The correct bridge is `CStr` and `CString`: `CStr::from_ptr(p).to_str()` checks
the bytes and gives you a `Result<&str, Utf8Error>`; `CString::new(s)` appends
the terminator and refuses anything with an interior NUL.[^1]

## Bit flags

Same story as error codes, just with bits:

```c
#define FLAG_READ  (1 << 0)
#define FLAG_WRITE (1 << 1)
#define FLAG_EXEC  (1 << 2)

int open_file(const char *path, int flags);
```

`flags` is an `int`, so nothing stops you from passing `ERR_FOO` or `42`. The
`bitflags` crate generates a newtype around the integer with the named
constants, the `|`, `&`, and `!` operators, and methods like `contains`,
`insert`, and `remove`:

```rust,compile_fail
use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Flags: u32 {
        const READ  = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC  = 1 << 2;
    }
}

fn open_file(path: &Path, flags: Flags);
```

Now the signature says what it accepts. For interop, the same tricks as before
apply: `#[repr(transparent)]` gives `Flags` the exact layout of the `u32` it
wraps, `bits()` gets the raw integer back out, and `from_bits(x)` returns `None`
if any bit you didn't define is set. If C hands you values with bits you don't
know about (newer headers, reserved bits), use `from_bits_retain`, which keeps
them as they are.

## Loops and iterators

Here is the `matches` function from `bm`'s index:

```c
for (size_t i = 0; i < b->n_tags; i++) {
    if (strstr(b->tags[i], query))
        return 1;
}
return 0;
```

The literal translation is:

```rust,no_run
# struct Bookmark { tags: Vec<String> }
# fn matches(b: &Bookmark, query: &str) -> bool {
for i in 0..b.tags.len() { if b.tags[i].contains(query) { return true; } }
# false
# }
```

It works, but every `b.tags[i]` is a bounds check the optimizer may or may not
be able to prove away, and Clippy will nag you about it (`needless_range_loop`).
Iterate the collection itself instead: `for tag in &b.tags`, with `.enumerate()`
if you actually need the index and `.iter_mut()` if you need to change elements.

From there it is a short step to the combinators, and the loop above is just
`any` and looks like this:

```rust,no_run
# struct Bookmark { tags: Vec<String> }
# fn matches(b: &Bookmark, query: &str) -> bool {
b.tags.iter().any(|tag| tag.contains(query))
# }
```

The other common shape is the pointer-bumping loop:

```c
for (const char *p = s; *p; p++) {
    if (*p == ',')
        n++;
}
```

This one walks bytes until it hits the NUL terminator. In Rust the string knows
its own length, so loops like these become `.bytes()` or `.chars()`.

The pointer-bumping loop would look like this in Rust:

```rust,no_run
# let s = "rust,ffi,c";
let n = s.bytes().filter(|&b| b == b',').count();
```

A loop that searches for an element is `find` or `position`; one with a running
total is `map(..).sum()`; one that fills a second array is
`filter(..).map(..).collect()`. These are lazy and compile down to the same code
as the hand-written loop, so there is no cost to the shorter form, and the name
of the combinator tells the reader what the loop was for.

[^1]: Since Rust 1.77 you can write `c"hello"`, which gets you a `&'static CStr`
    directly.
