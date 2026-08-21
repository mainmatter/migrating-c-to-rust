#pragma once

#include <stddef.h>

typedef struct RustBookmark RustBookmark;

RustBookmark *rust_bookmark_new(const char *url, const char *const *tags,
                                size_t n_tags);
void rust_bookmark_free(RustBookmark *bookmark);

const char *rust_bookmark_url(const RustBookmark *bookmark);
size_t rust_bookmark_tag_count(const RustBookmark *bookmark);
const char *rust_bookmark_tag(const RustBookmark *bookmark, size_t index);

size_t rust_bookmark_live_count(void);
