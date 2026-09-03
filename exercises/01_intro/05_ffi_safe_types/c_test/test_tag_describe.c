/*
 * Validation harness for the `bm_tag_describe` exercise.
 *
 * Build & run by hand (from this crate's directory):
 *
 *   cargo build
 *   cc -Wall -Wextra -std=c11 -I. \
 *      c_test/test_tag_describe.c \
 *      ../../../target/debug/libbm_ffi_safe_types.a \
 *      -o ../../../target/debug/test_tag_describe
 *   ../../../target/debug/test_tag_describe
 *
 * The `-I.` flag picks up the header that `cheadergen` writes into `c_test/`.
 */

#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "bm_ffi_safe_types.h"

int main(void) {
  /* A plain ASCII tag round-trips through `bm_tag_describe`. */
  Tag rust = {.name = "rust"};
  const char *desc = bm_tag_describe(rust);
  assert(strcmp(desc, "rust") == 0);
  /* The description must borrow the caller's buffer rather than allocate a new
   * one: anything that hands back its own allocation either leaks it (nothing
   * here can free it) or dangles once the call returns. */
  assert(desc == rust.name);

  /* An empty name still yields a valid, empty C string (not NULL). */
  Tag empty = {.name = ""};
  const char *empty_desc = bm_tag_describe(empty);
  assert(empty_desc != NULL);
  assert(strcmp(empty_desc, "") == 0);

  /* The name is a byte string, not text. A port that validates UTF-8 first is
   * wrong: the C original never did. Split so the hex escapes stop early. */
  char raw[] = "\xff\xfe"
               "tag";
  Tag not_utf8 = {.name = raw};
  const char *not_utf8_desc = bm_tag_describe(not_utf8);
  assert(not_utf8_desc == raw);
  assert(memcmp(not_utf8_desc,
                "\xff\xfe"
                "tag",
                6) == 0);

  /* A NULL name must not be dereferenced. The return value is deliberately not
   * asserted -- narrowing null-ness is the next exercise's lesson, this one
   * only requires that we don't read through it. */
  Tag missing = {.name = NULL};
  (void)bm_tag_describe(missing);

  printf("ok\n");
  return 0;
}
