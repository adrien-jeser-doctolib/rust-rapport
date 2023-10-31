# Rust rapport

Example : https://github.com/adrien-jeser-doctolib/rust-rapport/pull/7/files

## Getting started

### Github action

```
on:
  push:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ (github.event.pull_request.head.ref || github.ref_name) != 'master' }}

env:
  CARGO_TERM_COLOR: always

name: Lints
jobs:
  default:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: rustup component add clippy
      - run: cargo install --git https://github.com/adrien-jeser-doctolib/rust-rapport.git
      - name: Clippy
        run: |
          set +e
          cargo clippy --color always --message-format json \
            | tee >(rust-rapport github-summary >> "${GITHUB_STEP_SUMMARY}") >(rust-rapport github-pr-annotation) > /dev/null
           exit "${PIPESTATUS[0]}"
```
