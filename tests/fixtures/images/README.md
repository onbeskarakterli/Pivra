# PIV-09 image assets

18 original synthetic assets: four-quadrant RGB JPEG/PNG; wide/tall/small/large
PNG; JPEG EXIF orientations 1–8, no EXIF and malformed TIFF; PNG bytes with a
.jpg name; truncated JPEG/PNG and text named .jpg. No downloaded photographs.
The manifest records hashes, exact byte sizes, colors, corner mappings and JPEG
center tolerance. No project license is selected or granted by these fixtures.

`python3 generate.py --check` uses only stdlib and verifies frozen hashes/lengths.
`--write` requires the already available Pillow **12.3.0**, used only to encode
trusted synthetic pixels with fixed settings. This is not a runtime fallback
image decoder. Two regenerations produced the same manifest; TIFF tags 1–8 were
independently inspected with struct. Those are asset checks, not decode/render
or orientation success. The previous 86-byte PNG/model fixtures are unchanged.

`cargo run --locked --offline --example image_probe` (inside a graphical session)
requires real sandboxed Glycin. It compares real frame dimensions and downloaded
texture pixels at quadrant centers, including all eight orientations, if valid
baseline decoding works. `-- --jpeg-first` selects the valid JPEG baseline first.
A baseline failure returns **77/BLOCKED**, never PASS. In this Codex environment
both valid PNG and JPEG attempts failed: no compatible loaders configured.
Successful-branch pixel assertions are compiled but NOT RUN. Application Fit,
resize, presented pixels, active-decode close and successful concurrent image
windows still require a working 2+ loader plus namespace-capable native session.
