# Fixtures

Sample bookmark databases for the all-C `bm` binary.

File format (one bookmark per line):

    <url>\t<tag1>,<tag2>,...\n

## Files

- `sample.bm` — ~50 hand-curated entries, varied tag distribution. Quick
  iteration and demos.
- `large.bm` — ~500 entries. Exercises the index hashmap and triggers table
  growth.
- `corrupt.bm` — three valid entries, then a duplicate of the first (different
  case, same normalized URL). Loading stops there with `bm: ...:4: duplicate`
  and `bm` reports a corrupt database: this drives the storage error path. The
  malformed lines after it (bad scheme, no URL, no host, blank URL) would _not_
  fail on their own, because the C `bm_normalize_url` only rejects tabs.
