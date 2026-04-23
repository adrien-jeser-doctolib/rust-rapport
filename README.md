# rust-rapport

[![CI](https://github.com/adrien-jeser-doctolib/rust-rapport/actions/workflows/ci.yml/badge.svg)](https://github.com/adrien-jeser-doctolib/rust-rapport/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/rust-rapport.svg)](https://crates.io/crates/rust-rapport)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Formats `cargo clippy --message-format json` output into three views tailored for CI:

- **`github-summary`** — a Markdown table suitable for `$GITHUB_STEP_SUMMARY`.
- **`github-pr-annotation`** — GitHub workflow commands that render inline annotations on PR diffs.
- **`human`** — the plain rendered diagnostics, for local terminal use.

Malformed JSON lines are logged to stderr with the offending line number and skipped; valid lines still render. The exit code is `0` on success, `1` on I/O failure.

## Install

**From crates.io** (once released):

```sh
cargo install rust-rapport --locked
```

**In GitHub Actions** — zero-compile, downloads a prebuilt binary:

```yaml
- uses: taiki-e/install-action@v2
  with:
    tool: rust-rapport
```

**From GitHub Releases** — download a pre-built archive from the [Releases page](https://github.com/adrien-jeser-doctolib/rust-rapport/releases) and extract it onto your `PATH`.

## Usage

```sh
cargo clippy --message-format json | rust-rapport github-summary >> "$GITHUB_STEP_SUMMARY"
cargo clippy --message-format json | rust-rapport github-pr-annotation
cargo clippy --message-format json | rust-rapport human
```

## GitHub Actions

```yaml
- uses: actions/checkout@v4
- uses: dtolnay/rust-toolchain@stable
  with: { components: clippy }
- uses: taiki-e/install-action@v2
  with:
    tool: rust-rapport
- name: Clippy
  run: |
    set +e
    cargo clippy --message-format json \
      | tee >(rust-rapport github-summary >> "$GITHUB_STEP_SUMMARY") \
            >(rust-rapport github-pr-annotation) \
      > /dev/null
    exit "${PIPESTATUS[0]}"
```

## Requirements

- Rust 1.85+ (edition 2024, MSRV enforced in CI).

## Supply chain

Two orthogonal checks run in CI:

- **`cargo-audit`** (RUSTSEC) — fails if a published advisory matches any locked dependency. Also runs daily against `main` via `.github/workflows/audit.yml`.
- **`cargo-vet`** — requires every dependency version to be either human-audited or exempted. Audits are imported from [Mozilla](https://github.com/mozilla/supply-chain), [Google](https://github.com/google/supply-chain), [Bytecode Alliance](https://github.com/bytecodealliance/wasmtime), and [Divvi Up](https://github.com/divviup/libprio-rs); the project's own audits live in [`supply-chain/audits.toml`](supply-chain/audits.toml).

## Releasing

Releases are fully automated by [release-plz](https://release-plz.ieni.dev/). You never tag or bump `Cargo.toml` by hand — you just commit with [Conventional Commits](https://www.conventionalcommits.org/) messages:

- `feat: …` — minor bump
- `fix: …` — patch bump
- `feat!: …` or `BREAKING CHANGE:` in the body — major bump
- `chore: …`, `ci: …`, `docs: …`, `refactor: …`, `test: …` — no bump (but still visible in the PR)

**Flow:**

1. Land conventional commits on `main` (either directly or via merged PRs).
2. The `release-plz` workflow opens (or updates) a **release PR** titled `chore: release vX.Y.Z` that bumps the version in `Cargo.toml` and rewrites the `[Unreleased]` section of [`CHANGELOG.md`](CHANGELOG.md) into a dated version entry.
3. Review the PR. Edit the changelog prose freely — release-plz won't clobber your edits on subsequent runs as long as the version stays the same.
4. Merge the PR. release-plz then pushes the tag `vX.Y.Z`, creates the GitHub Release, and publishes to crates.io.
5. The `Upload release binaries` workflow reacts to the GitHub Release being published and attaches the four cross-platform archives as release assets.

## See it live

A permanently-open draft PR renders the tool's output on intentionally clippy-hostile code — step summary, inline PR annotations, the works. It re-runs automatically against each newly published release: [Demo PR](https://github.com/adrien-jeser-doctolib/rust-rapport/pulls?q=is%3Apr+is%3Adraft+head%3Ademo%2Fshowcase).

## License

MIT. See [`LICENSE`](LICENSE) and [`CHANGELOG.md`](CHANGELOG.md).
