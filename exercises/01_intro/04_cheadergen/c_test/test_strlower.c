/*
 * Validation harness for the `bm_strlower` exercise.
 *
 * Build & run by hand (from this crate's directory):
 *
 *   cargo build
 *   cc -Wall -Wextra -std=c11 -I. \
 *      c_test/test_strlower.c \
 *      target/debug/libex_01_cheadergen.a \
 *      -o target/debug/test_strlower
 *   ./target/debug/test_strlower
 *
 * The `-I.` flag picks up the header that `cheadergen` writes into the crate
 * root via `build.rs`.
 */

#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "cheadergen.h"

int main(void) {
  char mixed[] = "HELLO World 123";
  bm_strlower(mixed);
  assert(strcmp(mixed, "hello world 123") == 0);

  char already_lower[] = "abc";
  bm_strlower(already_lower);
  assert(strcmp(already_lower, "abc") == 0);

  char empty[] = "";
  bm_strlower(empty);
  assert(strcmp(empty, "") == 0);

  /* NULL must be a no-op (no crash). */
  bm_strlower(NULL);

  printf("ok\n");
  return 0;
}
