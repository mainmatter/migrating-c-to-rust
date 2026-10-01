// Part one: a Rust type that C only ever sees as a pointer.
//
// The header C compiles against is `c_test/stats.h`, and it says:
//
//     typedef struct Stats Stats;
//
//     Stats *stats_new(void);
//     void stats_add(Stats *stats, int32_t sample);
//     int64_t stats_sum(const Stats *stats);
//     void stats_free(Stats *stats);
//
// `c_test/test_stats.c` is a C program that uses those four functions, and
// `wr` builds and runs it against your implementation.
//
// The stubs below return placeholder values rather than calling `todo!()`,
// because a panic that reaches an `extern "C"` function aborts the process
// instead of unwinding, and an abort takes the whole test run with it.

/// Unlike the chapter's counter, `Stats` owns heap memory of its own.
pub struct Stats {
    samples: Vec<i32>,
}

/// Creates an empty `Stats`. The caller owns it, and has to pass it to
/// `stats_free`.
#[unsafe(no_mangle)]
pub extern "C" fn stats_new() -> *mut Stats {
    // TODO: put a `Stats` on the heap and hand ownership of it to C.
    std::ptr::null_mut()
}

/// Records one sample.
///
/// # Safety
///
/// `stats` must come from `stats_new` and must not have been freed. No other
/// call may use the same `Stats` for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stats_add(stats: *mut Stats, sample: i32) {
    // TODO: borrow the `Stats` mutably for this call, and record the sample.
    let _ = (stats, sample);
}

/// Returns the sum of the samples.
///
/// # Safety
///
/// `stats` must come from `stats_new` and must not have been freed. No other
/// call may modify the same `Stats` for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stats_sum(stats: *const Stats) -> i64 {
    // TODO: borrow the `Stats` for this call, and add up its samples.
    let _ = stats;
    0
}

/// Frees a `Stats`, including its samples. Passing null does nothing.
///
/// # Safety
///
/// `stats` must be null, or come from `stats_new` and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stats_free(stats: *mut Stats) {
    // TODO: take ownership back from C, and drop it.
    let _ = stats;
}

// Part two: a C type that Rust only ever sees as a pointer.
//
// `c_src/buffer.h` declares `typedef struct Buffer Buffer;` and the functions
// below. The struct itself is defined in `buffer.c`, so this crate has no way
// of knowing what is in it, and no business guessing.

use std::ptr::NonNull;

// TODO: replace this with a type that stands for "a C struct whose layout is
// none of my business": no public fields, and neither `Send` nor `Sync`.
pub struct Buffer;

unsafe extern "C" {
    fn buffer_new() -> *mut Buffer;
    fn buffer_push(buffer: *mut Buffer, byte: u8);
    fn buffer_len(buffer: *const Buffer) -> usize;
    fn buffer_free(buffer: *mut Buffer);
    /// How many buffers the C library currently has allocated.
    pub safe fn buffers_live() -> usize;
}

/// A buffer that frees itself.
pub struct OwnedBuffer(NonNull<Buffer>);

impl OwnedBuffer {
    /// Creates a buffer, or returns `None` if the C library couldn't.
    pub fn new() -> Option<Self> {
        // TODO: call `buffer_new`, and turn a null pointer into `None`.
        todo!()
    }

    /// Appends `byte`.
    pub fn push(&mut self, byte: u8) {
        // TODO: call `buffer_push`.
        todo!()
    }

    /// Returns how many bytes the buffer holds.
    pub fn len(&self) -> usize {
        // TODO: call `buffer_len`.
        todo!()
    }
}

// TODO: free the buffer when an `OwnedBuffer` goes out of scope.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_from_rust() {
        let stats = stats_new();
        assert!(!stats.is_null());

        unsafe {
            stats_add(stats, 1);
            stats_add(stats, 2);
            stats_add(stats, 6);
        }
        assert_eq!(unsafe { stats_sum(stats) }, 9);

        unsafe { stats_free(stats) };
        unsafe { stats_free(std::ptr::null_mut()) };
    }

    #[test]
    fn buffer_grows_and_is_freed() {
        let before = buffers_live();

        {
            let mut buffer = OwnedBuffer::new().expect("buffer_new returned null");
            assert_eq!(buffers_live(), before + 1);
            assert_eq!(buffer.len(), 0);

            for byte in b"opaque types" {
                buffer.push(*byte);
            }
            assert_eq!(buffer.len(), 12);
        }

        assert_eq!(buffers_live(), before);
    }
}
