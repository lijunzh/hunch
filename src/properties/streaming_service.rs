//! Streaming service detection — now fully handled by `rules/streaming_service.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn service(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("streaming_service")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_amzn() {
        assert_eq!(
            service("Movie.AMZN.WEB-DL.mkv"),
            Some("Amazon Prime".into())
        );
    }
    #[test]
    fn test_netflix() {
        assert_eq!(service("Movie.NF.WEB-DL.mkv"), Some("Netflix".into()));
    }
}
