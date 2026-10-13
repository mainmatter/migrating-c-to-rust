#pragma once

/* Same legacy slice as in 01_what_is_ffi. This time we won't transcribe it
 * into Rust by hand — bindgen will do it for us. */

int bm_add(int a, int b);
