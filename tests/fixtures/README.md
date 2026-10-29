# Fixture provenance and boundaries

`generate.py` produces these original bytes from hard-coded values; nothing is
downloaded. There are no third-party photographs, recordings or assets. The
repository license is still undecided: this task does not select a license or
introduce a license grant. Do not claim these files are public domain or CC0.

| File | Content | Current use |
|---|---|---|
| `pixels-2x2.png` | 86 bytes, original 2×2 RGBA red/green/blue/transparent-white | Copied into isolated Unicode/space-named paths for model tests; not decoded by Pivra |
| `model-video.txt` | 83 bytes, explicit NOT-playable-media message | Explicit `MediaKind::Video` metadata test only; **not a video fixture or playback test** |
| `manifest.json` | Exact byte lengths, SHA-256 and source/license status | Detect unintended fixture changes |

Validate read-only with `python3 tests/fixtures/generate.py --check`. Intentional
regeneration uses `--write`, followed by inspection of bytes and manifest diffs.
The PNG uses a fixed stored DEFLATE block, so encoder-version choices cannot
silently change its bytes. The generator requires only Python's standard library.
Neither the integrity check nor the `file` utility is a Pivra decoding test.

## Future fixture requirements (not implemented in PIV-04)

| Family | Small deterministic cases needed later |
|---|---|
| PNG | Current 2×2 alpha/colors; add orientation/metadata and corrupt/truncated cases when decoder tests exist |
| JPEG | Original tiny RGB pattern, known dimensions; separate EXIF orientation 1/6 cases; record generator/version, metadata and SHA-256 |
| WebP | Original lossless RGB/alpha pattern; later separate lossy case with tolerant pixel expectations |
| Animated GIF/APNG/WebP | Original two-frame patterns; fixed 100/200 ms durations and loop count; transparency/disposal expectations recorded |
| Video | Original tiny silent frame sequence, fixed size/rate/duration; explicit codec/container, timestamps and frame expectations; pin encoder/version and SHA-256 |

Future real encoded assets need recorded provenance and license status before
addition. Keep individual files below 32 KiB where possible; justify larger data.
Do not fetch random internet media, commit large collections, install an encoder
for the current model-only tests, or add a decoder/playback dependency here.

Tests read committed fixtures via `include_bytes!` and copy them into private
`tempfile` directories. The repository and user media are never test workspaces.
Fixtures do not determine MediaKind from a filename; callers supply that metadata.
