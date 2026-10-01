#include "buffer.h"

#include <stdlib.h>

struct Buffer {
  uint8_t *bytes;
  size_t len;
  size_t capacity;
};

static size_t live_buffers = 0;

Buffer *buffer_new(void) {
  Buffer *buffer = calloc(1, sizeof *buffer);
  if (!buffer) {
    return NULL;
  }
  live_buffers++;
  return buffer;
}

void buffer_push(Buffer *buffer, uint8_t byte) {
  if (buffer->len == buffer->capacity) {
    size_t capacity = buffer->capacity ? buffer->capacity * 2 : 8;
    uint8_t *bytes = realloc(buffer->bytes, capacity);
    if (!bytes) {
      abort();
    }
    buffer->bytes = bytes;
    buffer->capacity = capacity;
  }
  buffer->bytes[buffer->len++] = byte;
}

size_t buffer_len(const Buffer *buffer) { return buffer->len; }

void buffer_free(Buffer *buffer) {
  if (!buffer) {
    return;
  }
  free(buffer->bytes);
  free(buffer);
  live_buffers--;
}

size_t buffers_live(void) { return live_buffers; }
