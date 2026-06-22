/*
 * Validation harness for the `bm_tag_describe` exercise.
 *
 * Build & run by hand (from this crate's directory):
 *
 *   cargo build
 *   cc -Wall -Wextra -std=c11 -I. \
 *      c_test/test_tag_describe.c \
 *      ../../../target/debug/libffi_safe_types.a \
 *      -o ../../../target/debug/test_tag_describe
 *   ./target/debug/test_tag_describe
 *
 * The `-I.` flag picks up the header that `cheadergen` writes into the crate
 * root via `build.rs`.
 */

#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "ffi_safe_types.h"

int main(void) {
  /* A plain ASCII tag round-trips through `bm_tag_describe`. */
  Tag rust = {.name = "rust"};
  const char *desc = bm_tag_describe(rust);
  assert(strcmp(desc, "rust") == 0);

  /* An empty name still yields a valid, empty C string (not NULL). */
  Tag empty = {.name = ""};
  const char *empty_desc = bm_tag_describe(empty);
  assert(empty_desc != NULL);
  assert(strcmp(empty_desc, "") == 0);

  printf("ok\n");
  return 0;
}
