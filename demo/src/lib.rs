//! Intentionally clippy-hostile code. Every function below triggers at least
//! one lint when compiled with `cargo clippy --message-format json`. The
//! output is piped through rust-rapport on the demo PR to showcase how the
//! tool renders warnings in GitHub step summaries and inline annotations.

#![allow(dead_code)]

/// Triggers `unused_variables` and `clippy::needless_return`.
pub fn greet(name: &str) -> String {
    let unused = 42;
    return format!("Hello, {name}");
}

/// Triggers `clippy::or_fun_call` — allocates the `String::from` even when
/// the `Option` is `Some`.
pub fn name_or_default(opt: Option<String>) -> String {
    opt.unwrap_or(String::from("anonymous"))
}

/// Triggers `clippy::single_char_pattern` — splitting on `"a"` compiles to a
/// substring search when a `char` would be faster.
pub fn letters_around_a(input: &str) -> Vec<&str> {
    input.split("a").collect()
}

/// Triggers `clippy::redundant_clone` — the second `.clone()` is pointless
/// because the owned `String` could just be returned directly.
pub fn overly_cloney(s: String) -> String {
    let x = s.clone();
    x.clone()
}

/// Triggers `clippy::cast_possible_truncation` from the `pedantic` group.
pub fn truncate(big: i64) -> i32 {
    big as i32
}
