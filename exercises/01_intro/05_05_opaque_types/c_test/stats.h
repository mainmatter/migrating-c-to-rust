#pragma once

#include <stdint.h>

/*
 * Running statistics over `int32_t` samples. The type is defined on the Rust
 * side, so C only ever holds a pointer to one: it cannot allocate one, copy
 * one, or read a field.
 */
typedef struct Stats Stats;

/* Creates an empty `Stats`. The caller has to pass it to `stats_free`. */
Stats *stats_new(void);

/* Records one sample. */
void stats_add(Stats *stats, int32_t sample);

/* Returns the sum of the samples. */
int64_t stats_sum(const Stats *stats);

/* Frees a `Stats`. Passing NULL does nothing. */
void stats_free(Stats *stats);
