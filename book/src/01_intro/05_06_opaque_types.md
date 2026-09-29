# Opaque types

The previous section left a question open: what do you do with a type that can't
cross? A `Vec` is one of those, and so is a C `FILE` in the other direction.
Mirroring either of them field by field would mean two definitions that have to
be kept in step forever.

The alternative is to let a pointer cross and keep the type behind it private. C
calls this an opaque type, and it's the shape most C libraries already use:
`FILE *`, `sqlite3 *`, `pthread_mutex_t *`. One side owns the layout, the other
side only ever holds the address.

## A Rust type behind a C handle

The C side gets a type name with no definition, and functions that take a
pointer to it:

```c
typedef struct Counter Counter;

Counter *counter_new(uint32_t start);
void counter_add(Counter *counter, uint32_t amount);
uint32_t counter_value(const Counter *counter);
void counter_free(Counter *counter);
```

`typedef struct Counter Counter;` declares a struct that is never defined. C can
pass `Counter *` around, but it can't allocate one, copy one, or read a field,
because it doesn't know what's in there.

On the Rust side, the type is an ordinary Rust type. What makes it opaque is
that the boundary only ever hands out pointers to it:

```rust,no_run
pub struct Counter {
    value: u32,
}

/// Creates a counter. The caller owns it and must pass it to `counter_free`.
#[unsafe(no_mangle)]
pub extern "C" fn counter_new(start: u32) -> *mut Counter {
    Box::into_raw(Box::new(Counter { value: start }))
}

/// # Safety
///
/// `counter` must come from `counter_new` and must not have been freed. No
/// other call may touch the same counter for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_add(counter: *mut Counter, amount: u32) {
    // SAFETY: the caller guarantees a live counter, and no other reference to
    // it exists during this call.
    let counter = unsafe { &mut *counter };
    counter.value += amount;
}

/// # Safety
///
/// `counter` must come from `counter_new` and must not have been freed. No
/// other call may modify the same counter for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_value(counter: *const Counter) -> u32 {
    // SAFETY: the caller guarantees a live counter.
    unsafe { &*counter }.value
}

/// # Safety
///
/// `counter` must come from `counter_new` and must not have been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_free(counter: *mut Counter) {
    if counter.is_null() {
        return;
    }
    // SAFETY: the caller guarantees the pointer came from `Box::into_raw` in
    // `counter_new` and hasn't been freed yet.
    drop(unsafe { Box::from_raw(counter) });
}
```

`Box::into_raw` hands ownership of the allocation to the caller and stops Rust
from freeing it. `Box::from_raw` takes that ownership back, and dropping the box
runs the destructor and releases the memory. They come in pairs, and every
`counter_new` needs exactly one `counter_free`.

A few rules keep this sound:

- **Only the owning side dereferences.** C holds an address and nothing else.
- **Pair every constructor with a destructor**, and document which is which.
  After `counter_free`, the pointer C holds is dangling, and using it is
  undefined behavior that no amount of checking on the Rust side can catch.
- **Don't hand out references that outlive the call.** Turning the pointer into
  a `&mut Counter` for the duration of one function is fine. Storing that
  reference somewhere is not, because C may free the counter at any time.
- **Keep the struct out of the header.** If C can see the fields, it can read
  them, and then the layout is part of your API whether you meant it or not.

The C side of this is a header you write by hand or generate. Either way, the
declaration C sees is the `typedef` above, with no fields in it.

## A C type behind a Rust handle

In the other direction, a C library hands you a pointer to something you know
nothing about:

```c
typedef struct Widget Widget;

Widget *widget_open(int id);
int widget_id(const Widget *widget);
void widget_close(Widget *widget);
```

The wrong move is to guess. Declaring a Rust struct with a plausible field means
two definitions of the same memory, and the C library is free to change its
version in the next release.

Rust has a pattern for a type that stands for "something whose layout is none of
my business":

