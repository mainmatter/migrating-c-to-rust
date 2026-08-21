#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "result.h"

typedef struct BmDb BmDb;
typedef struct BmCursor BmCursor;

/* Open or create a database backed by a text file at `path`. */
BmResult bm_open(const char *path, BmDb **out);

/* Save and close. */
void bm_close(BmDb *db);

BmResult bm_add(BmDb *db, const char *url, const char *const *tags,
                size_t n_tags);
BmResult bm_del(BmDb *db, const char *url);
BmResult bm_tag(BmDb *db, const char *url, const char *const *add, size_t n_add,
                const char *const *rm, size_t n_rm);

/*
 * Iterate over bookmarks whose URL or any tag contains `query` as a
 * substring. Empty query matches all. Caller must close the cursor.
 */
BmCursor *bm_find(BmDb *db, const char *query);

/* Iterate over (tag, count) for every distinct tag. */
BmCursor *bm_list_tags(BmDb *db);

const char *bm_cursor_url(BmCursor *c);
size_t bm_cursor_tags(BmCursor *c, const char ***out_tags);
size_t bm_cursor_count(BmCursor *c);
bool bm_cursor_next(BmCursor *c);
void bm_cursor_close(BmCursor *c);
