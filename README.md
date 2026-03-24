# git-clean

[![CI](https://github.com/aaronflorey/git-clean/actions/workflows/ci.yml/badge.svg)](https://github.com/aaronflorey/git-clean/actions/workflows/ci.yml)

`git-clean` deletes branches that are already merged into your base branch (default: `main`).

It can delete:
- local branches
- remote branches
- both (default)

## Requirements

- `git` installed and available in `PATH`
- Rust `1.74+` only if building from source

## Install

Using [`bin`](https://github.com/marcosnils/bin) (recommended):

```bash
bin install aaronflorey/git-clean
```

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
-s, --squashes                  Detect squash-merged branches
-d, --delete-unpushed-branches  Treat local-only branches as deletable
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

```bash
cargo fmt -- --check
cargo test --locked
```

## License

MIT
