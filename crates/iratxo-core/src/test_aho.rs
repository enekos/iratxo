#[cfg(test)]
mod tests {
    #[test]
    fn aho_is_clone() {
        let ac = aho_corasick::AhoCorasick::new(["foo"]).unwrap();
        let _ = ac.clone();
    }
}
