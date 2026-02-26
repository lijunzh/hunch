//! Country detection — now fully handled by `rules/country.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn country(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("country")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_us() {
        assert_eq!(country("Movie.US.mkv"), Some("US".into()));
    }
}
