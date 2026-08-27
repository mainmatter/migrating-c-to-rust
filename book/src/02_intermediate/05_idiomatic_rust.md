# Replacing C-isms with Rust-isms

One of the hardest things when translating one language into another is
_idioms_: common patterns, practices, expressions. These are shaped by the
affordances of the language, the authors cultures, norms, backgrounds.

As with human language, translating computer languages suffers from the same
issues. You can do a _literal_ translation of a function or module, but it won't
be right: it won't sit well with other code, it will cause confusion, and
consequently destroy the confidence in our code we're trying so hard to
establish through a rewrite!

The good news is that these translations usually follow quite regular patterns
and are easy to wrap your head around. Throughout this chapter you will
experience a number of these patterns. To kick us off here are a few easy ones
to get us started:

## Out Parameters

C functions make frequent use of so-called "out parameters", pointers that the
function will write its outputs into. In a lot of situations this is great, the
caller has the flexbility to allocate the output however it sees fit. But with
this great flexbility comes great potential for confusion:

```c
int add(int *a, const int *b);
```

at a glance: what does this function return? how does it return? The second
argument is a const pointer, we can relatively safely assume thats simply an
argument. The first argument however is a mutable pointer, `add` could possibly
write the output there, but it also returns `int` so maybe thats where the
output goes?

We don't really have a way of knowing for sure. Here is the equivalent Rust
function:

```rust
fn add(a: &i32, b: &i32) -> Option<i32>;
```

It becomes obvious that `add` is some kind of "checked" addition function that
only sometimes returns an output, and sometimes fails.

This is fundamentally a limitation of C's (lack of) type system as expressing
fallibility necessarily means a return code of some sort which forces the output
to be expressed through an out parameter. In Rusts more expressive type system
we can write `Option<T>` to loudly signal the intent of our code to callers from
the signature alone.

## Error Codes

Likewise, in C code you will usually see the following:

```c
#define ERR_OK 0
#define ERR_FOO 1
#define ERR_BAR 2

int doSomething();
```

much like out parameters, the correct usage of this relies on conventions, good
documentation and proper code review. The compiler will not catch you
accidentally using the wrong constant, its all just `int`s after all!

In Rust we would express this using an `enum` (actually a combination of enums):

```rust
enum Error {
    Foo,
    Bar,
}

fn doSomething() -> Result<(), Error>;
```

here is a neat trick: since a Rust `enum` is just a `union` plus integer tag you
can cast an enum to an integer! As long as the enum doesn't have fields the
following works:

```rust
enum Error {
    Foo,
    Bar = 45, // excplicit tag
}

println!("{}", Error::Foo as usize); // prints 0
println!("{}", Error::Bar as usize); // print 45
```

by default enums have a `usize` tag and variants are just numbered starting at
zero. If you want to make sure tags match your old C version exactly (for
interop purposes for example) you might do this:

```rust
#[repr(u32)] // make sure we're using "int"s too
enum Error {
    Foo = 0,
    Bar = 1,
}

fn doSomething() -> Result<(), Error>;
```

now `Error` has the exact layout and bit pattern as the original C version had!

## Strings

Strings deserve extra care. The obvious translation would be `char *` => `&str`
(or `String`):

```c
void greet(const char *name);
```

```rust
fn greet(name: &str);
```

but C and Rust don't fully agree on what a string _is_: A C string is a pointer
to some bytes that end at the first NUL byte. A `&str` carries its length with
it, has no terminator and must be valid UTF-8. Handing a `&str` to C directly or
a casting a `char*` to a `&str` in Rust will cause immediate problems. C will
read past the end of a `&str` looking for a NUL that isn't there, and `char*`
must not necessarily be valid UTF8 at all.

The correct bridge is `CStr` and `CString`: `CStr::from_ptr(p).to_str()` checks
the bytes and gives you a `Result<&str, Utf8Error>`, `CString::new(s)` appends
the terminator and refuses anything with an interior NUL. [^1]

## Bit Flags

Same story as error codes, just with bits:

```c
#define FLAG_READ  (1 << 0)
#define FLAG_WRITE (1 << 1)
#define FLAG_EXEC  (1 << 2)

int open_file(const char *path, int flags);
```

`flags` is an `int`, so nothing stops you from passing `ERR_FOO` or `42`. The
`bitflags` crate generates a newtype around the integer with the named
constants, the `|`, `&` and `!` operators, and methods like `contains`, `insert`
and `remove`:

```rust
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

now the signature says what it accepts. For interop the same tricks as before
apply: `#[repr(transparent)]` gives `Flags` the exact layout of the `u32` it
wraps, `bits()` gets the raw integer back out, and `from_bits(x)` returns `None`
if any bit you didn't define is set. If C hands you values with bits you don't
know about (newer headers, reserved bits) use `from_bits_retain` which keeps
them as they are.

## Loops and Iterators

Here is the `matches` function from `bm`'s index:

```c
for (size_t i = 0; i < b->n_tags; i++) {
    if (strstr(b->tags[i], query))
        return 1;
}
return 0;
```

The literal translation is
`for i in 0..b.tags.len() { if b.tags[i].contains(query) { return true; } }` and
it compiles, but every `b.tags[i]` is a bounds check the optimizer may or may
not be able to prove away, and clippy will nag you about it
(`needless_range_loop`). Iterate the collection itself instead:
`for tag in &b.tags`, with `.enumerate()` if you actually need the index and
`.iter_mut()` if you need to change elements. Pointer-bumping loops like
`while (*p)` over a string usually become `.bytes()`, `.chars()` or
`split(',')`.

From there it is a short step to the combinators, and the loop above is just
`any`:

```rust
b.tags.iter().any(|tag| tag.contains(query))
```

A loop that searches for an element is `find` or `position`, one with a running
total is `map(..).sum()`, one that fills a second array is
`filter(..).map(..).collect()`. These are lazy and compile down to the same code
as the hand-written loop, so there is no cost to the shorter form, and the name
of the combinator tells the reader what the loop was for.

[^1]: Since Rust 1.77 you can even write `c"hello"` that gets you a a
    `&'static CStr` directly!
