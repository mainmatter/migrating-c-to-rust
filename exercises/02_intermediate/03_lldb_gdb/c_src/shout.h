#ifndef BM_SHOUT_H
#define BM_SHOUT_H

#include <stddef.h>

// Writes an uppercase `label!` into `out` and returns 0 on success.
// Returns -1 when `out` is too small.
int bm_shout(char *out, size_t out_len, const char *label);

#endif
