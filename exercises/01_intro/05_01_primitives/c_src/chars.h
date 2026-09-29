#pragma once

#include <stddef.h>

/* Lowercases an ASCII letter, and returns any other byte unchanged. */
char to_lower(char byte);

/* Returns 1 when `byte` is an ASCII letter, and 0 otherwise. */
int is_letter(char byte);

/* Lowercases the ASCII letters in the `len` bytes at `bytes`, in place. */
void lower_bytes(char *bytes, size_t len);
