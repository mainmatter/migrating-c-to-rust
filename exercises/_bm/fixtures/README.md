# Fixtures

Sample bookmark databases for the all-C `bm` binary.

File format (one bookmark per line):

    <url>\t<tag1>,<tag2>,...\n

## Files

- `sample.bm` — ~50 hand-curated entries, varied tag distribution. Quick
  iteration and demos.
- `large.bm` — ~500 entries. Exercises the index hashmap and triggers table
  growth.
- `corrupt.bm` — three valid entries followed by malformed lines. Drives the
  storage error path.
