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

## Releasing

1. Move the `[Unreleased]` section of [`CHANGELOG.md`](CHANGELOG.md) to `[X.Y.Z] - YYYY-MM-DD`.
2. Bump `version` in [`Cargo.toml`](Cargo.toml).
3. `git commit -am "chore: release vX.Y.Z" && git tag vX.Y.Z && git push --follow-tags`.

The `release.yml` workflow takes over from there: it creates a GitHub Release with the CHANGELOG entry as notes, uploads cross-platform binaries, and publishes the crate to crates.io.

## License

MIT. See [`LICENSE`](LICENSE) and [`CHANGELOG.md`](CHANGELOG.md).
