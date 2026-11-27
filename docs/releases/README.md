# Pivra release workflow

The [v0.2.8 release notes](PIVRA_V0.2.8_RELEASE_NOTES.md) describe the stable
source. The [validation record](PIVRA_V0.2.8_RC_VALIDATION.md) tracks the
accepted package and remaining publication gates. Earlier records retain their
original outcomes, including [v0.2.7](PIVRA_V0.2.7_RELEASE_NOTES.md),
[v0.2.5](PIVRA_V0.2.5_RELEASE_NOTES.md) and the unreleased
[v0.2.6 candidate](PIVRA_V0.2.6_RC_VALIDATION.md).

1. Develop on `dev` and freeze features for the target release. Complete
   bounded regression/stability testing and document verified fixes, skips and
   open defects. Run physical GNOME acceptance on the intended system.
2. Review README, user/developer documentation and release notes. Keep
   stable and candidate claims tied to their actual sources and outcomes.
3. Prepare the version only when authorized. Keep `Cargo.toml`, the root Pivra
   package in `Cargo.lock`, the version displayed by Pivra and the release
   packaging metadata consistent. Run the version-matched candidate/final
   preflight on the reviewed source and actual staged artifact.
4. Validate the native Arch candidate, `.SRCINFO`, installed package, runtime
   behavior and source/notice inventory with an exact local commit archive.
   After explicit approval, promote the stable source to `main` through the
   GitHub review workflow and create the annotated tag.
5. Verify the tag target and the SHA-256 of the immutable GitHub commit
   archive. Validate the package from that archive in a separate native Arch
   build. Publish the GitHub Release and AUR package after the final source
   and package checks, then record actual remote outcomes. [Packaging guide](../packaging/README.md)
6. Once both GitHub and AUR publication are verified, synchronize `dev` and
   bump its version to the next `-dev` version. v0.2.8 is followed by
   `0.2.9-dev`, and v0.2.9 by `0.2.10-dev`. Keep the Cargo and displayed
   versions in agreement. Numeric preflight targets apply to stable versions.

Attribute owner reports, controlled tests and final artifact verification to
their actual environment and source SHA.
