# Changelog

All notable changes to this project are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/adrien-jeser-doctolib/rust-rapport/compare/v0.1.3...v0.1.4) - 2026-04-23

### Other

- trust major Rust maintainers and import zcash/embark audits

## [0.1.3](https://github.com/adrien-jeser-doctolib/rust-rapport/compare/v0.1.2...v0.1.3) - 2026-04-23

### Other

- exempt rust-rapport from self-vet (first-party, not external dep)
- pin cargo-vet to 0.10.0 to match local toolchain (fixes imports.lock formatting drift)

## [0.1.2](https://github.com/adrien-jeser-doctolib/rust-rapport/compare/v0.1.1...v0.1.2) - 2026-04-23

### Other

- add workflow_dispatch trigger to release.yml for retroactive binary upload
- use RELEASE_PLZ_TOKEN PAT so release events trigger downstream workflows
- auto-merge release-plz PRs

## [0.1.1](https://github.com/adrien-jeser-doctolib/rust-rapport/compare/v0.1.0...v0.1.1) - 2026-04-23

### Other

- ignore .claude/ and use release-plz default changelog parsers

### Added
- `LICENSE` file (MIT) — was declared in `Cargo.toml` but missing at the repo root, blocking `cargo publish` and breaking the README link.
- `.editorconfig` for editor-agnostic indentation and line endings.
- `.github/workflows/audit.yml` — scheduled daily `cargo-audit` against the RUSTSEC advisory database.
- `.github/workflows/release-plz.yml` + `release-plz.toml` — fully automated releases: conventional commits on `main` open/update a release PR that bumps `Cargo.toml` and writes the `CHANGELOG.md` entry; merging the PR tags, creates the GitHub Release, and publishes to crates.io.
- `.github/workflows/release.yml` — listens for published GitHub Releases and attaches cross-platform binaries (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `universal-apple-darwin`, `x86_64-pc-windows-msvc`) as release assets.
- CI matrix extended to cross-platform: `test` runs on Linux, macOS, and Windows; separate `msrv` job verifies the 1.85 floor.
- CI `audit` job (`rustsec/audit-check`) running on every PR.
- Community files: `.github/CODEOWNERS`, issue templates (bug / feature / config), `PULL_REQUEST_TEMPLATE.md`.
- README badges (CI, crates.io, MSRV, license), install section covering crates.io / `taiki-e/install-action` / prebuilt archives, and a `Releasing` section.

### Changed
- README's GitHub Actions example now installs rust-rapport via `taiki-e/install-action@v2` (prebuilt binary) instead of `cargo install --git` (source compile on every run).

## [0.1.0] - 2026-04-23

### Added
- Rust edition 2024 and MSRV `1.85` pinned in `Cargo.toml`; toolchain pinned via `rust-toolchain.toml`.
- Library crate (`src/lib.rs`) exposing `run`, `Mode`, and `Error` — the binary is now a thin entrypoint.
- Integration tests in `tests/cli.rs` driven by JSON fixtures under `tests/fixtures/`.
- Unit tests in every module (`level`, `output`, `print`, top-level).
- Workspace-level lint configuration via `[lints]` (rust + clippy pedantic/nursery/cargo, `unsafe_code = "forbid"`).
- `rustfmt.toml` pinning format rules (stable only).
- `.github/workflows/ci.yml` replacing `rust.yml` + `clippy.yml`: separate jobs for fmt, clippy, test (matrix stable + MSRV), release build, and a self-check job that dogfoods the binary on the repo's own clippy output.
- `renovate.json` managing both `cargo` and `github-actions` updates, grouping non-major bumps and auto-merging patch/minor/digest/lockfile updates once CI is green (majors stay manual with a `major-update` label). Also enables `lockFileMaintenance` on a weekly schedule.
- `CHANGELOG.md`.

### Changed
- Bumped to `0.1.0` (from `0.0.1`).
- Diagnostic deduplication now uses `BTreeSet<Output>` instead of `HashSet<Output>`, guaranteeing deterministic output ordering.
- `success` detection for the "Cargo is Happy / Sad" summary branch is now computed on the full stream before filtering, so the happy branch actually fires on clean builds.
- Error handling rewritten in plain `std` (no `anyhow`/`thiserror`): a typed `Error` enum with hand-written `Display`, `Error`, and `From<io::Error>` impls; `main` returns `ExitCode`.
- CI actions upgraded to `actions/checkout@v4`, `dtolnay/rust-toolchain`, and `Swatinem/rust-cache@v2`.
- Switched from Dependabot to Renovate (`.github/dependabot.yml` removed; `renovate.json` added).
- `SECURITY.md` rewritten (was a GitHub template placeholder).

### Fixed
- `Output::column_start` previously returned `column_end` (copy-paste bug); it now returns `column_start`. GitHub PR annotations now point at the correct column.
- `print::github_summary` no longer prints the Markdown table header directly to stdout as a side effect — it returns the full table (header + body) as a single `String`.
- Typo `"Unknow"` → `"Unknown"` in the summary fallback.
- `github-pr-annotation` no longer emits the invalid `:: ::` form when a diagnostic carries no metadata.
- Malformed JSON lines are now reported on stderr with their line number and skipped, instead of being silently dropped by `.ok()` / `.flatten()`.
