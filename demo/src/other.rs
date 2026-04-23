//! "Unrelated" file: lives on `main`, never modified by the demo PR.
//!
//! It deliberately holds a handful of clippy findings. Because the demo PR
//! doesn't touch this file, GitHub will *not* render those findings as inline
//! annotations on the PR's Files changed tab (annotations are diff-scoped).
//! They still appear in the Markdown step summary produced by
//! `rust-rapport github-summary` — that's the whole point of showing them:
//! demonstrating that the summary catches what inline annotations miss.

#![allow(dead_code)]

/// Triggers `unused_variables` and `clippy::needless_return`.
pub fn echo_owned(s: &str) -> String {
    let unused = 1;
    return s.to_owned();
}

/// Triggers `clippy::len_zero` — should use `.is_empty()`.
pub fn looks_empty(s: &str) -> bool {
    s.len() == 0
}

/// Triggers `clippy::needless_collect` — the intermediate `Vec` is pointless.
pub fn byte_count(s: &str) -> usize {
    s.bytes().collect::<Vec<_>>().len()
}
