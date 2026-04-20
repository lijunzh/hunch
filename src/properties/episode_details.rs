//! Episode details detection — now fully handled by `src/rules/episode_details.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn details(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("episode_details")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_special() {
        assert_eq!(details("Show.S01.Special.mkv"), Some("Special".into()));
    }
    #[test]
    fn test_pilot() {
        assert_eq!(details("Show.S01.Pilot.mkv"), Some("Pilot".into()));
    }
}
