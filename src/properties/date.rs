//! Date detection.
//!
//! Detects air dates in filenames: 2014.12.25, 25-12-2014, etc.

use lazy_static::lazy_static;
use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// YYYY.MM.DD or YYYY-MM-DD format.
    static ref DATE_YMD: Regex = Regex::new(
        r"(?<![0-9])(?P<date>(?:19|20)\d{2})[.-](?P<month>0[1-9]|1[0-2])[.-](?P<day>0[1-9]|[12]\d|3[01])(?![0-9])"
    ).unwrap();

    /// DD.MM.YYYY or DD-MM-YYYY format.
    static ref DATE_DMY: Regex = Regex::new(
        r"(?<![0-9])(?P<day>0[1-9]|[12]\d|3[01])[.-](?P<month>0[1-9]|1[0-2])[.-](?P<year>(?:19|20)\d{2})(?![0-9])"
    ).unwrap();

    /// Month name format: 25 Dec 2014, Dec 25 2014.
    static ref DATE_NAMED: Regex = Regex::new(
        r"(?i)(?<![a-z])(?P<day>\d{1,2})[-. ]?(?P<month>Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*[-. ]?(?P<year>(?:19|20)\d{2})(?![0-9])"
    ).unwrap();
}

pub struct DateMatcher;

impl PropertyMatcher for DateMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        if let Ok(Some(cap)) = DATE_YMD.captures(input) {
            if let (Some(full), Some(year), Some(month), Some(day)) =
                (cap.get(0), cap.name("date"), cap.name("month"), cap.name("day"))
            {
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::Date,
                        format!("{}-{}-{}", year.as_str(), month.as_str(), day.as_str()),
                    )
                    .with_priority(2),
                );
            }
        }

        if matches.is_empty() {
            if let Ok(Some(cap)) = DATE_DMY.captures(input) {
                if let (Some(full), Some(year), Some(month), Some(day)) =
                    (cap.get(0), cap.name("year"), cap.name("month"), cap.name("day"))
                {
                    matches.push(
                        MatchSpan::new(
                            full.start(),
                            full.end(),
                            Property::Date,
                            format!("{}-{}-{}", year.as_str(), month.as_str(), day.as_str()),
                        )
                        .with_priority(2),
                    );
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ymd() {
        let m = DateMatcher.find_matches("Show.2014.12.25.HDTV.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2014-12-25");
    }

    #[test]
    fn test_dmy() {
        let m = DateMatcher.find_matches("Show.25-12-2014.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2014-12-25");
    }
}
