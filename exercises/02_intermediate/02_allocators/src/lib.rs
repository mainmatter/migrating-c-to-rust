//! A global allocator that counts, and a leak to find with it.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ffi::{CString, c_char};

/// Forwards every request to the system allocator.
pub struct CountingAllocator;

// TODO 1: make this the global allocator for the crate, and have it record
// every allocation with `record(1)` and every deallocation with `record(-1)`.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

/// Hands C a NUL-terminated copy of `label`. C has to pass it back to
/// `label_free`.
pub fn label_new(label: &str) -> *mut c_char {
    CString::new(label)
        .expect("labels have no NUL bytes")
        .into_raw()
}

/// Frees a label from `label_new`.
///
/// # Safety
///
/// `label` must come from `label_new`, and must not be used afterwards.
pub unsafe extern "C" fn label_free(label: *mut c_char) {
    // TODO 2: this looks like it takes the label back, but the tests say it
    // leaks. Fix it.
    let label = unsafe { std::ffi::CStr::from_ptr(label) };
    let _ = label;
}

// Each thread counts its own allocations, so tests running in parallel don't
// disturb each other's numbers. You don't need to change anything below.

thread_local! {
    static LIVE: Cell<isize> = const { Cell::new(0) };
}

/// Adds `delta` to the current thread's count of live allocations.
fn record(delta: isize) {
    let _ = LIVE.try_with(|live| live.set(live.get() + delta));
}

/// How many allocations made on this thread are still live.
pub fn live_allocations() -> isize {
    LIVE.with(Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many more allocations are live after `f` than before it.
    fn change_during(f: impl FnOnce()) -> isize {
        let before = live_allocations();
        f();
        live_allocations() - before
    }

    #[test]
    fn counts_allocations() {
        let mut boxed = None;

        assert_eq!(change_during(|| boxed = Some(Box::new(42))), 1);
        assert_eq!(change_during(|| boxed = None), -1);
    }

    #[test]
    fn labels_are_freed() {
        let mut label = std::ptr::null_mut();

        assert_eq!(change_during(|| label = label_new("rust")), 1);
        assert_eq!(change_during(|| unsafe { label_free(label) }), -1);
    }
}
