# GNOME thumbnail freshness — persistent validity and in-flight identity

Decision base: `90f13b593863efe0891bdde8d86117d32cdfe2e6` on dev.
This closes the obsolete GNOME cache test expectation, not a version/release gate.

## Decision and evidence

**An identical-byte atomic replacement at the same URI with unchanged mtime and
size may reuse the valid persistent GNOME thumbnail.** Regeneration is unnecessary.
The test expecting `OUTCOME Stale` in that case described an older policy.

The deliberate production change is
`d9bbcc11dd8bb202195c87d6bc620da628da6cf5`, “Accept valid GNOME thumbnails after
Trash restore changes source ctime”. It removed the source-ctime vs cache-mtime
comparison. Restoring a file from Trash changes ctime without changing its pixels,
URI or mtime; that comparison rejected correct thumbnails. Production query code
still documents this decision. The old TH-01 validation text and atomic-replace
Python assertion were not updated with it. The later baseline failure at
`4dcd84c30013cacc884bd99de9244087d27f635d` reproduced that mismatch.

No production freshness or provider-selection logic is changed by this fix.
No content hashing, additional cache, decoder or regeneration retry is added.

## Two distinct contracts

1. **Persistent GNOME/GIO metadata validity:** GIO checks Thumb::URI and whole-
   second Thumb::MTime, plus Thumb::Size when present. Pivra additionally bounds
   and validates the PNG container and checks source/cache FileKey stability
   during the query. Neither inode replacement before a new query, chmod, nor
   a source ctime newer than the cache's filesystem mtime alone invalidates it.
2. **Already-issued candidate / in-flight work:** Candidate::still_current checks
   cancellation and full source/cache FileKey identity (path, device, inode,
   length, nanosecond mtime and ctime). Replacing a source or cache file invalidates
   that old candidate, even if bytes, size and mtime were preserved. A new query
   may obtain a new valid candidate; the old candidate must not be presented.

A HIT means metadata/container valid, **not decoded pixels or cryptographic
content identity**. Standard thumbnails do not carry an original-source digest.
If a file is changed between independent queries while URI, stored size and
whole-second mtime are all preserved, that metadata can alias different content.
FileKey protects an existing snapshot; it does not supply an old source digest
for a cold lookup. No test claims that the persistent query detects this alias.
Restoring the ctime heuristic would reintroduce the Trash regression without
creating a content-hash guarantee. Extending the persistent protocol is outside
this focused test correction. Secure Glycin decoding and current-candidate
checks remain mandatory before pixel presentation.

This decision concerns `src/thumbnail.rs`, the GNOME/GIO provider. The alternative
Freedesktop provider's separately documented conservative policy is unchanged.
It is not asserted that every provider uses the same freshness rules.

## Explicit regressions

| Scenario | Required result and evidence |
|---|---|
| Identical atomic replacement, same URI/mtime/size | HIT, current=true; new inode asserted; source bytes identical and cache bytes/inode/mtime unchanged |
| Changed pixels in place, new mtime, same size | Stale; changed bytes and unchanged source inode asserted |
| Changed pixels by same-size atomic replacement, new mtime | Stale; changed bytes, new inode and equal size asserted |
| Changed source size, preserved mtime | Stale via stored Thumb::Size; source remains a valid PNG container |
| Identical pixels, changed mtime | Stale |
| Wrong Thumb::URI/MTime/Size | Existing Stale assertions retained |
| Private Trash-like rename, absent source, restore at original URI | SourceUnavailable while absent, then HIT; bytes/inode/mtime preserved; cache intentionally older than source ctime and not rewritten |
| Permissions-only metadata change | HIT; bytes/inode/mtime preserved, ctime newer than cache |
| Already-issued source replaced with identical or changed same-size bytes and preserved mtime | Old candidate rejected |
| Cache replaced with preserved size/mtime | Old candidate rejected; new snapshot valid until cancelled |
| Source rename or cache deletion | Old candidate rejected using an independent fresh snapshot before each mutation |

The restored-file test models filesystem rename/restore in a private temporary
directory. It does not operate on the user's Trash or claim physical GNOME UI
acceptance. Python cache tests query real production GIO code in the freshly
built thumbnail_provider_probe; synthetic valid PNGs exercise metadata validity,
not Glycin decoding. Rust tests exercise real filesystem FileKeys/cancellation.
All original corrupt-container, symlink, size-limit, desktop/capability, helper
failure, failed-marker and source-missing assertions remain.

## Validation

Final command results are recorded below after execution. No native GPU tests,
package versions, Arch recipe/source configuration or release tags are changed.

### Completed checks

Codex private Linux SDK, real GLib/GIO cache queries, 2026-11-07:

| Command/suite | Result |
|---|---|
| Fresh `cargo build --locked --example thumbnail_provider_probe` | PASS |
| Focused `test_cache.py` | 23 PASS |
| Complete `tests/thumbnail` Python suite, including real C helper/stub tests | 59 PASS, 0 FAIL |
| Python root suite | 50 PASS |
| Python FD05A / video_gpu / video_color suites | 9 / 32 / 13 PASS |
| Total Python (focused 23 are included, not double-counted) | **163 PASS, 0 FAIL** |
| `cargo fmt --check` | PASS |
| `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, exit 0, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | **210 PASS, 0 FAIL** |

Production `src/thumbnail.rs` before its cfg(test) module is byte-for-byte
unchanged against the decision base. The existing candidate test now takes fresh
snapshots before rename/cache-delete, so those checks cannot pass merely because
an earlier source replacement had already invalidated the candidate. Two added
Rust tests protect same-size/time-preserved source and cache replacement; all
prior security/error assertions are retained. No runtime assertion is removed.

The specific regression gate reported in the v0.2.5 preparation audit is closed.
This does not authorize a version bump, main promotion, tag or publication.
Physical GNOME Trash/Undo, secure decoding and GPU visual tests were not rerun
or attributed to these metadata/unit tests. The new native Arch staging results
are user-reported acceptance, not independent Codex makepkg/installation evidence.
