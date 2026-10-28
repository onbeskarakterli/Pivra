# Pivra 0.1.0 dependency license audit

> B6 recheck (2026-10-29): project license, Cargo.lock and the 184-package
> inventory still match this audit; exact crate archives/notices were collected
> again offline with checksum verification. This is source-license consistency,
> not a fresh CVE audit or a binary-install verification. See the
> [B6 report](PIVRA_V0.1_B6_READINESS_AUDIT.md).

Date: 2026-10-27 (Europe/Istanbul). Baseline: `be2b9e1a6cbd4880e54b5546bb5ec7727bd25e86`.
Project license decision: **GPL-3.0-only**.

## Result and evidence boundary

**B4: PASS for the current project/source license scope.** No incompatible
or unspecified grant was identified in the audited packages. Explicit published
SPDX declarations and pinned source grants, rather than mere absence/presence
of a file called LICENSE, determine the reviewed license choices. Binary/package
redistribution must still satisfy the notice/source/relinking duties below.
No incompatible license expression was found in the **184 external
Cargo.lock packages** or **135 registry packages in the recorded loader build**.
This is not approval of an arbitrary future OS codec, plugin or binary bundle.
Dependencies retain their own grants; compatible combination does not erase
upstream licenses, copyrights, source availability or relinking rights.

`cargo metadata --locked --offline --format-version 1` covered all 185 locked
packages including Pivra, build/dev dependencies and foreign-target entries.
All 184 cached `.crate` archive SHA-256 values match Cargo.lock; their archived
Cargo.toml files match the source metadata inspected here. The supplied
license/notice documents and special AND conditions were inspected. This is
not a CVE scan. No crate versions or Cargo.lock contents changed.
Direct runtime packages are gtk4 0.10.3, libadwaita 0.8.1 and glycin 3.0.8.
Direct dev dependencies and all transitives appear in the table with exact
versions. Listing a foreign-target/build/dev package is not a runtime-link claim.

## GPL text provenance

LICENSE is copied byte-for-byte from Debian `/usr/share/common-licenses/GPL-3`.
Debian base-files' copyright record says the GNU licenses came from ftp.gnu.org
and permits verbatim redistribution. It contains the complete preamble,
sections 0–17 and application appendix, without modifying placeholders or
inventing a Pivra copyright owner.
SHA-256: `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`.
The direct <https://www.gnu.org/licenses/gpl-3.0.txt> fetch returned HTTP 403;
no independent online byte comparison is claimed. The license document's FSF
copyright is not a claim that FSF owns Pivra.
Cargo/README explicitly choose GPL version 3 **only**; explanatory later-version
examples in the unchanged GNU text do not change that project decision.

## Notice codes and selected license branches

- **M — MIT:** preserve the upstream copyright and permission text with source
  or substantial portions, including in binary documentation/notices.
- **B — BSD-2/3-Clause:** retain copyright, conditions and disclaimer in source;
  reproduce them in binary documentation. BSD-3-Clause forbids endorsement.
- **I — ISC:** preserve the upstream copyright and permission notice.
- **L — LGPL-2.1-or-later:** select this branch of Glycin's MPL/LGPL alternatives.
  Retain licenses/notices/disclaimers, identify modifications and provide
  corresponding library source. Preserve replacement/relinking and permitted
  debugging rights (LGPL §6). Static inclusion can require relinkable application
  objects or a compliant corresponding-source/rebuild route; a stripped binary
  and GPL text alone are insufficient. LGPL §3 allows applying GPLv2 or a later
  version when needed. No upstream notice is removed or relicensed by this audit.
- **A — Apache-2.0:** preserve the license, applicable NOTICE and attribution,
  mark modifications, and respect patent/notice conditions. Compatible with GPLv3.
- **E — LLVM exception:** preserve the exception with Apache-2.0; it relaxes
  additional constraints and does not create a GPLv2-only restriction.
- **U — Unicode-3.0:** an additional AND condition; preserve Unicode's supplied
  copyright, permission and disclaimer, even when MIT is selected for code.
