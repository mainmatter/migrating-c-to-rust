# Resource management

Many C functions acquire several resources in a row: memory, a file, a lock.
Each acquisition might depend on the previous one. If a later step fails,
everything acquired before it must be released, and nothing acquired after it
exists yet. What a function has to clean up therefore depends on where it
failed.

There is a bit of a paradigm shift here, but we'll see that the Rust way is
simpler to maintain and less prone to errors.

## Working with a resource

Let's start building our intuition from the ground up. Consider a function that
simply opens a file, returning an error in the failure case.

```c
struct Store {
    FILE *file;
};

BmStatus store_open(const char *path, struct Store *store) {
    store->file = fopen(path, "r+");

    if (store->file == NULL) {
        return BM_ERR_IO;
    }

    return BM_OK;
}
```

There is nothing to clean up in the case of failure here because no resource was
allocated.

The Rust equivalent is pretty straightforward:

```rust
pub struct Store {
    file: File,
}

pub fn open_store(path: &Path) -> io::Result<Store> {
    let file = File::options().read(true).write(true).open(path)?;
    Ok(Store { file })
}
```

`io::Result<T>` is a shorthand for `Result<T, std::io::Error>` provided by the
standard library.

The `?` operator is syntactic sugar for propagating the error. In this case it's
equivalent to:[^2]

```rust
let file = match File::options().read(true).write(true).open(path) {
    Ok(file) => file,
    Err(err) => return Err(err.into()),
};
```

## RAII and Drop

Now `store_open` also loads the index, which allocates memory:

```c
struct Store {
    FILE *file;
    struct Index *index;
};

BmStatus store_open(const char *path, struct Store *store) {
    store->file = fopen(path, "r+");
    if (store->file == NULL) {
        return BM_ERR_IO;
    }

    store->index = index_load(store->file);
    if (store->index == NULL) {
        fclose(store->file);
        return BM_ERR_IO;
    }
    return BM_OK;
}
```

If `index_load` fails, the file is already open, so we must remember to manually
close it. It's worth mentioning that after an error, `*store` is left in an
unspecified state (`store->file` now dangles), so the caller must not use it or
pass it to `store_close`.

In Rust, this endeavor becomes as simple as adding an extra line:

```rust
pub struct Store {
    index: Index,
    file: File,
}


pub fn open_store(path: &Path) -> io::Result<Store> {
    let file = File::options().read(true).write(true).open(path)?;
    let index = Index::load(&file)?;
    Ok(Store { index, file })
}
```

When `Index::load` fails, `?` returns early and file is closed automatically.
But how? What is happening behind the scenes?

The philosophy of RAII (Resource Acquisition is Initialization) is that
resource management is tied to the object's lifetime: allocate on
initialization, deallocate on destruction. 

The way it works is that `File` implements the `Drop` trait.[^1] Rust calls its
`drop` method when it goes out of scope.

Notice that we introduced `index` before `file` in our `Store`. This will
determine the order in which they are cleaned up, as we'll see later.

Instead, our job is just to think about the cleanup once when defining the
struct. Rust takes care of calling it at the exact right moment and in the
correct sequence, something you'll appreciate more and more as your functions
deal with multiple resources.

## Cleanup labels in C

The store also keeps a journal, which is another file the function has to open:

```c
struct Store {
    FILE *file;
    struct Index *index;
    struct Journal *journal;
};

BmStatus store_open(const char *path, struct Store *store) {
    store->file = fopen(path, "r+");
    if (store->file == NULL) {
        return BM_ERR_IO;
    }

    store->index = index_load(store->file);
    if (store->index == NULL) {
        fclose(store->file);
        return BM_ERR_IO;
    }

    store->journal = journal_open(path);
    if (store->journal == NULL) {
        index_free(store->index);
        fclose(store->file);
        return BM_ERR_IO;
    }
    return BM_OK;
}
```

Now each failure has a different set of "things" to give back:

| Fails at | Already acquired | To release |
| :--- | :--- | :--- |
| fopen | nothing | nothing |
| index_load | file | file |
| journal_open | file, index | index, then file |

