#include "c_test/bm_allocators.h"

#include <assert.h>
#include <stddef.h>
#include <string.h>

int main(void) {
  const char *tags[] = {"rust", "ffi"};
  size_t before = rust_bookmark_live_count();

  RustBookmark *bookmark = rust_bookmark_new("https://rust-lang.org", tags, 2);
  assert(bookmark != NULL);
  assert(rust_bookmark_live_count() == before + 1);
  assert(strcmp(rust_bookmark_url(bookmark), "https://rust-lang.org") == 0);
  assert(rust_bookmark_tag_count(bookmark) == 2);
  assert(strcmp(rust_bookmark_tag(bookmark, 0), "rust") == 0);
  assert(strcmp(rust_bookmark_tag(bookmark, 1), "ffi") == 0);
  assert(rust_bookmark_tag(bookmark, 2) == NULL);

  rust_bookmark_free(bookmark);
  assert(rust_bookmark_live_count() == before);

  /* Match C allocation APIs: invalid input fails, and free(NULL) is a no-op. */
  assert(rust_bookmark_new(NULL, NULL, 0) == NULL);
  assert(rust_bookmark_new("https://example.com", NULL, 1) == NULL);
  rust_bookmark_free(NULL);

  return 0;
}
