/*
 * Validation harness for part one of the opaque types exercise: the `Counter`
 * handle, used the way C would really use it.
 *
 * Build & run by hand (from this crate's directory):
 *
 *   cargo build
 *   cc -Wall -Wextra -std=c11 -I. \
 *      c_test/test_counter.c \
 *      ../../../target/debug/libffi_opaque_types.a \
 *      -o ../../../target/debug/test_counter
 *   ../../../target/debug/test_counter
 */

#include <assert.h>
#include <stdio.h>

#include "counter.h"

int main(void) {
  Counter *counter = counter_new(1);
  /* A handle must be a real address: C has no other way to reach the value. */
  assert(counter != NULL);

  counter_add(counter, 41);
  assert(counter_value(counter) == 42);

  counter_add(counter, 0);
  assert(counter_value(counter) == 42);

  counter_free(counter);

  /* NULL must be a no-op, the way `free(NULL)` is. */
  counter_free(NULL);

  /* Every handle owns its own state: two counters must not share a value. */
  Counter *first = counter_new(0);
  Counter *second = counter_new(100);
  assert(first != NULL && second != NULL);
  assert(first != second);

  counter_add(first, 1);
  assert(counter_value(first) == 1);
  assert(counter_value(second) == 100);

  counter_free(first);
  counter_free(second);

  printf("ok\n");
  return 0;
}
