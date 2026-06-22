#include "index.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bookmark.h"
#include "internal.h"
#include "storage.h"
#include "tag.h"
#include "util.h"

/*
 * In-memory bookmark index. URL -> Bookmark hash table, open addressing
 * with linear probing. Doubles when load > 70%.
 */

Bookmark *const BM_TOMBSTONE = (Bookmark *)(uintptr_t)1;

enum CursorKind { CUR_FIND, CUR_LIST_TAGS };

struct BmCursor {
  BmDb *db;
  enum CursorKind kind;

  /* CUR_FIND state. */
  Bookmark **slot;   /* points into db->entries */
  Bookmark *current; /* current match, borrowed */
  char *query;       /* owned */
  int started;

  /* CUR_LIST_TAGS state: snapshot taken at cursor creation. */
  char **list_tags;
  size_t *list_counts;
  size_t list_n;
  size_t list_i;
};

/* ------------------------------------------------------------------ */
/* Hashing + slot lookup.                                             */
/* ------------------------------------------------------------------ */

static uint64_t hash_str(const char *s) {
  /* FNV-1a 64-bit. */
  uint64_t h = 0xcbf29ce484222325ULL;
  while (*s) {
    h ^= (unsigned char)*s++;
    h *= 0x100000001b3ULL;
  }
  return h;
}

/* Find the slot holding url. Returns slot index, or -1 on failure. */
static int find_url_slot(const BmDb *db, const char *url) {
  if (db->cap == 0)
    return -1;
  size_t mask = db->cap - 1;
  size_t h = hash_str(url) & mask;
  for (size_t probes = 0; probes < db->cap; probes++) {
    Bookmark *e = db->entries[h];
    if (e == NULL)
      return -1; /* empty: not present */
    if (e != BM_TOMBSTONE && strcmp(e->url, url) == 0)
      return (int)h;
    h = (h + 1) & mask;
  }
  return -1;
}

/* Find the slot to insert url into: existing match, first tombstone, or
 * first empty. Caller must run grow() first to keep load below 100%. */
static int find_insert_slot(const BmDb *db, const char *url) {
  if (db->cap == 0)
    return -1;
  size_t mask = db->cap - 1;
  size_t h = hash_str(url) & mask;
  int first_tomb = -1;
  for (size_t probes = 0; probes < db->cap; probes++) {
    Bookmark *e = db->entries[h];
    if (e == NULL)
      return first_tomb >= 0 ? first_tomb : (int)h;
    if (e == BM_TOMBSTONE) {
      if (first_tomb < 0)
        first_tomb = (int)h;
    } else if (strcmp(e->url, url) == 0) {
      return (int)h;
    }
    h = (h + 1) & mask;
  }
  return first_tomb;
}

/* ------------------------------------------------------------------ */
/* Table growth.                                                      */
/* ------------------------------------------------------------------ */

static BmResult grow(BmDb *db) {
  size_t new_cap = (db->cap == 0) ? BM_INITIAL_CAP : db->cap * 2;
  Bookmark **new_entries = calloc(new_cap, sizeof(Bookmark *));
  if (!new_entries)
    return BM_ERR_IO;

  Bookmark **old_entries = db->entries;
  size_t old_cap = db->cap;

  db->entries = new_entries;
  db->cap = new_cap;
  db->used = 0;
  db->len = 0;

  for (size_t i = 0; i < old_cap; i++) {
    Bookmark *b = old_entries[i];
    if (!b || b == BM_TOMBSTONE)
      continue;
    int s = find_insert_slot(db, b->url);
    db->entries[s] = b;
    db->used++;
    db->len++;
  }
  free(old_entries);
  return BM_OK;
}

/* ------------------------------------------------------------------ */
/* Lifecycle.                                                         */
/* ------------------------------------------------------------------ */

