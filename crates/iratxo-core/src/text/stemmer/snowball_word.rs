//! Minimal Snowball helper. Mirrors the subset of marrow's
//! `internal/stemmer/snowballword/snowballword.go` actually used by the
//! Basque stemmer: rune-indexed word with R1/R2/RV markers and a few
//! suffix utilities.
//!
//! Internally a `Vec<char>` so suffix matching by rune count is O(1) and
//! we avoid UTF-8 byte-boundary edge cases. `to_string()` round-trips back
//! into a normal `String`.

pub struct SnowballWord {
    pub rs: Vec<char>,
    pub r1_start: usize,
    pub r2_start: usize,
    pub rv_start: usize,
}

impl SnowballWord {
    pub fn new(word: &str) -> Self {
        let rs: Vec<char> = word.chars().collect();
        let n = rs.len();
        SnowballWord { rs, r1_start: n, r2_start: n, rv_start: n }
    }

    pub fn to_string(&self) -> String { self.rs.iter().collect() }

    pub fn has_suffix(&self, suffix: &[char]) -> bool {
        let n = self.rs.len();
        let m = suffix.len();
        if m > n { return false; }
        for i in 0..m {
            if self.rs[n - m + i] != suffix[i] {
                return false;
            }
        }
        true
    }

    pub fn remove_last_n(&mut self, n: usize) {
        let new_len = self.rs.len().saturating_sub(n);
        self.rs.truncate(new_len);
        self.reset_markers();
    }

    pub fn replace_suffix(&mut self, suffix: &[char], replacement: &[char]) {
        let new_len = self.rs.len().saturating_sub(suffix.len());
        self.rs.truncate(new_len);
        self.rs.extend_from_slice(replacement);
        self.reset_markers();
    }

    fn reset_markers(&mut self) {
        let n = self.rs.len();
        if self.r1_start > n { self.r1_start = n; }
        if self.r2_start > n { self.r2_start = n; }
        if self.rv_start > n { self.rv_start = n; }
    }
}

/// Snowball "vnv suffix" region marker (used by Romance/Basque). Returns the
/// index of the position immediately after the first non-vowel that follows a
/// vowel, starting from `start`. If there's no such pair, returns the end of
/// the word. Direct port of marrow's `romance.VnvSuffix`.
pub fn vnv_suffix<F: Fn(char) -> bool>(rs: &[char], is_vowel: F, start: usize) -> usize {
    let n = rs.len();
    if start >= n { return n; }
    // Iterate the same way marrow does: i=1..len(rs[start:]), j=start+i.
    let mut i = 1;
    while i < n - start {
        let j = start + i;
        if is_vowel(rs[j - 1]) && !is_vowel(rs[j]) {
            return j + 1;
        }
        i += 1;
    }
    n
}
