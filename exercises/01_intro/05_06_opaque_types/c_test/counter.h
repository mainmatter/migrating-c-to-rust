#pragma once

#include <stdint.h>

/*
 * A counter. The type is defined on the Rust side, so C only ever holds a
 * pointer to one: it cannot allocate one, copy one, or read a field.
 */
typedef struct Counter Counter;

/* Creates a counter. The caller has to pass it to `counter_free`. */
Counter *counter_new(uint32_t start);

/* Adds `amount` to the counter. */
void counter_add(Counter *counter, uint32_t amount);

/* Reads the counter's value. */
uint32_t counter_value(const Counter *counter);

/* Frees a counter. Passing NULL does nothing. */
void counter_free(Counter *counter);
