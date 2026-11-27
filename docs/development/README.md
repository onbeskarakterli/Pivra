# Develop Pivra

The `dev` branch reports `0.2.9-dev`. The current stable release and Arch
package are v0.2.8 and `0.2.8-1`, respectively.

Work on `dev` and keep commits small and independently reviewable. The
[architecture guide](../architecture/README.md) maps the source and shows where
new behavior belongs. Put independent features in dedicated modules when the
boundary is clear, and reuse existing actions, workers and caches.

Follow the [packaging guide](../packaging/README.md) for toolchain and native
dependencies. A normal Rust development build uses `cargo build --locked`.
During a focused change, run `cargo fmt --all -- --check`,
`cargo check --locked` and the most relevant existing test. Keep temporary
fixtures, build SDKs and machine-local settings outside tracked source.

Before a stable release, use the [testing guide](../testing/README.md) for
the full bounded regression run, physical GNOME acceptance and native package
checks. Use the [release workflow](../releases/README.md) for branch, version and
approval rules. The [feature specification](../PIVRA_FEATURE_SPEC_TR.md) and
[roadmap](../PIVRA_DEVELOPMENT_ROADMAP.md) describe planned behavior. The
[post-v0.2.7 feature record](NEXT_CYCLE_UI.md) describes implemented changes
and the scope of their original checks.
