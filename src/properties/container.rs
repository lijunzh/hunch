//! Container detection — now fully handled by pipeline PATH A (extension)
//! and `rules/container.toml` (standalone tokens).
//!
//! PATH A: Tokenizer strips extension → pipeline emits Container (priority 10)
//! PATH B: container.toml exact matches → standalone tokens (priority 5)

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn container(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("container")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_mkv() {
        assert_eq!(container("Movie.2020.mkv"), Some("mkv".into()));
    }

    #[test]
    fn test_srt() {
        assert_eq!(container("Movie.srt"), Some("srt".into()));
    }

    #[test]
    fn test_no_extension() {
        assert_eq!(container("Movie 2020 1080p"), None);
    }
}