The rows are cumulative, and the cleanup order is the reverse of the
acquisition order, since later resources may depend on earlier ones.

That's where the popular `goto` pattern with cleanup labels comes in handy:

> [!todo]
> decision
>
> can be introduced directly in previous code snipped to make section shorter
```c
BmStatus store_open(const char *path, struct Store *store) {
    BmStatus status;

    store->file = fopen(path, "r+");
    if (store->file == NULL) {
        return BM_ERR_IO;
    }

    store->index = index_load(store->file);
    if (store->index == NULL) {
        status = BM_ERR_IO;
        goto close_file;
    }

    store->journal = journal_open(path);
    if (store->journal == NULL) {
        status = BM_ERR_IO;
        goto free_index;
    }

    return BM_OK;

free_index:
    index_free(store->index);
close_file:
    fclose(store->file);
    return status;
}
```

The labels are in reverse order of acquisition and fall through into each other,
so a failure runs its own cleanup and everything below it. 

It works, but it depends on three rules that the programmer must be diligent
about:

- **Each failure jumps to the label of the last resource it acquired.** If you
  mess this up, you risk leaks or trying to release something that was never
acquired
- **Return before the labels on success** A missing `return BM_OK`
  frees what the caller is about to use.
- **A new resource needs a new label in the right position.** Inserting a step
  in the middle of the function changes the labels of everything below it.

In Rust, we simply add one more `let` and `?`:

```rust
pub struct Store {
    journal: Journal,
    index: Index,
    file: File,
}

pub fn open_store(path: &Path) -> io::Result<Store> {
    let file = File::options().read(true).write(true).open(path)?;
    let index = Index::load(&file)?;
    let journal = Journal::open(path)?;
    Ok(Store { journal, index, file })
}
```

## What happens on success

The C function returns `BM_OK` before reaching the labels, so the resources stay
open and the caller must close them later:

```c
void store_close(struct Store *store) {
    journal_close(store->journal);
    index_free(store->index);
    fclose(store->file);
}
```

In Rust, the ownership of `open_store` is transferred to the callee we return:

```rust
Ok(Store { journal, index, file })
```

Putting `journal`, `index` and `file` into the `Store` moves them. After a move,
the original variable no longer owns the value, so the function doesn't drop
those three locals when it returns. The `Store` owns them now, and `Ok(...)`
hands the `Store` to the caller. 

This also prevents drop-after-free bugs, since a value will never be dropped
twice.

> [!todo]
> Side note: worried about performance? we're not working with pointers here, what happens? are we copying
> data around? --> no, the caller reserves a stack slot for Store, and the object
> is created directly in the caller's function call stack.

The resources then live as long as the caller’s `Store` does:

```rust
{
    let store = open_store(Path::new("bookmarks.db"))?;
    // ... use `store` ...
} // `store` goes out of scope here: its fields are dropped
```

### Struct field drop order

It's important to keep in mind that when a struct is dropped, its fields are
dropped in *declaration* order (FIFO), which is different from what happens to
local variables, which are dropped in reverse order of declaration (LIFO).  This
is why `Store` declares `journal`, then `index`, then `file`. That order matches
the reverse-order cleanup in `store_close`, and it matters when one field
depends on another.

If `Store` needs extra work on close, implement `Drop` for `Store`.  Its drop
runs first *before* the fields' destructors, so your custom destructor still
sees `journal`, `index` and `file` intact.

## Side-by-side comparison

> [!todo]
> do we want a side-by-side comparison of the bm_open function?

## Head to the exercise

// suggestion for one of the exercises: make the reader implement the Drop trait
// to demystify it, so they understand what is actually happening.

[^1]: Strictly speaking, `File` has no `impl Drop for File` of its own. The
    cleanup happens one or more layers down, and `File` `inherits` it because it
owns a field whose type does implement `Drop`.

[^2] In reality, `?` expands to a call through the `Try` machinery, and for
`Result` the error is converted with `From::from`. When the error types are the
same, as in our example, that conversion is a no-op. It matters when you
propagate an error into a different error type.
