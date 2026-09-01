/*
 * URL and tag normalization. Both functions write into a buffer supplied by
 * the caller, so nothing here allocates.
 */

#include "normalize.h"

#include <ctype.h>
#include <string.h>

BmResult bm_normalize_url(const char *raw, char *out, size_t out_len) {
  (void)out_len;

  /* Lowercase the URL into `out`. */
  size_t i = 0;
  while (raw[i] != '\0') {
    out[i] = (char)tolower((unsigned char)raw[i]);
    i++;
  }
  out[i] = '\0';

  /* Reject embedded '\t' — it's our on-disk separator. */
  if (strchr(out, '\t'))
    return BM_ERR_INVALID_URL;

  return BM_OK;
}

BmResult tag_normalize(const char *raw, char *out, size_t out_len) {
  if (!raw || !out || out_len == 0)
    return BM_ERR_INVALID_URL;

  /* Skip leading whitespace. */
  while (*raw && isspace((unsigned char)*raw))
    raw++;

  size_t n = 0;
  while (*raw && !isspace((unsigned char)*raw) && *raw != ',' && *raw != '\t') {
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
