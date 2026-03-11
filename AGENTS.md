# Repository Guidelines

## Project Structure & Module Organization
`git-clean` is a single Rust crate.

- `src/main.rs`: CLI entrypoint.
- `src/lib.rs`: orchestration of option resolution, validation, and branch cleanup.
- `src/cli.rs`: Clap argument definitions.
- `src/options.rs`: effective option resolution (defaults, repo config, CLI overrides).
- `src/config.rs`: per-repository config load/save at `~/.config/git-clean/config.toml`.
- `src/branches.rs`, `src/commands.rs`, `src/error.rs`: branch logic, shell command helpers, error types.
- `tests/`: integration tests (`tests.rs` includes `local`, `remote`, `deletion`, `utility`) with helpers in `tests/support.rs`.
- `.github/workflows/`: CI (`ci.yml`), release automation (`release-please.yml`, `goreleaser.yml`).

## Build, Test, and Development Commands
- `cargo build` — compile debug binary.
- `cargo run -- -h` — run CLI locally.
- `cargo test --locked` — run unit + integration tests using lockfile.
- `cargo fmt` — format source.
- `cargo fmt -- --check` — formatting check for CI/local validation.
- `cargo clippy --all-targets --all-features -D warnings` — recommended lint gate before PR.

## Coding Style & Naming Conventions
- Rust edition: 2021.
- This repo enforces `#![deny(warnings)]`; keep code warning-free.
- Use `snake_case` for functions/variables/modules, `CamelCase` for types/enums, `SCREAMING_SNAKE_CASE` for constants.
- Keep module boundaries clear: CLI parsing in `cli.rs`, options/config in `options.rs` + `config.rs`, git behavior in `branches.rs`/`commands.rs`.

## Testing Guidelines
- Unit tests live near implementation (`#[cfg(test)]` modules in `src/*`).
- Integration tests live under `tests/` and should be scenario-focused (`test_git_clean_...` naming pattern).
- Always run `cargo test --locked` after behavior changes.
- If CLI flags or config behavior changes, update tests and README examples together.

## Commit & Pull Request Guidelines
- Prefer Conventional Commit style used in recent history (`feat: ...`, `ci: ...`, `fix: ...`).
- Keep commits scoped and reviewable (logic, tests, docs together for one feature).
- PRs should include:
  - concise summary of behavior changes,
  - test evidence (`cargo test --locked`, plus lint/format if run),
  - docs updates when flags/config/release behavior changes.
