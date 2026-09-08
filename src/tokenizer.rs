pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace().map(|s| s.to_string()).collect()
}