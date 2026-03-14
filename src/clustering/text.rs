
//! Utilities for clustering similar names.

use regex::Regex;

use std::cmp::min;

const USELESS_COMMON_TOKENS: &[&str] = &[
  "the", "an", "a",
];

pub fn token_containment_score(a: &str, b: &str) -> f64 {
  let a = normalize_and_tokenize(a);
  let b = normalize_and_tokenize(b);

  // Corner cases: division by zero
  if a.is_empty() || b.is_empty() {
    return if a.is_empty() && b.is_empty() { 1.0 } else { 0.0 };
  }

  // For small text, O(n^2) is going to be much faster than allocating
  // a bunch of hashsets.
  let intersection = a.iter().filter(|a_tok| b.contains(a_tok)).count() as f64;

  intersection / min(a.len(), b.len()) as f64
}

fn normalize_and_tokenize(text: &str) -> Vec<String> {
  normalize_text(text)
    .split_whitespace()
    .filter(|s| !USELESS_COMMON_TOKENS.contains(&s))
    .map(|s| s.to_owned())
    .collect()
}

fn normalize_text(text: &str) -> String {
  let symbols_re = Regex::new("[^a-zA-Z0-9 ]").unwrap();
  let text = text.to_lowercase();
  let text = symbols_re.replace_all(&text, "");
  text
    .trim()
    .to_owned()
}
