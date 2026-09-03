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

/// Unlike the chapter's counter, `Stats` owns heap memory of its own in the form
/// of a `Vec`.
pub struct Stats {
    samples: Vec<i32>,
}

/// Creates an empty `Stats`. The caller owns it, and has to pass it to
/// `stats_free`.
#[unsafe(no_mangle)]
pub extern "C" fn stats_new() -> *mut Stats {
    Box::into_raw(Box::new(Stats {
        samples: Vec::new(),
    }))

}

/// Records one sample.
///
/// # Safety
///
/// `stats` must come from `stats_new` and must not have been freed. No other
/// call may use the same `Stats` for the duration of this one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stats_add(stats: *mut Stats, sample: i32) {
    // SAFETY: the caller guarantees a live `Stats` that nothing else uses.
    let stats = unsafe { &mut *stats };
    stats.samples.push(sample);

}

/// Returns the sum of the samples.
///
/// # Safety
///
/// `stats` must come from `stats_new` and must not have been freed. No other
/// call may modify the same `Stats` for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stats_sum(stats: *const Stats) -> i64 {
    // SAFETY: the caller guarantees a live `Stats` that nothing modifies.
    let stats = unsafe { &*stats };
    stats.samples.iter().map(|&sample| i64::from(sample)).sum()

}

/// Frees a `Stats`, including its samples. Passing null does nothing.
///
/// # Safety
///
/// `stats` must be null, or come from `stats_new` and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stats_free(stats: *mut Stats) {
    if !stats.is_null() {
        // SAFETY: the pointer came from `Box::into_raw` in `stats_new`, and the
        // caller won't use it again.
        drop(unsafe { Box::from_raw(stats) });
    }

}

// Part two: a C type that Rust only ever sees as a pointer.
//
// `c_src/buffer.h` declares `typedef struct Buffer Buffer;` and the functions
// below. The struct itself is defined in `buffer.c`, so this crate has no way
// of knowing what is in it (and no business guessing).

use std::ptr::NonNull;

#[repr(C)]
pub struct Buffer {
    _data: (),
    _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}


unsafe extern "C" {
    fn buffer_new() -> *mut Buffer;
    fn buffer_push(buffer: *mut Buffer, byte: u8);
    fn buffer_len(buffer: *const Buffer) -> usize;
    fn buffer_free(buffer: *mut Buffer);
    /// How many buffers the C library currently has allocated.
    pub safe fn num_buffers_live() -> usize;
}

/// A buffer that frees itself.
pub struct OwnedBuffer(NonNull<Buffer>);

impl OwnedBuffer {
    /// Creates a buffer, or returns `None` if the C library couldn't.
    pub fn new() -> Option<Self> {
        // SAFETY: `buffer_new` returns either a valid buffer or null.
        let buffer = unsafe { buffer_new() };
        NonNull::new(buffer).map(OwnedBuffer)

    }

    /// Appends `byte`.
    pub fn push(&mut self, byte: u8) {
        // SAFETY: the buffer is live until `drop`, and `&mut self` means no
        // other call uses it.
        unsafe { buffer_push(self.0.as_ptr(), byte) }

    }

    /// Returns how many bytes the buffer holds.
    pub fn len(&self) -> usize {
        // SAFETY: the buffer is live until `drop`.
        unsafe { buffer_len(self.0.as_ptr()) }

    }
}

impl Drop for OwnedBuffer {
    fn drop(&mut self) {
        // SAFETY: the buffer came from `buffer_new`, and `OwnedBuffer` is its
        // only owner, so it is freed exactly once.
        unsafe { buffer_free(self.0.as_ptr()) }
    }
}


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
        let before = num_buffers_live();

        {
            let mut buffer = OwnedBuffer::new().expect("buffer_new returned null");
            assert_eq!(num_buffers_live(), before + 1);
            assert_eq!(buffer.len(), 0);

            for byte in b"opaque types" {
                buffer.push(*byte);
            }
            assert_eq!(buffer.len(), 12);
        }

        assert_eq!(num_buffers_live(), before);
    }
}
