# Pivra release workflow

The published stable release is [v0.2.7](PIVRA_V0.2.7_RELEASE_NOTES.md).
v0.2.8 is under preparation. Its [draft notes](PIVRA_V0.2.8_RELEASE_NOTES.md)
and [validation record](PIVRA_V0.2.8_RC_VALIDATION.md) describe the current
candidate. Earlier release records retain their original outcomes, including
published [v0.2.5](PIVRA_V0.2.5_RELEASE_NOTES.md) and the unreleased
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
4. Review the immutable source archive and verify the real SHA-256. Obtain
   explicit approval for that source. Validate the local native Arch build,
   `.SRCINFO`, installed package, runtime behavior and source/notice inventory.
   See the [packaging guide](../packaging/README.md).
5. After explicit publication approval, promote the authorized stable source
   to `main` through the applicable GitHub review/protection workflow.
   Verify the tag points to the approved source before creating the GitHub
   Release. Publish the AUR package against the approved archive and verify
   the package version, metadata and install. Record actual remote outcomes.
6. After stable publication, move `dev` to the next intended version with a
   `-dev` suffix. Publishing v0.2.8 would lead to `0.2.9-dev` on `dev`, and
   publishing v0.2.9 would lead to `0.2.10-dev`. Keep Cargo and the displayed
   version in agreement. Numeric-only stable release preflight applies when
   preparing a stable release version.

The versioning policy describes future work. Current v0.2.8 preparation leaves
the checked-in source and recipe at 0.2.7. Keep owner reports, controlled tests
and final artifact verification attributed to their real environment and SHA.
