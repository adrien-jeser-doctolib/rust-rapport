# rust-rapport

Formats `cargo clippy --message-format json` output into three views tailored for CI:

- **`github-summary`** — a Markdown table suitable for `$GITHUB_STEP_SUMMARY`.
- **`github-pr-annotation`** — GitHub workflow commands that render inline annotations on PR diffs.
- **`human`** — the plain rendered diagnostics, for local terminal use.

Malformed JSON lines are logged to stderr with the offending line number and skipped; valid lines still render. The exit code is `0` on success, `1` on I/O failure.

## Install

```sh
cargo install --git https://github.com/adrien-jeser-doctolib/rust-rapport.git
```

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
- run: cargo install --git https://github.com/adrien-jeser-doctolib/rust-rapport.git
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

## License

MIT. See [`LICENSE`](LICENSE) and [`CHANGELOG.md`](CHANGELOG.md).