```rust,no_run
use std::ffi::c_int;
use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub struct Widget {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

unsafe extern "C" {
    fn widget_open(id: c_int) -> *mut Widget;
    fn widget_id(widget: *const Widget) -> c_int;
    fn widget_close(widget: *mut Widget);
}
```

Each piece of that definition does a job:

- **`_data: ()`** keeps the struct zero-sized, and its fields are private, so no
  code outside this module can build a `Widget`. The only way to get one is a
  pointer from C.
- **`PhantomData<*mut u8>`** makes the type neither `Send` nor `Sync`, so it
  can't be moved or shared across threads until you decide that's sound and say
  so.
- **`PhantomPinned`** makes it `!Unpin`, which says the thing behind the pointer
  may not be safe to move. Anything C keeps an address to usually isn't.

Rust has a language feature for this, `extern type`, which says the same thing
in one line. It's still unstable, so the struct above is what you write today.

None of the three declarations is marked `safe`, and that's deliberate. An
`unsafe extern` block lets you mark an item `safe`, which means callers don't
need an `unsafe` block, and that's a promise that no argument the type system
allows can go wrong. `widget_id` reads through whatever address it is handed, so
calling it with null from safe Rust would be undefined behavior. The `unsafe`
stays on the declaration, and the wrapper below is what makes the calls safe. A
function that takes no pointers, such as a `widgets_open()` that returns a
count, is a fair candidate for `safe`.

Two rules apply here as well: never dereference the pointer, and never assume a
size. `size_of::<Widget>()` is 0 and means nothing, because the real size lives
in the C library and may change in its next release. That's the point of the
pattern, not a limitation of it.

Wrapping the handle in a Rust type with a `Drop` implementation turns the manual
`widget_close` into something the compiler tracks:

```rust,no_run
# use std::ffi::c_int;
# use std::marker::{PhantomData, PhantomPinned};
# use std::ptr::NonNull;
# #[repr(C)]
# pub struct Widget { _data: (), _marker: PhantomData<(*mut u8, PhantomPinned)> }
# unsafe extern "C" {
#     fn widget_open(id: c_int) -> *mut Widget;
#     fn widget_id(widget: *const Widget) -> c_int;
#     fn widget_close(widget: *mut Widget);
# }
# // Stand in for the C library, so that this example links.
# #[unsafe(export_name = "widget_open")]
# extern "C" fn widget_open_impl(id: c_int) -> *mut Widget {
#     Box::into_raw(Box::new(id)).cast()
# }
# #[unsafe(export_name = "widget_id")]
# extern "C" fn widget_id_impl(widget: *const Widget) -> c_int {
#     unsafe { *widget.cast::<c_int>() }
# }
# #[unsafe(export_name = "widget_close")]
# extern "C" fn widget_close_impl(widget: *mut Widget) {
#     drop(unsafe { Box::from_raw(widget.cast::<c_int>()) });
# }
pub struct OwnedWidget(NonNull<Widget>);

impl OwnedWidget {
    pub fn open(id: c_int) -> Option<Self> {
        // SAFETY: `widget_open` returns either a valid widget or null.
        let widget = unsafe { widget_open(id) };
        NonNull::new(widget).map(OwnedWidget)
    }

    pub fn id(&self) -> c_int {
        // SAFETY: the widget came from `widget_open`, and `OwnedWidget` closes
        // it only in `drop`, so it is still open here.
        unsafe { widget_id(self.0.as_ptr()) }
    }
}

impl Drop for OwnedWidget {
    fn drop(&mut self) {
        // SAFETY: the widget came from `widget_open` and is closed exactly
        // once, because `OwnedWidget` owns it and is not `Copy`.
        unsafe { widget_close(self.0.as_ptr()) };
    }
}
```

## Head to the exercise

The exercise has both directions. First, expose a Rust type to C as a handle,
with a constructor, two operations, and a destructor. Then take a handle from a
C library whose struct you can't see, and wrap it in a Rust type that closes it
exactly once. The tests check that every handle you open is closed again.
