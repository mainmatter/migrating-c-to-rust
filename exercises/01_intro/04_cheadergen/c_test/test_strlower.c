/*
 * Validation harness for the `bm_strlower` exercise.
 *
 * Build & run by hand (from this crate's directory):
 *
 *   cargo build
 *   cheadergen generate --lang c --output-dir c_test -p bm_cheadergen
 *   cc -Wall -Wextra -std=c11 -I. \
 *      c_test/test_strlower.c \
 *      ../../../target/debug/libbm_cheadergen.a \
 *      -o ../../../target/debug/test_strlower
 *   ../../../target/debug/test_strlower
 *
 * The `-I.` flag picks up the header that `cheadergen` writes into `c_test/`.
 */

#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "bm_cheadergen.h"

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

  /* Bytes that are not valid UTF-8 must survive: the C original lowercases
   * bytes, not characters, so a port that validates UTF-8 first is wrong.
   * The string is split so the hex escapes don't swallow the 'A'. */
  char not_utf8[] = "\xff\xfe"
                    "AB";
  bm_strlower(not_utf8);
  assert(memcmp(not_utf8,
                "\xff\xfe"
                "ab",
                5) == 0);

  /* Only the first NUL-terminated run is touched; bytes past it survive. */
  char embedded_nul[] = "AB\0CD";
  bm_strlower(embedded_nul);
  assert(memcmp(embedded_nul, "ab\0CD", 6) == 0);

  /* A port that lowercases via Unicode instead of bytes can produce more bytes
   * than it was given: U+0130 is two bytes, its lowercase form is three. `buf`
   * is sized to exactly the input plus its NUL, so any extra byte lands on the
   * canary. */
  struct {
    char buf[3];
    char canary;
  } guarded;
  memcpy(guarded.buf, "\xc4\xb0", 3);
  guarded.canary = '#';
  bm_strlower(guarded.buf);
  assert(guarded.canary == '#');
  assert(memcmp(guarded.buf, "\xc4\xb0", 3) == 0);

  printf("ok\n");
  return 0;
}
