# Resource management

Many times, in order to construct an object, we must first acquire several
resources, one by one, and each might depend on the previous ones. At each step,
an error can occur. If we fail halfway, we must clean up what was already
acquired.

In C, this is entirely up to us, and one wrong step can mean a leak (or worse).
When porting this logic to Rust, there is a bit of a paradigm shift initially,
but once you try it, you'll see why it's worth it. The compiler deals with much
of the boilerplate for you, so you won't have to worry about getting the steps
right every time.

## What needs cleaning up in C?

Let's imagine we want to port the constructor for a `Store`, which is the handle
used to work with a bookmark database on disk. `store_open` needs to open a
file, allocate memory for the `Store` out parameter and then load an index.

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

Any of these intermediate steps might fail, and we're left in an awkward,
halfway-done state. In this case, we want to return an error to let the caller
know the initialization wasn't successful, but we first need to clean up any
acquired resources. The typical pattern for structuring this cleanly without a
bunch of `if` statements is one you're likely familiar with: using `goto` labels.

We can picture the labels as a waterfall: each `goto` leads to the label of the
most recently acquired resource. Notice in the example how a failure in
`index_load` jumps to `free_store` because allocating memory for the `store` is
the most recent acquisition. The `store` gets freed, and then we cascade down
through the cleanup code for closing the `file`. A failure earlier enters lower:
if `calloc` fails, we skip `free_store` and jump directly to `close_file`.

It works, but it depends on the programmer being diligent. There are three rules
to keep in mind:

- **Each failure jumps to the label of the last resource it acquired.** Jump to
  the wrong one and you either leak a resource or free something that was never
  acquired.
- **The success path must return before the labels.** Forget `return BM_OK;` and
  we happily fall into the cleanup code, freeing what the caller is about to
  use.
- **A new resource needs a new label in the right position.** Inserting a step
  in the middle is tricky, since it changes where every later failure must jump
  to.

## How does Rust handle the cleanup?

Now, take a moment to appreciate how it translates to Rust, keeping in mind that
it achieves the same result:

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

At each step, we attempt to acquire a resource. If we fail, instead of jumping
somewhere, we return early with the help of `?`, which propagates the error. We
can see right away that there appears to be no cleanup code anywhere.
Suspicious, right?

That is the whole point, and it is where the magic happens. In the case that
`Index::load` fails, we can rest assured that the `file` that was already opened
gets properly closed as `?` returns early. But how and where does this happen?

### Ownership and the `Drop` trait

The answer lies in the concept of ownership, and it's the same idea we saw in
[Chapter 2.6](../02_basics/06_opaque_types.md#a-c-type-behind-a-rust-handle).
There, we wrapped a C pointer in a struct and wrote a custom destructor to
release the pointer.

The idea is that we delegate the responsibility of deallocating a resource to
the object that "owns" it. This way, we avoid working directly with raw
resources. The way to achieve this is by implementing the `Drop` trait for our
owner struct and defining a custom `drop` function. Inside it, we write the
destruction logic, which is exactly what we'd write under each `goto` label. The
awesome thing is that this `drop` function gets called automatically by the
compiler when the variable goes out of scope, replacing the need to write it out
by hand.

In our case, we didn't have to write a custom `drop` for `File` because the
standard library already provides one. A `File` wraps an owned file descriptor,
and that type's implementation calls `close` on drop. Therefore, to answer our
question, when `Index::load` fails, `file` goes out of scope, Rust drops its
fields, and the file descriptor is closed.

What about the order? With the `goto` waterfall, the rule was to tear down in
the reverse order of acquisition, or, in other words, last-in-first-out. This is
especially important when one resource might depend on earlier ones. Otherwise,
you risk pulling a dependency out from under something that's still using it.
No need to worry. This is also taken care of by Rust, since local variables are
dropped in reverse order of declaration. What this means for us is that the
cleanup happens automatically in the same order we'd write the `goto` labels.

This new paradigm, called *Resource Acquisition Is Initialization* (RAII), ties
resource management to an object's lifetime: allocate on initialization,
deallocate on destruction. Let it sink in for a second before jumping (no pun
intended) to the exercise.

## Head to the exercise
