# Resource management

Acquiring a resource, such as memory, a file, or a lock, comes with an
obligation to release it. When several are acquired in a row and each may depend
on the previous one, a failure partway through leaves some held and others
never acquired. Exactly the ones that are held must be released, so what needs
cleaning up depends on where the failure happened.

C leaves that bookkeeping to the programmer, who has to write it out for every
failure point. Rust makes it a question of ownership: whoever owns a value is
responsible for releasing it, and the compiler tracks who that is.

## What needs cleaning up in C?

Our `Store` needs three resources: a data file, an index loaded from that file,
and a journal. Here is `open_store` in C:

```c
struct Store {
    FILE *file;
    struct Index *index;
    struct Journal *journal;
};

BmStatus open_store(const char *path, struct Store *store) {
    BmStatus status;

    store->file = fopen(path, "r");
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

The table below summarizes what each failure point must release.

| Fails at | Already acquired | To release |
| :--- | :--- | :--- |
| `fopen` | nothing | nothing |
| `index_load` | `file` | `file` |
| `journal_open` | `file`, index` | `index`, then `file` |

The goto labels are in reverse order of acquisition and fall through into each
other, so a failure runs its own cleanup and everything below it.

It works, but it depends on three rules the programmer must follow diligently:

- **Each failure jumps to the label of the last resource it acquired.** Jumping
  to the wrong label either leaks a resource or releases something that was
  never acquired.
- **The success path must return before the labels.** A missing `return BM_OK;`
  frees what the caller is about to use.
- **A new resource needs a new label in the right position.** Inserting a step
  in the middle of the function changes which label every later failure must
  jump to.

There is also a subtler problem: after an error, `*store` can be left in an
unspecified state. Any failure after `fopen`, for example, leaves `store->file`
dangling. The caller must not use `*store` or pass it to `store_close` in that
case. In Rust there is no such state: a failed `open_store` returns no `Store`
at all, as we'll see next.

## How does Rust handle that failure?

Here is the same function in Rust:

```rust
# use std::fs::File;
# use std::io;
# use std::path::Path;
# struct Index;
# struct Journal;
# impl Index { fn load(_: &File) -> io::Result<Index> { Ok(Index) } }
# impl Journal { fn open(_: &Path) -> io::Result<Journal> { Ok(Journal) } }
pub struct Store {
    journal: Journal,
    index: Index,
    file: File,
}

pub fn open_store(path: &Path) -> io::Result<Store> {
    let file = File::open(path)?;
    let index = Index::load(&file)?;
    let journal = Journal::open(path)?;
    Ok(Store { journal, index, file })
}
```

When `Index::load` fails, `?` returns early and `file` is closed automatically.
When `Journal::open` fails, `index` and then `file` are released. But how does
that work behind the scenes?

### RAII and `Drop`

*Resource Acquisition Is Initialization* (RAII) ties resource management to an
object's lifetime: allocate on initialization, deallocate on destruction.

A type can implement the `Drop` trait to specify what happens when a value
is dropped. A `File` closes its underlying file descriptor when it is dropped.
A value is dropped when its owner goes out of scope, which is what happens
on an early return. Local variables are dropped in reverse order of declaration,
so the cleanup happens automatically in the correct order.

> **One caveat:** `drop` cannot return an error, so a failure during cleanup is
> silently ignored. If callers need to know about such failures, give the type
> an explicit close method that returns a `Result`.

## What happens on success?

On success, the C function returns `BM_OK` before reaching the labels, so the
resources stay open. The caller must close them later:

```c
void store_close(struct Store *store) {
    journal_close(store->journal);
    index_free(store->index);
    fclose(store->file);
}
```

In Rust, returning transfers ownership of the resources to the caller:

```rust,no-run
Ok(Store { journal, index, file })
```

Putting `journal`, `index`, and `file` into the `Store` *moves* them. After a move,
the original variable no longer owns the value, so the function doesn't drop
those three locals when it returns. The `Store` owns them now, and `Ok(...)`
hands the `Store` to the caller.

This is the Rust counterpart of returning before the labels. Since a moved value
is never dropped by its old owner, it can never be dropped twice.

The resources then live as long as the caller's `Store` does:

```rust,no-run
{
    let store = open_store(Path::new("bookmarks.db"))?;
    // ... use `store` ...
} // `store` goes out of scope here: its fields are dropped
```

## Head to the exercise
