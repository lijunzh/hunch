//! Edition detection (Director's Cut, Extended, Unrated, Collector, etc.).

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static EDITION_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        // Director's cuts.
        ValuePattern::new(r"(?i)(?<![a-z])DDC(?![a-z])", "Director's Definitive Cut"),
        ValuePattern::new(
            r"(?i)(?<![a-z])Director'?s?[-. ]?(?:Definitive[-. ]?)?Cut(?![a-z])",
            "Director's Cut",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])DC(?![a-z'])", "Director's Cut"),
        // Extended / Unrated / Theatrical.
        ValuePattern::new(
            r"(?i)(?<![a-z])Extended(?:[-. ]?(?:Cut|Edition))?(?![a-z])",
            "Extended",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])Unrated(?:[-. ]?(?:Cut|Edition))?(?![a-z])",
            "Unrated",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])Theatrical(?:[-. ]?(?:Cut|Edition))?(?![a-z])",
            "Theatrical",
        ),
        // Collector.
        ValuePattern::new(
            r"(?i)(?<![a-z])Collector'?s?(?:[-. ]?Edition)?(?![a-z])",
            "Collector",
        ),
        // Special.
        ValuePattern::new(r"(?i)(?<![a-z])Special[-. ]?Edition(?![a-z])", "Special"),
        ValuePattern::new(r"(?<![a-zA-Z])SE(?![a-zA-Z])", "Special"),
        // Ultimate.
        ValuePattern::new(r"(?i)(?<![a-z])Ultimate[-. ]?Edition(?![a-z])", "Ultimate"),
        ValuePattern::new(r"(?i)(?<![a-z])Ultimate(?![a-z])", "Ultimate"),
        // Deluxe.
        ValuePattern::new(r"(?i)(?<![a-z])Deluxe(?:[-. ]?Edition)?(?![a-z])", "Deluxe"),
        // Anniversary.
        ValuePattern::new(
            r"(?i)(?<![a-z])Anniversary(?:[-. ]?Edition)?(?![a-z])",
            "Anniversary Edition",
        ),
        // Criterion.
        ValuePattern::new(
            r"(?i)(?<![a-z])Criterion[-. ]?(?:Collection|Edition)(?![a-z])",
            "Criterion",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])CC(?![a-z])", "Criterion"),
        ValuePattern::new(r"(?i)(?<![a-z])Criterion(?![a-z])", "Criterion"),
        // IMAX.
        ValuePattern::new(r"(?i)(?<![a-z])IMAX(?:[-. ]?Edition)?(?![a-z])", "IMAX"),
        // Alternative Cut.
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Alternate|Alternative)(?:[-. ]?Cut)?(?![a-z])",
            "Alternative Cut",
        ),
        // Fan.
        ValuePattern::new(
            r"(?i)(?<![a-z])Fan[-. ]?(?:Edit|Edition|Collection)(?![a-z])",
            "Fan",
        ),
        // Limited.
        ValuePattern::new(
            r"(?i)(?<![a-z])Limited(?:[-. ]?Edition)?(?![a-z])",
            "Limited",
        ),
        // Remaster / Restore.
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:4[Kk][-. ]?)?Remaster(?:ed)?(?![a-z])",
            "Remastered",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:4[Kk][-. ]?)?Restor(?:ed?)?(?![a-z])",
            "Restored",
        ),
        // Uncensored.
        ValuePattern::new(r"(?i)(?<![a-z])Uncensored(?![a-z])", "Uncensored"),
        // Uncut.
        ValuePattern::new(
            r"(?i)(?<![a-z])Uncut(?:[-. ]?(?:Cut|Edition))?(?![a-z])",
            "Uncut",
        ),
        // Festival.
        ValuePattern::new(
            r"(?i)(?<![a-z])Festival(?:[-. ]?(?:Cut|Edition))?(?![a-z])",
            "Festival",
        ),
        // Edition Special (reversed order).
        ValuePattern::new(r"(?i)(?<![a-z])Edition[-. ]?Special(?![a-z])", "Special"),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in EDITION_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(MatchSpan::new(start, end, Property::Edition, pattern.value));
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directors_cut() {
        let m = find_matches("Movie.Directors.Cut.mkv");
        assert!(m.iter().any(|x| x.value == "Director's Cut"));
    }

    #[test]
    fn test_extended() {
        let m = find_matches("Movie.Extended.Edition.mkv");
        assert!(m.iter().any(|x| x.value == "Extended"));
    }

    #[test]
    fn test_collector() {
        let m = find_matches("Movie.Collector.Edition.mkv");
        assert!(m.iter().any(|x| x.value == "Collector"));
    }

    #[test]
    fn test_special_edition() {
        let m = find_matches("Movie.Special.Edition.mkv");
        assert!(m.iter().any(|x| x.value == "Special"));
    }

    #[test]
    fn test_imax() {
        let m = find_matches("Movie.IMAX.mkv");
        assert!(m.iter().any(|x| x.value == "IMAX"));
    }

    #[test]
    fn test_remastered() {
        let m = find_matches("Movie.Remastered.mkv");
        assert!(m.iter().any(|x| x.value == "Remastered"));
    }

    #[test]
    fn test_limited() {
        let m = find_matches("Movie.LiMiTED.mkv");
        assert!(m.iter().any(|x| x.value == "Limited"));
    }

    #[test]
    fn test_deluxe() {
        let m = find_matches("Movie.Deluxe.Edition.mkv");
        assert!(m.iter().any(|x| x.value == "Deluxe"));
    }

    #[test]
    fn test_alternative_cut() {
        let m = find_matches("Movie.Alternate.Cut.mkv");
        assert!(m.iter().any(|x| x.value == "Alternative Cut"));
    }

    #[test]
    fn test_ddc() {
        let m = find_matches("Movie.DDC.mkv");
        assert!(m.iter().any(|x| x.value == "Director's Definitive Cut"));
    }

    #[test]
    fn test_4k_remastered() {
        let m = find_matches("Movie.4k.Remastered.mkv");
        assert!(m.iter().any(|x| x.value == "Remastered"));
    }
}
