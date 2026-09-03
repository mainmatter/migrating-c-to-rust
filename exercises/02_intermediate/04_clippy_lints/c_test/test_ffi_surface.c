/*
 * Checks that this crate still exports the C API the legacy callers expect.
 *
 * Build & run by hand (from this crate's directory):
 *
 *   cargo build -p bm_clippy_lints
 *   cc -Wall -Wextra -std=c11 -I. \
 *      c_test/test_ffi_surface.c \
 *      ../../../target/debug/libbm_clippy_lints.a \
 *      -o ../../../target/debug/test_ffi_surface
 *   ../../../target/debug/test_ffi_surface
 */

#include <stdio.h>

#include "ffi_surface.h"

int main(void) {
  /* Calling it is the whole point: the symbol has to exist under the name the
   * C API promises, whatever the Rust function ends up being called. */
  BMNormalizeURL();

  printf("ok\n");
  return 0;
}
