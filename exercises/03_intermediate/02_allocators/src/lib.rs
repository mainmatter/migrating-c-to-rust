//! A global allocator that counts, and a leak to find with it.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ffi::{CString, c_char};

/// Forwards every request to the system allocator.
pub struct CountingAllocator;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

// SAFETY: every method forwards to `System` with the caller's arguments, so it
// upholds `GlobalAlloc`'s contract exactly as `System` does. `record` doesn't
// allocate.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller upholds `alloc`'s contract, which we pass on.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record(1);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record(-1);
        // SAFETY: `ptr` came from this allocator, which got it from `System`.
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
    // SAFETY: the caller guarantees `label` came from `CString::into_raw` in
    // `label_new` and isn't used again.
    drop(unsafe { CString::from_raw(label) });

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
