#include "bookmark.h"

#include <ctype.h>
#include <stdlib.h>
#include <string.h>

#include "util.h"

Bookmark *bookmark_new(const char *url, const char *const *tags,
                       size_t n_tags) {
  if (!url)
    return NULL;
  Bookmark *b = calloc(1, sizeof(*b));
  if (!b)
    return NULL;

  b->url = bm_strdup(url);
  if (!b->url)
    goto fail;

  if (n_tags > 0) {
    b->tags = calloc(n_tags, sizeof(char *));
    if (!b->tags)
      goto fail;
    for (size_t i = 0; i < n_tags; i++) {
      b->tags[i] = bm_strdup(tags[i]);
      if (!b->tags[i])
        goto fail;
      b->n_tags = i + 1;
    }
  }
  return b;

fail:
  bookmark_free(b);
  return NULL;
}

void bookmark_free(Bookmark *b) {
  if (!b)
    return;
  free(b->url);
  b->url = NULL; /* prevent double-free */
  for (size_t i = 0; i < b->n_tags; i++) {
    free(b->tags[i]);
  }
  free(b->tags);
  b->tags = NULL; /* prevent double-free */
  free(b);
}

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