- **J — IJG:** preserve the full source notice and identify source changes;
  binary documentation must acknowledge IJG. jpeg-encoder 0.6.1 `src/fdct.rs`
  contains the complete conditions. Its MIT/Apache choice does not remove IJG.
  If distributing that loader: **This software is based in part on the work of
  the Independent JPEG Group.** Do not imply endorsement by IJG authors.

Legacy slash expressions denote upstream alternatives (corroborated by shipped
license files); MIT is selected. OR selects a branch; AND retains every condition.
The inventory is not a replacement for shipping actual license texts/notices
and satisfying source/relinking duties in a future distributable. This task
creates no binary bundle or installer.

## Complete Pivra Cargo.lock inventory

Every row has a reviewed GPLv3-compatible branch, conditional on its notice
codes. Evidence paths are relative to the versioned upstream crate, available
through its crates.io version and verified Cargo.lock archive checksum.

| Package | Version | Upstream license/options | Selected branch | GPL-3.0-only compatible | Notices | Evidence |
|---|---|---|---|---|---|---|
| aho-corasick | 1.1.5 | Unlicense OR MIT | MIT | yes | M | `COPYING`, `LICENSE-MIT` |
| async-broadcast | 0.7.2 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-executor | 1.14.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-fs | 2.2.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-global-executor | 3.1.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-io | 2.6.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-lock | 3.4.2 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-process | 2.5.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-recursion | 1.2.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-signal | 0.2.14 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-task | 4.7.1 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| async-trait | 0.1.92 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-THIRD-PARTY` |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| bitflags | 1.3.2 | MIT/Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| blocking | 1.7.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-ZLIB` |
| cairo-rs | 0.21.5 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| cairo-sys-rs | 0.21.5 | MIT | MIT | yes | M | `LICENSE` |
| cc | 1.9.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| cfg-expr | 0.20.10 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| cfg_aliases | 0.2.2 | MIT | MIT | yes | M | `LICENSE`, `NOTICES.md` |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| crossbeam-deque | 0.8.8 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| crunchy | 0.2.4 | MIT | MIT | yes | M | `LICENSE` |
| defmt | 1.2.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| defmt-macros | 1.2.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| defmt-parser | 1.1.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| dlib | 0.5.3 | MIT | MIT | yes | M | `LICENSE.txt` |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | MIT | yes | M | `LICENSE` |
| either | 1.19.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| endi | 1.1.1 | MIT | MIT | yes | M | `LICENSE-MIT` |
| enumflags2 | 0.7.12 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| enumflags2_derive | 0.7.12 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| env_filter | 2.0.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| env_logger | 0.11.11 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| errno | 0.3.14 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| field-offset | 0.3.6 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| foreign-types | 0.5.0 | MIT/Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-channel | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-executor | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-THIRD-PARTY` |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-timer | 3.0.4 | MIT/Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| gdk-pixbuf | 0.21.5 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| gdk-pixbuf-sys | 0.21.5 | MIT | MIT | yes | M | `LICENSE` |
| gdk4 | 0.10.3 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| gdk4-sys | 0.10.3 | MIT | MIT | yes | M | `LICENSE` |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| gio | 0.21.5 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| gio-sys | 0.21.5 | MIT | MIT | yes | M | `LICENSE` |
| glib | 0.21.5 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| glib-macros | 0.21.5 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| glib-sys | 0.21.5 | MIT | MIT | yes | M | `LICENSE` |
| glycin | 3.0.8 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L | `LICENSE`, `LICENSE-LGPL-2.1`, `LICENSE-MPL-2.0` |
| glycin-common | 1.0.3 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L | `LICENSE`, `LICENSE-LGPL-2.1`, `LICENSE-MPL-2.0` |
| glycin-utils | 4.0.4 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L | `LICENSE`, `LICENSE-LGPL-2.1`, `LICENSE-MPL-2.0` |
| gobject-sys | 0.21.5 | MIT | MIT | yes | M | `LICENSE` |
| graphene-rs | 0.21.5 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| graphene-sys | 0.21.5 | MIT | MIT | yes | M | `LICENSE` |
| gsk4 | 0.10.3 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| gsk4-sys | 0.10.3 | MIT | MIT | yes | M | `LICENSE` |
| gtk4 | 0.10.3 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| gtk4-macros | 0.10.3 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| gtk4-sys | 0.10.3 | MIT | MIT | yes | M | `LICENSE` |
| gufo-common | 1.1.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L | `LICENSE`, `LICENSE-LGPL-2.1`, `LICENSE-MPL-2.0` |
| gufo-exif | 0.3.1 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L | `LICENSE`, `LICENSE-LGPL-2.1`, `LICENSE-MPL-2.0` |
| half | 2.7.1 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| heck | 0.5.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| hermit-abi | 0.5.3 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| hex | 0.4.3 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| jiff | 0.2.38 | Unlicense OR MIT | MIT | yes | M | `COPYING`, `LICENSE-MIT` |
| jiff-core | 0.1.1 | Unlicense OR MIT | MIT | yes | M | `COPYING`, `LICENSE-MIT` |
| jiff-static | 0.2.38 | Unlicense OR MIT | MIT | yes | M | `COPYING`, `LICENSE-MIT` |
| jobserver | 0.1.35 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| js-sys | 0.3.106 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| lcms2 | 6.2.0 | MIT | MIT | yes | M | `LICENSE` |
| lcms2-sys | 4.0.7 | MIT | MIT | yes | M | `vendor/LICENSE` |
| libadwaita | 0.8.1 | MIT | MIT | yes | M | `LICENCE` |
| libadwaita-sys | 0.8.1 | MIT | MIT | yes | M | `LICENCE` |
| libc | 0.2.191 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| libloading | 0.8.9 | ISC | ISC | yes | I | `LICENSE` |
| libseccomp | 0.4.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE`, `LICENSE-APACHE`, `LICENSE-MIT` |
| libseccomp-sys | 0.3.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE`, `LICENSE-APACHE`, `LICENSE-MIT` |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE-APACHE`, `LICENSE-Apache-2.0_WITH_LLVM-exception`, `LICENSE-MIT` |
| log | 0.4.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| memchr | 2.8.3 | Unlicense OR MIT | MIT | yes | M | `COPYING`, `LICENSE-MIT` |
| memfd | 0.6.6 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| memmap2 | 0.9.11 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| memoffset | 0.9.1 | MIT | MIT | yes | M | `LICENSE` |
| nix | 0.30.1 | MIT | MIT | yes | M | `LICENSE` |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| ordered-stream | 0.2.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| pango | 0.21.5 | MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE` |
| pango-sys | 0.21.5 | MIT | MIT | yes | M | `LICENSE` |
| parking | 2.2.1 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-THIRD-PARTY` |
| paste | 1.0.15 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| piper | 0.2.5 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| polling | 3.11.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| quote | 1.0.47 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | MIT | yes | M | `AUTHORS (full license terms)`, `README.md (license options)` |
| rayon | 1.12.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| rayon-core | 1.13.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| regex | 1.13.1 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT`, `src/unicode_tables/LICENSE-UNICODE` |
| rmp | 0.8.15 | MIT | MIT | yes | M | `LICENSE` |
| rmp-serde | 1.3.1 | MIT | MIT | yes | M | `LICENSE` |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| rustix | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | MIT | yes | M | `COPYRIGHT`, `LICENSE-APACHE`, `LICENSE-Apache-2.0_WITH_LLVM-exception`, `LICENSE-MIT` |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| semver | 1.0.28 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| serde | 1.0.229 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| serde_repr | 0.1.21 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| serde_spanned | 1.1.2 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| shlex | 2.0.1 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| slab | 0.4.12 | MIT | MIT | yes | M | `LICENSE` |
| smallvec | 1.16.3 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| syn | 2.0.119 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| syn | 3.0.9 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| system-deps | 7.0.8 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| target-lexicon | 0.13.5 | Apache-2.0 WITH LLVM-exception | Apache-2.0 WITH LLVM-exception | yes | A,E | `LICENSE` |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| toml | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| toml_writer | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| tracing | 0.1.44 | MIT | MIT | yes | M | `LICENSE` |
| tracing-attributes | 0.1.31 | MIT | MIT | yes | M | `LICENSE` |
| tracing-core | 0.1.36 | MIT | MIT | yes | M | `LICENSE`, `src/spin/LICENSE` |
| uds_windows | 1.2.1 | MIT | MIT | yes | M | `LICENSE` |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | MIT AND Unicode-3.0 | yes | M,U | `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-UNICODE` |
| uuid | 1.26.1 | Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| version-compare | 0.2.1 | MIT | MIT | yes | M | `LICENSE` |
| wasm-bindgen | 0.2.129 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| wasm-bindgen-macro | 0.2.129 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| wasm-bindgen-macro-support | 0.2.129 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| wasm-bindgen-shared | 0.2.129 | MIT OR Apache-2.0 | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-MIT` |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | MIT | yes | M | `license-apache-2.0`, `license-mit` |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | MIT | yes | M | `license-apache-2.0`, `license-mit` |
| winnow | 0.7.15 | MIT | MIT | yes | M | `LICENSE-MIT` |
| winnow | 1.0.4 | MIT | MIT | yes | M | `LICENSE-MIT` |
| yeslogic-fontconfig-sys | 6.0.1 | MIT | MIT | yes | M | `LICENSE` |
| zbus | 5.13.2 | MIT | MIT | yes | M | `LICENSE` |
| zbus_macros | 5.13.2 | MIT | MIT | yes | M | `LICENSE` |
| zbus_names | 4.3.1 | MIT | MIT | yes | M | `LICENSE` |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-BSD`, `LICENSE-MIT` |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | MIT | yes | M | `LICENSE-APACHE`, `LICENSE-BSD`, `LICENSE-MIT` |
| zvariant | 5.9.2 | MIT | MIT | yes | M | `LICENSE` |
| zvariant_derive | 5.9.2 | MIT | MIT | yes | M | `LICENSE` |
| zvariant_utils | 3.3.0 | MIT | MIT | yes | M | `LICENSE` |

## External sandboxed loader (outside Pivra Cargo.lock)

The client does not vendor an OS loader. The isolated SDK records
GNOME glycin-image-rs **2.0.0**, tag 2.0.0, commit
`724f18c5f3519885995428a7e2e6e5df9ca2ab27`, protocol **2+**.
The recorded and independently re-hashed loader binary SHA-256 is
`9b93831143bfe83e9a9d87aea4c2d94dab4f657bb0c853a60517c295172a416b`.
Workspace Cargo.toml/LICENSE grant MPL-2.0 OR LGPL-2.1-or-later; LGPL is selected.
This is license/build-artifact evidence, not a successful decode execution.

The conservative registry footprint below comes from compiled `.d` records:
135 packages including build-time code. It is not the whole upstream
multi-loader workspace. JPEG uses zune-jpeg, PNG uses png, and image/gufo provide
container/metadata functionality. Other codecs in the upstream loader bundle
are not Pivra v0.1 feature claims. License declarations for all rows were located; IJG's full source header
was separately inspected. The packages without standalone files were resolved using the pinned
upstream evidence below.
Workspace-wide offline metadata encountered uncached addr2line/ahash outside
this recorded build footprint; no claim is made about those unbuilt workspace
members. No downloads or version substitutions were made to resolve them.
A different distro loader package needs its own version-specific notices/SBOM;
historical user Arch Glycin 2.2.3 is not the exact binary audited here.

| Loader-build package | Version | Upstream license/options | Selected branch | GPLv3 compatible | Notices |
|---|---|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | MIT | yes | M |
| async-broadcast | 0.7.2 | MIT OR Apache-2.0 | MIT | yes | M |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | MIT | yes | M |
| async-executor | 1.13.3 | Apache-2.0 OR MIT | MIT | yes | M |
| async-io | 2.5.0 | Apache-2.0 OR MIT | MIT | yes | M |
| async-lock | 3.4.1 | Apache-2.0 OR MIT | MIT | yes | M |
| async-process | 2.4.0 | Apache-2.0 OR MIT | MIT | yes | M |
| async-signal | 0.2.12 | Apache-2.0 OR MIT | MIT | yes | M |
| async-task | 4.7.1 | Apache-2.0 OR MIT | MIT | yes | M |
| async-trait | 0.1.89 | MIT OR Apache-2.0 | MIT | yes | M |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | MIT | yes | M |
| autocfg | 1.5.0 | Apache-2.0 OR MIT | MIT | yes | M |
| bit_field | 0.10.3 | Apache-2.0/MIT | MIT | yes | M |
| bitflags | 2.9.3 | MIT OR Apache-2.0 | MIT | yes | M |
| blocking | 1.6.2 | Apache-2.0 OR MIT | MIT | yes | M |
| bytemuck | 1.23.2 | Zlib OR Apache-2.0 OR MIT | MIT | yes | M |
| byteorder | 1.5.0 | Unlicense OR MIT | MIT | yes | M |
| byteorder-lite | 0.1.0 | Unlicense OR MIT | MIT | yes | M |
| cfg-if | 1.0.3 | MIT OR Apache-2.0 | MIT | yes | M |
| cfg_aliases | 0.2.1 | MIT | MIT | yes | M |
| color_quant | 1.1.0 | MIT | MIT | yes | M |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | MIT | yes | M |
| crc32fast | 1.5.0 | MIT OR Apache-2.0 | MIT | yes | M |
| crossbeam-deque | 0.8.6 | MIT OR Apache-2.0 | MIT | yes | M |
| crossbeam-epoch | 0.9.18 | MIT OR Apache-2.0 | MIT | yes | M |
| crossbeam-utils | 0.8.21 | MIT OR Apache-2.0 | MIT | yes | M |
| either | 1.15.0 | MIT OR Apache-2.0 | MIT | yes | M |
| encoding_rs | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause | MIT AND BSD-3-Clause | yes | M,B |
| endi | 1.1.0 | MIT | MIT | yes | M |
| enumflags2 | 0.7.12 | MIT OR Apache-2.0 | MIT | yes | M |
| enumflags2_derive | 0.7.12 | MIT OR Apache-2.0 | MIT | yes | M |
| env_filter | 0.1.3 | MIT OR Apache-2.0 | MIT | yes | M |
| env_logger | 0.11.8 | MIT OR Apache-2.0 | MIT | yes | M |
| equivalent | 1.0.2 | Apache-2.0 OR MIT | MIT | yes | M |
| event-listener | 5.4.1 | Apache-2.0 OR MIT | MIT | yes | M |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | MIT | yes | M |
| exr | 1.73.0 | BSD-3-Clause | BSD-3-Clause | yes | B |
| fastrand | 2.3.0 | Apache-2.0 OR MIT | MIT | yes | M |
| fax | 0.2.6 | MIT | MIT | yes | M |
| fax_derive | 0.2.0 | MIT | MIT | yes | M |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | MIT | yes | M |
| flate2 | 1.1.2 | MIT OR Apache-2.0 | MIT | yes | M |
| futures-core | 0.3.31 | MIT OR Apache-2.0 | MIT | yes | M |
| futures-io | 0.3.31 | MIT OR Apache-2.0 | MIT | yes | M |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT | MIT | yes | M |
| futures-macro | 0.3.31 | MIT OR Apache-2.0 | MIT | yes | M |
| futures-task | 0.3.31 | MIT OR Apache-2.0 | MIT | yes | M |
| futures-util | 0.3.31 | MIT OR Apache-2.0 | MIT | yes | M |
| getrandom | 0.3.3 | MIT OR Apache-2.0 | MIT | yes | M |
| gif | 0.13.3 | MIT OR Apache-2.0 | MIT | yes | M |
| gufo | 0.3.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| gufo-common | 1.0.1 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| gufo-exif | 0.3.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| gufo-jpeg | 0.3.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| gufo-png | 0.3.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| gufo-tiff | 0.3.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| gufo-webp | 0.3.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| gufo-xmp | 0.3.0 | MPL-2.0 OR LGPL-2.1-or-later | LGPL-2.1-or-later | yes | L |
| half | 2.6.0 | MIT OR Apache-2.0 | MIT | yes | M |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | MIT | yes | M |
| hex | 0.4.3 | MIT OR Apache-2.0 | MIT | yes | M |
| image | 0.25.7 | MIT OR Apache-2.0 | MIT | yes | M |
| image-webp | 0.2.4 | MIT OR Apache-2.0 | MIT | yes | M |
| indexmap | 2.11.0 | Apache-2.0 OR MIT | MIT | yes | M |
| jiff | 0.2.15 | Unlicense OR MIT | MIT | yes | M |
| jpeg-encoder | 0.6.1 | (MIT OR Apache-2.0) AND IJG | MIT AND IJG | yes | M,J |
| lebe | 0.5.2 | BSD-3-Clause | BSD-3-Clause | yes | B |
| libc | 0.2.175 | MIT OR Apache-2.0 | MIT | yes | M |
| libseccomp | 0.4.0 | MIT OR Apache-2.0 | MIT | yes | M |
| libseccomp-sys | 0.3.0 | MIT OR Apache-2.0 | MIT | yes | M |
| linux-raw-sys | 0.9.4 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | MIT | yes | M |
| log | 0.4.27 | MIT OR Apache-2.0 | MIT | yes | M |
| memmap2 | 0.9.8 | MIT OR Apache-2.0 | MIT | yes | M |
| memoffset | 0.9.1 | MIT | MIT | yes | M |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | MIT | yes | M |
| moxcms | 0.7.5 | BSD-3-Clause OR Apache-2.0 | BSD-3-Clause | yes | B |
| nix | 0.30.1 | MIT | MIT | yes | M |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | MIT | yes | M |
| once_cell | 1.21.3 | MIT OR Apache-2.0 | MIT | yes | M |
| ordered-stream | 0.2.0 | MIT OR Apache-2.0 | MIT | yes | M |
| parking | 2.2.1 | Apache-2.0 OR MIT | MIT | yes | M |
| paste | 1.0.15 | MIT OR Apache-2.0 | MIT | yes | M |
| pin-project-lite | 0.2.16 | Apache-2.0 OR MIT | MIT | yes | M |
| pin-utils | 0.1.0 | MIT OR Apache-2.0 | MIT | yes | M |
| piper | 0.2.4 | MIT OR Apache-2.0 | MIT | yes | M |
| pkg-config | 0.3.32 | MIT OR Apache-2.0 | MIT | yes | M |
| png | 0.18.0 | MIT OR Apache-2.0 | MIT | yes | M |
| polling | 3.10.0 | Apache-2.0 OR MIT | MIT | yes | M |
| ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 | MIT | yes | M |
| proc-macro-crate | 3.3.0 | MIT OR Apache-2.0 | MIT | yes | M |
| proc-macro2 | 1.0.101 | MIT OR Apache-2.0 | MIT | yes | M |
| pxfm | 0.1.20 | BSD-3-Clause OR Apache-2.0 | BSD-3-Clause | yes | B |
| qoi | 0.4.1 | MIT/Apache-2.0 | MIT | yes | M |
| quick-error | 2.0.1 | MIT/Apache-2.0 | MIT | yes | M |
| quote | 1.0.40 | MIT OR Apache-2.0 | MIT | yes | M |
| rand | 0.9.2 | MIT OR Apache-2.0 | MIT | yes | M |
| rand_chacha | 0.9.0 | MIT OR Apache-2.0 | MIT | yes | M |
| rand_core | 0.9.3 | MIT OR Apache-2.0 | MIT | yes | M |
| rayon | 1.11.0 | MIT OR Apache-2.0 | MIT | yes | M |
| rayon-core | 1.13.0 | MIT OR Apache-2.0 | MIT | yes | M |
| rmp | 0.8.14 | MIT | MIT | yes | M |
| rmp-serde | 1.3.0 | MIT | MIT | yes | M |
| rustix | 1.0.8 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | MIT | yes | M |
| serde | 1.0.219 | MIT OR Apache-2.0 | MIT | yes | M |
| serde_derive | 1.0.219 | MIT OR Apache-2.0 | MIT | yes | M |
| serde_repr | 0.1.20 | MIT OR Apache-2.0 | MIT | yes | M |
| signal-hook-registry | 1.4.6 | Apache-2.0/MIT | MIT | yes | M |
| simd-adler32 | 0.3.7 | MIT | MIT | yes | M |
| slab | 0.4.11 | MIT | MIT | yes | M |
| smallvec | 1.15.1 | MIT OR Apache-2.0 | MIT | yes | M |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | MIT | yes | M |
| syn | 2.0.106 | MIT OR Apache-2.0 | MIT | yes | M |
| thiserror | 2.0.16 | MIT OR Apache-2.0 | MIT | yes | M |
| thiserror-impl | 2.0.16 | MIT OR Apache-2.0 | MIT | yes | M |
| tiff | 0.10.3 | MIT | MIT | yes | M |
| toml_datetime | 0.6.11 | MIT OR Apache-2.0 | MIT | yes | M |
| toml_edit | 0.22.27 | MIT OR Apache-2.0 | MIT | yes | M |
| tracing | 0.1.41 | MIT | MIT | yes | M |
| tracing-attributes | 0.1.30 | MIT | MIT | yes | M |
| tracing-core | 0.1.34 | MIT | MIT | yes | M |
| unicode-ident | 1.0.18 | (MIT OR Apache-2.0) AND Unicode-3.0 | MIT AND Unicode-3.0 | yes | M,U |
| weezl | 0.1.10 | MIT OR Apache-2.0 | MIT | yes | M |
| winnow | 0.7.13 | MIT | MIT | yes | M |
| xml-rs | 0.8.27 | MIT | MIT | yes | M |
| zbus | 5.10.0 | MIT | MIT | yes | M |
| zbus_macros | 5.10.0 | MIT | MIT | yes | M |
| zbus_names | 4.2.0 | MIT | MIT | yes | M |
| zerocopy | 0.8.26 | BSD-2-Clause OR Apache-2.0 OR MIT | MIT | yes | M |
| zerocopy-derive | 0.8.26 | BSD-2-Clause OR Apache-2.0 OR MIT | MIT | yes | M |
| zune-core | 0.4.12 | MIT OR Apache-2.0 OR Zlib | MIT | yes | M |
| zune-inflate | 0.2.54 | MIT OR Apache-2.0 OR Zlib | MIT | yes | M |
| zune-jpeg | 0.4.20 | MIT OR Apache-2.0 OR Zlib | MIT | yes | M |
| zvariant | 5.7.0 | MIT | MIT | yes | M |
| zvariant_derive | 5.7.0 | MIT | MIT | yes | M |
| zvariant_utils | 3.2.1 | MIT | MIT | yes | M |

## Packages without standalone license files: resolved evidence

The published archives for fax 0.2.6, fax_derive 0.2.0, zune-core 0.4.12,
zune-inflate 0.2.54 and zune-jpeg 0.4.20 have no standalone LICENSE file.
They are not classified as unknown grants: versioned Cargo manifests contain
explicit licenses, corroborated as follows.

- fax/fax_derive: both exact upstream manifests at pdf-rs/fax commit
  `40b226eb97b391355e971fd95a8dd532a4c127a0` explicitly state `license = "MIT"`,
  matching the published archives. Preserve these source declarations and any
  existing notices; include the MIT permission/disclaimer in binary notices.
  Do not turn Cargo's author field into an invented copyright assertion.
- zune-jpeg: etemesi254/zune-image commit
  `12c415278200f0bd42dc2ef19c1be97b7312251e`, root `LICENSE.md`, grants MIT or
  Apache-2.0 and names the zune-image developers. MIT is selected; no reliance
  on an unverified Zlib text is necessary.
- zune-core: same repository commit
  `f8fbb123d5ed04441e8324a555bfcda0cb1bd28f`, `LICENSE.md`, same grant/notice.
  Both retrieved notices hash to
  `c6dff146a9f31848ac296faa5a08a4253caf2c384c86f906dc99e7fc0a39cc8c`.
- zune-inflate: etemesi254/zune-image commit
  `69502ce83fdfecdd0beefd677e2abb3781b29d98`, root `LICENSE.md`, MIT/Apache
  grant with the original 2022 developer notice. SHA-256:
  `e870d7fd7819a9f0453c62798cb52ae95d148a8c91dc2de1d983f997143c72de`.

Upstream revisions are those recorded in each crate's `.cargo_vcs_info.json`.
Preserve the actual upstream notices and source declarations; a package author
or dependency metadata field is not an instruction to create new copyrights.
Initial raw requests for LICENSE/LICENSE-MIT returned 404; GitHub API returned
403. Exact Git trees and the actual `LICENSE.md` paths resolved the evidence,
so those failed URL guesses are not treated as license incompatibility.
The fax source repository also contains an ITU specification PDF that is
explicitly excluded by its crate manifest: that PDF is not a bundled runtime
or a granted Pivra asset, and is not copied or redistributed here.

## Native toolkit, color and sandbox components

Rust binding licenses do not replace native-library licenses. Versions describe
inspected evidence; no SDK libraries are copied into this repository or bundled.

| Component / evidence | License and compatibility with GPLv3 | Distribution obligations |
|---|---|---|
| GTK 4.18.6 / SDK libgtk-4-1 copyright | LGPL-2.1-or-later primary grant, compatible; bundled permissive notices include MIT/Expat, BSD-3-Clause, Apache-2.0, Unicode, CC0 and ZPL-2.1 | LGPL source/replacement rights; preserve complete package notices. Debian test-file GPL-3+ is not a runtime license switch |
| libadwaita 1.7.6 / SDK package copyright | LGPL-2.1-or-later, compatible. Documentation separately identifies CC-BY-SA-4.0 and CC0 | LGPL source/relinking notices; retain documentation licenses if those documents are distributed |
| GNOME desktop thumbnail API / gnome-desktop 44.4 source reference | Thumbnail implementation explicitly LGPL-2.0-or-later, compatible using later-version permission. Repository COPYING alone is not evidence of GPLv2-only library licensing | COPYING.LIB and file notices; corresponding library source/relinking rights; distro-specific package review on actual bundling |
| GLib/GIO/GObject, GdkPixbuf, Pango and Cairo | LGPL-family toolkit linkage, compatible; Cairo's LGPL-2.1 alternative is used rather than assuming MPL-1.1 equivalence | Original distro licenses/source/replacement rights; no static toolkit bundle made |
| LittleCMS / lcms2-sys 4.0.7 vendor LICENSE and system 2.16 copyright | MIT base library, compatible. Optional fast-float/threaded plugins carry GPLv3 terms | Preserve vendor MIT notice; do not apply it indiscriminately to separately licensed plugins; no plugin added |
| Bubblewrap 0.12.0 / system copyright | LGPL-2.1-or-later, compatible; separate executable | Original license/notices and corresponding source if redistributed; sandbox remains mandatory |
| libseccomp 2.6.0 / system copyright | LGPL-2.1, compatible (including LGPL §3 GPL conversion option where needed) | LGPL source/linking rights; Rust bindings have separate MIT/Apache choices |
| glibc 2.41 and GCC runtime 14.2 | LGPL-2.1-or-later libc; GPL-3.0-or-later WITH GCC-exception-3.1 libgcc; compatible under runtime exception | System notices/source/runtime exception when redistributed; no runtime/compiler vendoring |

Evidence: SDK `usr/share/doc/libgtk-4-1/copyright` and
`usr/share/doc/libadwaita-1-0/copyright`, system
`/usr/share/doc/{bubblewrap,libseccomp2,liblcms2-2}/copyright`,
crate `lcms2-sys/vendor/LICENSE`, and GNOME's versioned source header:
<https://github.com/GNOME/gnome-desktop/blob/44.4/libgnome-desktop/gnome-desktop-thumbnail.c>.
The optional GNOME factory development helper is unavailable in this SDK;
license review is not helper generation/runtime acceptance.

## Delivery and validation boundary

Source archives retain LICENSE and all existing notices. Binary conveyance
requires GPL corresponding source (including necessary build/install scripts),
license/notices and applicable installation information, plus the LGPL and
permissive obligations above. Cargo.lock is not that corresponding source.
A future AUR recipe or binary bundle must implement these duties and audit its
actual native packages. This task makes no package, tag, installer, release or
third-party binary bundle. This is B4 source licensing closure, not a claim
that installation/package/native acceptance gates have closed.

Validation in this task: `cargo fmt --check` PASS;
`CARGO_INCREMENTAL=0 cargo test --locked` PASS (186 tests, zero failures);
`cargo check --locked` PASS; `cargo clippy --locked` PASS with existing unused
`load_stats` and identical pin-icon branch warnings. Cargo metadata reports
GPL-3.0-only; LICENSE matches the Debian GNU text byte-for-byte and its stated
SHA-256; Cargo.lock is unchanged. No GUI/native test was rerun for this docs task.
The final delivery report records the commit/push outcome. Historical native validation remains attributed
to the user; no new GNOME or secure decode PASS is claimed here.
