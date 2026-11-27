# SRD importer

Fills the global catalog (`classes`, `species`, `spells`, `items`, `monsters`)
from the SRD 5.2 (CC-BY-4.0, see `docs/ATTRIBUTION.md`).

## Principle: two separate phases

```
             (network, rare)                    (offline, any time)
Open5e v2 ──── srd-import fetch ────▶ data/srd/srd-2024/ ──── srd-import import ────▶ PostgreSQL
                                      ├── manifest.json        (upsert by slug,
                                      ├── classes.json          one transaction)
                                      ├── species.json
                                      ├── spells.json
                                      ├── items.json
                                      └── monsters.json
```

- **`fetch`** is the only command that touches the network. It normalizes
  Open5e data into **our own format** (`crates/srd/src/model.rs`) and writes it
  to `data/srd/<document>/`. These files are **committed to git**.
- **`import`** reads only these files. Once the snapshot is committed, the
  project no longer depends on Open5e (CI, docker, fresh install, offline).

## Commands

```bash
cargo run -p srd -- fetch            # refresh snapshot from Open5e
cargo run -p srd -- verify           # check checksums + content (no DB)
cargo run -p srd -- import           # import into $DATABASE_URL
cargo run -p srd -- import --dry-run # report what would change, then roll back
cargo run -p srd -- import --force   # re-run even if checksum already imported
cargo run -p srd -- status           # snapshot checksum vs last DB import
cargo run -p srd -- rehash           # after hand-editing SRD files
```

Options: `--document srd-2024` (default), `--dir <path>`, `--base-url <url>`.

## Versioned

| Level | Mechanism |
|-------|-----------|
| SRD version | One directory per document key: `srd-2014` (5.1), `srd-2024` (5.2) |
| Snapshot content | `manifest.json` → sha256 per file + global `checksum` |
| Snapshot format | `format_version` (`FORMAT_VERSION` in `model.rs`), checked on read |
| Code history | Snapshot is in git: a re-fetch shows up as a reviewable diff |
| DB history | Table `srd_imports`: document, checksum, counts, date of each import |
| Row origin | `source_document` + `content_hash` columns on each catalog table |

The checksum only depends on the content (never on dates), so re-fetching
unchanged data gives the same checksum and **writes nothing**.

## Replayable

- **Idempotent**: if the latest `srd_imports` row for the document has the same
  checksum, `import` is a no-op ("already up to date").
- **Upsert on `slug`**: `ON CONFLICT (slug) DO UPDATE ... WHERE content_hash IS DISTINCT FROM`.
  UUIDs never change, so `characters.class_id` etc. keep pointing to the right row.
  Unchanged rows are not rewritten.
- **All or nothing**: a single transaction; an error leaves the DB untouched.
- **Concurrency-safe**: `pg_advisory_xact_lock` serializes simultaneous imports.
- **Never deletes**: entries removed from the snapshot are reported as `stale`
  (they may be referenced by characters).

## Importing from SRD files instead of Open5e

The importer does not care where the files come from. To use another source
(SRD PDF conversion, homebrew, fixes):

1. Write the JSON files in the format of `model.rs` into `data/srd/<document>/`
   (copy `manifest.json` and set `"source": "manual"`).
2. `cargo run -p srd -- rehash` to recompute checksums.
3. `cargo run -p srd -- import --dir data/srd/<document>`.

Any edit made without `rehash` is rejected by `verify` / `import` (checksum mismatch).

## Open5e mapping notes

- Endpoints: `classes`, `species`, `spells`, `items` + `magicitems` (merged into `items`), `creatures` → `monsters`.
- Filter `document__key__in=<document>` **plus** a client-side filter on
  `document.key`: `magicitems` ignores `document__key` and returns SRD 5.1 entries.
- Slug = Open5e key without the `<document>_` prefix (`srd-2024_fireball` → `fireball`).
- Subclasses → `classes.data.subclasses`, subspecies → `species.data.subspecies`.
- Species `speed` / `size` are parsed from the `SPEED` / `SIZE` traits.
- `data` keeps the full Open5e entry, minus `key`, `document`, `crossreferences`.

## Known limits

- `slug` is globally unique: importing `srd-2014` **and** `srd-2024` would make
  their common slugs (`fireball`...) overwrite each other. A multi-document
  catalog needs a `UNIQUE (source_document, slug)` constraint instead.
