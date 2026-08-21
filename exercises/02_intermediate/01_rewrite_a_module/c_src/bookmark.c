#include "bookmark.h"

#include <stdlib.h>

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
