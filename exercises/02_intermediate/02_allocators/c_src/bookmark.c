#include "bookmark.h"

#include <stdlib.h>
#include <string.h>

static size_t live_count = 0;

static char *duplicate_string(const char *source) {
  if (!source)
    return NULL;

  size_t len = strlen(source);
  char *copy = malloc(len + 1);
  if (!copy)
    return NULL;

  memcpy(copy, source, len + 1);
  return copy;
}

Bookmark *bookmark_new(const char *url, const char *const *tags,
                       size_t n_tags) {
  if (!url || (n_tags > 0 && !tags))
    return NULL;

  Bookmark *bookmark = calloc(1, sizeof(*bookmark));
  if (!bookmark)
    return NULL;

  bookmark->url = duplicate_string(url);
  if (!bookmark->url)
    goto fail;

  if (n_tags > 0) {
    bookmark->tags = calloc(n_tags, sizeof(*bookmark->tags));
    if (!bookmark->tags)
      goto fail;

    for (size_t i = 0; i < n_tags; i++) {
      bookmark->tags[i] = duplicate_string(tags[i]);
      if (!bookmark->tags[i])
        goto fail;
      bookmark->n_tags = i + 1;
    }
  }

  live_count++;
  return bookmark;

fail:
  free(bookmark->url);
  for (size_t i = 0; i < bookmark->n_tags; i++)
    free(bookmark->tags[i]);
  free(bookmark->tags);
  free(bookmark);
  return NULL;
}

void bookmark_free(Bookmark *bookmark) {
  if (!bookmark)
    return;

  free(bookmark->url);
  for (size_t i = 0; i < bookmark->n_tags; i++)
    free(bookmark->tags[i]);
  free(bookmark->tags);
  free(bookmark);

  live_count--;
}

size_t bookmark_live_count(void) { return live_count; }
