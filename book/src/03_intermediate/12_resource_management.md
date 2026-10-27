# Resource management

Many times, in order to construct an object, we must first acquire several
resources, one by one, and each might depend on the previous ones. At each step,
an error can occur. If we fail halfway, we must clean up what was already
acquired.

In C, this must be carefully done by the programmer. When porting this logic to
Rust, there is a bit of a paradigm shift to keep in mind. The compiler handles
the cleanup, so the code becomes easier to maintain and reason about.

## What needs cleaning up in C?

Let's imagine we want to port the constructor for a `Store`, which is the handle
used to work with a bookmark database on disk. `store_open` needs to open a
file, allocate memory for the `Store` out parameter and load an index.

```c
struct Store {
    FILE *file;
    struct Index *index;
};

BmStatus store_open(const char *path, struct Store **out) {
    BmStatus status;

    FILE *file = fopen(path, "r");
    if (file == NULL) {
        return BM_ERR_IO;
    }

    struct Store *store = calloc(1, sizeof(*store));
    if (store == NULL) {
        status = BM_ERR_IO;
        goto close_file;
    }
    store->file = file;

    store->index = index_load(file);
    if (store->index == NULL) {
        status = BM_ERR_IO;
        goto free_store;
    }

    *out = store;
    return BM_OK;

free_store:
    free(store);
close_file:
    fclose(file);
    return status;
}
```

Any of these intermediate steps might fail, and in this case, we need to clean
up any acquired resources before returning an error. The typical pattern for
achieving this involves using `goto` labels.

As you can see in the example, the `goto`s are in reverse order of acquisition
and fall through into each other, so a failure runs its own cleanup and
everything below it.

It works, but it depends on three rules the programmer must follow diligently:

- **Each failure jumps to the label of the last resource it acquired.** Jumping
  to the wrong label either leaks a resource or releases something that was
  never acquired.
- **The success path must return before the labels.** A missing `return BM_OK;`
  frees what the caller is about to use.
- **A new resource needs a new label in the right position.** Inserting a step
  in the middle of the function changes which label every later failure must
  jump to.

## How does Rust handle the cleanup?

We see that in Rust this same function looks much simpler:

```rust
# use std::fs::File;
# use std::io;
# use std::path::Path;
# struct Index;
# impl Index { fn load(_: &File) -> io::Result<Index> { Ok(Index) } }
pub struct Store {
    index: Index,
    file: File,
}

pub fn open_store(path: &Path) -> io::Result<Store> {
    let file = File::open(path)?;
    let index = Index::load(&file)?;
    Ok(Store { index, file })
}
```

At each step, we attempt to acquire a resource. If we fail, we return early with
the help of `?`, which propagates the error. We can see right away that there
appears to be no cleanup code anywhere. So, what happens with the `file` that
was already opened when `Index::load` fails and `?` returns early?

### Ownership and the `Drop` trait

The answer is ownership, and it's the same idea we saw in
[Chapter 2.6](../02_basics/06_opaque_types.md#a-c-type-behind-a-rust-handle).
There we wrapped a C pointer in a struct and wrote a custom destructor to
release the pointer.

When a struct owns a raw resource, it becomes responsible for deallocating it.
The way it achieves this is by implementing the `Drop` trait and defining a custom
`drop` function. Inside the `drop` function we write the destruction logic,
similar to what we do in C under each `goto` label. This `drop` function gets
called automatically by the compiler when the variable goes out of scope,
replacing the need to write it out by hand.

In our case, we didn't have to write a custom `drop` for `File` because the
standard library already has it. A `File` wraps an owned file descriptor, and
that type's implementation calls `close`. When `Index::load` fails, `file` goes
out of scope, Rust drops its fields, and the descriptor is closed.

Local variables are dropped in reverse order of declaration, so the cleanup
happens automatically, in the same order we'd write the `goto` labels in C.

This paradigm, called *Resource Acquisition Is Initialization* (RAII), ties
resource management to an object's lifetime: allocate on initialization,
deallocate on destruction. We write cleanup code only once, which means less
room for mistakes.

## Head to the exercise
