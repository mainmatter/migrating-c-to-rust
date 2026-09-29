// Part one: a Rust type that C only ever sees as a pointer.
//
// The header C compiles against is `c_test/counter.h`, and it says:
//
//     typedef struct Counter Counter;
//
//     Counter *counter_new(uint32_t start);
//     void counter_add(Counter *counter, uint32_t amount);
//     uint32_t counter_value(const Counter *counter);
//     void counter_free(Counter *counter);
//
// `c_test/test_counter.c` is a C program that uses those four functions, and
// `wr` builds and runs it against your implementation.
//
// The stubs below return placeholder values rather than calling `todo!()`,
// because a panic that reaches an `extern "C"` function aborts the process
// instead of unwinding, and an abort takes the whole test run with it.

pub struct Counter {
    value: u32,
}

/// Creates a counter. The caller owns it, and has to pass it to
/// `counter_free`.
#[unsafe(no_mangle)]
pub extern "C" fn counter_new(start: u32) -> *mut Counter {
    // TODO: put the counter on the heap and hand ownership of it to C.
    let _ = start;
    std::ptr::null_mut()
}

/// Adds `amount` to the counter.
///
/// # Safety
///
/// `counter` must come from `counter_new` and must not have been freed. No
/// other call may touch the same counter for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_add(counter: *mut Counter, amount: u32) {
    // TODO
    let _ = (counter, amount);
}

/// Reads the counter's value.
///
/// # Safety
///
/// `counter` must come from `counter_new` and must not have been freed. No
/// other call may modify the same counter for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_value(counter: *const Counter) -> u32 {
    // TODO
    let _ = counter;
    0
}

/// Frees a counter. Passing null does nothing.
///
/// # Safety
///
/// `counter` must come from `counter_new`, and must not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn counter_free(counter: *mut Counter) {
    // TODO: take ownership back from C, and drop it.
    let _ = counter;
}

// Part two: a C type that Rust only ever sees as a pointer.
//
// `c_src/widget.h` declares `typedef struct Widget Widget;` and three
// functions. The struct itself is defined in `widget.c`, so this crate has no
// way of knowing what is in it, and no business guessing.

use std::ffi::c_int;
use std::ptr::NonNull;

// TODO: replace this with a type that stands for "a C struct whose layout is
// none of my business". It must have no public fields, so nothing outside this
// module can build one, and it must be neither `Send` nor `Sync` nor `Unpin`,
// because the C library owns the memory and may care about its address.
pub struct Widget;

unsafe extern "C" {
    fn widget_open(id: c_int) -> *mut Widget;
    // Both of these read through the pointer they are given, so neither can be
    // marked `safe`: calling them with null has to stay a caller's problem.
    fn widget_id(widget: *const Widget) -> c_int;
    fn widget_close(widget: *mut Widget);
    /// How many widgets the C library currently has open. It takes no
    /// pointers, so there is nothing a caller can get wrong.
    pub safe fn widgets_open() -> c_int;
}

/// A widget handle that closes itself.
pub struct OwnedWidget(NonNull<Widget>);

impl OwnedWidget {
    /// Opens a widget, or returns `None` if the C library couldn't.
    pub fn open(id: c_int) -> Option<Self> {
        // SAFETY: `widget_open` returns a valid widget or null.
        let widget = unsafe { widget_open(id) };
        NonNull::new(widget).map(OwnedWidget)
    }

    /// Returns the widget's id.
    pub fn id(&self) -> c_int {
        // SAFETY: the widget came from `widget_open` and is still open, since
        // `OwnedWidget` owns it and only closes it on drop.
        unsafe { widget_id(self.0.as_ptr()) }
    }
}

// TODO: make `OwnedWidget` close its widget when it goes out of scope, so that
// no caller has to remember. The test below checks that nothing stays open.

#[cfg(test)]
mod tests {
    use super::{OwnedWidget, counter_add, counter_free, counter_new, counter_value, widgets_open};

    #[test]
    fn counts_up_and_frees() {
        let counter = counter_new(1);
        assert!(!counter.is_null());

        unsafe { counter_add(counter, 41) };
        assert_eq!(unsafe { counter_value(counter) }, 42);

        unsafe { counter_free(counter) };
    }

    #[test]
    fn freeing_null_does_nothing() {
        unsafe { counter_free(std::ptr::null_mut()) };
    }

    #[test]
    fn closes_the_widget_it_opened() {
        let before = widgets_open();

        {
            let widget = OwnedWidget::open(7).expect("widget_open returned null");
            assert_eq!(widget.id(), 7);
            assert_eq!(widgets_open(), before + 1);
        }

        assert_eq!(widgets_open(), before);
    }
}
