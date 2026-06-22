# Standalone build of the all-C `bm` binary.

CC      := cc
CFLAGS  := -Wall -Wextra -Werror -O0 -g -std=c11 -Isrc
SRCS    := $(wildcard src/*.c)
OBJS    := $(patsubst src/%.c,build/%.o,$(SRCS))

bm: $(OBJS)
	$(CC) -o $@ $^

build/%.o: src/%.c | build
	$(CC) $(CFLAGS) -c -o $@ $<

build:
	mkdir -p build

clean:
	rm -rf build bm bookmarks.bm bookmarks.bm.tmp

.PHONY: clean
