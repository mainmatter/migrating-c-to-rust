#include "tag.h"

#include <ctype.h>

BmResult tag_normalize(const char *raw, char *out, size_t out_len) {
  if (!raw || !out || out_len == 0)
    return BM_ERR_INVALID_URL;

  /* Skip leading whitespace. */
  while (*raw && isspace((unsigned char)*raw))
    raw++;

  size_t n = 0;
  while (*raw && !isspace((unsigned char)*raw) && *raw != ',') {
    if (n + 1 >= out_len)
      return BM_ERR_INVALID_URL; /* too long */
    out[n++] = (char)tolower((unsigned char)*raw);
    raw++;
  }
  out[n] = '\0';

  if (n == 0)
    return BM_ERR_INVALID_URL;
  return BM_OK;
}
