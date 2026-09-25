# `MaybeUninit` and `UnsafeCell`

Two of Rust's rules get awkward as soon as a C library is on the other side of
the call. Reading memory that hasn't been initialized is undefined behavior, and
while a `&T` exists, the value behind it must not change. C functions write into
memory the caller hands them, and C libraries keep modifying structs they have
given out pointers to. Each rule has a type that adjusts it.

## Memory C writes into: `MaybeUninit`

Most of the time you don't need anything special here. Build a value, pass a
pointer to it, and let C overwrite whatever you put there:

```rust,no_run
#[repr(C)]
#[derive(Default)]
pub struct Stats {
    pub count: u32,
    pub total: u64,
}

# #[unsafe(export_name = "read_stats")]
# unsafe extern "C" fn read_stats_impl(out: *mut Stats) { unsafe { out.write(Stats { count: 2, total: 7 }) } }
unsafe extern "C" {
    fn read_stats(out: *mut Stats);
}

let mut stats = Stats::default();
// SAFETY: `read_stats` writes through the pointer and doesn't keep it.
unsafe { read_stats(&mut stats) };
```

Zeroing two integers costs nothing, `Stats::default()` is a value that makes
sense on its own, and the code stays free of `unsafe` constructs beyond the call
itself. Reach for this first.

Two situations rule it out.

**The type has no valid default.** Consider a handle the library fills in, whose
first field is a pointer that is never null:

```rust,no_run
# use std::ffi::c_void;
# use std::ptr::NonNull;
#[repr(C)]
pub struct Handle {
    pub ptr: NonNull<c_void>,
    pub id: u32,
}
```

There is no sensible `Default` to write, because there is no address to put in
`ptr` until the library provides one. Zeroing the memory isn't an option either:
all-zero bytes are not a valid `NonNull`, and creating an invalid value is
undefined behavior the moment it exists, well before C gets a chance to
overwrite it.

**Initializing is wasted work.** A 4 KiB buffer that a C function is about to
fill doesn't need to be zeroed first. In a read loop, that's a memset per
iteration that has no effect on the result.

### Using `MaybeUninit`

`MaybeUninit<T>` is memory with the size and alignment of a `T`, and no
requirement that it currently holds one:

```rust,no_run
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

#[repr(C)]
pub struct Handle {
    pub ptr: NonNull<c_void>,
    pub id: u32,
}

# #[unsafe(export_name = "open_handle")]
# unsafe extern "C" fn open_handle_impl(out: *mut Handle) -> bool {
#     unsafe { out.write(Handle { ptr: NonNull::dangling(), id: 1 }) };
#     true
# }
unsafe extern "C" {
    fn open_handle(out: *mut Handle) -> bool;
}

pub fn open() -> Option<Handle> {
    let mut handle = MaybeUninit::<Handle>::uninit();

    // SAFETY: `as_mut_ptr` points at memory with the size and alignment of a
    // `Handle`, which is what `open_handle` requires.
    if !unsafe { open_handle(handle.as_mut_ptr()) } {
        return None;
    }

    // SAFETY: `open_handle` returned true, so it wrote every field.
    Some(unsafe { handle.assume_init() })
}
```

`assume_init` is the step where the type changes from `MaybeUninit<Handle>` to
`Handle`. Before it, the compiler assumes nothing about those bytes. After it,
it treats them as a valid `Handle`, including that `ptr` is non-null. Calling it
after a failed call, or when C filled in only part of the struct, is undefined
behavior even if nothing ever reads the fields.

Use the pointer that `as_mut_ptr` gives you, and don't take a `&mut Handle` to
memory that isn't initialized yet. The reference is invalid the moment it
exists, which is one step too early.

For the buffer case, the same type works per element, and only the part C
reports as written counts as initialized:

