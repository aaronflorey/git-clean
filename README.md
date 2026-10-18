# git-clean

[![CI](https://github.com/aaronflorey/git-clean/actions/workflows/ci.yml/badge.svg)](https://github.com/aaronflorey/git-clean/actions/workflows/ci.yml)

`git-clean` deletes branches that are already merged into your base branch (default: `main`).

It can delete:

- local branches
- remote branches
- both (default)

## Requirements

- `git` installed and available in `PATH`
- A Rust toolchain only if building from source (development uses the version in `mise.toml`)

## Install

Using Homebrew (recommended):

```bash
brew install aaronflorey/tap/git-clean
```

Using [`bin`](https://github.com/aaronflorey/bin):

```bash
bin install github.com/aaronflorey/git-clean
```

Alternatively, download an archive for your OS and architecture from
[GitHub Releases](https://github.com/aaronflorey/git-clean/releases), verify it
against `checksums.txt`, and put the extracted executable in your `PATH`.

With Cargo from this fork (not crates.io):

```bash
cargo install --git https://github.com/aaronflorey/git-clean.git --locked
```

Build locally from source:

```bash
cargo build --release
```

## Quick Start

Preview and confirm deletion:

```bash
git-clean
```

Run non-interactively:

```bash
git-clean -y
```

Delete only local or only remote:

```bash
git-clean -l -y
git-clean -r -y
```

## Common Options

```text
-l, --locals                    Only delete local branches
-r, --remotes                   Only delete remote branches
-y, --yes                       Skip confirmation prompt
    --dry-run                   Preview branches without deleting them
-s, --squashes                  Detect squash-merged branches
-d, --delete-unpushed-branches  Delete branches with no upstream that are absent on remote
-R, --remote <remote>           Remote name (default: origin)
-b, --branch <branch>           Base branch (default: main)
-i, --ignore <branch>           Ignore branch (repeatable)
    --color <auto|always|never> Control color output (default: auto)
    --save-config               Save provided flags for this repository
    --ignore-config             Ignore saved repository config
```

## Per-Repository Config

`git-clean` supports saved defaults per repository in:

`~/.config/git-clean/config.toml`

CLI flags always override config values.

Save flags for the current repository:

```bash
git-clean --save-config -s -d -i release -i keep-me
```

Example config:

```toml
["/Users/you/code/my-repo"]
squashes = true
delete_unpushed_branches = true
remote = "origin"
base_branch = "main"
ignored_branches = ["release", "keep-me"]
delete_mode = "both"
color_mode = "auto"
```

## Development

Install [mise](https://mise.jdx.dev/) and Git, then set up the pinned tools and
repository hooks:

```bash
mise install
mise run hooks
```

The toolchain needs the rustfmt and Clippy components (`rustup component add rustfmt clippy`
if they are missing). The pinned Node runtime supplies npm for the existing filename linter.

```bash
mise run build
mise run test
mise run lint
mise run check
```

`mise run format` formats Rust; `mise run fix` runs the configured hook fixers.
`check` runs all existing repository checks, including dependency and secret checks.
Hooks check Conventional Commit messages (`feat:`, `fix:`, `chore:`) for release-please.
The equivalent native Rust checks remain available:

```bash
cargo fmt -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

CI tests stable, beta, and nightly Rust (nightly failures are allowed) and checks
formatting. See [development and release notes](docs/DEVELOPMENT.md) for validation
limits and release prerequisites.

## License

MIT
