# Types that can't cross

The previous sections covered the types that do cross: scalars, and aggregates
that carry a `repr` attribute. That leaves the ones that don't, and it is worth
knowing why they don't, because the compiler's warning tells you that a type is
a problem without saying what to do about it.

The list is long, but as a rule of thumb, two kinds of type cannot be shared:
anything generic, and any Rust type that is not explicitly FFI-safe, such as
`String`.

`String` is the easier case. It is a pointer, a length, and a capacity, laid out
however the compiler likes, and it owns its allocation. C has no type that means
all of that, so there is nothing to pair it with. The same goes for `Vec<T>`,
`&str`, and every other type whose layout Rust reserves the right to change.

_Generics_ are not FFI-safe because the compiler will monomorphize a concrete
version of the struct for each type passed into the generic. If we pass the type
across the FFI boundary, the C compiler, which does not know about
monomorphization, cannot know which version to pick. There exists no ABI that
represents generics.

The compiler does tell you about all of this, in both directions: declaring such
a type in an `extern` block, or defining an `extern "C"` function that takes or
returns one, produces a warning that names the type and says it isn't FFI-safe.

What to reach for instead depends on the type. A string becomes a pointer plus a
length, or a NUL-terminated `*const c_char`. A `Vec` becomes a pointer and a
length. A type that has no useful shape to expose at all becomes a handle, which
is the next section.

## Head to the exercise

There you will find an FFI function that attempts to pass types that are not
FFI-safe. Notice the compiler-generated warnings: it is your job to fix them by
using FFI-safe types.
