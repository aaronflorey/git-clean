# Development and releases

Use the setup and check commands in the [README](../README.md#development).
Git and a native Rust linker/C toolchain are platform prerequisites. mise owns
the development tools; Cargo owns the application and test dependencies.

## Existing check limits

Clippy currently reports violations of the lint policy in `Cargo.toml`. The
`lint` task and existing hk Clippy step expose those failures; CI does not yet
make Clippy a required gate. Resolve that baseline before adding the gate.

The dependency license check also rejects the locked `unicode-ident` dependency's
`Unicode-3.0` license, which is absent from the existing allowlist. Review that
license before changing the policy. No license exception is added by setup.

The existing `pinact_update` hook requests major upgrades to Checkout,
release-please-action, and goreleaser-action. Setup pins their existing major
versions; review those major upgrades separately. `pinact run --check` verifies
the current pins without requesting upgrades.

The manifest's Rust 1.74 minimum predates the current lockfile: Clap now requires
Rust 1.85. Use the pinned development toolchain; the minimum supported Rust
version needs separate validation before being advertised.

The release configuration still uses GoReleaser's deprecated `brews` publisher
to preserve the existing Homebrew formula and Linux installation support.
`goreleaser check` reports this deprecation. Migrate the distribution method
before upgrading to a GoReleaser version that removes formula publishing.

## Releases

release-please owns the version in `Cargo.toml`, changelog, and `vX.X.X` tags.
Releasable Conventional Commits on `master` produce a release PR. Merging that PR
creates the release; the same workflow checks out its tag and runs GoReleaser.

The existing build publishes Linux and macOS archives for x86_64 and aarch64,
plus a Windows x86_64 zip, with `checksums.txt`. Rust, Zig, cargo-zigbuild, and
GoReleaser are provisioned by the release workflow. The Homebrew formula is
written to `aaronflorey/homebrew-tap` on `main`.

GitHub Actions must be allowed to create release PRs. The workflow uses
`GITHUB_TOKEN` for releases and requires `HOMEBREW_TAP_GITHUB_TOKEN` with write
access to the tap. Configure those repository settings and credentials outside
local setup. This project does not configure crates.io publishing.

For local release configuration validation, use `goreleaser check` if
GoReleaser v2 is installed, allowing for the known Homebrew deprecation above.
Do not run a live release to validate setup.

## Setup validation

Local setup added shared mise tasks, an explicit Node owner for the existing
npm tool, release-binary installation of cargo-deny, and repository-local hooks.
CI now checks Rust formatting, uses pinned actions and scoped permissions, and
avoids restoring build caches into release packaging. Dependabot checks Cargo
and GitHub Actions weekly with a seven-day cooldown. Installation instructions
cover the existing Homebrew tap, release binaries, and Cargo source builds.

Validation completed during setup:

- `mise install`, `mise run hooks`, and `mise exec -- hk validate` passed.
- `mise run build`, `mise run test` (`cargo test --locked`), and Rust formatting passed.
- The Conventional Commit hook accepted a valid test message.
- `mise exec -- pinact run --check` and `git diff --check` passed.
- All other hk checks passed with `cargo_clippy`, `cargo_deny`, and
  `pinact_update` excluded for the baseline issues documented above.
- `goreleaser check` validated the configuration but exited unsuccessfully
  because the retained `brews` publisher is deprecated.

Cross-platform archive builds, live GitHub workflows, tap credentials, and
publication were not exercised. No daemon configuration was added: this CLI
has no long-running development service.
