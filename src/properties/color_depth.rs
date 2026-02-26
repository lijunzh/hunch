//! Color depth detection — now fully handled by `rules/color_depth.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn depth(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("color_depth")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_10bit() {
        assert_eq!(depth("Movie.10bit.mkv"), Some("10-bit".into()));
    }
    #[test]
    fn test_8bit() {
        assert_eq!(depth("Movie.8bit.mkv"), Some("8-bit".into()));
    }
}
