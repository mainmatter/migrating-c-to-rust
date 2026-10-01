#pragma once

#include <stddef.h>
#include <stdint.h>

/*
 * A growable byte buffer. The struct is defined in buffer.c and nowhere else,
 * so callers only ever hold a pointer to one.
 */
typedef struct Buffer Buffer;

/* Creates an empty buffer, or returns NULL. */
Buffer *buffer_new(void);

/* Appends `byte`. */
void buffer_push(Buffer *buffer, uint8_t byte);

/* Returns how many bytes the buffer holds. */
size_t buffer_len(const Buffer *buffer);

/* Frees a buffer. Passing NULL does nothing. */
void buffer_free(Buffer *buffer);

/* Returns how many buffers are currently allocated. */
size_t buffers_live(void);
