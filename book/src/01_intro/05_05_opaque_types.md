# Opaque types

Some types can't be shared with C at all. A `Vec` is one of those, and so is a C
`FILE` in the other direction. Neither can be mirrored field by field, because
neither layout is guaranteed: Rust reserves the right to change `Vec`'s, and the
C standard leaves `FILE`'s up to each C library.

The alternative is to let a pointer cross and keep the type behind it private. C
calls this an opaque type, and it's the shape most C libraries already use:
`FILE *`, `sqlite3 *`, `pthread_mutex_t *`. One side owns the object, its
layout, and its memory, and the other side only ever holds the address. So an
opaque type is really a question of ownership: who creates the object, and who
frees it.

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
pass `Counter *` around, but it can't allocate one, copy one, or read a field.

On the Rust side, `Counter` is an ordinary Rust struct. The constructor moves it
to the heap and hands ownership to C, and the destructor takes it back:

```rust,no_run
pub struct Counter {
    value: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn counter_new(start: u32) -> *mut Counter {
    Box::into_raw(Box::new(Counter { value: start }))
}

/// # Safety
///
/// `counter` must be null, or come from `counter_new` and not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_free(counter: *mut Counter) {
    if !counter.is_null() {
        // SAFETY: the pointer came from `Box::into_raw` in `counter_new`.
        drop(unsafe { Box::from_raw(counter) });
    }
}
```

`Box::into_raw` stops Rust from freeing the counter, and `Box::from_raw` makes
Rust responsible for it again, so dropping the box frees it.

The operations in between borrow the counter for the duration of one call.
`&mut *counter` turns the raw pointer into a `&mut Counter`, and `&*counter`
into a `&Counter`:

```rust,no_run
# pub struct Counter { value: u32 }
/// # Safety
///
/// `counter` must come from `counter_new` and must not have been freed. No
/// other call may use the same counter for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_add(counter: *mut Counter, amount: u32) {
    // SAFETY: the caller guarantees a live counter that nothing else uses.
    let counter = unsafe { &mut *counter };
    counter.value += amount;
}

/// # Safety
///
/// `counter` must come from `counter_new` and must not have been freed. No
/// other call may modify the same counter for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_value(counter: *const Counter) -> u32 {
    // SAFETY: the caller guarantees a live counter that nothing modifies.
    let counter = unsafe { &*counter };
    counter.value
}
```

These references are only sound because of the promises in the `# Safety`
sections: the pointer is live, and nothing else uses the counter while the
reference exists. A `&mut Counter` must be the only way to reach the counter, so
two overlapping calls on the same counter would be undefined behavior.

Two rules keep this sound:

- **Pair every constructor with a destructor.** Every `counter_new` needs
  exactly one `counter_free`. After that, the pointer C holds is dangling.
- **Don't keep references past the call.** C may free the counter at any time,
  so a reference stored somewhere would outlive it.

## A C type behind a Rust handle

In the other direction, a C library hands you a pointer to something you know
nothing about:

```c
typedef struct Widget Widget;

Widget *widget_open(int id);
int widget_id(const Widget *widget);
void widget_close(Widget *widget);
```

Don't guess its fields. Rust has a pattern for a type that stands for "something
whose layout is none of my business":

```rust,no_run
use std::ffi::c_int;
use std::marker::PhantomData;

#[repr(C)]
pub struct Widget {
    _data: (),
    _marker: PhantomData<*mut u8>,
}

unsafe extern "C" {
    fn widget_open(id: c_int) -> *mut Widget;
    fn widget_id(widget: *const Widget) -> c_int;
    fn widget_close(widget: *mut Widget);
}
```

Both fields are private, so the only way to get a `Widget` is a pointer from C.
`_data` keeps the `improper_ctypes` lint quiet, which would otherwise flag a
struct made only of `PhantomData`. The marker makes the type neither `Send` nor
`Sync`, so Rust won't assume it can be used from another thread.

To make sure every widget is closed exactly once, wrap the pointer in a type
that owns it. `NonNull` turns C's null into `None`, and `Drop` closes the widget
when the wrapper goes out of scope:

```rust,no_run
# use std::ffi::c_int;
# use std::marker::PhantomData;
# use std::ptr::NonNull;
# #[repr(C)]
# pub struct Widget { _data: (), _marker: PhantomData<*mut u8> }
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
        // SAFETY: the widget is still open, because `OwnedWidget` only closes
        // it in `drop`.
        unsafe { widget_id(self.0.as_ptr()) }
    }
}

impl Drop for OwnedWidget {
    fn drop(&mut self) {
        // SAFETY: the widget came from `widget_open`, and `OwnedWidget` is its
        // only owner, so it is closed exactly once.
        unsafe { widget_close(self.0.as_ptr()) };
    }
}
```

## Head to the exercise

The exercise has both directions, each with a twist the examples above don't
have. First, expose a Rust `Stats` type to C as a handle. Unlike the counter, it
owns a `Vec` of its own, which has to be freed along with it. Then wrap a C
`Buffer` in a Rust type that frees it exactly once.
