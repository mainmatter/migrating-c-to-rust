/*
 * Validation harness for part one of the opaque types exercise: the `Stats`
 * handle, used the way C would really use it.
 *
 * Build & run by hand (from this crate's directory):
 *
 *   cargo build
 *   cc -Wall -Wextra -std=c11 -I. \
 *      c_test/test_stats.c \
 *      ../../../target/debug/libffi_opaque_types.a \
 *      -o ../../../target/debug/test_stats
 *   ../../../target/debug/test_stats
 */

#include <assert.h>
#include <stdio.h>

#include "stats.h"

int main(void) {
  Stats *stats = stats_new();
  /* A handle must be a real address: C has no other way to reach the value. */
  assert(stats != NULL);

  stats_add(stats, 1);
  stats_add(stats, 2);
  stats_add(stats, 6);
  assert(stats_sum(stats) == 9);

  stats_free(stats);

  /* NULL must be a no-op, the way `free(NULL)` is. */
  stats_free(NULL);

  /* Every handle owns its own samples. */
  Stats *first = stats_new();
  Stats *second = stats_new();
  assert(first != NULL && second != NULL && first != second);

  stats_add(first, -4);
  stats_add(second, 10);
  assert(stats_sum(first) == -4);
  assert(stats_sum(second) == 10);

  stats_free(first);
  stats_free(second);

  printf("ok\n");
  return 0;
}