```rust,no_run
use std::mem::MaybeUninit;
use std::slice;

# #[unsafe(export_name = "read_block")]
# unsafe extern "C" fn read_block_impl(buf: *mut u8, len: usize) -> usize {
#     let n = len.min(5);
#     unsafe { slice::from_raw_parts_mut(buf, n) }.copy_from_slice(b"hello");
#     n
# }
unsafe extern "C" {
    fn read_block(buf: *mut u8, len: usize) -> usize;
}

let mut buf = [const { MaybeUninit::<u8>::uninit() }; 4096];

// SAFETY: the pointer and length describe `buf`, and `read_block` only writes.
let written = unsafe { read_block(buf.as_mut_ptr().cast::<u8>(), buf.len()) };

// SAFETY: `read_block` reported writing `written` bytes, so that prefix is
// initialized. The rest of `buf` is not, and isn't read here.
let data = unsafe { slice::from_raw_parts(buf.as_ptr().cast::<u8>(), written) };
assert_eq!(data, b"hello");
```

## Mutation behind a shared reference: `UnsafeCell`

C APIs hand out `const struct foo *` constantly. In C, that `const` is a
restriction on the caller: this code will not write through the pointer. It says
nothing about the object. The library keeps updating fields while the caller
holds that pointer: a reference count, a cached length, a status word, a lock
inside the struct. Callers are expected to read only the fields the
documentation describes as stable, and nothing in C enforces the split.

Rust has no way to express that split. A `&Foo` covers the entire value, and for
as long as it exists, no part of that value may change. The compiler depends on
it: it can read a field once and reuse the result, or reorder reads, because
nothing is allowed to write in between. Handing out a `&Foo` for a struct the C
library mutates is undefined behavior even when your code only reads the fields
the library never touches.

There are two ways to deal with it:

- **Don't create the reference.** Keep the raw pointer and read individual
  fields through it. The `const` half of the C contract is then something your
  code follows by hand, which is what C callers do as well.
- **Mark the fields that change.** If the type definition lives on the Rust
  side, wrap those fields in `UnsafeCell<T>`. The compiler then stops assuming
  they hold still, while the rest of the struct keeps the guarantees that make
  it optimizable.

`UnsafeCell<T>` is the only way to get mutation through a shared reference, and
`Cell` and `RefCell` are built on top of it:

```rust,no_run
use std::cell::UnsafeCell;

#[repr(C)]
pub struct Counter {
    // `UnsafeCell` is `repr(transparent)`, so C still sees a plain `uint32_t`.
    value: UnsafeCell<u32>,
}

# #[unsafe(export_name = "counter_bump")]
# unsafe extern "C" fn counter_bump_impl(value: *mut u32) { unsafe { *value += 1 } }
unsafe extern "C" {
    fn counter_bump(value: *mut u32);
}

impl Counter {
    pub fn new(value: u32) -> Self {
        Counter {
            value: UnsafeCell::new(value),
        }
    }

    /// Note the `&self`: the C side changes the value while Rust holds nothing
    /// but a shared reference to it.
    pub fn bump(&self) {
        // SAFETY: the value sits in an `UnsafeCell`, so writing to it through a
        // shared reference is allowed, and nothing else reads it here.
        unsafe { counter_bump(self.value.get()) }
    }

    pub fn get(&self) -> u32 {
        // SAFETY: nothing else is accessing the value here.
        unsafe { *self.value.get() }
    }
}
```

`get` hands back a `*mut T`, and everything you do with it is `unsafe`, because
`UnsafeCell` only removes an assumption the compiler would otherwise make. It
provides no synchronization: two overlapping accesses are still a data race,
including a C thread writing while Rust reads.

One consequence is easy to miss. A type containing an `UnsafeCell` is not
`Sync`, so Rust won't let you share it across threads until you state that the
sharing is sound. For memory a C library writes to on its own schedule, that is
the right default.

## What they have in common

Both types narrow what the compiler is allowed to assume. `MaybeUninit` drops
the assumption that the bytes hold a valid value, and `UnsafeCell` drops the
assumption that the value stays put while it's borrowed. In exchange, you take
on the corresponding condition yourself, which is what lets you describe what
the code on the other side of the boundary actually does.

## Head to the exercise

The exercise is one function: call a C-style function that fills a struct
through an out-parameter, and hand back the struct it wrote, or nothing when the
call reports failure. The struct has a `NonZero` field, so starting from a
zeroed or default value isn't an option.
