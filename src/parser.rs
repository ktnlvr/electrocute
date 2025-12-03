pub fn tokenize(input: &str) -> Vec<Vec<String>> {
    input
        .split("\n")
        .filter_map(|s| Some(s.trim()).filter(|s| !s.is_empty()))
        .map(|s| s.split_ascii_whitespace().map(|s| s.to_owned()).collect())
        .collect()
}
