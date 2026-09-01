# Allocation, allocators, and mixing them

In the previous section you saw how to replace a whole C module with Rust while
keeping the interface identical. But `bm` passes more than integers and borrowed
strings across the boundary: it passes _ownership_. `bookmark_new` allocates a
`Bookmark` that someone else will free later. Who frees it, and with what? This
is where mixed-language codebases hide some of their nastiest bugs and most
difficult design challenges.

## The golden rule

Every allocation must be released through its corresponding deallocation API.

That's the whole rule. C's `malloc`/`free`[^1] and Rust's global allocator are
two independent bookkeeping systems[^2]. Handing a Rust-allocated pointer to C's
`free` (or a `malloc`'d pointer to `Box::from_raw`) is undefined behavior, even
if it happens to "work" on your machine because both forward to the same
underlying `malloc` today. They might stop doing so tomorrow, or not do so at
all on a different architecture.

```text
┌──────────────┐  Box::into_raw   ┌──────────────┐
│ Rust global  │ ───────────────► │  C code      │
│ allocator    │ ◄─────────────── │  (borrows)   │
└──────────────┘  Box::from_raw   └──────────────┘
```

## Crossing the boundary in practice

There are several workable strategies:

1. **Use C allocation APIs from Rust.** Rust can call a C constructor such as
   `bookmark_new` and wrap the returned pointer in an owning Rust type. Its
   `Drop` implementation releases the allocation by calling the matching C
   function, `bookmark_free`. Keep allocation and deallocation within the same C
   API; export the functions through FFI to Rust.

   Here is the complete shape of that wrapper. `OwnedCBookmark` owns the C
   allocation, so callers cannot accidentally pass it to Rust's allocator. A
   null pointer from C becomes `None`; dropping a successfully created wrapper
   always calls back into C to free it.

   ```rust
   use std::ffi::CStr;
   use std::ptr::{self, NonNull};

   #[repr(C)]
   // An opaque type, C owns the layout and allocation.
   struct CBookmark {
       _data: (),
       _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
   }

   // FFI (de)allocation functions
   unsafe extern "C" {
       fn bookmark_new(
           url: *const std::ffi::c_char,
           tags: *const *const std::ffi::c_char,
           n_tags: usize,
       ) -> *mut CBookmark;
       fn bookmark_free(bookmark: *mut CBookmark);
   }

   // The Rust wrapper
   struct OwnedCBookmark(NonNull<CBookmark>);

   impl OwnedCBookmark {
       fn new(url: &CStr) -> Option<Self> {
           // SAFETY: `url` is NUL-terminated. This example passes no tags,
           // matching the C function's documented null-and-zero contract.
           let bookmark = unsafe { bookmark_new(url.as_ptr(), ptr::null(), 0) };
           NonNull::new(bookmark).map(Self)
       }
   }

   impl Drop for OwnedCBookmark {
       fn drop(&mut self) {
           // SAFETY: this pointer came from `bookmark_new` and this wrapper is
           // its unique owner, so `bookmark_free` is called exactly once.
           unsafe { bookmark_free(self.0.as_ptr()) };
       }
   }
   ```

2. **Use Rust allocators from C.** A Rust type that crosses the boundary will
   need a matching `_new`/`_free` pair exported through FFI. `_new` in Rust is
   paired with `_free` in Rust; C only ever holds the pointer and calls the Rust
   functions to allocate or free an object of that type.

   The basic pattern for this is to convert a `Box` to and from a raw pointer
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

   pub struct RustType {
       foo: i32,
       bar: i32,
   }

   #[unsafe(no_mangle)]
   pub extern "C" fn rust_type_new(foo: i32, bar: i32) -> *mut RustType {
       Box::into_raw(Box::new(RustType { foo, bar }))
   }

   #[unsafe(no_mangle)]
   pub extern "C" fn rust_type_free(b: Option<NonNull<RustType>>) {
       // To match C's free(NULL) semantics, return immediately when b is null.
       let Some(b) = b else {
           return;
       };

       // SAFETY: `b` was created by `rust_type_new` via Box::into_raw and is
       // not used again after this call (documented in bookmark.h).
       drop(unsafe { Box::from_raw(b.as_ptr()) });
   }
   ```

3. **Transfer ownership of a C allocation across the boundary.** Use this when
   one language calls a C allocation API, transfers ownership of the result to
   the other language, and the receiving side later calls the matching C
   deallocation API. For example, Rust can allocate a NUL-terminated string with
   [libc::malloc](https://docs.rs/libc/latest/libc/fn.malloc.html) and pass it
   to C code that retains it and later calls `free`.

   The reverse direction is valid too: C can transfer ownership of a value it
   allocated, for example with `malloc`, to Rust, and Rust can later call the
   matching deallocation function through FFI. In both cases, the allocation and
   deallocation calls originate on opposite sides of the FFI boundary. The
   layout and ownership contract must exactly match what the receiving API
   expects.

4. **Use a `malloc`/`free` adapter as Rust's global allocator.** On Linux,
   Rust's default global allocator typically already uses the system allocator,
   so this is usually unnecessary. It can still be useful when Rust must use a
   specific C allocation API. Implementing the adapter is non-trivial: it's
   inherently `unsafe`, and you need to carefully uphold all the safety
   guarantees required by the
   [GlobalAlloc](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html)
   trait. Do not pass Rust-owned values to C for it to free. Even with a
   `malloc`/`free` global allocator, C cannot run Rust destructors; freeing a
   value containing a `String`, `Vec`, or other owned resource leaks what that
   value owns.

   **Avoid if possible.** An allocation adapter is trickier than it seems. The
   code is small, but getting its allocation contract right requires a lot of
   implicit knowledge. Only use one when you have a specific interoperability
   need.

5. **Use the Allocator API.** As of this writing, the API is still experimental
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

   `foo` and `bar` now use different allocators.

   **Avoid if possible.** The same warning as for `GlobalAlloc` applies here:
   implementing an allocator is a non-trivial undertaking and is only worth it
   if you have a specific use case.

   This API is primarily useful for arena or bump allocators, where many related
   values are allocated from the same memory region and released together.
   Another useful case is selectively integrating parts of your program with a
   different allocation system while still using standard owning types such as
   `Box` and `Vec`.

## Spotting allocator mismatches

Tools like Valgrind give every allocation a birth certificate: run the test
binary under `valgrind --leak-check=full` and it will tell you not just that a
block leaked, but which call stack allocated it.

## Head to the exercise

You'll port `bm`'s `Bookmark` type, including its allocation and deallocation
functions, to Rust without leaking or double-freeing a single byte.

You'll practice both ownership directions from this section in
`exercises/02_intermediate/02_allocators`: C holding a Rust allocation, and Rust
holding a C allocation.

[^1]: `malloc`/`free` are standard C allocation functions, but nothing prevents
    you from using other allocation functions or even rolling your own. For the
    sake of simplicity, we will always refer to `malloc`/`free` in this section,
    but read it as `malloc`/`free` and all other custom (de)allocation
    functions.

[^2]: Currently, Rust's default global allocator is unspecified, but on many
    platforms it _is_ the system `malloc`. Libraries such as cdylibs and
    staticlibs, however, are guaranteed to use the `System` allocator by
    default. Beware though: C's `malloc`/`free` API and Rust's global-allocation
    API are not interchangeable, even when both happen to use the same
    underlying system allocator.

[^3]: There are some other useful structs, besides `Box`, that come with their
    own `from_raw` and `into_raw` functions, like `Vec` or `CString`. The same
    pattern as for `Box` applies here: allocate in Rust → convert into raw
    pointer and transfer ownership to C → use the pointer in C → free the
    pointer by transferring ownership back to Rust.
