//! Edition detection — now fully handled by `src/rules/edition.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn edition(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("edition")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_directors_cut() {
        assert_eq!(
            edition("Movie.Directors.Cut.mkv"),
            Some("Director's Cut".into())
        );
    }
    #[test]
    fn test_extended() {
        assert_eq!(edition("Movie.EXTENDED.mkv"), Some("Extended".into()));
    }
    #[test]
    fn test_remastered() {
        assert_eq!(edition("Movie.REMASTERED.mkv"), Some("Remastered".into()));
    }
    #[test]
    fn test_imax() {
        assert_eq!(edition("Movie.IMAX.mkv"), Some("IMAX".into()));
    }
    #[test]
    fn test_special_edition() {
        assert_eq!(edition("Movie.Special.Edition.mkv"), Some("Special".into()));
    }
    #[test]
    fn test_limited() {
        assert_eq!(edition("Movie.LIMITED.mkv"), Some("Limited".into()));
    }
    #[test]
    fn test_deluxe() {
        assert_eq!(edition("Movie.DELUXE.mkv"), Some("Deluxe".into()));
    }
    #[test]
    fn test_collector() {
        assert_eq!(
            edition("Movie.Collectors.Edition.mkv"),
            Some("Collector".into())
        );
    }
    #[test]
    fn test_ddc() {
        assert_eq!(edition("Movie.DC.mkv"), Some("Director's Cut".into()));
    }
    #[test]
    fn test_alternative_cut() {
        assert_eq!(
            edition("Movie.Alternative.Cut.mkv"),
            Some("Alternative Cut".into())
        );
    }
    #[test]
    fn test_4k_remastered() {
        assert_eq!(
            edition("Movie.4K.Remastered.mkv"),
            Some("Remastered".into())
        );
    }
}
