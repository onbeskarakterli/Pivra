# Modified LGPL adapter — notices, source and relinking

Pivra is GPL-3.0-only. The GTK-derived adapter files retain their original
copyright headers and LGPL-2.1-or-later grant. Both supplied LGPL texts and the
upstream notices in `native/video/` must be retained; copying only Pivra's GPL
LICENSE is insufficient. System GTK/GStreamer libraries remain shared libraries
and are supplied by the distribution, not bundled forks.

The adapter is a static archive linked into the Rust executable. We use the
complete-source route under LGPL 2.1 section 6(a): provide the modified adapter,
complete application source and corresponding locked crate sources, with build
instructions allowing modification and rebuilding/relinking. No opaque Rust
object files are offered in place of source, and no restriction on debugging
modified library code is added. This is an implementation compliance review,
not a claim of legal certification of every downstream distribution.

Arch packages include the exact approved application source archive, complete
checksum-verified crate archives and notices collected by `collect-licenses.py`,
the recipe/source configuration/collector, plus an easy-to-find adapter source
copy and both LGPL texts. Portable source staging includes an application source
snapshot with per-file hashes, all locked crate archives/notices and adapter
sources. The snapshot is not an approved immutable release archive.

Installed locations (PREFIX is /usr in an Arch package):

* `share/licenses/pivra/LICENSE`
* `share/licenses/pivra/video-adapter/COPYING.LGPL-{2,2.1}`
* `share/licenses/pivra/third-party/` (including inventory.json)
* `share/doc/pivra/corresponding-source/` (application and crate sources)
* `share/doc/pivra/corresponding-source/video-adapter/`

## Rebuild or relink with a modified adapter

1. Extract the application source archive into a writable private directory.
   The portable snapshot has a `Pivra-source/` top directory. For an Arch release,
   use the included exact approved archive, recipe and source configuration.
2. Install the native development/runtime dependencies listed in the package
   candidate report; run `bash packaging/linux/dependencies.sh check`.
3. Modify the LGPL adapter sources under `native/video/` as desired.
4. `cargo fetch --locked`, then
   `PIVRA_DISTRIBUTION_BUILD=1 cargo build --release --locked --offline --bin pivra`.
   This recompiles the static adapter and relinks the application against it.
   Install or test in a private DESTDIR with the unchanged source frontend.
5. The corresponding `.crate` archives are supplied alongside the application.
   Their SHA-256 values are in Cargo.lock and the notice inventory. Cargo's
   registry index/native toolchains are not bundled: the fetch step may need
   registry access. To work offline, populate a normal Cargo registry cache/index
   or use `cargo vendor --locked` in a connected preparation environment and
   preserve its generated source configuration. Native GTK/GStreamer development
   packages and their corresponding sources are available from their distributors.

Record `rustc -Vv`, native `pkg-config --modversion` output, recipe/toolchain flags
and the source manifest for a reproducibility comparison. Locked source selection
and deterministic staging archives do not by themselves prove bit-identical
binaries across compiler/driver versions. Do not copy local SDK environment paths
into a distributed executable or release recipe.
