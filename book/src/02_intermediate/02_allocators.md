# Allocation, allocators, and mixing allocators

In the previous exercise you replaced a whole C module with Rust while keeping
the interface identical. But `bm` passes more than integers and borrowed strings
across the boundary: it passes _ownership_. `bookmark_new` allocates a
`Bookmark` that someone else will free later. Who frees it, and with what? This
is where mixed-language codebases hide some of their nastiest bugs and most
difficult design challenges.

## The golden rule

Every allocation must be released through its corresponding deallocation API.

That's the whole rule. C's `malloc`/`free`[^1] and Rust's global allocator are
two independent bookkeeping systems[^2]. Handing a Rust-allocated pointer to C's
`free` (or a `malloc`'d pointer to `Box::from_raw`) is undefined behavior, even
if it happens to "work" on your machine, because both may forward to the same
underlying `malloc` today. It might stop doing so tomorrow or not work at all on
a different machine architecture.

```text
┌──────────────┐  Box::into_raw   ┌──────────────┐
│ Rust global  │ ───────────────► │  C code      │
│ allocator    │ ◄─────────────── │  (borrows)   │
└──────────────┘  Box::from_raw   └──────────────┘
```

## Crossing the boundary in practice

There are several workable strategies:

1. **Use Rust allocators from C.** A Rust type that crosses the boundary will
   need a matching `_new`/`_free` pair exported through FFI. `_new` in Rust is
   paired with `_free` in Rust; C only ever holds the pointer and calls the Rust
   functions to allocate or free an object of that type.

   The basic pattern for this is to convert raw pointers from and to a `Box`
   with `into_raw` and `from_raw`:[^3]

   ```rust
   let foo = Box::new(42);
   let ptr = Box::into_raw(foo);
   // This will convert the raw pointer back into a Box and when dropped free the memory:
   let _foo = unsafe { Box::from_raw(ptr) };
   ```
   And a full FFI split would look something like this:

   ```rust
   use std::ptr::NonNull;

   struct RustType {
      foo: i32,
      bar: i32
   };

   #[unsafe(no_mangle)]
   pub extern "C" fn rust_type_new(foo: i32, bar: i32) -> *mut RustType {
      Box::into_raw(Box::new(RustType{ foo, bar }))
   }

   #[unsafe(no_mangle)]
   pub extern "C" fn rust_type_free(b: Option<NonNull<RustType>>) {
      // To match C's free(NULL) semantics, return immediately when b is null.
      let Some(b) = b else {
         return;
      }

      // SAFETY: `b` was created by `rust_type_new` via Box::into_raw and is
      // not used again after this call (documented in bookmark.h).
      drop(unsafe { Box::from_raw(b.as_ptr()) });
   }
   ```

2. **Use C allocators from Rust.** C owns the allocation. When existing C code
   insists on calling `free` on what we hand it, we allocate with
   [libc::malloc](https://docs.rs/libc/latest/libc/fn.malloc.html) on the Rust
   side. This doesn't apply only to `malloc`/`free` of course, but to all
   allocation APIs written in C, like in our example where we use `bookmark_new`
   and `bookmark_free`. To use them, they need to be exported through FFI to
   Rust.

3. **Make Rust use `malloc` globally.** Registering a custom `GlobalAlloc` that
   forwards to `malloc`/`free` (or any other allocator in C) makes both worlds
   share one allocator. **Avoid if possible!** Only go this route when you
   absolutely have to. Writing an allocator is a non-trivial task: it's
   inherently `unsafe`, and you need to be very careful to uphold all the safety
   guarantees required by the
   [GlobalAlloc](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html)
   trait. Sharing an allocator ensures that both languages use the same
   mechanism for allocating and deallocating memory. It does not make arbitrary
   Rust values safe for C to destroy: C will not run Rust destructors, so values
   containing String, Vec, or other owned resources may leak.

4. **Use the Allocator API.** As of this writing the API is still experimental
   and only available on nightly Rust. It functions similarly to `GlobalAlloc`:
   you have to implement your own allocator, but unlike the global one, it is a
   lot more flexible and can be used on a per-variable basis. The new API adds
   an [Allocator](https://doc.rust-lang.org/std/alloc/trait.Allocator.html)
   trait and `Allocator` type parameters to many allocation-owning types, such
   as `Box` or `Vec`.

   ```rust,ignore
   pub struct Box<T, A = Global>()
   // Global is the global allocator implementing the Allocator trait
   where
       A: Allocator,
       T: ?Sized;
   ```

   Additionally, it adds a few new functions where you can pass an allocator,
   for example `new_in`:

   ```rust,ignore
   #![feature(allocator_api)]

   use std::alloc::System;
   let foo = Box::new_in(1, System);

   let custom_alloc = SomeCustomAlloc::new();
   let bar = Box::new_in(2, custom_alloc);
   ```

   Both `foo` and `bar` will now use different allocators.

   **Avoid if possible!** The same warning as for `GlobalAlloc` applies here:
   implementing an allocator is a non-trivial undertaking and is only worth it
   if you have a very specific use case.

   This API is primarily useful for arena or bump allocators, where many related
   values are allocated from the same memory region and released together.
   Another useful case is selectively integrating parts of your program with a
   different allocation system while still using standard owning types such as
   `Box` and `Vec`.

## Spotting allocator mismatches

You can use tools like Valgrind, which gives every allocation a birth
certificate: run the test binary under `valgrind --leak-check=full` and it will
tell you not just that a block leaked, but which call stack allocated it. We'll
go deeper on this in chapter 3.

## Head to the exercise

Head to the exercise, where you'll port `bm`'s `Bookmark` type, including its
allocation and deallocation functions, to Rust without leaking or double-freeing
a single byte.

You'll practice both ownership directions from this chapter in
`exercises/02_intermediate/02_allocators`: C holding a Rust allocation, and Rust
holding a C allocation.

[^1]: `malloc`/`free` are standard C allocation functions, but nothing prevents
    you from using other allocation functions or even rolling your own. For the
    sake of simplicity we will always refer to malloc/free in this chapter, but
    read it as malloc/free + all other custom (de)allocation functions.

[^2]: Currently Rust's default global allocator is unspecified, but on many
    platforms it _is_ the system `malloc`. Libraries, however, like cdylibs and
    staticlibs, are guaranteed to use the System allocator by default. Beware
    though: C's malloc/free API and Rust's global-allocation API are not
    interchangeable, even when both happen to use the same underlying system
    allocator.

[^3]: There are some other useful structs, besides `Box`, that come with their
    own `from_raw` and `into_raw` functions, like `Vec` or `CString`. The same
    pattern as for `Box` applies here: allocate in Rust -> convert into raw
    pointer and transfer ownership to C -> use the pointer in C -> free the
    pointer by transferring ownership back to Rust.