BmResult bm_open(const char *path, BmDb **out) {
  if (!path || !out)
    return BM_ERR_IO;
  *out = NULL;

  BmDb *db = calloc(1, sizeof(*db));
  if (!db)
    return BM_ERR_IO;

  db->path = bm_strdup(path);
  if (!db->path)
    goto err_path;

  db->cap = BM_INITIAL_CAP;
  db->entries = calloc(db->cap, sizeof(Bookmark *));
  if (!db->entries)
    goto err_entries;

  BmResult r = storage_load(path, db);
  if (r != BM_OK)
    goto err_load;
  db->dirty = 0; /* successful load == clean state */

  *out = db;
  return BM_OK;

err_load:
  free(db->entries);
err_entries:
  free(db->path);
err_path:
  free(db);
  return BM_ERR_IO;
}

void bm_close(BmDb *db) {
  if (!db)
    return;

  if (db->path && db->dirty) {
    BmResult r = storage_save(db, db->path);
    if (r != BM_OK) {
      fprintf(stderr, "bm: warning: could not save %s: %s\n", db->path,
              bm_result_to_str(r));
    }
  }

  for (size_t i = 0; i < db->cap; i++) {
    Bookmark *b = db->entries[i];
    if (b && b != BM_TOMBSTONE)
      bookmark_free(b);
  }
  free(db->entries);
  db->entries = NULL; /* prevent double-free */
  free(db->path);
  free(db);
}

/* ------------------------------------------------------------------ */
/* Mutations.                                                         */
/* ------------------------------------------------------------------ */

BmResult bm_add(BmDb *db, const char *url, const char *const *tags,
                size_t n_tags) {
  if (!db || !url)
    return BM_ERR_INVALID_URL;

  char normalized[BM_MAX_URL_LEN];
  BmResult r = bm_normalize_url(url, normalized, sizeof normalized);
  if (r != BM_OK)
    return r;

  /* Grow before searching: keeps load < 70% and keeps find_insert_slot
   * from hitting the overloaded -1 in find_url_slot. */
  if ((db->used + 1) * 10 >= db->cap * 7) {
    r = grow(db);
    if (r != BM_OK)
      return r;
  }

  int slot = find_insert_slot(db, normalized);
  if (slot < 0)
    return BM_ERR_IO;
  Bookmark *existing = db->entries[slot];
  if (existing && existing != BM_TOMBSTONE)
    return BM_ERR_DUPLICATE;

  Bookmark *b = bookmark_new(normalized, tags, n_tags);
  if (!b)
    return BM_ERR_IO;

  if (existing == NULL)
    db->used++;
  db->entries[slot] = b;
  db->len++;

  db->next_id++;
  db->dirty = 1;

  return BM_OK;
}

BmResult bm_del(BmDb *db, const char *url) {
  if (!db || !url)
    return BM_ERR_INVALID_URL;

  char normalized[BM_MAX_URL_LEN];
  BmResult r = bm_normalize_url(url, normalized, sizeof normalized);
  if (r != BM_OK)
    return r;

  int slot = find_url_slot(db, normalized);
  if (slot < 0)
    return BM_ERR_NOT_FOUND;

  bookmark_free(db->entries[slot]);
  db->entries[slot] = BM_TOMBSTONE;
  db->len--;
  db->dirty = 1;
  return BM_OK;
}

BmResult bm_tag(BmDb *db, const char *url, const char *const *add, size_t n_add,
                const char *const *rm, size_t n_rm) {
  if (!db || !url)
    return BM_ERR_INVALID_URL;

  char normalized[BM_MAX_URL_LEN];
  BmResult r = bm_normalize_url(url, normalized, sizeof normalized);
  if (r != BM_OK)
    return r;

  int slot = find_url_slot(db, normalized);
  if (slot < 0)
    return BM_ERR_NOT_FOUND;

  Bookmark *b = db->entries[slot];

  r = tag_set_diff(&b->tags, &b->n_tags, rm, n_rm);
  if (r != BM_OK)
    return r;
  r = tag_set_union(&b->tags, &b->n_tags, add, n_add);
  if (r != BM_OK)
    return r;

  db->dirty = 1;
  return BM_OK;
}

/* ------------------------------------------------------------------ */
/* Cursors.                                                           */
/* ------------------------------------------------------------------ */

static int matches(const Bookmark *b, const char *query) {
  if (!*query)
    return 1;
  if (strstr(b->url, query))
    return 1;
  for (size_t i = 0; i < b->n_tags; i++) {
    if (strstr(b->tags[i], query))
      return 1;
  }
  return 0;
}

BmCursor *bm_find(BmDb *db, const char *query) {
  if (!db)
    return NULL;
  BmCursor *c = calloc(1, sizeof(*c));
  if (!c)
    return NULL;
  c->db = db;
  c->kind = CUR_FIND;
  c->query = bm_strdup(query ? query : "");
  if (!c->query) {
    free(c);
    return NULL;
  }
  c->slot = db->entries;
  return c;
}

BmCursor *bm_list_tags(BmDb *db) {
  if (!db)
    return NULL;
  BmCursor *c = calloc(1, sizeof(*c));
  if (!c)
    return NULL;
  c->db = db;
  c->kind = CUR_LIST_TAGS;

  /* Snapshot every (tag, count) pair. O(N * unique_tags) but the database
   * is small. */
  for (size_t i = 0; i < db->cap; i++) {
    Bookmark *b = db->entries[i];
    if (!b || b == BM_TOMBSTONE)
      continue;
    for (size_t j = 0; j < b->n_tags; j++) {
      const char *tag = b->tags[j];
      size_t k;
      for (k = 0; k < c->list_n; k++) {
        if (strcmp(c->list_tags[k], tag) == 0)
          break;
      }
      if (k < c->list_n) {
        c->list_counts[k]++;
        continue;
      }
      char **nt = realloc(c->list_tags, (c->list_n + 1) * sizeof(char *));
      size_t *nc = realloc(c->list_counts, (c->list_n + 1) * sizeof(size_t));
      if (!nt || !nc) {
        free(nt ? nt : c->list_tags);
        free(nc ? nc : c->list_counts);
        c->list_tags = NULL;
        c->list_counts = NULL;
        bm_cursor_close(c);
        return NULL;
      }
      c->list_tags = nt;
      c->list_counts = nc;
      c->list_tags[c->list_n] = bm_strdup(tag);
      if (!c->list_tags[c->list_n]) {
        bm_cursor_close(c);
        return NULL;
      }
      c->list_counts[c->list_n] = 1;
      c->list_n++;
    }
  }
  return c;
}

bool bm_cursor_next(BmCursor *c) {
  if (!c)
    return false;

  if (c->kind == CUR_FIND) {
    Bookmark **end = c->db->entries + c->db->cap;
    if (!c->started)
      c->started = 1;
    else
      c->slot++;

    for (; c->slot < end; c->slot++) {
      Bookmark *b = *c->slot;
      if (!b || b == BM_TOMBSTONE)
        continue;
      if (matches(b, c->query)) {
        c->current = b;
        return true;
      }
    }
    c->current = NULL;
    return false;
  }

  /* CUR_LIST_TAGS */
  if (!c->started) {
    c->started = 1;
    c->list_i = 0;
  } else {
    c->list_i++;
  }
  return c->list_i < c->list_n;
}

const char *bm_cursor_url(BmCursor *c) {
  if (!c)
    return NULL;
  if (c->kind == CUR_FIND) {
    return c->current ? c->current->url : NULL;
  }
  if (c->list_i >= c->list_n)
    return NULL;
  return c->list_tags[c->list_i];
}

size_t bm_cursor_tags(BmCursor *c, const char ***out_tags) {
  if (!c || c->kind != CUR_FIND || !c->current) {
    if (out_tags)
      *out_tags = NULL;
    return 0;
  }
  if (out_tags)
    *out_tags = (const char **)c->current->tags;
  return c->current->n_tags;
}

size_t bm_cursor_count(BmCursor *c) {
  if (!c || c->kind != CUR_LIST_TAGS)
    return 0;
  if (c->list_i >= c->list_n)
    return 0;
  return c->list_counts[c->list_i];
}

void bm_cursor_close(BmCursor *c) {
  if (!c)
    return;
  free(c->query);
  for (size_t i = 0; i < c->list_n; i++)
    free(c->list_tags[i]);
  free(c->list_tags);
  free(c->list_counts);
  free(c);
}
